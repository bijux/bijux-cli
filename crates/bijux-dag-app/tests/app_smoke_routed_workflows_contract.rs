use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

use support::{graph_chain, graph_diamond};

fn run_ok(args: &[String]) {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let matches = dag_command().try_get_matches_from(refs).expect("parse command");
    let code = dag_run(&matches).expect("run command");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

fn run_ok_with_internal_lane(args: &[String]) {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let matches = dag_command().try_get_matches_from(refs).expect("parse command");
    let previous = std::env::var_os("BIJUX_DAG_ENABLE_INTERNAL");
    std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", "1");
    let result = dag_run(&matches);
    if let Some(value) = previous {
        std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", value);
    } else {
        std::env::remove_var("BIJUX_DAG_ENABLE_INTERNAL");
    }
    let code = result.expect("run command");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

fn write_graph(path: &Path, graph: &bijux_dag_core::Graph) {
    let payload = serde_json::to_vec_pretty(graph).expect("serialize graph");
    fs::write(path, payload).expect("write graph");
}

fn first_run_dir(root: &Path) -> PathBuf {
    let mut runs: Vec<PathBuf> = fs::read_dir(root)
        .expect("read runs root")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();
    runs.sort();
    runs.pop().expect("at least one run dir")
}

fn run_alias_of(run_dir: &Path) -> String {
    run_dir.file_name().and_then(|v| v.to_str()).unwrap_or("run").to_string()
}

fn first_artifact_ref(run_dir: &Path) -> (String, PathBuf) {
    let raw = fs::read_to_string(run_dir.join("outputs/index.json")).expect("read outputs index");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("parse outputs index");
    let file = value["files"]
        .as_array()
        .and_then(|items| items.first())
        .expect("at least one output file");
    let node_id = file["node_id"].as_str().expect("node id");
    let path = file["path"].as_str().expect("path");
    let name = Path::new(path).file_name().and_then(|v| v.to_str()).expect("artifact file name");
    (format!("{node_id}:{name}"), run_dir.join(path))
}

#[test]
fn smoke_validate_plan_run_inspect_replay_diff() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("graph.json");
    let runs_root = tmp.path().join("runs");
    fs::create_dir_all(&runs_root).expect("create runs root");
    write_graph(&graph_path, &graph_chain());

    run_ok(&["bijux-dag".to_string(), "validate".to_string(), graph_path.display().to_string()]);
    run_ok(&[
        "bijux-dag".to_string(),
        "plan".to_string(),
        "explain".to_string(),
        graph_path.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "run".to_string(),
        graph_path.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);

    let first = first_run_dir(&runs_root);
    run_ok(&["bijux-dag".to_string(), "status".to_string(), first.display().to_string()]);
    run_ok(&[
        "bijux-dag".to_string(),
        "replay".to_string(),
        first.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);
    let second = first_run_dir(&runs_root);
    run_ok(&[
        "bijux-dag".to_string(),
        "diff".to_string(),
        first.display().to_string(),
        second.display().to_string(),
    ]);
}

#[test]
fn smoke_artifact_hash_inspect_trace() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("graph.json");
    let runs_root = tmp.path().join("runs");
    fs::create_dir_all(&runs_root).expect("create runs root");
    write_graph(&graph_path, &graph_chain());
    run_ok(&[
        "bijux-dag".to_string(),
        "run".to_string(),
        graph_path.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);
    let run_dir = first_run_dir(&runs_root);
    let (artifact_ref, artifact_path) = first_artifact_ref(&run_dir);
    run_ok(&[
        "bijux-dag".to_string(),
        "hash".to_string(),
        "artifact".to_string(),
        artifact_path.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "artifact-inspect".to_string(),
        run_dir.display().to_string(),
        artifact_ref.clone(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "trace-artifact".to_string(),
        run_dir.display().to_string(),
        artifact_ref,
    ]);
}

#[test]
fn smoke_export_import_verify_only_and_fsck() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("graph.json");
    let runs_root = tmp.path().join("runs");
    let bundle = tmp.path().join("bundle.json");
    fs::create_dir_all(&runs_root).expect("create runs root");
    write_graph(&graph_path, &graph_chain());
    run_ok(&[
        "bijux-dag".to_string(),
        "run".to_string(),
        graph_path.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);
    let run_dir = first_run_dir(&runs_root);
    run_ok(&[
        "bijux-dag".to_string(),
        "export".to_string(),
        run_dir.display().to_string(),
        "--out".to_string(),
        bundle.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "import".to_string(),
        bundle.display().to_string(),
        "--verify-only".to_string(),
    ]);
    run_ok(&["bijux-dag".to_string(), "fsck".to_string(), run_dir.display().to_string()]);
}

#[test]
fn smoke_history_show_summary_timeline() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("graph.json");
    let runs_root = tmp.path().join("runs");
    fs::create_dir_all(&runs_root).expect("create runs root");
    write_graph(&graph_path, &graph_diamond());
    run_ok(&[
        "bijux-dag".to_string(),
        "run".to_string(),
        graph_path.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);
    let run_dir = first_run_dir(&runs_root);
    let run_id = run_alias_of(&run_dir);
    run_ok(&[
        "bijux-dag".to_string(),
        "runs".to_string(),
        "history".to_string(),
        "--root".to_string(),
        runs_root.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "runs".to_string(),
        "show".to_string(),
        run_id.clone(),
        "--root".to_string(),
        runs_root.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "runs".to_string(),
        "summary".to_string(),
        "--root".to_string(),
        runs_root.display().to_string(),
    ]);
    run_ok(&[
        "bijux-dag".to_string(),
        "runs".to_string(),
        "timeline".to_string(),
        run_id,
        "--root".to_string(),
        runs_root.display().to_string(),
    ]);
}

#[test]
fn smoke_prove_verify_and_surface_queries() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let graph_path = tmp.path().join("graph.json");
    let runs_root = tmp.path().join("runs");
    fs::create_dir_all(&runs_root).expect("create runs root");
    write_graph(&graph_path, &graph_chain());
    run_ok(&[
        "bijux-dag".to_string(),
        "run".to_string(),
        graph_path.display().to_string(),
        "--out".to_string(),
        runs_root.display().to_string(),
    ]);
    let run_dir = first_run_dir(&runs_root);
    run_ok(&["bijux-dag".to_string(), "prove".to_string(), run_dir.display().to_string()]);
    run_ok(&["bijux-dag".to_string(), "verify".to_string(), run_dir.display().to_string()]);
    run_ok_with_internal_lane(&[
        "bijux-dag".to_string(),
        "semantic-portability".to_string(),
        "--backend".to_string(),
        "hpc".to_string(),
    ]);
    run_ok_with_internal_lane(&[
        "bijux-dag".to_string(),
        "capabilities".to_string(),
        "--backend".to_string(),
        "hpc".to_string(),
    ]);
}
