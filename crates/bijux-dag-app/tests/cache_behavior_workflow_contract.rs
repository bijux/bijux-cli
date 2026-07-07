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

fn run_json_with_status(args: &[&str], cwd: &Path) -> (i32, Value) {
    let (code, stdout, stderr) = support::run_dag_command(args, cwd);
    assert!(code == 0 || code == 3, "command failed unexpectedly: code={code} stderr={stderr}");
    (code, serde_json::from_str(&stdout).expect("parse json envelope"))
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

fn corrupt_first_cached_payload(entry_dir: &Path) -> PathBuf {
    let outputs_dir = entry_dir.join("outputs");
    let mut stack = vec![outputs_dir.clone()];
    while let Some(path) = stack.pop() {
        for entry in fs::read_dir(&path).expect("read dir") {
            let entry = entry.expect("dir entry");
            let entry_path = entry.path();
            if entry.file_type().expect("file type").is_dir() {
                stack.push(entry_path);
                continue;
            }
            if entry_path.file_name() == Some(OsStr::new("index.json")) {
                continue;
            }
            fs::write(&entry_path, b"corrupted-cache-payload\n").expect("corrupt payload");
            return entry_path;
        }
    }
    panic!("expected at least one cache payload under {}", outputs_dir.display());
}

fn taxonomy_strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("taxonomy array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

#[test]
fn cache_behavior_workflow_proves_reuse_invalidation_corruption_and_explanation() {
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

    let cold = run_json_owned(
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

    let cold_run = run_dir_from_response(&cold);
    let cold_manifest = read_manifest(&cold_run);
    assert_eq!(cold_manifest["node_counts"]["cached"], 0);
    for node_id in [
        "ingest_orders",
        "clean_orders",
        "derive_region_totals",
        "derive_segment_totals",
        "load_targets",
        "validate_outputs",
        "publish_final_table",
    ] {
        assert_eq!(read_trace(&cold_run, node_id)["status"], "success");
    }

    let warm = run_json_owned(
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
            orders_arg.clone(),
            "--input".to_string(),
            targets_arg.clone(),
            "--input".to_string(),
            title_arg.clone(),
        ],
        &root,
    );

    let warm_run = run_dir_from_response(&warm);
    let warm_manifest = read_manifest(&warm_run);
    assert!(
        warm_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 7,
        "expected the full workflow to reuse cache on the warm run"
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
        assert_eq!(read_trace(&warm_run, node_id)["status"], "cached");
    }

    let warm_table = fs::read_to_string(find_unique_file(&warm_run, "revenue_attainment.csv"))
        .expect("warm table");
    assert!(
        warm_table.contains("Regional Revenue Attainment,North,170.00,150.00,20.00,above-target")
    );

    let original_orders = fs::read_to_string(&orders_csv).expect("orders csv");
    let updated_orders =
        original_orders.replace("A-102,north,mid-market,3,15.00", "A-102,north,mid-market,5,15.00");
    assert_ne!(original_orders, updated_orders, "expected to change one order row");
    let updated_orders_csv = temp.path().join("inputs").join("orders-updated.csv");
    fs::write(&updated_orders_csv, updated_orders).expect("write updated orders");

    let updated = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "regional-sales-updated".to_string(),
            "--cache".to_string(),
            "readwrite".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
            "--input".to_string(),
            format!("orders_csv={}", output_path_string(&updated_orders_csv)),
            "--input".to_string(),
            targets_arg.clone(),
            "--input".to_string(),
            title_arg.clone(),
        ],
        &root,
    );

    let updated_run = run_dir_from_response(&updated);
    let updated_manifest = read_manifest(&updated_run);
    assert!(
        updated_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 1,
        "expected at least one independent branch to stay cached"
    );
    assert_eq!(read_trace(&updated_run, "load_targets")["status"], "cached");
    for node_id in [
        "ingest_orders",
        "clean_orders",
        "derive_region_totals",
        "derive_segment_totals",
        "validate_outputs",
        "publish_final_table",
    ] {
        assert_eq!(
            read_trace(&updated_run, node_id)["status"],
            "success",
            "expected {node_id} to rerun after the orders input changed"
        );
    }

    let updated_miss = run_json_owned(
        vec![
            "--json".to_string(),
            "why-cache-missed".to_string(),
            "--run-dir".to_string(),
            output_path_string(&updated_run),
            "--node".to_string(),
            "clean_orders".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
        ],
        &root,
    );
    assert_eq!(updated_miss["data"]["mode"], "node");
    assert_eq!(updated_miss["data"]["outcome"], "miss");
    assert!(
        taxonomy_strings(&updated_miss["data"]["taxonomy"])
            .iter()
            .any(|item| item == "changed_input_hashes"),
        "expected cache miss explanation to surface changed input hashes"
    );

    let load_targets_key = read_trace(&warm_run, "load_targets")["cache_identity"]["cache_key"]
        .as_str()
        .expect("load_targets cache key")
        .to_string();
    let corrupted_payload = corrupt_first_cached_payload(&cache_dir.join(&load_targets_key));
    assert!(corrupted_payload.exists(), "expected to tamper with an existing cache payload");

    let (verify_code, verify_payload) = run_json_with_status(
        &["--json", "cache", "verify", "--cache-dir", output_path_string(&cache_dir).as_str()],
        &root,
    );
    assert_eq!(verify_code, 3);
    assert!(
        verify_payload["data"]["corrupt_total"].as_u64().unwrap_or(0) >= 1,
        "expected cache verification to classify the tampered entry as corrupt"
    );

    let corrupt_miss = run_json_owned(
        vec![
            "--json".to_string(),
            "why-cache-missed".to_string(),
            "--run-dir".to_string(),
            output_path_string(&warm_run),
            "--node".to_string(),
            "load_targets".to_string(),
            "--cache-dir".to_string(),
            output_path_string(&cache_dir),
        ],
        &root,
    );
    assert_eq!(corrupt_miss["data"]["outcome"], "unsafe_reuse_refused");
    assert_eq!(corrupt_miss["data"]["exact_entry_report"]["eligible"], false);
    assert!(
        !taxonomy_strings(&corrupt_miss["data"]["taxonomy"]).is_empty(),
        "expected corruption explanation taxonomy to be populated"
    );

    let corrupt = run_json_owned(
        vec![
            "run".to_string(),
            "--json".to_string(),
            output_path_string(&graph),
            "--out".to_string(),
            output_path_string(&runs_dir),
            "--run-id".to_string(),
            "regional-sales-corrupt".to_string(),
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

    let corrupt_run = run_dir_from_response(&corrupt);
    let corrupt_manifest = read_manifest(&corrupt_run);
    assert!(
        corrupt_manifest["node_counts"]["cached"].as_u64().unwrap_or(0) >= 5,
        "expected unaffected stages to remain cached after one cache entry was corrupted"
    );
    assert_eq!(read_trace(&corrupt_run, "load_targets")["status"], "success");
    for node_id in
        ["ingest_orders", "clean_orders", "derive_region_totals", "derive_segment_totals"]
    {
        assert_eq!(
            read_trace(&corrupt_run, node_id)["status"],
            "cached",
            "expected {node_id} to remain cached when only the targets cache entry was corrupted"
        );
    }

    let corrupt_table =
        fs::read_to_string(find_unique_file(&corrupt_run, "revenue_attainment.csv"))
            .expect("corrupt table");
    assert_eq!(warm_table, corrupt_table);
}
