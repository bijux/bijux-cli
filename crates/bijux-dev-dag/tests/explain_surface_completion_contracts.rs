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
fn explain_specs_and_schema_surfaces_exist() {
    for rel in [
        "docs/spec/EXPLAIN_SURFACES_CONTRACT.md",
        "docs/spec/TRACE_CONTRACT.md",
        "configs/schema/operator/run_explain_failure.schema.json",
        "configs/schema/operator/artifact_trace.schema.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing explain or trace contract surface: {rel}"
        );
    }
}

#[test]
fn explain_behavior_and_wording_contract_tests_cover_required_surfaces() {
    let diagnostics_routes = read("crates/bijux-dag-app/src/routes/diagnostics_routes.rs");
    for token in [
        "why_rerun_reports_graph_drift_group",
        "why_rerun_reports_environment_drift_group",
        "why_rerun_reports_artifact_drift_group",
        "why_rerun_reports_replay_ancestry_drift_group",
    ] {
        assert!(
            diagnostics_routes.contains(token),
            "missing explain drift grouping coverage: {token}"
        );
    }

    let replay_surface = read("crates/bijux-dag-app/tests/replay_semantic_surface_contracts.rs");
    for token in [
        "explain_why_rerun_supports_imported_run_ancestry_context",
        "explain_why_cache_missed_reports_corrupt_entry_verification_failure",
        "trace_artifact_supports_replayed_run_provenance_surface",
    ] {
        assert!(
            replay_surface.contains(token),
            "missing explain surface contract test: {token}"
        );
    }

    let wording = read("crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs");
    for token in [
        "route_level_concise_wording_snapshot_is_stable",
        "route_level_detailed_wording_snapshot_is_stable",
    ] {
        assert!(
            wording.contains(token),
            "missing concise/detailed wording contract test: {token}"
        );
    }
}

#[test]
fn explain_regression_corpus_suite_and_benchmark_reports_exist() {
    for rel in [
        "evidence/cache/explain/regression_corpus.json",
        "configs/suites/explain_surface_stress.json",
        "docs/reports/foundation/explainability_benchmarks.md",
        "docs/reports/foundation/wording_drift_equivalent_commands_report.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing explain governance artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/explain/regression_corpus.json"))
            .expect("parse explain regression corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(
        corpus["cases"].as_array().expect("cases").len() >= 6,
        "expected explain regression corpus breadth"
    );

    let suite: Value = serde_json::from_str(&read("configs/suites/explain_surface_stress.json"))
        .expect("parse explain suite");
    assert_eq!(suite["id"], "explain-surface-stress");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "diff_explain_contract",
        "replay_semantic_surface_contracts",
        "route_output_wording_snapshot_contracts",
        "explain_surface_completion_contracts",
    ] {
        assert!(
            commands.contains(token),
            "missing suite command token: {token}"
        );
    }
}
