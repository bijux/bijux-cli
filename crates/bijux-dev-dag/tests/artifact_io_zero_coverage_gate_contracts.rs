use bijux_dag_testkit as _;
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
fn artifact_io_storage_files_are_not_allowlisted_for_zero_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let allowlist = root.join("configs/policy/protected_zero_coverage_allowlist.json");
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
        "crates/bijux-dag-artifacts/src/io/fs.rs",
        "crates/bijux-dag-artifacts/src/storage/services.rs",
    ] {
        assert!(
            !entries.contains(&forbidden),
            "artifact io/storage file must not be allowlisted at 0%: {forbidden}"
        );
    }

    // llvm-cov currently reports these path-routed modules as 0% despite direct execution.
    for allowed_exception in [
        "crates/bijux-dag-artifacts/src/io/store.rs",
        "crates/bijux-dag-artifacts/src/storage/hardening.rs",
    ] {
        assert!(
            entries.contains(&allowed_exception),
            "artifact coverage exception must be explicitly allowlisted: {allowed_exception}"
        );
    }
}
