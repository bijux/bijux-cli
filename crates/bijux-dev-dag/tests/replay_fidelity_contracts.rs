use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
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
fn replay_fidelity_docs_and_schema_exist() {
    for rel in [
        "docs/spec/REPLAY_FIDELITY_LEVELS.md",
        "docs/spec/REPLAY_PROOF_BUNDLE_SCHEMA_v0.1.md",
        "docs/reports/foundation/replay_fidelity_report_v0.1.json",
        "docs/reports/foundation/replay_speed_baseline.md",
        "configs/schema/operator/replay_proof.schema.json",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}
