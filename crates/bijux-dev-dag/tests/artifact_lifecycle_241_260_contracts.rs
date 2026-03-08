use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn artifact_241_260_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/artifact_lifecycle_241_260_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "241-248 write integrity, interruption, metadata, and checksum behavior",
        "249-256 orphan, GC, rebuild, and lineage integrity behavior",
        "257-258 integrity and lifecycle invariants reports",
        "259 artifact lifecycle verification suite",
        "260 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn artifact_241_260_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/artifact_lifecycle_241_260_status_report.md",
        "docs/reports/foundation/artifact_store_integrity_report.md",
        "docs/reports/foundation/artifact_lifecycle_invariants_report.md",
        "docs/reports/foundation/artifact_lifecycle_dashboard.md",
        "docs/adr/20260308-artifact-lifecycle-guarantees.md",
        "configs/suites/artifact_lifecycle_invariants.json",
        "configs/suites/artifact_storage_lifecycle_stress.json",
        "configs/suites/artifact_durability_verification.json",
        "docs/reports/foundation/artifact_gc_dry_run_explain.md",
        "docs/reports/foundation/artifact_lineage_anomaly_report.md",
        "crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs",
        "crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs",
        "crates/bijux-dag-artifacts/tests/artifact_identity_and_lineage_contracts.rs",
        "crates/bijux-dag-artifacts/tests/artifact_identity_lifecycle_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/artifact_storage_lifecycle_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/artifact_durability_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/artifact_lineage_completion_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
