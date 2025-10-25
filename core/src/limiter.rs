use crate::config::{
    CheckDecision, CheckRequest, CheckResult, CompiledPolicy, FluxgateConfig, FluxgateInit,
    PolicyAction,
};
use crate::error::{FluxgateError, Result};
use crate::gcra::TokenBucket;
use crate::key_builder::KeyBuilder;
use crate::metrics::Metrics;
use crate::time;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fluxgate {
    config: FluxgateConfig,
    key_builder: KeyBuilder,
    policies: Vec<PolicyState>,
    metrics: Metrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PolicyState {
    compiled: CompiledPolicy,
    buckets: HashMap<u64, TokenBucket>,
}

impl Fluxgate {
    pub fn new(init: FluxgateInit) -> Result<Self> {
        let config = init.into_config()?;
        Self::from_config(config)
    }

    fn from_config(config: FluxgateConfig) -> Result<Self> {
        let key_builder = KeyBuilder::new(config.key_secret.as_deref());
        let policies = config
            .policies
            .iter()
            .cloned()
            .map(PolicyState::new)
            .collect();

        Ok(Self {
            config,
            key_builder,
            policies,
            metrics: Metrics::default(),
        })
    }

    pub fn check(&mut self, request: CheckRequest) -> CheckResult {
        let now_ms = time::now_ms();
        let mut decisions = IndexMap::new();
        let mut allowed = true;
        let mut retry_after: Option<u32> = None;

        for policy in &mut self.policies {
            if let Some((decision, enforce)) = policy.check(&self.key_builder, &request, now_ms) {
                if enforce && !decision.allowed {
                    allowed = false;
                    retry_after = match (retry_after, decision.retry_after_ms) {
                        (Some(existing), Some(new_retry)) => Some(existing.max(new_retry)),
                        (None, Some(new_retry)) => Some(new_retry),
                        (existing, None) => existing,
                    };
                }
                decisions.insert(policy.policy_id().to_string(), decision);
            }
        }

        self.metrics.record(allowed);

        if allowed {
            CheckResult {
                allowed: true,
                retry_after_ms: None,
                decisions,
            }
        } else {
            CheckResult::denied(retry_after, decisions)
        }
    }

    pub fn check_batch(&mut self, requests: Vec<CheckRequest>) -> Vec<CheckResult> {
        requests.into_iter().map(|req| self.check(req)).collect()
    }

    pub fn rotate(&mut self) {
        // For the initial WASM build the rotation hook is a lightweight no-op. The
        // method exists to maintain API compatibility with the native library and
        // can later incorporate time-sliced eviction when Tier B approximations are
        // implemented.
    }

    pub fn reload(&mut self, init: FluxgateInit) -> Result<()> {
        let config = init.into_config()?;
        let rebuilt = Self::from_config(config)?;
        *self = rebuilt;
        Ok(())
    }

    pub fn snapshot(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|err| FluxgateError::Serialization(err.to_string()))
    }

    pub fn restore(&mut self, bytes: &[u8]) -> Result<()> {
        let restored: Fluxgate = bincode::deserialize(bytes)
            .map_err(|err| FluxgateError::Serialization(err.to_string()))?;
        *self = restored;
        Ok(())
    }

    pub fn metrics(&self) -> IndexMap<String, u64> {
        self.metrics.as_map()
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

impl PolicyState {
    fn new(compiled: CompiledPolicy) -> Self {
        Self {
            compiled,
            buckets: HashMap::new(),
        }
    }

    fn policy_id(&self) -> &str {
        &self.compiled.definition.id
    }

    fn check(
        &mut self,
        key_builder: &KeyBuilder,
        request: &CheckRequest,
        now_ms: u64,
    ) -> Option<(CheckDecision, bool)> {
        let captured = self.compiled.matcher.matches(request)?;
        let key = key_builder.build_key(&self.compiled.definition.id, &captured);
        let bucket = self
            .buckets
            .entry(key)
            .or_insert_with(|| TokenBucket::new(self.compiled.definition.burst, now_ms));
        let (allowed, retry_after_ms) = bucket.consume(
            self.compiled.definition.limit_per_second,
            self.compiled.definition.burst,
            now_ms,
        );
        let decision = CheckDecision {
            allowed,
            retry_after_ms,
        };
        let enforce = matches!(
            self.compiled.definition.action,
            None | Some(PolicyAction::Reject)
        );
        Some((decision, enforce))
    }
}
