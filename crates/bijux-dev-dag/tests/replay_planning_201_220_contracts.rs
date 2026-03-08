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
fn replay_201_220_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/replay_planning_201_220_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "201-206 replay planning path and stability checks",
        "207-208 replay drift fixtures and determinism checks",
        "209-215 explain, schema, failure, and corruption behavior",
        "216-217 complexity and determinism reports",
        "218 replay planning invariants verification suite",
        "219 replay plan consistency dashboard",
        "220 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn replay_201_220_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/replay_planning_201_220_status_report.md",
        "docs/reports/foundation/replay_planning_complexity_report.md",
        "docs/reports/foundation/replay_planning_determinism_report.md",
        "docs/reports/foundation/replay_plan_consistency_dashboard.md",
        "docs/adr/20260308-replay-planning-guarantees.md",
        "configs/suites/replay_planning_invariants.json",
        "crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs",
        "crates/bijux-dag-core/tests/planner_fixture_contracts.rs",
        "crates/bijux-dag-core/tests/planner_validation_remaining_contracts.rs",
        "crates/bijux-dag-core/tests/planner_error_and_schema_contracts.rs",
        "crates/bijux-dag-runtime/tests/replay_determinism_fuzz_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_hardening_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_fidelity_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_mismatch_corpus_contracts.rs",
        "configs/schema/execution_plan.schema.json",
        "configs/schema/planner_explain.schema.json",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
