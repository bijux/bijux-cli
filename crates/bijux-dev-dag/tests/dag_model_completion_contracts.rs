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
fn dag_model_contract_and_reports_exist() {
    for rel in [
        "docs/spec/DAG_MODEL_COMPLETENESS_CONTRACT.md",
        "docs/reports/foundation/dag_model_coverage_report.md",
        "docs/reports/foundation/dag_model_benchmarks_report.md",
        "docs/reports/foundation/dag_model_anomaly_report.md",
        "docs/reports/foundation/dag_model_verification_tools_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty dag model artifact: {rel}");
    }
}

#[test]
fn dag_model_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read("evidence/cache/dag_model/regression_corpus.json"))
        .expect("parse dag model corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 12, "expected broad dag model corpus");

    for coverage in [
        "node-semantics",
        "dependency-semantics",
        "artifact-dependency-semantics",
        "node-io-contract-semantics",
        "execution-ordering-guarantees",
        "validation-completeness",
        "semantic-validation",
        "normalization-determinism",
        "schema-compliance",
        "semantic-drift-detection",
        "validation-performance-benchmarks",
        "semantic-consistency",
        "fuzz-suite",
        "anomaly-detection",
        "verification-tools",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing dag model coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/dag_model_verification.json"))
            .expect("parse dag model suite");
    assert_eq!(suite["id"], "dag-model-verification");
}

#[test]
fn dag_model_surfaces_anchor_existing_dag_commands_and_tests() {
    let cli = read("crates/bijux-dev-dag/src/commands/cli.rs");
    for token in [
        "ExplainValidation",
        "SchemaExport",
        "Lint",
        "DryRun",
        "PlanDump",
    ] {
        assert!(cli.contains(token), "missing DAG CLI token: {token}");
    }

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "run_dag_lint",
        "run_dag_dry_run",
        "run_dag_plan_dump",
        "run_dag_explain_validation",
        "run_dag_schema_export",
    ] {
        assert!(commands.contains(token), "missing DAG command token: {token}");
    }

    let large_dag = read("crates/bijux-dev-dag/tests/large_dag_scalability_completion_contracts.rs");
    assert!(
        large_dag.contains("validation_stress_thousands_of_nodes"),
        "missing DAG validation scalability anchor"
    );
}

