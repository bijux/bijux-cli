use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: args={args:?} code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn write_host_path_graph(root: &Path) -> PathBuf {
    let path = root.join("path-preview-host.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"const",
              "kind":"const",
              "outputs":[{"name":"result","path":"result.txt"}],
              "params":{
                "value":"ok",
                "preview_path":"{outputs_dir}/result.txt"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write shell graph");
    path
}

fn write_execution_cost_graph(root: &Path) -> PathBuf {
    let path = root.join("execution-cost.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {
              "id":"b",
              "kind":"shell",
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","b"]},
              "resources":{"cpu":4,"mem_mb":2048},
              "tags":["gpu:2"],
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":250},
              "cache":{"enabled":false,"reason":"network-bound"}
            },
            {
              "id":"c",
              "kind":"shell",
              "inputs":["left","right"],
              "outputs":[{"name":"out","path":"c/out"}],
              "params":{"argv":["echo","c"]},
              "resources":{"cpu":2,"mem_mb":1024}
            }
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"left"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"right"}}
          ]
        }"#,
    )
    .expect("write execution cost graph");
    path
}

#[test]
fn plan_explain_json_reports_previewed_run_layout_and_paths() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_host_path_graph(tmp.path());
    let out_dir = tmp.path().join("runs");
    let cache_dir = tmp.path().join("cache");

    let payload = run_json(
        &[
            "plan",
            "explain",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "preview-shell",
            "--cache-dir",
            &output_path_string(&cache_dir),
        ],
        &root,
    );

    assert_eq!(payload["data"]["run_layout"]["run_id"], "preview-shell");
    assert_eq!(
        payload["data"]["run_layout"]["final_path"],
        output_path_string(&out_dir.join("run-preview-shell"))
    );
    let resolved_paths =
        payload["data"]["path_previews"][0]["resolved_paths"].as_array().expect("resolved paths");
    assert_eq!(resolved_paths[0]["expression"], "{outputs_dir}/result.txt");
    assert_eq!(
        resolved_paths[0]["resolved_path"],
        output_path_string(&out_dir.join("run.tmp-preview-shell/nodes/const/outputs/result.txt"))
    );
    assert!(payload["data"]["path_previews"][0]["resolved_argv"].is_null());
}

#[test]
fn run_json_reuses_previewed_run_layout_for_execution_and_scheduling() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_host_path_graph(tmp.path());
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir runs");

    let payload = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "executed-shell",
            "--explain-scheduling",
        ],
        &root,
    );

    assert_eq!(payload["data"]["run_layout"]["run_id"], "executed-shell");
    assert_eq!(payload["data"]["scheduling"]["run_layout"]["run_id"], "executed-shell");
    assert_eq!(
        payload["data"]["run_dir"],
        output_path_string(&out_dir.join("run-executed-shell"))
    );
    assert_eq!(
        payload["data"]["scheduling"]["path_previews"][0]["resolved_paths"][0]["resolved_path"],
        output_path_string(&out_dir.join("run.tmp-executed-shell/nodes/const/outputs/result.txt"))
    );
    assert!(payload["data"]["scheduling"]["path_previews"][0]["resolved_argv"].is_null());
}

#[test]
fn plan_explain_json_reports_execution_cost_estimate() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_execution_cost_graph(tmp.path());

    let payload = run_json(&["plan", "explain", "--json", &output_path_string(&graph)], &root);

    assert_eq!(payload["data"]["execution_cost_estimate"]["node_count"], 3);
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["root_nodes"],
        serde_json::json!(["a", "b"])
    );
    assert_eq!(payload["data"]["execution_cost_estimate"]["critical_path_length"], 2);
    assert_eq!(payload["data"]["execution_cost_estimate"]["max_parallelism"], 2);
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["demand"]["cpu_cores_total"],
        7
    );
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["cache_exposure"]["non_cacheable_node_ids"],
        serde_json::json!(["b"])
    );
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["timeout_exposure"]["max_timeout_ms"],
        5000
    );
    assert_eq!(
        payload["data"]["execution_cost_estimate"]["retry_exposure"]["max_attempts"],
        3
    );
}
