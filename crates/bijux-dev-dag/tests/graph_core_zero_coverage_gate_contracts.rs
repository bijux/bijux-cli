use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[test]
fn graph_pipeline_core_files_are_not_allowlisted_for_zero_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let allowlist = root.join("configs/policy/protected_zero_coverage_allowlist.json");
    let payload: Value = serde_json::from_str(&fs::read_to_string(allowlist).expect("read allowlist"))
        .expect("parse allowlist");
    let entries = payload["protected_zero_coverage_allowlist"]
        .as_array()
        .expect("allowlist array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    for forbidden in [
        "crates/bijux-dag-core/src/graph/canonical.rs",
        "crates/bijux-dag-core/src/graph/edge.rs",
        "crates/bijux-dag-core/src/graph/topology.rs",
        "crates/bijux-dag-core/src/pipeline/validate.rs",
        "crates/bijux-dag-core/src/pipeline/resolve.rs",
    ] {
        assert!(
            !entries.contains(&forbidden),
            "graph/pipeline core file must not be allowlisted at 0%: {forbidden}"
        );
    }
}
