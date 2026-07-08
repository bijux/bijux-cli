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
    cache_entry_has_required_proof, cache_key_explanation, cache_metadata_version_supported,
    CacheKeyInput,
};

#[test]
fn cache_key_is_stable_for_cosmetic_omissions_and_changes_for_meaningful_inputs() {
    let base = CacheKeyInput {
        execution_fingerprint: "exec-fp-1".to_string(),
        node_definition_fingerprint: "node-def-fp-1".to_string(),
        declared_environment_fingerprint: "env-fp-1".to_string(),
        input_lineage_fingerprint: "inputs-fp-1".to_string(),
        adapter_id: "shell".to_string(),
        adapter_version: "1.0.0".to_string(),
        output_schema_version: "schema/v1".to_string(),
        policy_fingerprint: "policy-a".to_string(),
        execution_contract_fingerprint: "exec-contract-a".to_string(),
        backend_class: "local".to_string(),
    };
    let e1 = cache_key_explanation(&base);
    let e2 = cache_key_explanation(&base);
    assert_eq!(e1.key, e2.key);

    let mut changed = base.clone();
    changed.adapter_version = "1.1.0".to_string();
    assert_ne!(e1.key, cache_key_explanation(&changed).key);

    changed = base.clone();
    changed.output_schema_version = "schema/v2".to_string();
    assert_ne!(e1.key, cache_key_explanation(&changed).key);

    changed = base.clone();
    changed.policy_fingerprint = "policy-b".to_string();
    assert_ne!(e1.key, cache_key_explanation(&changed).key);

    changed = base.clone();
    changed.execution_contract_fingerprint = "exec-contract-b".to_string();
    assert_ne!(e1.key, cache_key_explanation(&changed).key);
}

#[test]
fn cache_proof_metadata_and_version_checks_reject_stale_or_missing() {
    let ok = serde_json::json!({
        "cache_key": "cache-key-1",
        "node_fingerprint": "exec-fp-1",
        "node_definition_fingerprint": "node-def-fp-1",
        "declared_environment_fingerprint": "env-fp-1",
        "input_lineage_fingerprint": "inputs-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1",
        "policy_fingerprint": "policy-1",
        "execution_contract_fingerprint": "exec-contract-1",
        "backend_class": "local",
        "produces_outputs_schema_version": "schema/v1",
        "cache_metadata_version": "cache-meta/v0.2"
    });
    assert!(cache_entry_has_required_proof(&ok));
    assert!(cache_metadata_version_supported(&ok));

    let missing_proof = serde_json::json!({"cache_metadata_version": "cache-meta/v0.2"});
    assert!(!cache_entry_has_required_proof(&missing_proof));

    let stale = serde_json::json!({
        "cache_key": "cache-key-1",
        "node_fingerprint": "exec-fp-1",
        "node_definition_fingerprint": "node-def-fp-1",
        "declared_environment_fingerprint": "env-fp-1",
        "input_lineage_fingerprint": "inputs-fp-1",
        "adapter_id": "shell",
        "adapter_version": "1",
        "policy_fingerprint": "policy-1",
        "execution_contract_fingerprint": "exec-contract-1",
        "backend_class": "local",
        "produces_outputs_schema_version": "schema/v1",
        "cache_metadata_version": "cache-meta/v9.9"
    });
    assert!(!cache_metadata_version_supported(&stale));
}
