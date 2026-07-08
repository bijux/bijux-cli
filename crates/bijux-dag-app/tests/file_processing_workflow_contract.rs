use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = support::run_dag_command(args, cwd);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_owned(args: Vec<String>, cwd: &Path) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs, cwd)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/file-processing-report.dag.json")
}

fn write_source_dir(root: &Path) -> PathBuf {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).expect("source dir");
    fs::write(source_dir.join("alpha.txt"), "alpha\nbeta\n").expect("alpha");
    fs::write(source_dir.join("beta.txt"), "gamma\ndelta\nepsilon\n").expect("beta");
    fs::write(source_dir.join("gamma.txt"), "zeta\n").expect("gamma");
    source_dir
}

fn report_path(run_dir: &Path) -> PathBuf {
    run_dir.join("nodes").join("render_report").join("outputs").join("report").join("report.md")
}

fn artifact_id_for_report(registry_payload: &Value) -> String {
    registry_payload["data"]["artifacts"]
        .as_array()
        .expect("artifacts array")
        .iter()
        .find(|artifact| {
            artifact["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("nodes/render_report/outputs/report/report.md"))
        })
        .and_then(|artifact| artifact["artifact_id"].as_str())
        .expect("report artifact id")
        .to_string()
}

#[test]
fn file_processing_workflow_uses_runtime_inputs_and_reuses_cache() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = write_source_dir(temp.path());
    let runs_dir = temp.path().join("runs");
    let cache_dir = temp.path().join("cache");
    fs::create_dir_all(&runs_dir).expect("runs dir");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let source_dir_arg = format!("source_dir={}", output_path_string(&source_dir));
    let report_title_arg = "report_title=Clinical Intake Summary".to_string();
    let graph = workflow_graph(&root);

    let first = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "file-processing-first".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            source_dir_arg.clone(),
            "--input".to_string(),
            report_title_arg.clone(),
        ],
        &root,
    );

    let first_run = run_dir_from_response(&first);
    let first_manifest = read_manifest(&first_run);
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["source_dir"],
        output_path_string(&source_dir)
    );
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["report_title"],
        "Clinical Intake Summary"
    );
    assert_eq!(first_manifest["node_counts"]["cached"], 0);

    let first_report = fs::read_to_string(report_path(&first_run)).expect("first report");
    assert!(first_report.contains("# Clinical Intake Summary"));
    assert!(first_report.contains("Processed files: 3"));
    assert!(first_report.contains("Total non-empty lines: 6"));

    let second = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "file-processing-second".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            source_dir_arg,
            "--input".to_string(),
            report_title_arg,
        ],
        &root,
    );

    let second_run = run_dir_from_response(&second);
    let second_manifest = read_manifest(&second_run);
    assert!(
        second_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 3,
        "expected warm cache reuse for validate, transform, and aggregate nodes"
    );

    let second_report = fs::read_to_string(report_path(&second_run)).expect("second report");
    assert_eq!(first_report, second_report);
}

#[test]
fn file_processing_workflow_supports_lineage_partial_rerun_and_promotion() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let source_dir = write_source_dir(temp.path());
    let runs_dir = temp.path().join("runs");
    let cache_dir = temp.path().join("cache");
    let deliverables_dir = temp.path().join("deliverables");
    fs::create_dir_all(&runs_dir).expect("runs dir");
    fs::create_dir_all(&cache_dir).expect("cache dir");
    fs::create_dir_all(&deliverables_dir).expect("deliverables dir");

    let graph = workflow_graph(&root);
    let source = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "file-processing-source".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            format!("source_dir={}", output_path_string(&source_dir)),
            "--input".to_string(),
            "report_title=Lineage Proof Report".to_string(),
        ],
        &root,
    );
    let source_run = run_dir_from_response(&source);

    let registry = run_json_owned(
        vec![
            "artifact".to_string(),
            "registry".to_string(),
            output_path_string(&source_run),
            "--json".to_string(),
        ],
        &root,
    );
    let artifact_id = artifact_id_for_report(&registry);

    let artifact_inspect = run_json_owned(
        vec![
            "artifact-inspect".to_string(),
            output_path_string(&source_run),
            artifact_id.clone(),
            "--json".to_string(),
        ],
        &root,
    );
    assert!(
        artifact_inspect["data"]["lineage"]["upstream_artifact_ids"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "expected final report to expose upstream lineage"
    );

    let lineage = run_json_owned(
        vec![
            "artifact".to_string(),
            "lineage".to_string(),
            output_path_string(&source_run),
            "--json".to_string(),
        ],
        &root,
    );
    assert!(
        lineage["data"]["edge_count"].as_u64().unwrap_or(0) >= 3,
        "expected workflow lineage snapshot to record the file-processing chain"
    );

    let promote = run_json_owned(
        vec![
            "artifact".to_string(),
            "promote".to_string(),
            output_path_string(&source_run),
            artifact_id,
            "--deliverables-root".to_string(),
            output_path_string(&deliverables_dir),
            "--to".to_string(),
            "release".to_string(),
            "--json".to_string(),
        ],
        &root,
    );

    let destination = PathBuf::from(promote["data"]["destination"].as_str().expect("destination"));
    assert!(destination.join("payload").join("report.md").exists());
    assert!(destination.join("promotion.json").exists());

    let promoted_manifest = read_manifest(&source_run);
    assert_eq!(promoted_manifest["run_summary"]["promoted_outputs"][0]["output_name"], "report");

    let replay = run_json_owned(
        vec![
            "replay".to_string(),
            "--json".to_string(),
            "--source-run-id".to_string(),
            "file-processing-source".to_string(),
            "--source-run-root".to_string(),
            output_path_string(&runs_dir),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "file-processing-rerun".to_string(),
            "--from-node".to_string(),
            "render_report".to_string(),
        ],
        &root,
    );
    assert_eq!(replay["data"]["node_rerun_diff"]["node_id"], "render_report");

    let replay_run = run_dir_from_response(&replay);
    let replay_manifest = read_manifest(&replay_run);
    assert_eq!(replay_manifest["run_metadata"]["parent_run_id"], "file-processing-source");
}
