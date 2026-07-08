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
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: args={args:?} code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn write_node_fixture(root: &Path) -> PathBuf {
    let run_dir = root.join("run-node");
    fs::create_dir_all(run_dir.join("nodes").join("extract").join("outputs"))
        .expect("mkdir outputs");
    fs::create_dir_all(run_dir.join("nodes").join("extract").join("inputs")).expect("mkdir inputs");
    fs::create_dir_all(run_dir.join("nodes").join("extract").join("attempts").join("1"))
        .expect("mkdir attempt one");
    fs::create_dir_all(run_dir.join("nodes").join("extract").join("attempts").join("2"))
        .expect("mkdir attempt two");

    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-node",
            "created_unix_ms":1,
            "started_unix_ms":1,
            "finished_unix_ms":5,
            "graph_snapshot":"graph.snapshot.json",
            "status":"failed",
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"graph-node",
            "tool_version":"0.4.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":0,"failed":1,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "graph":{
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"inspect-node","owners":[],"tags":[]},
                "inputs":{"dataset_uri":"s3://warehouse/catalog"},
                "nodes":[
                    {
                        "id":"seed",
                        "kind":"const",
                        "inputs":[],
                        "outputs":[{"name":"out","path":"seed/out"}],
                        "params":{"value":"seed"},
                        "cache":{"enabled":true}
                    },
                    {
                        "id":"extract",
                        "kind":"shell",
                        "inputs":["seed_in"],
                        "outputs":[{"name":"out","path":"extract/out","kind":"file","required":true,"media_type":"text/plain"}],
                        "params":{"argv":["/bin/sh","-c","cat {inputs.seed_in} > {outputs.out}"]},
                        "retry":{"max_attempts":2,"backoff_ms":40},
                        "cache":{"enabled":true},
                        "effects":["filesystem"]
                    }
                ],
                "edges":[{"from":{"node_id":"seed","port":"out"},"to":{"node_id":"extract","port":"seed_in"}}]
            },
            "graph_fingerprint":"graph-node"
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run_dir.join("nodes").join("extract").join("trace.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "node_id":"extract",
            "status":"failed",
            "started_unix_ms":1u64,
            "finished_unix_ms":5u64,
            "attempt":2,
            "fingerprint":"fp-extract",
            "adapter_id":"shell",
            "adapter_version":"0.1",
            "adapter_outputs_schema_version":"1",
            "inputs_index":"inputs/index.json",
            "resolved_params":{"argv":["/bin/sh","-c","cat nodes/seed/outputs/out > nodes/extract/outputs/out"]},
            "cache_proof":{"hit":false,"key":"cache-key","source":"local","verified":true,"reason":"fingerprint_changed"},
            "failure":{"class":"execution","kind":"Execution","code":"EXEC_FAIL","message":"shell exited 1"},
            "transition_cause":"ExecutionFailed",
            "lifecycle_state":"failed",
            "outputs":[{
                "name":"out",
                "path":"extract/out",
                "kind":"file",
                "required":true,
                "present":false,
                "media_type":"text/plain"
            }]
        }))
        .expect("trace"),
    )
    .expect("write trace");
    fs::write(
        run_dir.join("nodes").join("extract").join("inputs").join("index.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "files":[{
                "local_path":"seed/out",
                "source_sha256":"seed-sha",
                "source_node_id":"seed",
                "source_node_fingerprint":"seed-fp",
                "source_output_name":"out",
                "materialization_mode":"copy"
            }]
        }))
        .expect("inputs"),
    )
    .expect("write inputs index");
    fs::write(
        run_dir.join("nodes").join("extract").join("outputs").join("index.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "files":[{
                "name":"out",
                "path":"extract/out",
                "kind":"file",
                "media_type":"text/plain",
                "size_bytes":4,
                "sha256":"out-sha",
                "node_id":"extract",
                "node_fingerprint":"fp-extract"
            }]
        }))
        .expect("outputs"),
    )
    .expect("write outputs index");
    fs::write(
        run_dir.join("nodes").join("extract").join("resolved_params.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "argv":["/bin/sh","-c","cat nodes/seed/outputs/out > nodes/extract/outputs/out"]
        }))
        .expect("resolved params"),
    )
    .expect("write resolved params");
    fs::write(
        run_dir.join("nodes").join("extract").join("attempts.json"),
        serde_json::to_vec_pretty(&serde_json::json!([
            {
                "attempt":1,
                "started_unix_ms":1u64,
                "finished_unix_ms":2u64,
                "status":"Failed",
                "stdout_path":"attempts/1/stdout.log",
                "stderr_path":"attempts/1/stderr.log",
                "failure":{"class":"execution","kind":"Execution","code":"EXEC_FAIL","message":"first attempt failed"},
                "scheduled_backoff_ms":40
            },
            {
                "attempt":2,
                "started_unix_ms":4u64,
                "finished_unix_ms":5u64,
                "status":"Failed",
                "stdout_path":"attempts/2/stdout.log",
                "stderr_path":"attempts/2/stderr.log",
                "failure":{"class":"execution","kind":"Execution","code":"EXEC_FAIL","message":"second attempt failed"}
            }
        ]))
        .expect("attempts"),
    )
    .expect("write attempts");
    fs::write(run_dir.join("nodes").join("extract").join("stdout.log"), "terminal stdout\n")
        .expect("write stdout");
    fs::write(run_dir.join("nodes").join("extract").join("stderr.log"), "terminal stderr\n")
        .expect("write stderr");
    fs::write(
        run_dir.join("nodes").join("extract").join("attempts").join("1").join("stdout.log"),
        "attempt one stdout\n",
    )
    .expect("write attempt one stdout");
    fs::write(
        run_dir.join("nodes").join("extract").join("attempts").join("1").join("stderr.log"),
        "attempt one stderr\n",
    )
    .expect("write attempt one stderr");
    fs::write(
        run_dir.join("nodes").join("extract").join("attempts").join("2").join("stdout.log"),
        "attempt two stdout\n",
    )
    .expect("write attempt two stdout");
    fs::write(
        run_dir.join("nodes").join("extract").join("attempts").join("2").join("stderr.log"),
        "attempt two stderr\n",
    )
    .expect("write attempt two stderr");

    run_dir
}

fn write_blocked_node_fixture(root: &Path) -> PathBuf {
    let run_dir = root.join("run-blocked-node");
    fs::create_dir_all(run_dir.join("nodes").join("publish")).expect("mkdir publish");

    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-blocked-node",
            "created_unix_ms":1,
            "started_unix_ms":1,
            "finished_unix_ms":1,
            "graph_snapshot":"graph.snapshot.json",
            "status":"running",
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"graph-blocked-node",
            "tool_version":"0.4.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":0,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "graph":{
                "spec":"bijux-dag/v0.1",
                "meta":{"name":"inspect-blocked-node","owners":[],"tags":[]},
                "inputs":{},
                "nodes":[
                    {
                        "id":"publish",
                        "kind":"shell",
                        "outputs":[{"name":"out","path":"publish/out","kind":"file","required":true,"media_type":"text/plain"}],
                        "params":{"argv":["/bin/sh","-c","echo blocked > {outputs.out}"]},
                        "cache":{"enabled":true},
                        "effects":["filesystem"]
                    }
                ],
                "edges":[]
            },
            "graph_fingerprint":"graph-blocked-node"
        }))
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run_dir.join("run-log.index.json"),
        serde_json::to_vec_pretty(&vec![serde_json::json!({
            "event":"node_blocked",
            "ts":3u64,
            "node_id":"publish",
            "reason":"blocked_by_cpu"
        })])
        .expect("events"),
    )
    .expect("write run-log index");

    run_dir
}

#[test]
fn node_command_json_surfaces_planned_runtime_and_log_evidence() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    let run_dir = write_node_fixture(temp.path());

    let payload =
        run_json(&["node", &output_path_string(&run_dir), "--id", "extract", "--json"], &root);

    assert_eq!(payload["command"], "dag.node");
    assert_eq!(payload["data"]["node_id"], "extract");
    assert_eq!(payload["data"]["status"], "failed");
    assert_eq!(payload["data"]["planned"]["kind"], "shell");
    assert_eq!(payload["data"]["dependencies"], serde_json::json!(["seed"]));
    assert_eq!(payload["data"]["resolved_params"]["argv"][0], "/bin/sh");
    assert_eq!(payload["data"]["input_artifacts"][0]["source_node_id"], "seed");
    assert_eq!(payload["data"]["output_artifacts"][0]["sha256"], "out-sha");
    assert_eq!(payload["data"]["terminal_attempt"], 2);
    assert_eq!(payload["data"]["attempts"][0]["scheduled_backoff_ms"], 40);
    assert_eq!(payload["data"]["logs"]["stdout"]["path"], "nodes/extract/stdout.log");
    assert_eq!(payload["data"]["logs"]["stderr"]["tail"][0], "terminal stderr");
    assert_eq!(payload["data"]["cache"]["observed_result"], "evaluated_without_reuse");
    assert_eq!(payload["data"]["failure"]["failure"]["code"], "EXEC_FAIL");
    assert_eq!(payload["data"]["execution_explanation"]["classification"], "executed");
    assert_eq!(payload["data"]["execution_explanation"]["executed"], true);
    assert_eq!(payload["data"]["execution_explanation"]["reason"], "EXEC_FAIL");
    assert!(payload["data"].get("evidence_gaps").is_none());
}

#[test]
fn node_command_human_output_surfaces_attempts_cache_and_failure() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    let run_dir = write_node_fixture(temp.path());

    let (code, stdout, stderr) =
        run_dag(&["node", &output_path_string(&run_dir), "--id", "extract"], &root);

    assert_eq!(code, 0, "command failed: {stderr}");
    assert!(stdout.contains("node: extract"));
    assert!(stdout.contains("status: failed"));
    assert!(stdout.contains("planned_kind: shell"));
    assert!(stdout.contains("input_artifact_count: 1"));
    assert!(stdout.contains("output_artifact_count: 1"));
    assert!(stdout.contains("attempt=1 status=Failed backoff_ms=40"));
    assert!(stdout.contains("attempt=2 status=Failed backoff_ms=-"));
    assert!(stdout.contains("cache_status: configured=enabled observed=evaluated_without_reuse"));
    assert!(stdout.contains("failure_info: {\"class\":\"execution\""));
    assert!(stdout.contains("execution_explanation: executed=true classification=executed"));
    assert!(stdout.contains("stdout_path: nodes/extract/stdout.log"));
    assert!(stdout.contains("stderr_path: nodes/extract/stderr.log"));
    assert!(stdout.contains("terminal stderr"));
}

#[test]
fn explain_node_json_reports_resource_block_when_trace_is_missing() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    let run_dir = write_blocked_node_fixture(temp.path());

    let payload =
        run_json(&["explain", &output_path_string(&run_dir), "--node", "publish", "--json"], &root);

    assert_eq!(payload["command"], "dag.explain");
    assert_eq!(payload["data"]["node"], "publish");
    assert!(payload["data"]["trace"].is_null());
    assert_eq!(payload["data"]["execution_explanation"]["classification"], "resource_blocked");
    assert_eq!(payload["data"]["execution_explanation"]["executed"], false);
    assert_eq!(payload["data"]["execution_explanation"]["reason"], "blocked_by_cpu");
    assert_eq!(
        payload["data"]["execution_explanation"]["evidence_sources"],
        serde_json::json!(["run-log.index.json"])
    );
}

#[test]
fn explain_node_human_reports_missing_trace_and_block_reason() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    let run_dir = write_blocked_node_fixture(temp.path());

    let (code, stdout, stderr) =
        run_dag(&["explain", &output_path_string(&run_dir), "--node", "publish"], &root);

    assert_eq!(code, 0, "command failed: {stderr}");
    assert!(stdout.contains("node: publish"));
    assert!(
        stdout.contains("execution_explanation: executed=false classification=resource_blocked")
    );
    assert!(stdout.contains("reason=blocked_by_cpu"));
    assert!(stdout.contains("trace: <missing>"));
}
