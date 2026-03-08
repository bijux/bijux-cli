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
fn repository_structure_contract_and_reports_exist() {
    for rel in [
        "docs/spec/REPOSITORY_STRUCTURAL_HEALTH_CONTRACT.md",
        "docs/reports/foundation/repository_structure_coverage_report.md",
        "docs/reports/foundation/repository_structure_dashboard_report.md",
        "docs/reports/foundation/repository_structure_complexity_report.md",
        "docs/reports/foundation/repository_structure_dependency_report.md",
        "docs/reports/foundation/repository_structure_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty repository structure artifact: {rel}"
        );
    }
}

#[test]
fn repository_structure_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/repository_structure/regression_corpus.json",
    ))
    .expect("parse repository structure corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 14,
        "expected broad repository structure corpus"
    );

    for coverage in [
        "largest-modules-report",
        "highest-churn-report",
        "lowest-coverage-report",
        "duplicate-helper-detection",
        "unused-module-detection",
        "cyclic-dependency-detection",
        "dependency-graph-visualization",
        "module-ownership-mapping",
        "module-complexity-scoring",
        "refactoring-candidates-report",
        "documentation-coverage-report",
        "dependency-drift-tests",
        "hygiene-regression-fixtures",
        "health-dashboard",
        "complexity-benchmarks",
        "structural-lint-rules",
        "dependency-verification",
        "architectural-conformance",
        "telemetry",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing repository structure coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/repository_structure_verification.json",
    ))
    .expect("parse repository structure suite");
    assert_eq!(suite["id"], "repository-structure-verification");
}

#[test]
fn repository_structure_surfaces_anchor_existing_reports_and_hygiene_guards() {
    for rel in [
        "docs/reports/foundation/largest_files_by_crate_report.md",
        "docs/reports/foundation/top_25_highest_churn_files_remaining_report.md",
        "docs/reports/foundation/line_coverage_under_50_report.md",
        "docs/reports/foundation/duplicate_helper_modules_report.md",
        "docs/reports/foundation/dependency_cycle_report.md",
        "docs/reports/foundation/crate_dependency_graph_overlays.md",
        "docs/reports/foundation/runtime_module_ownership_report.md",
        "docs/reports/foundation/runtime_module_complexity_report.md",
        "docs/reports/foundation/repo_tree_cleanup_candidates_report.md",
        "docs/reports/foundation/docs_pages_by_owner_and_codepaths_report.md",
        "docs/reports/foundation/repo_tree_health_dashboard.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing repository structure anchor report: {rel}"
        );
    }

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "fn run_repo_hygiene_suite_guard()",
        "repo hygiene suite missing required guard",
        "root-directory-guard",
        "docs-governance",
        "config-drift",
        "evidence-authority",
    ] {
        assert!(
            commands.contains(token),
            "missing repository hygiene command anchor token: {token}"
        );
    }

    let repo_contract = read("crates/bijux-dev-dag/tests/repo_health_contracts.rs");
    assert!(
        repo_contract.contains("dependency_cycle_report.md"),
        "missing dependency cycle contract anchor"
    );
}
