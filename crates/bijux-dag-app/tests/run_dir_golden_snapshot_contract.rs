use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

mod support;

use support::{
    collect_run_dir_snapshot, fixture_path_string, fixture_snapshot_path, graph_map_reduce_fixture,
    graph_semantic_map_reduce_fixture, update_or_assert_snapshot, write_graph_fixture,
};

fn dag_bin(cwd: &Path) -> Command {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let mut command = Command::new(cargo_bin);
    command.current_dir(cwd).env("CARGO_TARGET_DIR", cwd.join("artifacts/target")).args([
        "run",
        "--quiet",
        "-p",
        "bijux-dag-cli",
        "--",
    ]);
    command
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let output = dag_bin(cwd).args(args).output().expect("run dag command");
    assert!(output.status.success(), "command failed: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("parse json envelope")
}

fn run_dir_from(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run_dir"))
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__hello_workflow_run_dir_snapshot_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    let graph = root.join("evidence/dag/authoring/examples/hello.dag.json");
    let payload = run_json(
        &[
            "--json",
            "run",
            &fixture_path_string(&graph),
            "--out",
            &fixture_path_string(&out_dir),
            "--run-id",
            "hello-fixed",
        ],
        &root,
    );
    let snapshot = collect_run_dir_snapshot(&run_dir_from(&payload));
    update_or_assert_snapshot(
        &fixture_snapshot_path(env!("CARGO_MANIFEST_DIR"), "tests/snapshots/run_dir_hello.json"),
        &snapshot,
    );
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__cached_branch_workflow_run_dir_snapshot_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let out_dir = temp.path().join("runs");
    let graph = root.join("evidence/dag/authoring/examples/cached-branched-report.dag.json");
    let payload = run_json(
        &[
            "--json",
            "run",
            &fixture_path_string(&graph),
            "--out",
            &fixture_path_string(&out_dir),
            "--run-id",
            "branch-fixed",
        ],
        &root,
    );
    let snapshot = collect_run_dir_snapshot(&run_dir_from(&payload));
    update_or_assert_snapshot(
        &fixture_snapshot_path(
            env!("CARGO_MANIFEST_DIR"),
            "tests/snapshots/run_dir_cached_branch.json",
        ),
        &snapshot,
    );
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__map_reduce_workflow_run_dir_snapshot_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("map_reduce.json");
    write_graph_fixture(&graph_path, &graph_map_reduce_fixture());
    let out_dir = temp.path().join("runs");
    let payload = run_json(
        &[
            "--json",
            "run",
            &fixture_path_string(&graph_path),
            "--out",
            &fixture_path_string(&out_dir),
            "--run-id",
            "map-reduce-fixed",
        ],
        &root,
    );
    let snapshot = collect_run_dir_snapshot(&run_dir_from(&payload));
    update_or_assert_snapshot(
        &fixture_snapshot_path(
            env!("CARGO_MANIFEST_DIR"),
            "tests/snapshots/run_dir_map_reduce.json",
        ),
        &snapshot,
    );
}

#[test]
#[allow(non_snake_case, reason = "nextest slow-tier namespace contract")]
fn slow__semantic_map_reduce_workflow_run_dir_snapshot_is_stable() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let graph_path = temp.path().join("semantic_map_reduce.json");
    write_graph_fixture(&graph_path, &graph_semantic_map_reduce_fixture());
    let out_dir = temp.path().join("runs");
    let payload = run_json(
        &[
            "--json",
            "run",
            &fixture_path_string(&graph_path),
            "--out",
            &fixture_path_string(&out_dir),
            "--run-id",
            "semantic-map-reduce-fixed",
        ],
        &root,
    );
    let snapshot = collect_run_dir_snapshot(&run_dir_from(&payload));
    update_or_assert_snapshot(
        &fixture_snapshot_path(
            env!("CARGO_MANIFEST_DIR"),
            "tests/snapshots/run_dir_semantic_map_reduce.json",
        ),
        &snapshot,
    );
}
