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
fn cache_integrity_contract_and_reports_exist() {
    for rel in [
        "docs/spec/CACHE_SYSTEM_INTEGRITY_CONTRACT.md",
        "docs/reports/foundation/cache_integrity_benchmarks.md",
        "docs/reports/foundation/cache_integrity_telemetry_report.md",
        "docs/reports/foundation/cache_explainability_integration_report.md",
        "docs/reports/foundation/cache_integrity_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty cache integrity artifact: {rel}"
        );
    }
}

#[test]
fn cache_integrity_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/cache_integrity/regression_corpus.json",
    ))
    .expect("parse cache integrity corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 18, "expected broad cache integrity corpus");

    for coverage in [
        "key-determinism",
        "lookup-consistency",
        "invalidation",
        "graph-change-invalidation",
        "environment-drift-invalidation",
        "artifact-change-invalidation",
        "replay-ancestry-invalidation",
        "integrity-verification",
        "corruption-detection",
        "concurrency",
        "eviction-safety",
        "retention",
        "lifecycle",
        "regression-fixtures",
        "stress",
        "performance",
        "telemetry",
        "explainability-integration",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing cache integrity coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/cache_integrity_verification.json"))
            .expect("parse cache integrity suite");
    assert_eq!(suite["id"], "cache-integrity-verification");
}

#[test]
fn cache_integrity_surfaces_anchor_existing_cache_contracts() {
    let cache_hardening = read("crates/bijux-dev-dag/tests/cache_hardening_contracts.rs");
    for token in ["dag cache explain", "tp_cache_integrity"] {
        assert!(
            cache_hardening.contains(token),
            "missing cache hardening anchor token: {token}"
        );
    }

    let cache_contract = read("docs/spec/CACHE_CONTRACT.md");
    for token in [
        "dag cache verify",
        "Cache key invalidation on planner-significant changes",
        "Cache key invalidation on policy/config changes",
    ] {
        assert!(
            cache_contract.contains(token),
            "missing cache contract anchor token: {token}"
        );
    }

    let explain_surface =
        read("crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs");
    assert!(
        explain_surface
            .contains("explain_why_cache_missed_reports_corrupt_entry_verification_failure"),
        "missing cache explain integration anchor"
    );
}
