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
fn evidence_141_160_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/evidence_rationalization_141_160_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "141-145 evidence command and report classification",
        "146-148 duplicate-output rationalization",
        "149 machine-stability tests for release-critical outputs",
        "150 advisory isolation from blockers",
        "153-154 docs-to-evidence and evidence-to-suite mappings",
        "157-158 compact evidence packs",
        "159 low-decision-value evidence output report",
        "160 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn evidence_141_160_governance_artifacts_exist() {
    for rel in [
        "configs/policy/evidence_rationalization_policy.json",
        "docs/reports/foundation/release_critical_evidence_commands_only_report.md",
        "docs/reports/foundation/release_supporting_evidence_commands_report.md",
        "docs/reports/foundation/advisory_only_evidence_commands_report.md",
        "docs/reports/foundation/evidence_outputs_duplicate_signal_report.md",
        "docs/reports/foundation/evidence_docs_mapping_report.md",
        "docs/reports/foundation/evidence_suite_exercise_mapping_report.md",
        "docs/reports/foundation/evidence_commands_not_exercised_in_ci_report.md",
        "docs/reports/foundation/compact_release_evidence_pack.md",
        "docs/reports/foundation/compact_advisory_evidence_pack.md",
        "docs/reports/foundation/compact_release_evidence_pack.json",
        "docs/reports/foundation/compact_advisory_evidence_pack.json",
        "docs/reports/foundation/top_25_evidence_outputs_low_decision_value_report.md",
        "docs/adr/20260308-evidence-severity-rationalization.md",
        "crates/bijux-dev-dag/tests/evidence_rationalization_contracts.rs",
        "crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
