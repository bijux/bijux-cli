use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn backend_adapter_registry_reports_and_adr_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/runtime_adapter_registry_coverage_dashboard.md",
        "docs/reports/foundation/backend_capability_drift_release_report.md",
        "docs/reports/foundation/backend_claims_without_direct_tests_report.md",
        "docs/reports/foundation/backend_adapter_registry_completion_report.md",
        "docs/adr/20260308-runtime-adapter-registry-end-state.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing backend adapter registry artifact: {rel}"
        );
    }

    let completion = fs::read_to_string(
        root.join("docs/reports/foundation/backend_adapter_registry_completion_report.md"),
    )
    .expect("read completion");
    for required in [
        "521-540",
        "adapter_registry_capability_contracts.rs",
        "backend_adapter_registry_fast.json",
        "shipped_adapters_registry_direct_tests_contracts.rs",
    ] {
        assert!(
            completion.contains(required),
            "backend adapter completion report missing {required}"
        );
    }
}
