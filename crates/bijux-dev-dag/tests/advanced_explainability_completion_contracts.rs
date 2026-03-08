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
fn advanced_explainability_contract_and_reports_exist() {
    for rel in [
        "docs/spec/ADVANCED_EXPLAINABILITY_MODEL_CONTRACT.md",
        "docs/reports/foundation/explainability_completeness_report.md",
        "docs/reports/foundation/explainability_anomaly_detection_report.md",
        "docs/reports/foundation/advanced_explainability_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty advanced explainability artifact: {rel}"
        );
    }
}

#[test]
fn advanced_explainability_corpus_and_suite_are_machine_readable() {
    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/explainability/regression_corpus.json"))
            .expect("parse advanced explainability corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 16, "expected broad explainability corpus");

    for coverage in [
        "node-level",
        "scheduler-decision",
        "replay-decision",
        "cache-hit",
        "cache-miss",
        "artifact-lineage",
        "dependency-chain",
        "environment-drift",
        "backend-capability-mismatch",
        "output-consistency",
        "json-schema",
        "text-snapshot",
        "ordering-determinism",
        "large-dag-stress",
        "performance",
        "regression-fixtures",
        "explain-completeness",
        "anomaly-detection",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing explainability coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/advanced_explainability_regression.json"))
            .expect("parse advanced explainability suite");
    assert_eq!(suite["id"], "advanced-explainability-regression");
}

#[test]
fn advanced_explainability_is_anchored_to_existing_explain_surfaces() {
    let explain_surface = read("crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs");
    for token in [
        "why_rerun_reports_graph_drift_group",
        "why_rerun_reports_environment_drift_group",
        "why_rerun_reports_artifact_drift_group",
        "why_rerun_reports_replay_ancestry_drift_group",
        "explain_why_cache_missed_reports_corrupt_entry_verification_failure",
    ] {
        assert!(
            explain_surface.contains(token),
            "missing explain surface anchor token: {token}"
        );
    }

    let wording_snapshot = read("crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt");
    assert!(
        wording_snapshot.contains("why-rerun") || wording_snapshot.contains("cache"),
        "missing concise explain wording anchor"
    );
}
