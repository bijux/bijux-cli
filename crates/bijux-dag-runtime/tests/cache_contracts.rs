use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    cache_entry_has_required_proof, cache_entry_manifest_version_supported, cache_key_explanation,
    cache_metadata_version_supported, CacheEntryManifest, CacheKeyInput, CacheManifestOutput,
};
use serde_json::json;

fn sample_input() -> CacheKeyInput {
    CacheKeyInput {
        execution_fingerprint: "exec-fp-1".to_string(),
        node_definition_fingerprint: "node-def-fp-1".to_string(),
        declared_environment_fingerprint: "env-fp-1".to_string(),
        input_lineage_fingerprint: "inputs-fp-1".to_string(),
        adapter_id: "shell".to_string(),
        adapter_version: "1.0.0".to_string(),
        adapter_binary_sha256: None,
        output_schema_version: "out/v1".to_string(),
        policy_fingerprint: "policy-fp-1".to_string(),
        execution_contract_fingerprint: "exec-contract-fp-1".to_string(),
        backend_class: "local-shell".to_string(),
    }
}

#[test]
fn cache_key_is_stable_for_same_semantics() {
    let a = sample_input();
    let b = sample_input();
    let key_a = cache_key_explanation(&a).key;
    let key_b = cache_key_explanation(&b).key;
    assert_eq!(key_a, key_b);
}

#[test]
fn cache_key_changes_when_planner_meaning_changes() {
    let mut input = sample_input();
    let key_a = cache_key_explanation(&input).key;
    input.node_definition_fingerprint = "node-def-fp-2".to_string();
    let key_b = cache_key_explanation(&input).key;
    assert_ne!(key_a, key_b);
}

#[test]
fn cache_key_changes_on_backend_capability_change() {
    let mut input = sample_input();
    let key_a = cache_key_explanation(&input).key;
    input.backend_class = "remote-container".to_string();
    let key_b = cache_key_explanation(&input).key;
    assert_ne!(key_a, key_b);
}

#[test]
fn cache_key_changes_when_external_adapter_binary_identity_changes() {
    let mut input = sample_input();
    input.adapter_binary_sha256 = Some("sha256-a".to_string());
    let key_a = cache_key_explanation(&input).key;
    input.adapter_binary_sha256 = Some("sha256-b".to_string());
    let key_b = cache_key_explanation(&input).key;
    assert_ne!(key_a, key_b);
}

#[test]
fn cache_key_changes_on_policy_or_config_change() {
    let mut input = sample_input();
    let key_a = cache_key_explanation(&input).key;
    input.policy_fingerprint = "policy-fp-2".to_string();
    let key_b = cache_key_explanation(&input).key;
    assert_ne!(key_a, key_b);

    let mut input_two = sample_input();
    let key_c = cache_key_explanation(&input_two).key;
    input_two.execution_contract_fingerprint = "exec-contract-fp-2".to_string();
    let key_d = cache_key_explanation(&input_two).key;
    assert_ne!(key_c, key_d);
}

#[test]
fn cache_proof_requires_explicit_metadata_fields() {
    let valid = json!({
        "cache_key": "cache-key-1",
        "node_fingerprint": "exec-fp-1",
        "node_definition_fingerprint": "node-def-fp-1",
        "declared_environment_fingerprint": "env-fp-1",
        "input_lineage_fingerprint": "inputs-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "adapter_binary_sha256": null,
        "policy_fingerprint": "policy-fp-1",
        "execution_contract_fingerprint": "exec-contract-fp-1",
        "backend_class": "local-shell",
        "cache_metadata_version": "cache-meta/v0.4",
        "produces_outputs_schema_version": "out/v1",
    });
    assert!(cache_entry_has_required_proof(&valid));
    assert!(cache_metadata_version_supported(&valid));

    let missing_proof = json!({
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "cache_metadata_version": "cache-meta/v0.4",
    });
    assert!(!cache_entry_has_required_proof(&missing_proof));

    let stale_version = json!({
        "cache_key": "cache-key-1",
        "node_fingerprint": "exec-fp-1",
        "node_definition_fingerprint": "node-def-fp-1",
        "declared_environment_fingerprint": "env-fp-1",
        "input_lineage_fingerprint": "inputs-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "adapter_binary_sha256": null,
        "policy_fingerprint": "policy-fp-1",
        "execution_contract_fingerprint": "exec-contract-fp-1",
        "backend_class": "local-shell",
        "produces_outputs_schema_version": "out/v1",
        "cache_metadata_version": "cache-meta/v0.0",
    });
    assert!(!cache_metadata_version_supported(&stale_version));
}

#[test]
fn cache_entry_manifest_requires_supported_version_and_output_contracts() {
    let manifest = CacheEntryManifest {
        manifest_version: "cache-entry/v0.1".to_string(),
        cache_key: "cache-key-1".to_string(),
        node_id: "node-a".to_string(),
        outputs: vec![
            CacheManifestOutput {
                name: "report".to_string(),
                path: "report.txt".to_string(),
                kind: "file".to_string(),
                media_type: "text/plain".to_string(),
                required: true,
            },
            CacheManifestOutput {
                name: "debug-log".to_string(),
                path: "debug.log".to_string(),
                kind: "log".to_string(),
                media_type: "text/plain".to_string(),
                required: false,
            },
        ],
    };
    assert!(cache_entry_manifest_version_supported(&manifest));

    let stale =
        CacheEntryManifest { manifest_version: "cache-entry/v9.9".to_string(), ..manifest.clone() };
    assert!(!cache_entry_manifest_version_supported(&stale));
}
