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
fn fixture_tooling_contract_and_reports_exist() {
    for rel in [
        "docs/spec/FIXTURE_TOOLING_GOVERNANCE_CONTRACT.md",
        "docs/reports/foundation/fixture_tooling_coverage_report.md",
        "docs/reports/foundation/fixture_duplication_detection_report.md",
        "docs/reports/foundation/fixture_cleanup_automation_report.md",
        "docs/reports/foundation/fixture_lifecycle_governance_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty fixture tooling artifact: {rel}");
    }
}

#[test]
fn fixture_tooling_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/fixture_tooling/regression_corpus.json",
    ))
    .expect("parse fixture tooling corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 10, "expected fixture tooling corpus breadth");

    for coverage in [
        "graph-generation",
        "run-generation",
        "artifact-generation",
        "replay-generation",
        "diff-generation",
        "bundle-generation",
        "fixture-validation-cli",
        "fuzz-corpus-generation",
        "benchmark-scenario-generation",
        "duplication-detection",
        "cleanup-automation",
        "lifecycle-governance",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing fixture tooling coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read("configs/suites/fixture_tooling_governance.json"))
        .expect("parse fixture tooling suite");
    assert_eq!(suite["id"], "fixture-tooling-governance");
}

#[test]
fn fixture_tooling_surfaces_are_anchored_in_repo() {
    let testkit = read("crates/bijux-dag-testkit/src/lib.rs");
    for helper in [
        "load_graph_fixture_json",
        "load_run_fixture_json",
        "load_artifact_fixture_json",
        "load_bundle_fixture_json",
        "load_replay_fixture_json",
        "load_benchmark_fixture_json",
    ] {
        assert!(
            testkit.contains(helper),
            "missing fixture helper surface in testkit: {helper}"
        );
    }

    let governance_bin = read("crates/bijux-dev-dag/src/bin/generate_fixture_governance_reports.rs");
    assert!(
        governance_bin.contains("fixture_governance_missing_owner_report.md"),
        "missing fixture governance report generation anchor"
    );

    let duplicate_bin =
        read("crates/bijux-dev-dag/src/bin/generate_duplicate_fixture_loader_report.rs");
    assert!(
        duplicate_bin.contains("duplicate_fixture_loader_helpers_report.md"),
        "missing duplicate fixture helper report generation anchor"
    );
}
