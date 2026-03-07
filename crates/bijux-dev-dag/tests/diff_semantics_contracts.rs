use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn diff_semantics_docs_and_schemas_exist() {
    for rel in [
        "docs/spec/GRAPH_DIFF_SEMANTICS.md",
        "docs/spec/RUN_DIFF_SEMANTICS.md",
        "docs/spec/ARTIFACT_DIFF_SEMANTICS.md",
        "docs/spec/DIFF_CLASSIFICATION_CONTRACT.md",
        "configs/schema/operator/graph_diff.schema.json",
        "configs/schema/operator/run_diff.schema.json",
        "configs/schema/operator/artifact_trace.schema.json",
        "docs/reports/foundation/diff_speed_baseline.md",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}
