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
fn replay_equivalence_contract_and_reports_exist() {
    for rel in [
        "docs/spec/REPLAY_EQUIVALENCE_COMPLETENESS_CONTRACT.md",
        "docs/reports/foundation/replay_equivalence_coverage_report.md",
        "docs/reports/foundation/replay_equivalence_benchmarks_report.md",
        "docs/reports/foundation/replay_equivalence_telemetry_report.md",
        "docs/reports/foundation/replay_equivalence_diagnostics_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty replay equivalence artifact: {rel}"
        );
    }
}

#[test]
fn replay_equivalence_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/replay_equivalence/regression_corpus.json",
    ))
    .expect("parse replay equivalence corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 12,
        "expected broad replay equivalence corpus"
    );

    for coverage in [
        "equivalence-detection",
        "correctness-guarantees",
        "mismatch-classification",
        "fidelity-levels",
        "deterministic-planning",
        "proof-verification",
        "environment-drift-semantics",
        "artifact-drift-semantics",
        "regression-fixtures",
        "fuzz-suite",
        "anomaly-detection",
        "performance-benchmarks",
        "explainability",
        "telemetry",
        "diagnostics-tooling",
        "documentation",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing replay equivalence coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/replay_equivalence_verification.json"))
            .expect("parse replay equivalence suite");
    assert_eq!(suite["id"], "replay-equivalence-verification");
}

#[test]
fn replay_equivalence_surfaces_anchor_existing_app_runtime_and_contract_tests() {
    let app_replay_proof = read("crates/bijux-dag-app/tests/replay_proof_contract.rs");
    for token in [
        "replay_prove_reports_strict_equivalent_on_exact_pair",
        "replay_prove_reports_diverged_on_corrupt_source_pair",
        "fidelity_level",
    ] {
        assert!(
            app_replay_proof.contains(token),
            "missing replay proof token: {token}"
        );
    }

    let app_replay_diff = read("crates/bijux-dag-app/tests/replay_diff_hardening_contract.rs");
    assert!(
        app_replay_diff
            .contains("replay_missing_artifacts_and_environment_mismatch_downgrade_fidelity"),
        "missing replay drift fidelity anchor"
    );

    let runtime_replay = read("crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs");
    assert!(
        runtime_replay.contains("replay_mismatch_is_detected"),
        "missing runtime replay mismatch anchor"
    );

    let replay_hardening = read("crates/bijux-dev-dag/tests/replay_hardening_contracts.rs");
    assert!(
        replay_hardening.contains("replay_contract_covers_definition_explainability_and_non_goals"),
        "missing replay hardening contract anchor"
    );
}
