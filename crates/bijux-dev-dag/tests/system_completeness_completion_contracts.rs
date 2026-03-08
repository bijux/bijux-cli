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
fn system_completeness_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_COMPLETENESS_VERIFICATION_CONTRACT.md",
        "docs/reports/foundation/system_completeness_verification_report.md",
        "docs/reports/foundation/system_completeness_coverage_matrix.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty completeness artifact: {rel}");
    }
}

#[test]
fn system_completeness_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_completeness/regression_corpus.json",
    ))
    .expect("parse system completeness corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 15, "expected broad system completeness corpus");

    for coverage in [
        "system-invariants-coverage",
        "determinism-coverage",
        "replay-equivalence-coverage",
        "diff-semantics-coverage",
        "artifact-lineage-coverage",
        "runtime-scheduler-coverage",
        "backend-adapter-coverage",
        "portability-coverage",
        "schema-compatibility-coverage",
        "introspection-coverage",
        "observability-coverage",
        "security-coverage",
        "performance-benchmark-coverage",
        "stress-coverage",
        "fuzz-coverage",
        "conceptual-integrity",
        "architecture-coherence",
        "correctness-guarantees",
        "reliability-guarantees",
        "final-report",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing system completeness coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/system_completeness_verification.json"))
            .expect("parse system completeness suite");
    assert_eq!(suite["id"], "system-completeness-verification");
}

#[test]
fn system_completeness_surfaces_anchor_all_major_completion_contract_domains() {
    for rel in [
        "crates/bijux-dev-dag/tests/system_invariants_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/dag_model_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/artifact_lineage_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/semantic_diff_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/system_introspection_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/system_introspection_architecture_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/system_conceptual_integrity_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/system_reliability_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/system_maintainability_completion_contracts.rs",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing completion domain anchor: {rel}"
        );
    }
}

