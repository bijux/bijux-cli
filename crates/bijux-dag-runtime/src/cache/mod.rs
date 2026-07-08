//! Runtime cache models and helpers.
#![allow(unused_imports)]

pub(crate) mod key;
pub(crate) mod lineage;
pub(crate) mod proof;
pub(crate) mod store;

pub use crate::CacheMode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CACHE_METADATA_VERSION: &str = "cache-meta/v0.4";
pub const CACHE_METADATA_VERSION_PREVIOUS: &str = "cache-meta/v0.3";
pub const CACHE_METADATA_VERSION_LEGACY: &str = "cache-meta/v0.2";
pub const CACHE_ENTRY_MANIFEST_VERSION: &str = "cache-entry/v0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyInput {
    pub execution_fingerprint: String,
    pub node_definition_fingerprint: String,
    pub declared_environment_fingerprint: String,
    pub input_lineage_fingerprint: String,
    pub adapter_id: String,
    pub adapter_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_binary_sha256: Option<String>,
    pub output_schema_version: String,
    pub policy_fingerprint: String,
    pub execution_contract_fingerprint: String,
    pub backend_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKeyExplanation {
    pub key: String,
    pub intentional_inputs: Vec<(String, String)>,
    pub accidental_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheManifestOutput {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub media_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntryManifest {
    pub manifest_version: String,
    pub cache_key: String,
    pub node_id: String,
    pub outputs: Vec<CacheManifestOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheExplainabilityProof {
    pub params_fingerprint: String,
    pub command_fingerprint: Option<String>,
}

pub fn cache_key_explanation(input: &CacheKeyInput) -> CacheKeyExplanation {
    let intentional_inputs = vec![
        ("node_definition_fingerprint".to_string(), input.node_definition_fingerprint.clone()),
        (
            "declared_environment_fingerprint".to_string(),
            input.declared_environment_fingerprint.clone(),
        ),
        ("input_lineage_fingerprint".to_string(), input.input_lineage_fingerprint.clone()),
        ("adapter_id".to_string(), input.adapter_id.clone()),
        ("adapter_version".to_string(), input.adapter_version.clone()),
        (
            "adapter_binary_sha256".to_string(),
            input.adapter_binary_sha256.clone().unwrap_or_default(),
        ),
        ("output_schema_version".to_string(), input.output_schema_version.clone()),
        ("policy_fingerprint".to_string(), input.policy_fingerprint.clone()),
        (
            "execution_contract_fingerprint".to_string(),
            input.execution_contract_fingerprint.clone(),
        ),
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
    CacheKeyExplanation { key, intentional_inputs, accidental_inputs: Vec::new() }
}

pub fn cache_key_input_from_meta(meta: &serde_json::Value) -> Option<CacheKeyInput> {
    Some(CacheKeyInput {
        execution_fingerprint: meta.get("node_fingerprint").and_then(|v| v.as_str())?.to_string(),
        node_definition_fingerprint: meta
            .get("node_definition_fingerprint")
            .and_then(|v| v.as_str())?
            .to_string(),
        declared_environment_fingerprint: meta
            .get("declared_environment_fingerprint")
            .and_then(|v| v.as_str())?
            .to_string(),
        input_lineage_fingerprint: meta
            .get("input_lineage_fingerprint")
            .and_then(|v| v.as_str())?
            .to_string(),
        adapter_id: meta.get("adapter_id").and_then(|v| v.as_str())?.to_string(),
        adapter_version: meta.get("adapter_version").and_then(|v| v.as_str())?.to_string(),
        adapter_binary_sha256: meta
            .get("adapter_binary_sha256")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        output_schema_version: meta
            .get("produces_outputs_schema_version")
            .or_else(|| meta.get("output_schema_version"))
            .and_then(|v| v.as_str())?
            .to_string(),
        policy_fingerprint: meta.get("policy_fingerprint").and_then(|v| v.as_str())?.to_string(),
        execution_contract_fingerprint: meta
            .get("execution_contract_fingerprint")
            .and_then(|v| v.as_str())?
            .to_string(),
        backend_class: meta.get("backend_class").and_then(|v| v.as_str())?.to_string(),
    })
}

pub fn cache_entry_has_required_proof(meta: &serde_json::Value) -> bool {
    meta.get("cache_key").and_then(|v| v.as_str()).is_some()
        && cache_key_input_from_meta(meta).is_some()
}

pub fn cache_explainability_proof_from_meta(
    meta: &serde_json::Value,
) -> Option<CacheExplainabilityProof> {
    Some(CacheExplainabilityProof {
        params_fingerprint: meta.get("params_fingerprint").and_then(|v| v.as_str())?.to_string(),
        command_fingerprint: meta
            .get("command_fingerprint")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
    })
}

pub fn cache_metadata_version_supported(meta: &serde_json::Value) -> bool {
    meta.get("cache_metadata_version")
        .and_then(|v| v.as_str())
        .map(|version| {
            version == CACHE_METADATA_VERSION
                || version == CACHE_METADATA_VERSION_PREVIOUS
                || version == CACHE_METADATA_VERSION_LEGACY
        })
        .unwrap_or(false)
}

pub fn cache_entry_manifest_version_supported(manifest: &CacheEntryManifest) -> bool {
    manifest.manifest_version == CACHE_ENTRY_MANIFEST_VERSION
}
