use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_internal_json_owned_allow_failure(args: Vec<String>, cwd: &Path) -> (i32, Value, String) {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) =
        support::run_dag_command_with_env(&refs, cwd, &[("BIJUX_DAG_ENABLE_INTERNAL", "1")]);
    let payload = serde_json::from_str(&stdout).expect("parse json envelope");
    (code, payload, stderr)
}

fn write_basic_run_snapshot(path: &Path, run_id: &str, selected_nodes: &[&str]) {
    fs::write(
        path.join("run.snapshot.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id": run_id,
            "graph_snapshot_path": "graph.snapshot.json",
            "planner_config": "{}",
            "scheduler_config": "{}",
            "policy_config": "{}",
            "provenance": "{}",
            "submission_source": "manual",
            "trigger_source": "cli",
            "operator": "ops",
            "labels": [],
            "parent_run_id": null,
            "requested_selectors": [],
            "selected_nodes": selected_nodes,
            "dependency_closure_enabled": true,
            "replay_source_run_id": null,
            "partial_rerun_contract": null
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
}

#[test]
fn runtime_repair_apply_recovers_manifest_and_event_index_without_rerun_plan() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let run_dir = temp.path().join("run-metadata-only");
    fs::create_dir_all(run_dir.join("nodes").join("extract")).expect("nodes");
    fs::create_dir_all(run_dir.join("outputs")).expect("outputs");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        r#"{"graph":{"nodes":[],"edges":[]},"graph_fingerprint":"fp"}"#,
    )
    .expect("graph snapshot");
    write_basic_run_snapshot(&run_dir, "run-metadata-only", &["extract"]);
    fs::write(
        run_dir.join("nodes").join("extract").join("trace.json"),
        r#"{
          "node_id":"extract",
          "status":"success",
          "started_unix_ms":1,
          "finished_unix_ms":2,
          "attempt":1,
          "fingerprint":"fp-node",
          "adapter_id":"shell",
          "adapter_version":"v1",
          "adapter_outputs_schema_version":"schema/v1"
        }"#,
    )
    .expect("trace");
    fs::write(run_dir.join("run.log.jsonl"), r#"{"event":"run_started","ts":1}"#).expect("run log");
    fs::write(run_dir.join("outputs").join("index.json"), r#"{"files":[]}"#).expect("outputs");

    let (code, repair, stderr) = run_internal_json_owned_allow_failure(
        vec![
            "runtime".to_string(),
            "repair".to_string(),
            "--json".to_string(),
            "--apply".to_string(),
            output_path_string(&run_dir),
        ],
        &root,
    );
    assert_eq!(code, 0, "metadata repair should succeed: stderr={stderr}");
    assert_eq!(repair["ok"], true);
    assert_eq!(repair["data"]["metadata"]["manifest_rewritten"], true);
    assert_eq!(repair["data"]["metadata"]["index_rewritten"], true);
    assert!(run_dir.join("manifest.json").exists());
    assert!(run_dir.join("run-log.index.json").exists());
}

#[test]
fn runtime_repair_apply_rejects_nested_output_root_for_repairable_run() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tempdir");
    let run_dir = temp.path().join("run-source");
    fs::create_dir_all(run_dir.join("nodes").join("publish").join("outputs")).expect("nodes");
    fs::create_dir_all(run_dir.join("outputs")).expect("outputs");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "graph": {
                "spec":"bijux-dag/v0.1",
                "nodes":[
                    {
                        "id":"publish",
                        "kind":"const",
                        "outputs":[{"name":"bulletin","path":"publish/bulletin.md","required":true}],
                        "params":{"value":"ok"}
                    }
                ],
                "edges":[]
            },
            "graph_fingerprint":"fp"
        }))
        .expect("graph"),
    )
    .expect("write graph");
    write_basic_run_snapshot(&run_dir, "run-source", &["publish"]);
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "run_id":"run-source",
            "status":"success",
            "graph_fingerprint":"fp",
            "policy":{"deny_network":false,"deny_env":false,"deny_clock":false,"clean_env":false}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("nodes").join("publish").join("trace.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "node_id":"publish",
            "status":"success",
            "started_unix_ms":1,
            "finished_unix_ms":2,
            "attempt":1,
            "fingerprint":"fp-publish",
            "adapter_id":"const",
            "adapter_version":"v1",
            "adapter_outputs_schema_version":"schema/v1"
        }))
        .expect("trace"),
    )
    .expect("write trace");
    fs::write(run_dir.join("run.log.jsonl"), "{\"event\":\"run_started\",\"ts\":1}\n")
        .expect("write log");
    fs::write(
        run_dir.join("run-log.index.json"),
        serde_json::to_vec_pretty(&vec![serde_json::json!({"event":"run_started","ts":1})])
            .expect("index"),
    )
    .expect("write log index");
    fs::write(run_dir.join("outputs").join("index.json"), "{\"files\":[]}").expect("run outputs");

    let refs = vec![
        "runtime".to_string(),
        "repair".to_string(),
        "--json".to_string(),
        "--apply".to_string(),
        "--out".to_string(),
        output_path_string(&run_dir.join("nested")),
        output_path_string(&run_dir),
    ];
    let refs = refs.iter().map(String::as_str).collect::<Vec<_>>();
    let (code, stdout, stderr) =
        support::run_dag_command_with_env(&refs, &root, &[("BIJUX_DAG_ENABLE_INTERNAL", "1")]);
    assert_eq!(code, 3, "nested repair output should be rejected: stderr={stderr}");
    assert!(stdout.is_empty(), "unexpected stdout for nested output rejection: {stdout}");
}
