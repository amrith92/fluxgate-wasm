use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenBucket {
    tokens: f64,
    last_ms: u64,
}

impl TokenBucket {
    pub fn new(burst: u32, now_ms: u64) -> Self {
        Self {
            tokens: burst as f64,
            last_ms: now_ms,
        }
    }

    pub fn consume(
        &mut self,
        limit_per_second: u32,
        burst: u32,
        now_ms: u64,
    ) -> (bool, Option<u32>) {
        let rate = limit_per_second.max(1) as f64;
        let elapsed_ms = now_ms.saturating_sub(self.last_ms) as f64;
        let refill = (elapsed_ms / 1000.0) * rate;
        self.tokens = (self.tokens + refill).min(burst as f64);
        self.last_ms = now_ms;

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            return (true, None);
        }

        let missing = 1.0 - self.tokens;
        let wait_ms = ((missing / rate) * 1000.0).ceil();
        (false, Some(wait_ms.max(0.0) as u32))
    }

    pub fn remaining_tokens(&self) -> f64 {
        self.tokens
    }
}
