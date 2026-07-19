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
fn future_direction_defines_promotion_gates_without_blurring_current_boundary() {
    let direction = read_repo_file("docs/bijux-dag/foundation/future-direction.md");

    for token in [
        "# Future Direction",
        "## Promotion Model",
        "## Current Foundation",
        "## Graph Expressiveness",
        "## Durable Scheduling And Backfill",
        "## Remote Workers",
        "## Broader Batch And Cluster Support",
        "## Long-Lived Compatibility",
        "## Decision Record",
        "Release Boundary",
        "Known Limitations",
        "local-first",
        "execution claim without this lifecycle",
        "narrower release claim remains authoritative",
    ] {
        assert!(direction.contains(token), "future direction page missing token: {token}");
    }
}

#[test]
fn future_direction_preserves_shipped_backend_truth() {
    let direction = read_repo_file("docs/bijux-dag/foundation/future-direction.md");

    for token in [
        "concrete container, Kubernetes Job, and shared-filesystem SLURM boundaries",
        "current Kubernetes and SLURM lanes are intentionally concrete",
        "storage and path visibility assumptions",
        "scheduler resource and timeout mappings",
        "controller restart and backend outage behavior",
        "explicit unsupported and downgrade cases",
        "One abstract backend interface must not hide material differences",
    ] {
        assert!(direction.contains(token), "future direction page missing token: {token}");
    }
}

#[test]
fn entry_points_and_boundary_docs_route_promotion_questions_to_future_direction() {
    for path in [
        "README.md",
        "docs/index.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/foundation/release-boundary.md",
        "docs/bijux-dag/foundation/scope-and-boundaries.md",
        "docs/bijux-dag/quality/known-limitations.md",
        "docs/bijux-dag/interfaces/support-matrix.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("future-direction.md") || content.contains("Future Direction"),
            "{path} must route capability promotion questions to Future Direction"
        );
    }
}
