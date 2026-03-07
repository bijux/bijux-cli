//! Runtime cache models and helpers.
pub(crate) mod key;
pub(crate) mod proof;
pub(crate) mod store;
pub(crate) mod lineage;

pub use crate::CacheMode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyInput {
    pub node_fingerprint: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub output_schema_version: String,
    pub policy_fingerprint: String,
    pub config_fingerprint: String,
    pub backend_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyExplanation {
    pub key: String,
    pub intentional_inputs: Vec<(String, String)>,
    pub accidental_inputs: Vec<String>,
}

pub fn cache_key_explanation(input: &CacheKeyInput) -> CacheKeyExplanation {
    let intentional_inputs = vec![
        ("node_fingerprint".to_string(), input.node_fingerprint.clone()),
        ("adapter_id".to_string(), input.adapter_id.clone()),
        ("adapter_version".to_string(), input.adapter_version.clone()),
        (
            "output_schema_version".to_string(),
            input.output_schema_version.clone(),
        ),
        ("policy_fingerprint".to_string(), input.policy_fingerprint.clone()),
        ("config_fingerprint".to_string(), input.config_fingerprint.clone()),
        ("backend_class".to_string(), input.backend_class.clone()),
    ];
    let mut hasher = Sha256::new();
    for (k, v) in &intentional_inputs {
        hasher.update(k.as_bytes());
        hasher.update(b"=");
        hasher.update(v.as_bytes());
        hasher.update(b";");
    }
    let key = hex::encode(hasher.finalize());
    CacheKeyExplanation {
        key,
        intentional_inputs,
        accidental_inputs: Vec::new(),
    }
}

pub fn cache_entry_has_required_proof(meta: &serde_json::Value) -> bool {
    meta.get("node_fingerprint").and_then(|v| v.as_str()).is_some()
        && meta.get("adapter_id").and_then(|v| v.as_str()).is_some()
        && meta.get("adapter_version").and_then(|v| v.as_str()).is_some()
}

pub fn cache_metadata_version_supported(meta: &serde_json::Value) -> bool {
    meta.get("cache_metadata_version")
        .and_then(|v| v.as_str())
        .map(|v| v == "cache-meta/v0.1")
        .unwrap_or(false)
}
