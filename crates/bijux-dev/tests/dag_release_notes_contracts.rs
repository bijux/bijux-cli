use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn read_repo_file(path: &str) -> String {
    let absolute = workspace_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn dag_v040_release_notes_cover_the_operator_release_boundary_honestly() {
    let notes = read_repo_file("docs/bijux-dag/operations/v0-4-0-release-notes.md");

    for token in [
        "# `bijux-dag` v0.4.0 Release Notes",
        "## Stable Features",
        "## Experimental Features",
        "## Internal And Future Features",
        "## Known Limitations",
        "## Breaking Changes",
        "## Migration Notes",
        "## Examples",
        "## Validation Commands",
        "`BIJUX_DAG_ENABLE_INTERNAL=1`",
        "`BIJUX_DAG_ENABLE_SIMULATED=1`",
        "`make dag-demo`",
        "`make release-validate-rs`",
        "shared-filesystem SLURM",
        "Kubernetes Job execution for container nodes",
        "general Airflow replacement",
        "Migration Guide",
        "Known Limitations",
        "Release Boundary",
    ] {
        assert!(notes.contains(token), "release notes missing token: {token}");
    }
}

#[test]
fn release_entrypoints_route_readers_to_the_dag_release_notes() {
    for path in [
        "README.md",
        "CHANGELOG.md",
        "docs/index.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/operations/index.md",
        "docs/bijux-dag/operations/release-and-versioning.md",
        "docs/bijux-core/operations/release-and-versioning.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("v0-4-0-release-notes.md") || content.contains("v0.4.0 Release Notes"),
            "{path} must route release-boundary readers to the DAG release notes"
        );
    }
}
