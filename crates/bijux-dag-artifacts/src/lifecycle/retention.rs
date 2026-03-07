use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub local_cache_ttl_days: u32,
    pub run_artifacts_ttl_days: u32,
    pub promoted_artifacts_ttl_days: u32,
    pub exported_bundles_ttl_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            local_cache_ttl_days: 7,
            run_artifacts_ttl_days: 30,
            promoted_artifacts_ttl_days: 365,
            exported_bundles_ttl_days: 180,
        }
    }
}

impl RetentionPolicy {
    pub fn retain_prefixes(&self) -> Vec<&'static str> {
        let _ = self;
        vec!["run-", "promoted-", "export-", "cache-"]
    }

    pub fn should_prune_run_days(&self, age_days: u32) -> bool {
        age_days > self.run_artifacts_ttl_days
    }

    pub fn should_prune_cache_days(&self, age_days: u32) -> bool {
        age_days > self.local_cache_ttl_days
    }
}
