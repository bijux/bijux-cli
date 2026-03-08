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
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("read file")
}

#[test]
fn system_invariants_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_FORMAL_INVARIANTS_CONTRACT.md",
        "docs/reports/foundation/system_invariants_coverage_report.md",
        "docs/reports/foundation/system_invariants_failure_logging_report.md",
        "docs/reports/foundation/system_invariants_drift_detection_report.md",
        "docs/reports/foundation/system_invariants_tooling_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty system invariants artifact: {rel}"
        );
    }
}

#[test]
fn system_invariants_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_invariants/regression_corpus.json",
    ))
    .expect("parse system invariants corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 12, "expected broad system invariants corpus");

    for coverage in [
        "core-execution-invariants",
        "artifact-lineage-invariants",
        "replay-equivalence-invariants",
        "diff-semantic-invariants",
        "scheduler-fairness-invariants",
        "run-identity-invariants",
        "artifact-identity-invariants",
        "backend-equivalence-invariants",
        "determinism-invariants",
        "successful-runs",
        "failed-runs",
        "partial-runs",
        "replay-operations",
        "import-export-flows",
        "failure-logging",
        "invariant-verification-tooling",
        "invariant-regression-fixtures",
        "invariant-drift-detection",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing system invariants coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/system_invariants_verification.json"))
            .expect("parse system invariants suite");
    assert_eq!(suite["id"], "system-invariants-verification");
}

#[test]
fn system_invariants_surfaces_anchor_existing_invariant_runtime_and_app_flows() {
    let formal = read("docs/spec/FORMAL_INVARIANTS.md");
    for token in [
        "INV-GRAPH-SHAPE-001",
        "INV-RUN-COUNTS-001",
        "INV-TRACE-TIME-001",
        "INV-REPLAY-EQUIV-001",
    ] {
        assert!(
            formal.contains(token),
            "missing formal invariant token: {token}"
        );
    }

    let cli = read("crates/bijux-dev-dag/src/commands/cli.rs");
    assert!(
        cli.contains("InvariantsReport"),
        "missing invariants report CLI surface"
    );

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "invariants-report",
        "run_invariants_report",
        "run_evidence_replay_verify",
    ] {
        assert!(
            commands.contains(token),
            "missing invariant command anchor token: {token}"
        );
    }

    let replay_lineage = read("crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs");
    assert!(
        replay_lineage.contains("replay_manifest_keeps_parent_run_ancestry_chain"),
        "missing replay invariant anchor"
    );

    let import_export = read("crates/bijux-dag-app/tests/run_dir_import_export_contract.rs");
    assert!(
        import_export.contains("import_verify_only_roundtrip_contract"),
        "missing import/export invariant anchor"
    );
}
