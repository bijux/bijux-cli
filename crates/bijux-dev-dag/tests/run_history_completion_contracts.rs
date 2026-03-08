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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn run_history_specs_and_performance_reports_exist() {
    for rel in [
        "docs/spec/RUN_IDENTITY_CONTRACT.md",
        "docs/spec/RUN_HISTORY_CONTRACT.md",
        "docs/spec/RUN_MANIFEST_SCHEMA_v0.1.md",
        "docs/spec/RUN_SUMMARY_SCHEMA_v0.1.md",
        "docs/reports/foundation/run_history_query_benchmarks.md",
        "docs/reports/foundation/run_history_query_latency_report.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing run history surface: {rel}"
        );
    }
}

#[test]
fn run_manifest_regression_corpus_and_stress_suite_are_present() {
    let corpus_path = root().join("evidence/cache/replay/run_manifest_regression_corpus.json");
    let corpus: Value =
        serde_json::from_str(&fs::read_to_string(corpus_path).expect("read corpus"))
            .expect("parse corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(corpus["cases"].as_array().expect("cases").len() >= 3);

    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root().join("configs/suites/run_history_many_runs_stress.json"))
            .expect("read stress suite"),
    )
    .expect("parse stress suite");
    assert_eq!(suite["id"], "run-history-many-runs-stress");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "run_history_stress_suite_many_runs_is_deterministic",
        "run_history_query_performance_contract_on_large_fixture_set",
    ] {
        assert!(
            commands.contains(token),
            "missing stress suite command: {token}"
        );
    }
}
