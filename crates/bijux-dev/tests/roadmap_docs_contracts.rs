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
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

fn read_repo_file(path: &str) -> String {
    let absolute = workspace_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn roadmap_page_covers_post_v040_release_lanes_without_blurring_current_boundary() {
    let roadmap = read_repo_file("docs/tracking/bijux-dag-roadmap.md");

    for token in [
        "# Bijux Dag Roadmap",
        "## Release Ladder",
        "## v0.4.x Hardening",
        "## v0.5 Graph Expressiveness",
        "## v0.6 Scheduling and Backfill",
        "## v0.7 Remote Workers",
        "## v0.8 HPC and Kubernetes",
        "## v1.0 Stable API",
        "Release Boundary",
        "Known Limitations",
        "local-first",
        "distributed scheduler",
        "Promotion Rule",
        "the narrower claim wins",
    ] {
        assert!(roadmap.contains(token), "roadmap page missing token: {token}");
    }
}

#[test]
fn entry_points_and_boundary_docs_route_future_release_questions_to_roadmap() {
    for path in [
        "README.md",
        "docs/index.md",
        "docs/bijux-dag/index.md",
        "docs/bijux-dag/foundation/release-boundary.md",
        "docs/bijux-dag/foundation/scope-and-non-goals.md",
        "docs/bijux-dag/quality/known-limitations.md",
        "docs/bijux-dag/interfaces/reference/support-matrix.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("bijux-dag-roadmap.md") || content.contains("Bijux Dag Roadmap"),
            "{path} must route future release questions to the roadmap"
        );
    }
}
