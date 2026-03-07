use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn benchmark_claim_surfaces_cite_raw_evidence_paths() {
    let root = repo_root();
    let docs = [
        "docs/reports/foundation/performance_evidence_report.md",
        "docs/reports/foundation/portability_scorecard.md",
        "docs/reports/foundation/portability_determinism_scorecard.md",
    ];
    for rel in docs {
        let body = fs::read_to_string(root.join(rel)).expect("read report");
        assert!(
            body.contains("evidence/") || body.contains("artifacts/benchmarks/"),
            "benchmark claim surface must cite raw evidence path: {rel}"
        );
    }
}

#[test]
fn scenario_registry_and_metadata_are_normalized() {
    let root = repo_root();
    let registry: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/perf/scenario_registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/perf/metadata.json")).expect("read metadata"),
    )
    .expect("parse metadata");

    let entries = registry["entries"].as_array().expect("entries array");
    let scenarios = metadata["scenarios"]
        .as_object()
        .expect("metadata scenarios");

    let mut ids = BTreeSet::new();
    let mut last_id = String::new();
    for entry in entries {
        let id = entry["id"].as_str().expect("id");
        let path = entry["path"].as_str().expect("path");
        assert!(ids.insert(id.to_string()), "duplicate registry id: {id}");
        assert!(
            last_id.is_empty() || id >= last_id.as_str(),
            "registry entries must be sorted by id"
        );
        last_id = id.to_string();
        assert!(
            scenarios.contains_key(path),
            "registry path missing metadata: {path}"
        );
    }
}

#[test]
fn competitor_mapping_docs_and_matrix_template_exist() {
    let root = repo_root();
    for rel in [
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_AIRFLOW.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_DAGSTER.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_PREFECT.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_NEXTFLOW.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_SNAKEMAKE.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_ARGO_WORKFLOWS.md",
        "docs/reference/COMPETITOR_SCENARIO_MAPPING_LUIGI.md",
        "docs/reference/COMPARISON_MATRIX_TEMPLATE.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing competitor mapping surface: {rel}"
        );
    }
}

#[test]
fn benchmark_run_and_retention_docs_exist() {
    let root = repo_root();
    for rel in [
        "docs/reference/RUN_BENCHMARKS.md",
        "docs/spec/BENCHMARK_RESULT_FORMAT.md",
        "docs/spec/BENCHMARK_RAW_DATA_RETENTION.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing benchmark governance doc: {rel}"
        );
    }
}
