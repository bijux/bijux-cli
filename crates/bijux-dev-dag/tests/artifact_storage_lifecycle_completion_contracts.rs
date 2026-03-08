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
fn artifact_storage_specs_and_reports_exist() {
    for rel in [
        "docs/spec/ARTIFACT_STORAGE_LIFECYCLE_CONTRACT.md",
        "docs/spec/ARTIFACT_LIFECYCLE.md",
        "docs/spec/ARTIFACT_RETENTION_POLICY.md",
        "docs/reports/foundation/artifact_storage_lifecycle_benchmarks.md",
        "docs/reports/foundation/artifact_storage_lifecycle_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty artifact lifecycle surface: {rel}"
        );
    }
}

#[test]
fn artifact_storage_corpus_and_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/artifact_lifecycle/regression_corpus.json",
        "configs/suites/artifact_storage_lifecycle_stress.json",
        "crates/bijux-dag-artifacts/tests/fixtures/artifact_hash_regression_corpus.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing artifact lifecycle artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/artifact_lifecycle/regression_corpus.json",
    ))
    .expect("parse artifact lifecycle corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 14, "expected artifact lifecycle corpus breadth");
    for coverage in [
        "lifecycle-roundtrip",
        "replay-lifecycle",
        "imported-lifecycle",
        "partial-rerun",
        "gc-eligibility",
        "retention",
        "ancestry-chain",
        "gc-concurrency",
        "gc-replay",
        "fragmentation",
        "store-repair",
        "partial-write-recovery",
        "checksum",
        "corruption-detection",
        "gc-explain",
        "lifecycle-stress",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing artifact lifecycle coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/artifact_storage_lifecycle_stress.json",
    ))
    .expect("parse artifact lifecycle suite");
    assert_eq!(suite["id"], "artifact-storage-lifecycle-stress");
}

#[test]
fn artifact_crate_tests_anchor_lifecycle_contracts() {
    let completion = read(
        "crates/bijux-dag-artifacts/tests/artifact_identity_lifecycle_completion_contracts.rs",
    );
    for token in [
        "artifact_store_roundtrip_corruption_and_recovery_contracts_hold",
        "artifact_store_concurrency_and_integrity_verification_contracts_hold",
        "artifact_lineage_from_imported_bundle_and_replay_run_remains_distinct",
    ] {
        assert!(
            completion.contains(token),
            "missing artifact lifecycle completion token: {token}"
        );
    }

    let resilience =
        read("crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs");
    assert!(
        resilience.contains("gc_explain_and_cleanup_plan_are_dry_run_safe_and_retention_aligned"),
        "missing artifact storage resilience gc token"
    );

    let hardening = read(
        "crates/bijux-dag-artifacts/tests/artifact_io_store_hardening_expansion_contracts.rs",
    );
    for token in [
        "gc_explain_covers_retained_roots_and_collectable_leaves",
        "retention_explain_for_imported_bundle_prefixes_is_stable",
    ] {
        assert!(
            hardening.contains(token),
            "missing artifact store hardening token: {token}"
        );
    }
}
