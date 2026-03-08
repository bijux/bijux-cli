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
fn runtime_architecture_specs_and_reports_exist() {
    for rel in [
        "docs/spec/RUNTIME_ARCHITECTURE_CLEANUP_CONTRACT.md",
        "docs/spec/ARCHITECTURE_REVIEW_CHECKLIST.md",
        "docs/spec/RUNTIME_PUBLIC_API_BOUNDARY.md",
        "docs/reports/foundation/runtime_module_coverage_report.md",
        "docs/reports/foundation/runtime_module_complexity_report.md",
        "docs/reports/foundation/runtime_architecture_telemetry_report.md",
        "docs/reports/foundation/runtime_architecture_health_dashboard.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty runtime architecture surface: {rel}"
        );
    }
}

#[test]
fn runtime_architecture_corpus_and_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/runtime_architecture/regression_corpus.json",
        "configs/suites/runtime_architecture_health.json",
        "docs/reports/foundation/runtime_boundary_report.md",
        "docs/reports/foundation/runtime_module_ownership_report.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing runtime architecture artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/runtime_architecture/regression_corpus.json",
    ))
    .expect("parse runtime architecture corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 9,
        "expected runtime architecture corpus breadth"
    );
    for coverage in [
        "module-boundary",
        "ownership",
        "oversized-module",
        "split-rationale",
        "duplicate-helper",
        "unused-path",
        "dependency-graph",
        "architecture-invariants",
        "module-coverage",
        "module-complexity",
        "architecture-telemetry",
        "health-dashboard",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing runtime architecture coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/runtime_architecture_health.json"))
            .expect("parse runtime architecture suite");
    assert_eq!(suite["id"], "runtime-architecture-health");
}

#[test]
fn runtime_dev_tests_anchor_architecture_cleanup_contracts() {
    let guardrails = read("crates/bijux-dev-dag/tests/runtime_scope_v2_guardrails.rs");
    assert!(
        guardrails.contains("runtime_scope_v2 policy inventory"),
        "missing runtime scope guardrail anchor"
    );

    let ownership = read("crates/bijux-dev-dag/tests/runtime_broad_surface_ownership_contracts.rs");
    assert!(
        ownership.contains("missing runtime module"),
        "missing runtime ownership anchor"
    );

    let reports = read("crates/bijux-dev-dag/tests/runtime_scope_reports_contracts.rs");
    assert!(
        reports.contains("runtime_contract_backing_report"),
        "missing runtime scope reports anchor"
    );
}
