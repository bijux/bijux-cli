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
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn replay_performance_benchmark_surfaces_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/replay_speed_baseline.md",
        "docs/reports/foundation/replay_proof_generation_benchmarks.md",
        "docs/reports/foundation/replay_proof_latency_report.md",
        "docs/reports/foundation/replay_diff_benchmark_focus_report.json",
    ] {
        let path = root.join(rel);
        assert!(path.exists(), "missing replay benchmark surface: {rel}");
        let body = fs::read_to_string(path).expect("read benchmark surface");
        assert!(
            !body.trim().is_empty(),
            "benchmark surface should be non-empty: {rel}"
        );
    }
}

#[test]
fn replay_regression_corpus_is_machine_readable_and_complete() {
    let root = repo_root();
    let payload: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/replay/regression_corpus.json"))
            .expect("read replay regression corpus"),
    )
    .expect("parse replay regression corpus");
    assert_eq!(payload["version"], "v1");
    let cases = payload["cases"].as_array().expect("cases");
    assert!(cases.len() >= 6);
    for required in [
        "strict-equivalent-replay",
        "partial-selection-replay",
        "imported-run-replay",
        "artifact-corruption-replay",
        "missing-artifact-replay",
        "incompatible-backend-replay",
    ] {
        assert!(
            cases.iter().any(|case| case["id"] == required),
            "missing replay regression case: {required}"
        );
    }
}
