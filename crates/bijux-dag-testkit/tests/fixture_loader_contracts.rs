use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit::{
    load_artifact_fixture_json, load_benchmark_fixture_json, load_bundle_fixture_json,
    load_capability_fixture_json, load_graph_fixture_json, load_replay_fixture_json,
    load_run_fixture_json,
};
use serde as _;
use serde_json as _;
use tempfile as _;

#[test]
fn graph_fixture_loader_reads_schema_fixture() {
    let value = load_graph_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "configs/dag/schema/fixtures/v0.1/positive/empty-graph.json",
    );
    assert_eq!(value["spec"], "bijux-dag/v0.1");
}

#[test]
fn run_fixture_loader_reads_manifest_fixture() {
    let value = load_run_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-artifacts/tests/fixtures/run_manifest_minimal.json",
    );
    assert_eq!(value["manifest_version"], "run-manifest/v0.1");
}

#[test]
fn artifact_fixture_loader_reads_identity_fixture() {
    let value = load_artifact_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/compat/ecosystem/shared_identity/artifact_identity_fixture.json",
    );
    assert!(value.get("artifact_id").is_some());
}

#[test]
fn bundle_fixture_loader_reads_workflow_fixture() {
    let value = load_bundle_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/battle/workflows/import_export/export_import_with_files.json",
    );
    assert!(value.get("scenario_id").is_some());
}

#[test]
fn replay_fixture_loader_reads_replay_fixture() {
    let value = load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/cache/replay/mismatch_fixture_corpus.json",
    );
    assert!(value.get("cases").is_some());
}

#[test]
fn capability_fixture_loader_reads_capability_catalog() {
    let value = load_capability_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "configs/dag/policy/runtime_adapter_surface_catalog.json",
    );
    assert!(value.get("surfaces").is_some());
}

#[test]
fn benchmark_fixture_loader_reads_baseline_fixture() {
    let value = load_benchmark_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/perf/baselines/benchmark_baseline_fixtures_v1.json",
    );
    assert!(value.get("scenarios").is_some());
}
