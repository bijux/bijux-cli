use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

#[test]
fn backend_conformance_fast_suite_covers_local_and_modeled_capability_surfaces() {
    let suite = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/suites/backend_conformance_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(suite).expect("suite")).expect("json");
    assert_eq!(payload["id"], "backend-conformance-fast");

    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "capability_query_output_is_stable_for_local",
        "capability_query_output_is_stable_for_kubernetes",
        "capability_query_output_is_stable_for_hpc",
        "capability_query_output_is_stable_for_remote",
        "adapter_registry_capability_contracts",
        "backend_capability_docs_generation_contracts",
    ] {
        assert!(
            commands.contains(required),
            "backend conformance fast suite missing {required}"
        );
    }
}
