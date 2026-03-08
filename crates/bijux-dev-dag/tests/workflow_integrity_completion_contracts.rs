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
fn workflow_e2e_contract_tests_cover_major_command_chains() {
    let smoke = read("crates/bijux-dag-cli/tests/smoke_pipeline.rs");
    for token in [
        "cli_smoke_minimal_pipeline_validate_plan_run_replay_diff",
        "cli_smoke_export_import_and_fsck_verify_only",
        "cli_smoke_artifact_inspect_and_verify",
    ] {
        assert!(
            smoke.contains(token),
            "missing workflow smoke contract token: {token}"
        );
    }

    let routed = read("crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs");
    for token in [
        "smoke_validate_plan_run_inspect_replay_diff",
        "smoke_export_import_verify_only_and_fsck",
        "smoke_artifact_hash_inspect_trace",
        "smoke_history_show_summary_timeline",
        "smoke_prove_verify_and_surface_queries",
    ] {
        assert!(
            routed.contains(token),
            "missing routed workflow contract token: {token}"
        );
    }

    let slow = read("crates/bijux-dag-app/tests/e2e_integration_scenarios.rs");
    for token in [
        "e2e_minimal_parse_validate_run_inspect_replay",
        "e2e_replay_semantic_comparison_and_import_export",
        "e2e_selection_policy_compat_validation_and_no_partial_run_dir",
    ] {
        assert!(
            slow.contains(token),
            "missing slow e2e workflow token: {token}"
        );
    }
}

#[test]
fn workflow_regression_corpus_and_super_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/workflow/regression_corpus.json",
        "configs/suites/workflow_smoke_super_suite.json",
        "docs/reports/foundation/workflow_latency_benchmarks.md",
        "docs/reports/foundation/workflow_memory_benchmarks.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing workflow integrity artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/workflow/regression_corpus.json"))
            .expect("parse workflow corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 16,
        "expected broad workflow regression corpus"
    );

    for coverage in [
        "workflow-e2e",
        "bundle-roundtrip",
        "diagnostics",
        "traceability",
        "integrity-check",
        "failure-path",
        "recovery-path",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing workflow coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/workflow_smoke_super_suite.json"))
            .expect("parse workflow super suite");
    assert_eq!(suite["id"], "workflow-smoke-super-suite");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "smoke_pipeline",
        "app_smoke_routed_workflows_contract",
        "e2e_integration_scenarios",
        "workflow_integrity_completion_contracts",
    ] {
        assert!(
            commands.contains(token),
            "missing workflow super-suite command token: {token}"
        );
    }
}
