use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use siphasher::sip::SipHasher13;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyBuilder {
    k0: u64,
    k1: u64,
}

impl KeyBuilder {
    pub fn new(secret: Option<&str>) -> Self {
        let seed = secret.unwrap_or("fluxgate::default-secret");
        let mut hasher_a = DefaultHasher::new();
        seed.hash(&mut hasher_a);
        let k0 = hasher_a.finish();
        let mut hasher_b = DefaultHasher::new();
        format!("{seed}::secondary").hash(&mut hasher_b);
        let k1 = hasher_b.finish();
        Self { k0, k1 }
    }

    pub fn build_key(&self, policy_id: &str, captured: &IndexMap<String, String>) -> u64 {
        let mut hasher = SipHasher13::new_with_keys(self.k0, self.k1);
        policy_id.hash(&mut hasher);
        for (name, value) in captured.iter() {
            name.hash(&mut hasher);
            value.hash(&mut hasher);
        }
        hasher.finish()
    }
}
