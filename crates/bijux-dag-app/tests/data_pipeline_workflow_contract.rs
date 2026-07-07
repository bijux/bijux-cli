use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::ffi::OsStr;
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

fn read_trace(run_dir: &Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json")).expect("trace"),
    )
    .expect("trace json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/regional-sales-pipeline.dag.json")
}

fn copy_workflow_inputs(root: &Path, destination: &Path) -> (PathBuf, PathBuf) {
    let source_dir = root.join("evidence/dag/authoring/examples/regional-sales-source");
    fs::create_dir_all(destination).expect("inputs dir");
    let orders_csv = destination.join("orders.csv");
    let targets_json = destination.join("targets.json");
    fs::copy(source_dir.join("orders.csv"), &orders_csv).expect("copy orders");
    fs::copy(source_dir.join("targets.json"), &targets_json).expect("copy targets");
    (orders_csv, targets_json)
}

fn find_unique_file(root: &Path, file_name: &str) -> PathBuf {
    let mut stack = vec![root.to_path_buf()];
    let mut matches = Vec::new();
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(path).expect("read dir") {
            let entry = entry.expect("dir entry");
            let entry_path = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path.file_name() == Some(OsStr::new(file_name)) {
                matches.push(entry_path);
            }
        }
    }
    assert_eq!(matches.len(), 1, "expected exactly one {file_name} under {}", root.display());
    matches.pop().expect("unique match")
}

#[test]
fn regional_sales_pipeline_renders_final_table_and_reuses_cache() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let (orders_csv, targets_json) = copy_workflow_inputs(&root, &temp.path().join("inputs"));
    let runs_dir = temp.path().join("runs");
    let cache_dir = temp.path().join("cache");
    fs::create_dir_all(&runs_dir).expect("runs dir");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    let graph = workflow_graph(&root);
    let orders_arg = format!("orders_csv={}", output_path_string(&orders_csv));
    let targets_arg = format!("targets_json={}", output_path_string(&targets_json));
    let title_arg = "report_title=Regional Revenue Attainment".to_string();

    let first = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "regional-sales-cold".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            orders_arg.clone(),
            "--input".to_string(),
            targets_arg.clone(),
            "--input".to_string(),
            title_arg.clone(),
        ],
        &root,
    );

    let first_run = run_dir_from_response(&first);
    let first_manifest = read_manifest(&first_run);
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["orders_csv"],
        output_path_string(&orders_csv)
    );
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["targets_json"],
        output_path_string(&targets_json)
    );
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["report_title"],
        "Regional Revenue Attainment"
    );
    assert_eq!(first_manifest["node_counts"]["cached"], 0);

    let first_table = fs::read_to_string(find_unique_file(&first_run, "revenue_attainment.csv"))
        .expect("first final table");
    assert!(first_table.contains("report_title,region,revenue,target,variance,status"));
    assert!(first_table.contains("Regional Revenue Attainment,North,170.00,150.00,20.00,above-target"));
    assert!(first_table.contains("Regional Revenue Attainment,South,80.00,70.00,10.00,above-target"));
    assert!(first_table.contains("Regional Revenue Attainment,West,63.00,50.00,13.00,above-target"));

    let second = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "regional-sales-warm".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            orders_arg,
            "--input".to_string(),
            targets_arg,
            "--input".to_string(),
            title_arg,
        ],
        &root,
    );

    let second_run = run_dir_from_response(&second);
    let second_manifest = read_manifest(&second_run);
    assert!(
        second_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 7,
        "expected all data-pipeline stages to reuse cache on the warm run"
    );

    for node_id in [
        "ingest_orders",
        "clean_orders",
        "derive_region_totals",
        "derive_segment_totals",
        "load_targets",
        "validate_outputs",
        "publish_final_table",
    ] {
        assert_eq!(
            read_trace(&second_run, node_id)["status"],
            "cached",
            "expected {node_id} to be reused from cache on the warm run"
        );
    }

    let second_table = fs::read_to_string(find_unique_file(&second_run, "revenue_attainment.csv"))
        .expect("second final table");
    assert_eq!(first_table, second_table);
}
