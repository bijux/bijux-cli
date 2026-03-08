use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn artifact_identity_docs_and_schema_exist() {
    for rel in [
        "docs/spec/ARTIFACT_IDENTITY_CONTRACT.md",
        "docs/spec/ARTIFACT_INSPECT_SCHEMA_v0.1.md",
        "docs/spec/ARTIFACT_BUNDLE_MANIFEST_EXAMPLES.md",
        "docs/reports/foundation/artifact_store_capability_matrix.md",
        "docs/reports/foundation/content_addressed_storage_model.md",
        "configs/schema/operator/artifact_inspect.schema.json",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}

#[test]
fn artifact_capability_report_command_is_wired_and_report_is_implementation_backed() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read dev command source");
    for token in [
        "RepoCommand::ArtifactCapabilityReports",
        "repo.artifact-capability-reports",
        "run_repo_artifact_capability_reports",
    ] {
        assert!(
            source.contains(token),
            "missing artifact capability command token: {token}"
        );
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/artifact_store_capability_matrix.md"),
    )
    .expect("artifact capability matrix report");
    assert!(report.contains("filesystem store"));
    assert!(report.contains("object store model"));
    assert!(report.contains("implemented"));
    assert!(report.contains("modeled"));
}
