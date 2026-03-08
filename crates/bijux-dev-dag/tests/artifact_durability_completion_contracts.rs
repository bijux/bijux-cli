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
fn artifact_durability_contract_and_reports_exist() {
    for rel in [
        "docs/spec/ARTIFACT_DURABILITY_GUARANTEES_CONTRACT.md",
        "docs/reports/foundation/artifact_durability_benchmarks.md",
        "docs/reports/foundation/artifact_durability_telemetry_report.md",
        "docs/reports/foundation/artifact_durability_anomaly_report.md",
        "docs/reports/foundation/artifact_durability_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty artifact durability artifact: {rel}");
    }
}

#[test]
fn artifact_durability_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/artifact_durability/regression_corpus.json",
    ))
    .expect("parse artifact durability corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 18, "expected broad artifact durability corpus");

    for coverage in [
        "write-atomicity",
        "read-consistency",
        "partial-write-recovery",
        "concurrent-write-protection",
        "checksum-verification",
        "corruption-recovery",
        "store-rebuild",
        "store-compaction",
        "fragmentation",
        "retention-durability",
        "lifecycle-recovery",
        "gc-consistency",
        "gc-race-safety",
        "regression-fixtures",
        "durability-benchmarks",
        "durability-telemetry",
        "anomaly-detection",
        "store-rebuild-performance",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing artifact durability coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/artifact_durability_verification.json"))
            .expect("parse artifact durability suite");
    assert_eq!(suite["id"], "artifact-durability-verification");
}

#[test]
fn artifact_durability_surfaces_anchor_existing_storage_contracts() {
    let lifecycle = read("crates/bijux-dev-dag/tests/artifact_storage_lifecycle_completion_contracts.rs");
    for token in [
        "artifact_store_roundtrip_corruption_and_recovery_contracts_hold",
        "gc_explain_and_cleanup_plan_are_dry_run_safe_and_retention_aligned",
        "retention_explain_for_imported_bundle_prefixes_is_stable",
    ] {
        assert!(
            lifecycle.contains(token),
            "missing artifact lifecycle anchor token: {token}"
        );
    }

    let resilience =
        read("crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs");
    for token in [
        "atomic_durable_write_replaces_previous_json_payload",
        "gc_explain_and_cleanup_plan_are_dry_run_safe_and_retention_aligned",
    ] {
        assert!(
            resilience.contains(token),
            "missing artifact resilience anchor token: {token}"
        );
    }

    let hardening = read("crates/bijux-dag-artifacts/src/storage/hardening.rs");
    assert!(
        hardening.contains("write_json_atomic_durable"),
        "missing atomic durable write anchor"
    );
}
