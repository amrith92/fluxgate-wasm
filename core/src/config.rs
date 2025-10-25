use crate::error::{FluxgateError, Result};
use crate::policy::PolicyMatcher;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FluxgateInit {
    #[serde(default)]
    pub policies: Option<Vec<FluxgatePolicy>>,
    #[serde(default)]
    pub config_text: Option<String>,
    #[serde(default)]
    pub key_secret: Option<String>,
    #[serde(default)]
    pub slices: Option<u32>,
    #[serde(default)]
    pub sketch_width: Option<u32>,
    #[serde(default)]
    pub sketch_depth: Option<u32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub shard_a_hot_capacity: Option<u32>,
    #[serde(default)]
    pub admission_hits_to_promote: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FluxgatePolicy {
    pub id: String,
    #[serde(rename = "match")]
    pub match_rule: String,
    pub limit_per_second: u32,
    pub burst: u32,
    pub window_seconds: u32,
    #[serde(default)]
    pub action: Option<PolicyAction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckRequest {
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub route: Option<String>,
    #[serde(default)]
    pub headers: Option<IndexMap<String, Option<String>>>,
    #[serde(default)]
    pub attrs: Option<IndexMap<String, serde_json::Value>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckDecision {
    pub allowed: bool,
    #[serde(default)]
    pub retry_after_ms: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub allowed: bool,
    #[serde(default)]
    pub retry_after_ms: Option<u32>,
    #[serde(default)]
    pub decisions: IndexMap<String, CheckDecision>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FluxgateConfig {
    pub policies: Vec<CompiledPolicy>,
    #[serde(default)]
    pub key_secret: Option<String>,
    #[serde(default)]
    pub slices: Option<u32>,
    #[serde(default)]
    pub sketch_width: Option<u32>,
    #[serde(default)]
    pub sketch_depth: Option<u32>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub shard_a_hot_capacity: Option<u32>,
    #[serde(default)]
    pub admission_hits_to_promote: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompiledPolicy {
    pub definition: FluxgatePolicy,
    pub matcher: PolicyMatcher,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PolicyAction {
    #[serde(alias = "reject")]
    Reject,
    #[serde(alias = "annotate")]
    Annotate,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentPolicies {
    pub policies: Vec<FluxgatePolicy>,
}

impl FluxgateInit {
    pub fn into_config(self) -> Result<FluxgateConfig> {
        let mut policies = self.policies.unwrap_or_default();

        if let Some(text) = self.config_text {
            if !text.trim().is_empty() {
                #[cfg(feature = "yaml")]
                {
                    let doc: DocumentPolicies = serde_yaml::from_str(&text).map_err(|err| {
                        FluxgateError::InvalidConfig(format!("yaml parse error: {err}"))
                    })?;
                    policies.extend(doc.policies);
                }

                #[cfg(not(feature = "yaml"))]
                {
                    return Err(FluxgateError::InvalidConfig(
                        "configText provided but YAML support is disabled".to_string(),
                    ));
                }
            }
        }

        if policies.is_empty() {
            return Err(FluxgateError::InvalidConfig(
                "at least one policy must be provided".to_string(),
            ));
        }

        let compiled = policies
            .into_iter()
            .map(|policy| {
                let matcher = PolicyMatcher::from_rule(&policy.match_rule).map_err(|err| {
                    FluxgateError::InvalidConfig(format!(
                        "policy {} match parse error: {err}",
                        policy.id
                    ))
                })?;
                Ok(CompiledPolicy {
                    definition: policy,
                    matcher,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(FluxgateConfig {
            policies: compiled,
            key_secret: self.key_secret,
            slices: self.slices,
            sketch_width: self.sketch_width,
            sketch_depth: self.sketch_depth,
            top_k: self.top_k,
            shard_a_hot_capacity: self.shard_a_hot_capacity,
            admission_hits_to_promote: self.admission_hits_to_promote,
        })
    }
}

impl CheckResult {
    pub fn denied(retry_after_ms: Option<u32>, decisions: IndexMap<String, CheckDecision>) -> Self {
        Self {
            allowed: false,
            retry_after_ms,
            decisions,
        }
    }
}
