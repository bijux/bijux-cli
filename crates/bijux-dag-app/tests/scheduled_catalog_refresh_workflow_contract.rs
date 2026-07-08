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
    root.join("evidence/dag/authoring/examples/scheduled-catalog-refresh.dag.json")
}

fn write_json(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).expect("json bytes")).expect("write json");
}

fn write_schedule_registry(path: &Path) {
    write_json(
        path,
        &json!({
            "definitions": [
                {
                    "id": "catalog-refresh-hourly",
                    "dag_name": "atlas.catalog-refresh",
                    "dag_version_policy": "run-latest",
                    "input_contract": {
                        "scheduled_at_unix_ms": { "type": "integer", "required": true },
                        "refresh_label": { "type": "string", "default": "Nightly Catalog Refresh" },
                        "dataset_name": { "type": "string", "default": "atlas.catalog" }
                    },
                    "input_bindings": {
                        "scheduled_at_unix_ms": { "source": "requested_unix_ms" }
                    },
                    "trigger": {
                        "Cron": {
                            "expression": "0 * * * *",
                            "timezone": "UTC"
                        }
                    },
                    "queue": { "queue_name": "catalog-refresh", "tenant": "atlas" },
                    "priority": "High",
                    "concurrency": {
                        "per_dag": 1,
                        "per_queue": 2,
                        "per_tenant": 2,
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
fn scheduled_catalog_refresh_proves_preview_dedup_and_run_linkage() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let registry = temp.path().join("schedule-registry.json");
    let submit_inputs = temp.path().join("schedule-inputs.json");
    let first_ledger = temp.path().join("schedule-ledger.json");
    let duplicate_ledger = temp.path().join("schedule-ledger-duplicate.json");
    let queue_state = temp.path().join("queue-state.json");
    let dispatched_ledger = temp.path().join("schedule-ledger-dispatched.json");
    let completed_ledger = temp.path().join("schedule-ledger-completed.json");
    let status_updates = temp.path().join("schedule-status-updates.json");
    let runs_dir = temp.path().join("runs");
    fs::create_dir_all(&runs_dir).expect("runs dir");

    write_schedule_registry(&registry);
    write_json(&submit_inputs, &json!({ "now_unix_ms": 1768474800000u64 }));

    let validate = run_json_with_env(
        &["--json", "schedule", "validate", registry.to_string_lossy().as_ref()],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(validate["ok"], true);

    let preview = run_json_with_env(
        &[
            "--json",
            "schedule",
            "preview",
            registry.to_string_lossy().as_ref(),
            "--now-unix-ms",
            "1768473000000",
            "--next-runs",
            "1",
        ],
        &root,
        &INTERNAL_ENV,
    );
    let preview_entry = &preview["data"]["previews"][0];
    assert_eq!(preview_entry["schedule_id"], "catalog-refresh-hourly");
    assert_eq!(preview_entry["preview"]["next_fire_unix_ms"], 1768474800000u64);
    assert_eq!(preview_entry["materialized_runs"]["schedule_id"], "catalog-refresh-hourly");
    assert_eq!(preview_entry["materialized_runs"]["next_run_unix_ms"][0], 1768474800000u64);

    let submit = run_json_with_env(
        &[
            "--json",
            "schedule",
            "submit",
            registry.to_string_lossy().as_ref(),
            submit_inputs.to_string_lossy().as_ref(),
            "--out",
            first_ledger.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(submit["ok"], true);
    assert_eq!(submit["data"]["generated_requests"].as_array().map(Vec::len), Some(1));
    assert_eq!(submit["data"]["duplicate_suppressions"].as_array().map(Vec::len), Some(0));

    let request = &submit["data"]["generated_requests"][0];
    let scheduled_run_id = request["run_id"].as_str().expect("run id").to_string();
    assert_eq!(request["graph_inputs"]["scheduled_at_unix_ms"], 1768474800000u64);
    assert_eq!(request["graph_inputs"]["refresh_label"], "Nightly Catalog Refresh");
    assert_eq!(request["graph_inputs"]["dataset_name"], "atlas.catalog");

    let duplicate = run_json_with_env(
        &[
            "--json",
            "schedule",
            "submit",
            registry.to_string_lossy().as_ref(),
            submit_inputs.to_string_lossy().as_ref(),
            "--ledger",
            first_ledger.to_string_lossy().as_ref(),
            "--out",
            duplicate_ledger.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(duplicate["ok"], true);
    assert_eq!(duplicate["data"]["generated_requests"].as_array().map(Vec::len), Some(0));
    assert_eq!(duplicate["data"]["recorded_submissions"].as_array().map(Vec::len), Some(1));

    let queue = run_json_with_env(
        &[
            "--json",
            "schedule",
            "queue",
            "status",
            registry.to_string_lossy().as_ref(),
            "--ledger",
            first_ledger.to_string_lossy().as_ref(),
            "--out",
            queue_state.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    let queue_entry = &queue["data"]["queue_state"]["queues"][0];
    assert_eq!(queue_entry["queue_name"], "catalog-refresh");
    assert_eq!(queue_entry["active_runs"], 1);
    assert_eq!(queue_entry["runs"][0]["run_id"], scheduled_run_id);
    assert_eq!(queue_entry["runs"][0]["status"], "Pending");

    let dispatch = run_json_with_env(
        &[
            "--json",
            "schedule",
            "queue",
            "dispatch",
            first_ledger.to_string_lossy().as_ref(),
            "--max-dispatches",
            "1",
            "--out",
            dispatched_ledger.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(dispatch["ok"], true);
    assert_eq!(
        dispatch["data"]["dispatch_report"]["dispatched_runs"][0]["run_id"],
        scheduled_run_id
    );
    assert_eq!(dispatch["data"]["updated_ledger"]["entries"][0]["status"], "Running");

    let validate_graph =
        run_json(&["validate", "--json", workflow_graph(&root).to_string_lossy().as_ref()], &root);
    assert_eq!(validate_graph["ok"], true);

    let mut run_args = vec![
        "run".to_string(),
        "--json".to_string(),
        output_path_string(&workflow_graph(&root)),
        "--out".to_string(),
        output_path_string(&runs_dir),
        "--run-id".to_string(),
        scheduled_run_id.clone(),
    ];
    append_graph_inputs(&mut run_args, &request["graph_inputs"]);
    let run = run_json_owned(run_args, &root);

    let run_dir = run_dir_from_response(&run);
    let manifest = read_manifest(&run_dir);
    assert_eq!(manifest["run_id"], scheduled_run_id);
    assert_eq!(manifest["status"], "success");
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["scheduled_at_unix_ms"], 1768474800000u64);
    assert_eq!(
        manifest["run_metadata"]["graph_inputs"]["refresh_label"],
        "Nightly Catalog Refresh"
    );
    assert_eq!(manifest["run_metadata"]["graph_inputs"]["dataset_name"], "atlas.catalog");

    let report = fs::read_to_string(
        run_dir
            .join("nodes")
            .join("render_refresh_report")
            .join("outputs")
            .join("publish")
            .join("report.md"),
    )
    .expect("scheduled report");
    assert!(report.contains("# Nightly Catalog Refresh"));
    assert!(report.contains("Dataset: atlas.catalog"));
    assert!(report.contains("Scheduled at: 2026-01-15T11:00:00Z"));
    assert!(report.contains("Scheduled at unix ms: 1768474800000"));

    let verify =
        run_json(&["verify", "--json", output_path_string(&run_dir).as_str(), "--strict"], &root);
    assert_eq!(verify["ok"], true);

    write_json(
        &status_updates,
        &json!({
            "updates": [
                {
                    "run_id": scheduled_run_id,
                    "status": "Completed",
                    "updated_unix_ms": 1768475100000u64
                }
            ]
        }),
    );
    let update = run_json_with_env(
        &[
            "--json",
            "schedule",
            "queue",
            "update",
            dispatched_ledger.to_string_lossy().as_ref(),
            status_updates.to_string_lossy().as_ref(),
            "--out",
            completed_ledger.to_string_lossy().as_ref(),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(update["ok"], true);
    assert_eq!(update["data"]["updated_ledger"]["entries"][0]["status"], "Completed");

    let drained = run_json_owned_with_env(
        vec![
            "--json".to_string(),
            "schedule".to_string(),
            "queue".to_string(),
            "status".to_string(),
            output_path_string(&registry),
            "--ledger".to_string(),
            output_path_string(&completed_ledger),
        ],
        &root,
        &INTERNAL_ENV,
    );
    assert_eq!(drained["data"]["queue_state"]["queues"][0]["active_runs"], 0);
    assert_eq!(
        drained["data"]["queue_state"]["queues"][0]["runs"].as_array().map(Vec::len),
        Some(0)
    );
}
