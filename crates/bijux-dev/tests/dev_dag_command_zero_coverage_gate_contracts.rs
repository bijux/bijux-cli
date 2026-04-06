use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[test]
fn dev_dag_command_files_are_not_allowlisted_for_zero_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let allowlist = root.join("configs/dag/policy/protected_zero_coverage_allowlist.json");
    let payload: Value =
        serde_json::from_str(&fs::read_to_string(allowlist).expect("read allowlist"))
            .expect("parse allowlist");
    let entries = payload["protected_zero_coverage_allowlist"]
        .as_array()
        .expect("allowlist array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    for forbidden in [
        "crates/bijux-dev/src/commands/authoring_evidence.rs",
        "crates/bijux-dev/src/commands/battle_evidence.rs",
        "crates/bijux-dev/src/commands/benchmark_harness.rs",
        "crates/bijux-dev/src/commands/compare_evidence.rs",
        "crates/bijux-dev/src/commands/evidence_access.rs",
        "crates/bijux-dev/src/commands/evidence_control_plane.rs",
        "crates/bijux-dev/src/commands/evidence_registry.rs",
        "crates/bijux-dev/src/commands/model.rs",
        "crates/bijux-dev/src/commands/perf_evidence.rs",
        "crates/bijux-dev/src/commands/suite_catalog.rs",
    ] {
        assert!(
            !entries.contains(&forbidden),
            "dev-dag command file must not be allowlisted at 0%: {forbidden}"
        );
    }
}
