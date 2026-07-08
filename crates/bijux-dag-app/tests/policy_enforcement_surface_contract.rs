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

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_dag_with_internal_lane(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command_with_env(args, cwd, &[("BIJUX_DAG_ENABLE_INTERNAL", "1")])
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: args={args:?} code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn run_json_with_internal_lane(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag_with_internal_lane(args, cwd);
    assert!(code == 0, "command failed: args={args:?} code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn write_shell_graph(root: &Path) -> PathBuf {
    let path = root.join("policy-surface-shell.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{"argv":["/bin/sh","-c","printf 'ok' > ../outputs/value.txt"]},
              "effects":["filesystem","network","clock"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write shell graph");
    path
}

fn write_container_graph(root: &Path) -> PathBuf {
    let path = root.join("policy-surface-container.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"container",
              "kind":"container",
              "outputs":[{"name":"value","path":"value.txt"}],
              "params":{},
              "container":{
                "image":"alpine:3.19",
                "argv":["echo","ok"],
                "env_allowlist":[],
                "engine":"docker"
              },
              "effects":["filesystem","network"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write container graph");
    path
}

#[test]
fn run_preflight_reports_best_effort_subprocess_policy_surface() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_shell_graph(tmp.path());
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");

    let payload = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--preflight-only",
            "--hermetic",
        ],
        &root,
    );

    assert_eq!(
        payload["data"]["policy_surface"]["profile"]["mode"],
        "best_effort_local_policy_profile"
    );
    let surfaces =
        payload["data"]["policy_surface"]["enforcement"]["surfaces"].as_array().expect("surfaces");
    let subprocess = surfaces
        .iter()
        .find(|surface| surface["executor_surface"] == "local-subprocess")
        .expect("subprocess surface");
    assert_eq!(subprocess["isolation_claim"], "best_effort_process_boundary");
    assert!(subprocess["limitations"].as_array().expect("limitations").iter().any(|entry| entry
        .as_str()
        .is_some_and(|value| value.contains("does not firewall network access"))));
    assert!(subprocess["guards"]
        .as_array()
        .expect("guards")
        .iter()
        .any(|guard| guard["guard"] == "deny-network"
            && guard["enforcement_mode"] == "declared_effect_gate"));
}

#[test]
fn replay_dry_run_reports_source_write_boundary_without_process_sandbox_claim() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let out_dir = tmp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let graph = write_shell_graph(tmp.path());

    let run = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "policy-surface-source",
        ],
        &root,
    );
    assert_eq!(run["ok"], true);

    let source_run = out_dir.join("run-policy-surface-source");
    let replay = run_json(
        &[
            "replay",
            "--json",
            &output_path_string(&source_run),
            "--out",
            &output_path_string(&out_dir),
            "--dry-run",
            "--sandbox",
            "--hermetic",
        ],
        &root,
    );

    assert_eq!(replay["data"]["sandbox_scope"]["mode"], "source_run_write_boundary");
    assert!(replay["data"]["sandbox_scope"]["limitations"]
        .as_array()
        .expect("sandbox limitations")
        .iter()
        .any(|entry| entry
            .as_str()
            .is_some_and(|value| value.contains("does not create a process sandbox"))));
    assert_eq!(
        replay["data"]["policy_surface"]["profile"]["mode"],
        "best_effort_local_policy_profile"
    );
}

#[test]
fn runtime_isolation_reports_container_network_runtime_enforcement() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_container_graph(tmp.path());

    let payload = run_json_with_internal_lane(
        &["runtime", "isolation", "--json", &output_path_string(&graph)],
        &root,
    );

    let surfaces =
        payload["data"]["policy_surface"]["enforcement"]["surfaces"].as_array().expect("surfaces");
    let container = surfaces
        .iter()
        .find(|surface| surface["executor_surface"] == "container-engine")
        .expect("container surface");
    assert_eq!(container["isolation_claim"], "container_runtime_boundary");
    assert!(container["guards"]
        .as_array()
        .expect("guards")
        .iter()
        .any(|guard| guard["guard"] == "deny-network"
            && guard["enforcement_mode"] == "container_runtime_flag"));
    assert!(container["guards"]
        .as_array()
        .expect("guards")
        .iter()
        .any(|guard| guard["guard"] == "container-image-reference"
            && guard["enforcement_mode"] == "reference_digest_gate"));
}
