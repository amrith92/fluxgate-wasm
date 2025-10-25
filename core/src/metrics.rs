use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metrics {
    checks_total: u64,
    allowed_total: u64,
    denied_total: u64,
}

impl Metrics {
    pub fn record(&mut self, allowed: bool) {
        self.checks_total += 1;
        if allowed {
            self.allowed_total += 1;
        } else {
            self.denied_total += 1;
        }
    }

    pub fn as_map(&self) -> IndexMap<String, u64> {
        let mut map = IndexMap::new();
        map.insert("checks_total".to_string(), self.checks_total);
        map.insert("allowed_total".to_string(), self.allowed_total);
        map.insert("denied_total".to_string(), self.denied_total);
        map
    }
}
