use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    cache_entry_has_required_proof, cache_key_explanation, cache_metadata_version_supported, CacheKeyInput,
};
use serde_json::json;

fn sample_input() -> CacheKeyInput {
    CacheKeyInput {
        node_fingerprint: "node-fp-1".to_string(),
        adapter_id: "shell".to_string(),
        adapter_version: "1.0.0".to_string(),
        output_schema_version: "out/v1".to_string(),
        policy_fingerprint: "policy-fp-1".to_string(),
        config_fingerprint: "config-fp-1".to_string(),
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
    input.node_fingerprint = "node-fp-2".to_string();
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
fn cache_key_changes_on_policy_or_config_change() {
    let mut input = sample_input();
    let key_a = cache_key_explanation(&input).key;
    input.policy_fingerprint = "policy-fp-2".to_string();
    let key_b = cache_key_explanation(&input).key;
    assert_ne!(key_a, key_b);

    let mut input_two = sample_input();
    let key_c = cache_key_explanation(&input_two).key;
    input_two.config_fingerprint = "config-fp-2".to_string();
    let key_d = cache_key_explanation(&input_two).key;
    assert_ne!(key_c, key_d);
}

#[test]
fn cache_proof_requires_explicit_metadata_fields() {
    let valid = json!({
        "node_fingerprint": "node-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "cache_metadata_version": "cache-meta/v0.1",
    });
    assert!(cache_entry_has_required_proof(&valid));
    assert!(cache_metadata_version_supported(&valid));

    let missing_proof = json!({
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "cache_metadata_version": "cache-meta/v0.1",
    });
    assert!(!cache_entry_has_required_proof(&missing_proof));

    let stale_version = json!({
        "node_fingerprint": "node-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "cache_metadata_version": "cache-meta/v0.0",
    });
    assert!(!cache_metadata_version_supported(&stale_version));
}
