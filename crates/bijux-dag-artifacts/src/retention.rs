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
