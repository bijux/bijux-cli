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
fn provenance_specs_and_schema_surfaces_exist() {
    for rel in [
        "docs/spec/PROVENANCE_MODEL_CONTRACT.md",
        "docs/spec/RUN_VS_ARTIFACT_LINEAGE.md",
        "docs/spec/TRACE_CONTRACT.md",
        "configs/schema/operator/artifact_trace.schema.json",
        "configs/schema/operator/artifact_inspect.schema.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing provenance contract or schema surface: {rel}"
        );
    }
}

#[test]
fn provenance_linkage_and_trace_tests_cover_required_contracts() {
    let artifact_identity = read("crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs");
    for token in [
        "artifact_identity_explain_covers_provenance_and_lineage_traversal",
        "provenance_traversal_is_deterministic_across_repeated_inspection",
        "provenance_serialization_is_stable_for_repeated_inspection",
        "provenance_query_latency_contract_on_large_lineage_snapshot",
    ] {
        assert!(
            artifact_identity.contains(token),
            "missing provenance traceability contract test: {token}"
        );
    }

    let replay_surfaces = read("crates/bijux-dag-app/tests/replay_semantic_surface_contracts.rs");
    assert!(
        replay_surfaces.contains("trace_artifact_supports_replayed_run_provenance_surface"),
        "missing replayed artifact provenance contract"
    );

    let artifact_lineage =
        read("crates/bijux-dag-artifacts/tests/artifact_identity_and_lineage_contracts.rs");
    assert!(
        artifact_lineage.contains("lineage_traversal_is_stable_for_upstream_and_downstream_queries"),
        "missing artifact lineage traversal contract"
    );
}

#[test]
fn provenance_regression_corpus_suite_and_latency_report_are_present() {
    for rel in [
        "evidence/cache/provenance/regression_corpus.json",
        "configs/suites/provenance_traceability_stress.json",
        "docs/reports/foundation/provenance_query_latency_report.md",
        "docs/reports/foundation/artifact_provenance_field_map.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing provenance governance artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/provenance/regression_corpus.json"))
            .expect("parse provenance corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(
        corpus["cases"].as_array().expect("cases").len() >= 6,
        "expected provenance regression corpus breadth"
    );

    let suite: Value =
        serde_json::from_str(&read("configs/suites/provenance_traceability_stress.json"))
            .expect("parse provenance suite");
    assert_eq!(suite["id"], "provenance-traceability-stress");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "artifact_identity_explain_contract",
        "replay_semantic_surface_contracts",
        "semantic_lineage_contracts",
        "provenance_traceability_completion_contracts",
    ] {
        assert!(commands.contains(token), "missing suite command token: {token}");
    }
}
