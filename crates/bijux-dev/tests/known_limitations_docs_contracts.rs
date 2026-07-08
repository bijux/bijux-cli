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
fn known_limitations_page_covers_backlog_sections_and_current_scope_records() {
    let limitations = read_repo_file("docs/bijux-dag/quality/known-limitations.md");

    for token in [
        "## Stable Local Execution Limitations",
        "## Shell Isolation Limitations",
        "## Container Limitations",
        "## Scheduling Limitations",
        "## Remote/Distributed Limitations",
        "## API Stability Limitations",
        "## Cache/Replay Limitations",
        "### LIM-007 Stable local execution remains a single-controller runtime",
        "### LIM-008 Internal schedule and backfill lanes are not stable scheduler APIs",
        "### LIM-009 Remote coordination and batch backends are modeled, not shipped",
        "### LIM-010 Cache and replay proof depends on exact retained evidence",
        "submitters into the local `bijux-dag run` surface",
        "the schedule namespace remains internal throughout `v0.4.x`",
        "remote and distributed execution remain outside the stable",
        "`export --with-files`",
        "`manifest-only`",
        "`without-artifacts`",
    ] {
        assert!(limitations.contains(token), "known limitations page missing token: {token}");
    }
}

#[test]
fn release_boundary_and_scope_pages_route_readers_to_known_limitations() {
    for path in [
        "docs/bijux-dag/foundation/release-boundary.md",
        "docs/bijux-dag/interfaces/reference/support-matrix.md",
        "docs/bijux-dag/architecture/reference/local-only-vs-remote-coordinated-runtime.md",
        "docs/bijux-dag/architecture/reference/local-vs-batch-execution-constraints.md",
        "docs/bijux-dag/operations/common-workflows.md",
        "docs/bijux-dag/operations/performance-and-scaling.md",
    ] {
        let content = read_repo_file(path);
        assert!(
            content.contains("known-limitations.md") || content.contains("Known Limitations"),
            "{path} must route release-boundary readers to known limitations"
        );
    }
}
