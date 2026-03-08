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
fn system_maintainability_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_MAINTAINABILITY_CONTRACT.md",
        "docs/reports/foundation/system_maintainability_coverage_report.md",
        "docs/reports/foundation/system_maintainability_dashboard_report.md",
        "docs/reports/foundation/system_maintainability_telemetry_report.md",
        "docs/reports/foundation/system_maintainability_review_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty system maintainability artifact: {rel}"
        );
    }
}

#[test]
fn system_maintainability_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_maintainability/regression_corpus.json",
    ))
    .expect("parse system maintainability corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected broad system maintainability corpus");

    for coverage in [
        "maintainability-guidelines",
        "module-ownership-rules",
        "module-boundary-policies",
        "architectural-layering-guidelines",
        "dependency-hygiene-policies",
        "architectural-boundaries",
        "dependency-cycle-detection",
        "module-complexity-monitoring",
        "regression-fixtures",
        "telemetry-reporting",
        "anomaly-detection",
        "documentation",
        "review-checklist",
        "conformance-tests",
        "architecture-visualization-tooling",
        "dashboard",
        "maintainability-benchmarks",
        "architecture-review",
        "verification-tools",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing system maintainability coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/system_maintainability_verification.json",
    ))
    .expect("parse system maintainability suite");
    assert_eq!(suite["id"], "system-maintainability-verification");
}

#[test]
fn system_maintainability_surfaces_anchor_existing_boundary_and_hygiene_controls() {
    let repo_structure = read("crates/bijux-dev-dag/tests/repository_structure_completion_contracts.rs");
    assert!(
        repo_structure.contains("repository_structure_surfaces_anchor_existing_reports_and_hygiene_guards"),
        "missing repository structure maintainability anchor"
    );

    let conceptual = read("crates/bijux-dev-dag/tests/system_conceptual_integrity_completion_contracts.rs");
    assert!(
        conceptual.contains("conceptual_integrity_surfaces_anchor_existing_architecture_and_conformance_docs"),
        "missing conceptual integrity maintainability anchor"
    );

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "run_crate_ownership_guard",
        "run_crate_boundary_foundation_guard",
        "run_repo_hygiene_suite_guard",
        "Dependency Cycle Report",
    ] {
        assert!(
            commands.contains(token),
            "missing maintainability command anchor token: {token}"
        );
    }

    for rel in [
        "docs/architecture/module_ownership_map.md",
        "docs/reports/foundation/dependency_cycle_report.md",
        "docs/reports/foundation/runtime_module_complexity_report.md",
        "docs/reports/foundation/repo_tree_health_dashboard.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing maintainability report anchor: {rel}"
        );
    }
}

