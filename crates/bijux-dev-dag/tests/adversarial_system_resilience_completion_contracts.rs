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
fn adversarial_system_contract_and_reports_exist() {
    for rel in [
        "docs/spec/ADVERSARIAL_SYSTEM_RESILIENCE_CONTRACT.md",
        "docs/reports/foundation/adversarial_system_coverage_report.md",
        "docs/reports/foundation/adversarial_system_benchmarks_report.md",
        "docs/reports/foundation/adversarial_system_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty adversarial artifact: {rel}");
    }
}

#[test]
fn adversarial_system_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/adversarial_system/regression_corpus.json",
    ))
    .expect("parse adversarial corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 15, "expected broad adversarial corpus");

    for coverage in [
        "adversarial-dag-generator",
        "scheduler-stress",
        "scheduler-starvation",
        "artifact-store-stress",
        "replay-mismatch",
        "backend-communication",
        "bundle-import",
        "run-history-corruption",
        "provenance-traversal",
        "diff-adversarial",
        "explain-adversarial",
        "cache-poisoning",
        "environment-drift",
        "concurrency",
        "filesystem-behavior",
        "determinism-drift",
        "runtime-crash",
        "data-corruption",
        "fuzzing",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing adversarial coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/adversarial_system_resilience_verification.json",
    ))
    .expect("parse adversarial suite");
    assert_eq!(suite["id"], "adversarial-system-resilience-verification");
}

#[test]
fn adversarial_system_surfaces_anchor_existing_runtime_and_app_tests() {
    let runtime_adv = read("crates/bijux-dag-runtime/tests/runtime_adversarial_contracts.rs");
    assert!(
        runtime_adv.contains("adversarial_cache_entry_without_proof_is_rejected"),
        "missing runtime adversarial cache anchor"
    );

    let runtime_sched =
        read("crates/bijux-dag-runtime/tests/scheduler_ordering_fairness_contracts.rs");
    assert!(
        runtime_sched.contains("scheduler_starvation_prevention_prefers_oldest_starved_first"),
        "missing scheduler starvation anchor"
    );

    let app_replay = read("crates/bijux-dag-app/tests/replay_contract.rs");
    assert!(
        app_replay.contains("corruption_case.json"),
        "missing replay corruption anchor"
    );

    let battle_adv = read("crates/bijux-dev-dag/tests/battle_adversarial_contracts.rs");
    assert!(
        battle_adv.contains("adversarial_release_blocking_subset_is_enforced"),
        "missing adversarial battle coverage anchor"
    );
}
