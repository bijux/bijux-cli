use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromotionEnvironment {
    Local,
    Staging,
    Release,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactPromotionRecord {
    pub artifact_id: String,
    pub from: PromotionEnvironment,
    pub to: PromotionEnvironment,
    pub promoted_unix_ms: u128,
}
