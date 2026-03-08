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
fn conceptual_integrity_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_CONCEPTUAL_INTEGRITY_CONTRACT.md",
        "docs/reports/foundation/system_conceptual_integrity_coverage_report.md",
        "docs/reports/foundation/system_conceptual_integrity_drift_report.md",
        "docs/reports/foundation/system_conceptual_integrity_verification_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty conceptual integrity artifact: {rel}"
        );
    }
}

#[test]
fn conceptual_integrity_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_conceptual_integrity/regression_corpus.json",
    ))
    .expect("parse conceptual integrity corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 13,
        "expected broad conceptual integrity corpus"
    );

    for coverage in [
        "conceptual-architecture-overview",
        "execution-model-overview",
        "artifact-model-overview",
        "replay-model-overview",
        "diff-model-overview",
        "provenance-model-overview",
        "backend-abstraction-overview",
        "runtime-execution-overview",
        "scheduler-behavior-overview",
        "determinism-overview",
        "conceptual-consistency-tests",
        "system-boundary-consistency",
        "architecture-conformance",
        "conceptual-drift-detection",
        "documentation-generation",
        "anomaly-detection",
        "architecture-verification-tooling",
        "architecture-verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing conceptual integrity coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/system_conceptual_integrity_verification.json",
    ))
    .expect("parse conceptual integrity suite");
    assert_eq!(suite["id"], "system-conceptual-integrity-verification");
}

#[test]
fn conceptual_integrity_surfaces_anchor_existing_architecture_and_conformance_docs() {
    for rel in [
        "docs/architecture/runtime-execution-flow.md",
        "docs/architecture/controller_backend_artifact_boundary.md",
        "docs/architecture/engine-backend-responsibilities.md",
        "docs/architecture/runtime-concurrency-boundaries.md",
        "docs/architecture/runtime_core_architecture.md",
        "docs/architecture/runtime_scope_v2.md",
        "docs/reports/foundation/repository_architecture_report.md",
        "docs/reports/foundation/runtime_execution_conformance_suite.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing conceptual architecture anchor: {rel}"
        );
    }

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "run_repo_hygiene_suite_guard",
        "run_anti_drift_governance_guard",
        "run_drift_dashboard",
        "run_evidence_drift_verify",
    ] {
        assert!(
            commands.contains(token),
            "missing conceptual integrity command anchor token: {token}"
        );
    }

    let prior_completion =
        read("crates/bijux-dev-dag/tests/repository_structure_completion_contracts.rs");
    assert!(
        prior_completion
            .contains("repository_structure_surfaces_anchor_existing_reports_and_hygiene_guards"),
        "missing repository structure conceptual anchor"
    );
}
