use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

mod support;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn dag_command(root: &Path) -> Command {
    let mut command = Command::new(support::resolve_bijux_dag_binary(root));
    command.current_dir(root);
    command
}

fn run_json(root: &Path, args: &[&str]) -> Value {
    let output = dag_command(root).args(args).output().expect("run dag command");
    assert_eq!(
        output.status.code(),
        Some(0),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse json envelope")
}

fn wait_for(path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for {}", path.display());
}

fn wait_for_run_start(child: &mut Child, path: &Path, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        if let Some(status) = child.try_wait().expect("poll child") {
            panic!("run command exited before startup with status {status}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    let _ = child.kill();
    let _ = child.wait();
    panic!("timed out waiting for {}", path.display());
}

fn write_stoppable_graph(path: &Path) {
    fs::write(
        path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "spec": "bijux-dag/v0.1",
            "nodes": [
                {
                    "id": "prepare",
                    "kind": "const",
                    "inputs": [],
                    "outputs": [{"name": "value", "path": "prepare.txt"}],
                    "params": {"value": "ready"}
                },
                {
                    "id": "execute",
                    "kind": "shell",
                    "inputs": ["in"],
                    "outputs": [{"name": "value", "path": "execute.txt"}],
                    "params": {
                        "argv": [
                            "/bin/sh",
                            "-c",
                            "sleep 2; cat ../inputs/prepare/in > ../outputs/execute.txt"
                        ]
                    },
                    "effects": ["filesystem"]
                },
                {
                    "id": "publish",
                    "kind": "shell",
                    "inputs": ["in"],
                    "outputs": [{"name": "value", "path": "publish.txt"}],
                    "params": {
                        "argv": [
                            "/bin/sh",
                            "-c",
                            "cat ../inputs/execute/in > ../outputs/publish.txt"
                        ]
                    },
                    "effects": ["filesystem"]
                }
            ],
            "edges": [
                {"from": {"node_id": "prepare", "port": "value"}, "to": {"node_id": "execute", "port": "in"}},
                {"from": {"node_id": "execute", "port": "value"}, "to": {"node_id": "publish", "port": "in"}}
            ]
        }))
        .expect("graph"),
    )
    .expect("write graph");
}

#[test]
fn runs_stop_command_cancels_live_run_by_run_id() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tempdir");
    let graph_path = tmp.path().join("stoppable.dag.json");
    let out_dir = tmp.path().join("runs");
    write_stoppable_graph(&graph_path);

    let graph_arg = graph_path.to_string_lossy().to_string();
    let out_arg = out_dir.to_string_lossy().to_string();
    let mut child = dag_command(&root);
    let mut child = child
        .args(["run", graph_arg.as_str(), "--out", out_arg.as_str(), "--run-id", "run-stoppable"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn run command");

    let staging_dir = out_dir.join("run.tmp-stoppable");
    wait_for_run_start(&mut child, &staging_dir.join("manifest.json"), Duration::from_secs(30));
    let prepare_trace = staging_dir.join("nodes").join("prepare").join("trace.json");
    wait_for(&prepare_trace, Duration::from_secs(10));

    let stop =
        run_json(&root, &["runs", "stop", "run-stoppable", "--root", out_arg.as_str(), "--json"]);
    assert_eq!(stop["ok"], true);
    assert_eq!(stop["data"]["state"], "requested");
    assert_eq!(stop["data"]["run_id"], "stoppable");

    let output = child.wait_with_output().expect("wait for run command");
    assert_eq!(
        output.status.code(),
        Some(0),
        "run command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let final_dir = out_dir.join("run-stoppable");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(final_dir.join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["status"], "cancelled");
    assert_eq!(manifest["run_cancellation_cause"], "operator_request");
    assert_eq!(manifest["node_counts"]["success"], 1);
    assert_eq!(manifest["node_counts"]["cancelled"], 2);
    assert!(final_dir.join("run.stop-request.json").exists());

    let publish_trace: Value = serde_json::from_str(
        &fs::read_to_string(final_dir.join("nodes").join("publish").join("trace.json"))
            .expect("publish trace"),
    )
    .expect("parse publish trace");
    assert_eq!(publish_trace["status"], "cancelled");
    assert!(!final_dir.join("nodes").join("publish").join("outputs").join("publish.txt").exists());
}
