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
fn artifact_lineage_contract_and_reports_exist() {
    for rel in [
        "docs/spec/ARTIFACT_LINEAGE_COMPLETENESS_CONTRACT.md",
        "docs/reports/foundation/artifact_lineage_coverage_report.md",
        "docs/reports/foundation/artifact_lineage_benchmarks_report.md",
        "docs/reports/foundation/artifact_lineage_anomaly_report.md",
        "docs/reports/foundation/artifact_lineage_visualization_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty artifact lineage artifact: {rel}"
        );
    }
}

#[test]
fn artifact_lineage_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/artifact_lineage/regression_corpus.json",
    ))
    .expect("parse artifact lineage corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 12, "expected broad artifact lineage corpus");

    for coverage in [
        "reconstruction-correctness",
        "partial-run-completeness",
        "replay-lineage-correctness",
        "import-lineage-correctness",
        "gc-lineage-correctness",
        "serialization-stability",
        "traversal-guarantees",
        "traversal-benchmarks",
        "consistency-checks",
        "corruption-detection",
        "regression-fixtures",
        "fuzz-suite",
        "anomaly-detection",
        "explainability",
        "visualization-data",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing artifact lineage coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/artifact_lineage_verification.json"))
            .expect("parse artifact lineage suite");
    assert_eq!(suite["id"], "artifact-lineage-verification");
}

#[test]
fn artifact_lineage_surfaces_anchor_existing_runtime_app_and_artifact_tests() {
    let app_lineage = read("crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs");
    for token in [
        "artifact_identity_explain_covers_provenance_and_lineage_traversal",
        "provenance_traversal_is_deterministic_across_repeated_inspection",
        "provenance_serialization_is_stable_for_repeated_inspection",
        "provenance_query_latency_contract_on_large_lineage_snapshot",
    ] {
        assert!(
            app_lineage.contains(token),
            "missing app lineage token: {token}"
        );
    }

    let artifact_lineage =
        read("crates/bijux-dag-artifacts/tests/artifact_identity_and_lineage_contracts.rs");
    for token in [
        "lineage_traversal_is_stable_for_upstream_and_downstream_queries",
        "gc_plan_and_explain_outputs_stay_consistent",
    ] {
        assert!(
            artifact_lineage.contains(token),
            "missing artifact lineage token: {token}"
        );
    }

    let lifecycle =
        read("crates/bijux-dev-dag/tests/artifact_storage_lifecycle_completion_contracts.rs");
    assert!(
        lifecycle.contains("artifact_lineage_from_imported_bundle_and_replay_run_remains_distinct"),
        "missing lifecycle lineage continuity anchor"
    );

    let provenance =
        read("crates/bijux-dev-dag/tests/provenance_traceability_completion_contracts.rs");
    assert!(
        provenance.contains("provenance_linkage_and_trace_tests_cover_required_contracts"),
        "missing provenance lineage anchor"
    );
}
