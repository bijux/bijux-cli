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

fn write_shell_argv_resolution_graph(root: &Path) -> PathBuf {
    let path = root.join("command-argv-resolution.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"seed",
              "kind":"const",
              "outputs":[{"name":"out","path":"seed.txt"}],
              "params":{"value":"hello"}
            },
            {
              "id":"copy",
              "kind":"shell",
              "inputs":["reads"],
              "outputs":[{"name":"result","path":"result.txt"}],
              "effects":["filesystem"],
              "params":{
                "suffix":"!",
                "argv":[
                  "/bin/sh",
                  "-c",
                  "cat {inputs.reads} > {outputs.result} && printf '%s' {params.suffix} >> {outputs.result}"
                ]
              }
            }
          ],
          "edges":[
            {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"copy","port":"reads"}}
          ]
        }"#,
    )
    .expect("write graph");
    path
}

fn write_unresolved_shell_argv_graph(root: &Path) -> PathBuf {
    let path = root.join("command-argv-unresolved.dag.json");
    fs::write(
        &path,
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"copy",
              "kind":"shell",
              "outputs":[{"name":"result","path":"result.txt"}],
              "effects":["filesystem"],
              "params":{"argv":["echo","{params.missing}"]}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("write graph");
    path
}

#[test]
fn run_executes_resolved_shell_argv_with_inputs_outputs_and_params() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_shell_argv_resolution_graph(tmp.path());
    let out_dir = tmp.path().join("runs");

    let payload = run_json(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "argv-resolution",
        ],
        &root,
    );

    let run_dir = out_dir.join("run-argv-resolution");
    assert_eq!(payload["data"]["run_dir"], output_path_string(&run_dir));
    let result =
        fs::read_to_string(run_dir.join("nodes").join("copy").join("outputs").join("result.txt"))
            .expect("read templated output");
    assert_eq!(result, "\"hello\"!");
}

#[test]
fn run_rejects_unresolved_shell_argv_before_execution() {
    let root = repo_root();
    let tmp = tempfile::tempdir().expect("tmp");
    let graph = write_unresolved_shell_argv_graph(tmp.path());
    let out_dir = tmp.path().join("runs");

    let (code, _stdout, _stderr) = run_dag(
        &[
            "run",
            "--json",
            &output_path_string(&graph),
            "--out",
            &output_path_string(&out_dir),
            "--run-id",
            "argv-unresolved",
        ],
        &root,
    );

    assert_ne!(code, 0, "run must fail for unresolved command templates");
    assert!(
        !out_dir.join("run-argv-unresolved").exists(),
        "run directory must not be created when command templates fail to resolve"
    );
}
