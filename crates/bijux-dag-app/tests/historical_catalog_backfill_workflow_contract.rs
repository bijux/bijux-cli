use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

mod support;

const INTERNAL_ENV: [(&str, &str); 1] = [("BIJUX_DAG_ENABLE_INTERNAL", "1")];

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

fn run_json_with_env(args: &[&str], cwd: &Path, envs: &[(&str, &str)]) -> Value {
    let (code, stdout, stderr) = support::run_dag_command_with_env(args, cwd, envs);
    assert_eq!(code, 0, "command failed: stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_owned(args: Vec<String>, cwd: &Path) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs, cwd)
}

fn run_json_owned_with_env(args: Vec<String>, cwd: &Path, envs: &[(&str, &str)]) -> Value {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json_with_env(&refs, cwd, envs)
}

fn run_dir_from_response(payload: &Value) -> PathBuf {
    PathBuf::from(payload["data"]["run_dir"].as_str().expect("run dir"))
}

fn read_manifest(run_dir: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
        .expect("manifest json")
}

fn workflow_graph(root: &Path) -> PathBuf {
    root.join("evidence/dag/authoring/examples/historical-catalog-backfill.dag.json")
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("json bytes")).expect("write json");
}

fn write_backfill_registry(path: &Path) {
    write_json(
        path,
        &json!({
            "definitions": [
                {
                    "id": "catalog-history",
                    "dag_name": "atlas.catalog-backfill",
                    "dag_version_policy": "run-latest",
                    "input_contract": {
                        "requested_unix_ms": { "type": "integer", "required": true },
                        "backfill_window_start_unix_ms": { "type": "integer", "required": true },
                        "backfill_window_end_unix_ms": { "type": "integer", "required": true },
                        "backfill_partition_key": { "type": "string", "required": true },
                        "catalog_name": { "type": "string", "default": "atlas.catalog" },
                        "publication_title": { "type": "string", "default": "Historical Catalog Backfill" }
                    },
                    "input_bindings": {
                        "requested_unix_ms": { "source": "requested_unix_ms" },
                        "backfill_window_start_unix_ms": { "source": "backfill_window_start_unix_ms" },
                        "backfill_window_end_unix_ms": { "source": "backfill_window_end_unix_ms" },
                        "backfill_partition_key": { "source": "backfill_partition_key" }
                    },
                    "trigger": {
                        "Backfill": {
                            "window_start_unix_ms": 1704067200000u64,
                            "window_end_unix_ms": 1704067320000u64,
                            "partition_by": "region",
                            "partition_keys": ["north-america", "europe"],
                            "max_parallelism": 2,
                            "failure_policy": "pause"
                        }
                    },
                    "queue": { "queue_name": "catalog-backfill", "tenant": "atlas" },
                    "priority": "High",
                    "concurrency": {
                        "per_dag": 2,
                        "per_queue": 4,
                        "per_tenant": 4,
                        "per_node_group": null
                    },
                    "catch_up": { "enabled": false, "max_catch_up_runs": 0 }
                }
            ]
        }),
    );
}

fn append_graph_inputs(args: &mut Vec<String>, graph_inputs: &Value) {
    let object = graph_inputs.as_object().expect("graph inputs object");
    for (key, value) in object {
        args.push("--input".to_string());
        args.push(format!("{key}={}", serde_json::to_string(value).expect("input literal")));
    }
}

#[test]
fn historical_catalog_backfill_proves_partition_fanout_retry_and_summary() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = temp.path().join("backfill-registry.json");
    let state = temp.path().join("backfill-state.json");
    let summary = temp.path().join("backfill-summary.json");
    let advance_request = temp.path().join("backfill-advance-request.json");
    let dispatched_state = temp.path().join("backfill-dispatched.json");
    let failed_request = temp.path().join("backfill-failed-request.json");
    let failed_state = temp.path().join("backfill-failed.json");
    let retried_state = temp.path().join("backfill-retried.json");
    let retried_summary = temp.path().join("backfill-retried-summary.json");
    let retry_dispatch_state = temp.path().join("backfill-retry-dispatched.json");
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_backfill_registry(&registry);
    write_json(
        &advance_request,
        &json!({
            "now_unix_ms": 1704067105000u64,
            "pending_live_runs": 0,
            "throttling_policy": {
                "max_backfill_submissions_per_tick": 6,
                "reserve_live_capacity_percent": 0
            },
            "status_updates": []
        }),
    );

    let validate_schedule = run_json_with_env(
        &["--json", "schedule", "validate", registry.to_string_lossy().as_ref()],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(validate_schedule["ok"], true);

    let validate_graph =
        run_json(&["validate", "--json", workflow_graph(&root).to_string_lossy().as_ref()], &root);
    assert_eq!(validate_graph["ok"], true);

    let plan = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "plan",
            registry.to_string_lossy().as_ref(),
            "--schedule-id",
            "catalog-history",
            "--planned-unix-ms",
            "1704067100000",
            "--backfill-id",
            "catalog-history-january",
            "--out",
            state.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(plan["ok"], true);
    assert_eq!(plan["data"]["runs"].as_array().map(Vec::len), Some(6));
    assert_eq!(plan["data"]["runs"][0]["partition_key"], "north-america");
    assert_eq!(plan["data"]["runs"][1]["partition_key"], "europe");
    assert_eq!(plan["data"]["runs"][2]["requested_unix_ms"], 1704067260000u64);

    let initial_summary = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "summary",
            state.to_string_lossy().as_ref(),
            "--out",
            summary.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(initial_summary["ok"], true);
    assert_eq!(initial_summary["data"]["queued_runs"], 6);
    assert_eq!(initial_summary["data"]["total_retry_attempts"], 0);
    assert_eq!(initial_summary["data"]["partitions"].as_array().map(Vec::len), Some(6));

    let dispatch = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "advance",
            state.to_string_lossy().as_ref(),
            advance_request.to_string_lossy().as_ref(),
            "--out",
            dispatched_state.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(dispatch["ok"], true);
    assert_eq!(dispatch["data"]["dispatched_requests"].as_array().map(Vec::len), Some(2));

    let first_request = dispatch["data"]["dispatched_requests"][0].clone();
    assert_eq!(first_request["graph_inputs"]["requested_unix_ms"], 1704067200000u64);
    assert_eq!(first_request["graph_inputs"]["backfill_window_start_unix_ms"], 1704067200000u64);
    assert_eq!(first_request["graph_inputs"]["backfill_window_end_unix_ms"], 1704067320000u64);
    assert_eq!(first_request["graph_inputs"]["backfill_partition_key"], "north-america");

    let mut first_run_args = vec![
        "run".to_string(),
        "--json".to_string(),
        output_path_string(&workflow_graph(&root)),
        "--out".to_string(),
        output_path_string(&runs_dir),
        "--run-id".to_string(),
        first_request["run_id"].as_str().expect("first run id").to_string(),
    ];
    append_graph_inputs(&mut first_run_args, &first_request["graph_inputs"]);
    let first_run = run_json_owned(first_run_args, &root);

    let first_run_dir = run_dir_from_response(&first_run);
    let first_manifest = read_manifest(&first_run_dir);
    assert_eq!(first_manifest["status"], "success");
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["requested_unix_ms"],
        1704067200000u64
    );
    assert_eq!(
        first_manifest["run_metadata"]["graph_inputs"]["backfill_partition_key"],
        "north-america"
    );
    let first_report = fs::read_to_string(
        first_run_dir
            .join("nodes")
            .join("render_partition_report")
            .join("outputs")
            .join("publish")
            .join("report.md"),
    )
    .expect("first report");
    assert!(first_report.contains("# Historical Catalog Backfill"));
    assert!(first_report.contains("Catalog: atlas.catalog"));
    assert!(first_report.contains("Partition: north-america"));
    assert!(first_report.contains("Requested slot: 2024-01-01T00:00:00Z"));
    assert!(first_report.contains("Window start: 2024-01-01T00:00:00Z"));
    assert!(first_report.contains("Window end: 2024-01-01T00:02:00Z"));

    let verify_first = run_json(
        &["verify", "--json", output_path_string(&first_run_dir).as_str(), "--strict"],
        &root,
    );
    assert_eq!(verify_first["ok"], true);

    let failed_run_id = dispatch["data"]["dispatched_requests"][1]["run_id"]
        .as_str()
        .expect("failed run id")
        .to_string();
    write_json(
        &failed_request,
        &json!({
            "now_unix_ms": 1704067110000u64,
            "pending_live_runs": 0,
            "throttling_policy": {
                "max_backfill_submissions_per_tick": 6,
                "reserve_live_capacity_percent": 0
            },
            "status_updates": [
                {
                    "run_id": failed_run_id,
                    "status": "failed",
                    "updated_unix_ms": 1704067109000u64
                }
            ]
        }),
    );

    let failed = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "advance",
            dispatched_state.to_string_lossy().as_ref(),
            failed_request.to_string_lossy().as_ref(),
            "--out",
            failed_state.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(failed["ok"], true);
    assert_eq!(failed["data"]["operation"]["lifecycle"], "paused");
    assert_eq!(failed["data"]["dispatched_requests"].as_array().map(Vec::len), Some(0));

    let retry = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "retry-failed",
            failed_state.to_string_lossy().as_ref(),
            "--at-unix-ms",
            "1704067115000",
            "--out",
            retried_state.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(retry["ok"], true);
    assert_eq!(retry["data"]["lifecycle"], "active");

    let retried_partition = retry["data"]["runs"]
        .as_array()
        .expect("retried runs")
        .iter()
        .find(|run| {
            run["partition_key"] == "europe"
                && run["requested_unix_ms"] == 1704067200000u64
                && run["attempt"] == 2
        })
        .expect("retried partition")
        .clone();
    assert_eq!(retried_partition["previous_run_ids"].as_array().map(Vec::len), Some(1));
    assert_eq!(retried_partition["previous_run_ids"][0], json!(failed_run_id));

    let summary_after_retry = run_json_with_env(
        &[
            "--json",
            "schedule",
            "backfill",
            "summary",
            retried_state.to_string_lossy().as_ref(),
            "--out",
            retried_summary.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(summary_after_retry["ok"], true);
    assert_eq!(summary_after_retry["data"]["submitted_runs"], 1);
    assert_eq!(summary_after_retry["data"]["queued_runs"], 5);
    assert_eq!(summary_after_retry["data"]["failed_runs"], 0);
    assert_eq!(summary_after_retry["data"]["total_retry_attempts"], 1);

    let retry_dispatch = run_json_owned_with_env(
        vec![
            "--json".to_string(),
            "schedule".to_string(),
            "backfill".to_string(),
            "advance".to_string(),
            output_path_string(&retried_state),
            output_path_string(&advance_request),
            "--out".to_string(),
            output_path_string(&retry_dispatch_state),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(retry_dispatch["ok"], true);
    assert_eq!(retry_dispatch["data"]["dispatched_requests"].as_array().map(Vec::len), Some(1));

    let retry_request = retry_dispatch["data"]["dispatched_requests"][0].clone();
    assert_eq!(retry_request["run_id"], retried_partition["run_id"]);
    assert_eq!(retry_request["graph_inputs"]["backfill_partition_key"], "europe");
    assert_eq!(retry_request["graph_inputs"]["requested_unix_ms"], 1704067200000u64);

    let mut retry_run_args = vec![
        "run".to_string(),
        "--json".to_string(),
        output_path_string(&workflow_graph(&root)),
        "--out".to_string(),
        output_path_string(&runs_dir),
        "--run-id".to_string(),
        retry_request["run_id"].as_str().expect("retry run id").to_string(),
    ];
    append_graph_inputs(&mut retry_run_args, &retry_request["graph_inputs"]);
    let retry_run = run_json_owned(retry_run_args, &root);

    let retry_run_dir = run_dir_from_response(&retry_run);
    let retry_manifest = read_manifest(&retry_run_dir);
    assert_eq!(retry_manifest["status"], "success");
    assert_eq!(retry_manifest["run_metadata"]["graph_inputs"]["backfill_partition_key"], "europe");
    let retry_report = fs::read_to_string(
        retry_run_dir
            .join("nodes")
            .join("render_partition_report")
            .join("outputs")
            .join("publish")
            .join("report.md"),
    )
    .expect("retry report");
    assert!(retry_report.contains("Partition: europe"));
    assert!(retry_report.contains("Requested slot: 2024-01-01T00:00:00Z"));

    let verify_retry = run_json(
        &["verify", "--json", output_path_string(&retry_run_dir).as_str(), "--strict"],
        &root,
    );
    assert_eq!(verify_retry["ok"], true);
}
