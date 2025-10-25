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
        if limit_per_second == 0 {
            self.tokens = 0.0;
            self.last_ms = now_ms;
            return (false, None);
        }

        let rate = limit_per_second as f64;
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

    #[cfg(test)]
    pub fn remaining_tokens(&self) -> f64 {
        self.tokens
    }
}

#[cfg(test)]
mod tests {
    use super::TokenBucket;

    #[test]
    fn zero_rate_always_denies() {
        let mut bucket = TokenBucket::new(5, 0);

        let (allowed, retry_after) = bucket.consume(0, 5, 0);
        assert!(!allowed);
        assert_eq!(retry_after, None);
        assert_eq!(bucket.remaining_tokens(), 0.0);

        // Even after time has passed, the bucket should not refill.
        let (allowed, retry_after) = bucket.consume(0, 5, 5_000);
        assert!(!allowed);
        assert_eq!(retry_after, None);
        assert_eq!(bucket.remaining_tokens(), 0.0);
    }
}
