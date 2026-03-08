use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::path::Path;
use tempfile as _;

#[test]
fn shipped_adapters_have_direct_registry_tests_and_dump_entries() {
    let dump = bijux_dag_runtime::adapter_registry_dump();
    let adapters = dump["adapters"].as_array().expect("adapters");
    assert!(!adapters.is_empty(), "shipped adapter registry must not be empty");
    for adapter in adapters {
        assert!(adapter["adapter_id"].as_str().is_some_and(|v| !v.is_empty()));
        assert!(adapter["adapter_version"].as_str().is_some_and(|v| !v.is_empty()));
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tests = std::fs::read_to_string(
        root.join("crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs"),
    )
    .expect("adapter registry tests");

    for required in [
        "runtime_registry_query_output_is_stable",
        "adapter_metadata_is_present_in_registry_output_surface",
        "adapter_registry_rejects_duplicate_identities_by_reported_list",
        "incomplete_capability_declaration_is_rejected_by_conformance",
        "adapter_registry_dump_has_stable_identity_without_preference_override_surface",
    ] {
        assert!(
            tests.contains(required),
            "missing required direct registry test anchor {required}"
        );
    }

    let coverage_matrix: Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join("docs/reports/foundation/adapter_conformance_coverage_matrix.json"),
        )
        .expect("matrix"),
    )
    .expect("parse matrix");
    assert_eq!(coverage_matrix["format"], "adapter-conformance-coverage/v1");
}
