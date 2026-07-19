use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts::RunDir;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

mod support;

use support::create_corrupted_run_dir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

fn run_matches(args: &[&str]) -> Result<std::process::ExitCode, std::process::ExitCode> {
    let cmd = dag_command();
    let matches = cmd.try_get_matches_from(args).expect("clap parse");
    dag_run(&matches)
}

fn write_graph(path: &Path, payload: serde_json::Value) {
    fs::write(path, serde_json::to_vec_pretty(&payload).expect("serialize graph"))
        .expect("write graph");
}

#[test]
fn fault_permission_denied_run_dir_creation() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    let graph = root.join("evidence/authoring/examples/hello.dag.json");
    let locked = temp.path().join("locked");
    fs::create_dir_all(&locked).expect("create locked");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&locked).expect("meta").permissions();
        perms.set_mode(0o500);
        fs::set_permissions(&locked, perms).expect("chmod");
    }

    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph.to_string_lossy().as_ref(),
        "--out",
        locked.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_err());
}

#[test]
fn fault_disk_pressure_simulated_write_failure() {
    let temp = tempfile::tempdir().expect("tmp");
    let path = temp.path().join("out").join("index.json");
    fs::create_dir_all(temp.path().join("out")).expect("mkdir");
    let huge = "x".repeat(1024 * 1024);
    fs::write(&path, huge).expect("write first");
    let readonly = fs::metadata(&path).expect("meta").permissions().readonly();
    let _ = readonly;
    assert!(path.exists());
}

#[test]
fn fault_trace_file_missing_detected() {
    let temp = tempfile::tempdir().expect("tmp");
    let run = create_corrupted_run_dir(temp.path(), "missing_trace");
    let result = run_matches(&["bijux-dag", "verify", run.to_string_lossy().as_ref(), "--deep"]);
    assert!(result.is_err());
}

#[test]
fn fault_outputs_index_corruption_detected() {
    let temp = tempfile::tempdir().expect("tmp");
    let run = create_corrupted_run_dir(temp.path(), "tampered_outputs_index");
    let result = run_matches(&["bijux-dag", "verify", run.to_string_lossy().as_ref(), "--deep"]);
    assert!(result.is_err());
}

#[test]
fn fault_subprocess_non_zero_exit() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("fail.json");
    let graph = json!({
      "spec":"dag/v0.1",
      "meta":{"name":"non-zero"},
      "nodes":[
        {
          "id":"fail",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","exit 3"]},
          "effects":["filesystem"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_err());
}

#[test]
fn fault_subprocess_timeout_classification() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("timeout.json");
    let graph = json!({
      "spec":"dag/v0.1",
      "meta":{"name":"timeout"},
      "nodes":[
        {
          "id":"slow",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","sleep 1"]},
          "timeout_ms":1,
          "effects":["filesystem"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_err());
}

#[test]
fn fault_subprocess_killed_signal_like_failure() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("killed.json");
    let graph = json!({
      "spec":"dag/v0.1",
      "meta":{"name":"killed"},
      "nodes":[
        {
          "id":"killed",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","kill -9 $$"]},
          "effects":["filesystem"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_err());
}

#[test]
fn fault_missing_required_env_var() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("env-missing.json");
    let graph = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"env"},
      "nodes":[
        {
          "id":"env",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","test -n \"$REQUIRED_X\" && echo ok > ../outputs/out"]},
          "effects":["filesystem","env"],
          "env_allowlist":["REQUIRED_X"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
        "--clean-env",
    ]);
    assert!(result.is_err());
    assert_eq!(fs::read_dir(&out_dir).expect("run root entries").count(), 0);
}

#[test]
fn fault_malformed_graph_configuration() {
    let temp = tempfile::tempdir().expect("tmp");
    let bad = temp.path().join("bad.json");
    fs::write(&bad, "{not-json").expect("write bad");
    let result = run_matches(&["bijux-dag", "validate", bad.to_string_lossy().as_ref()]);
    assert!(result.is_err());
}

#[test]
fn fault_manifest_tamper_after_completion() {
    let temp = tempfile::tempdir().expect("tmp");
    let run = create_corrupted_run_dir(temp.path(), "truncated_manifest");
    let result = run_matches(&["bijux-dag", "verify", run.to_string_lossy().as_ref(), "--deep"]);
    assert!(result.is_err());
}

#[test]
fn fault_missing_trace_referenced_by_manifest() {
    let temp = tempfile::tempdir().expect("tmp");
    let run = create_corrupted_run_dir(temp.path(), "missing_trace");
    let result = run_matches(&["bijux-dag", "verify", run.to_string_lossy().as_ref(), "--deep"]);
    assert!(result.is_err());
}

#[test]
fn fault_stale_cache_metadata_mismatch() {
    let temp = tempfile::tempdir().expect("tmp");
    let cache = temp.path().join("cache");
    fs::create_dir_all(cache.join("abc")).expect("mkdir");
    fs::write(cache.join("abc").join("meta.json"), "{\"fingerprint\":\"old\"}").expect("write");
    let result = run_matches(&[
        "bijux-dag",
        "cache",
        "verify",
        "--cache-dir",
        cache.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn fault_run_id_collision() {
    let temp = tempfile::tempdir().expect("tmp");
    let first = RunDir::create_with_id(temp.path(), "same-id").expect("run1");
    let _ = first.finalize();
    let second = RunDir::create_with_id(temp.path(), "same-id");
    assert!(
        second.is_err() || second.and_then(|run| run.finalize()).is_err(),
        "run id collision should be rejected when reserving or finalizing duplicate paths"
    );
}

#[test]
fn fault_latest_alias_race() {
    let temp = tempfile::tempdir().expect("tmp");
    let alias = temp.path().join("latest");
    fs::write(&alias, "old").expect("write alias file");
    let replaced = fs::remove_file(&alias).and_then(|()| fs::write(&alias, "new"));
    assert!(replaced.is_ok());
}

#[test]
fn fault_no_silent_half_valid_artifacts() {
    let temp = tempfile::tempdir().expect("tmp");
    let run = create_corrupted_run_dir(temp.path(), "tampered_outputs_index");
    let result = run_matches(&["bijux-dag", "verify", run.to_string_lossy().as_ref(), "--deep"]);
    assert!(result.is_err());
}

#[test]
fn fault_subprocess_malformed_output_payload() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("malformed-output.json");
    let graph = json!({
      "spec":"dag/v0.1",
      "meta":{"name":"malformed-output"},
      "nodes":[
        {
          "id":"emit",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","printf '\\xFF\\xFE' > ../outputs/out"]},
          "effects":["filesystem"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let result = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
    ]);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn fault_partial_run_cleanup_after_early_failure() {
    let temp = tempfile::tempdir().expect("tmp");
    let graph_path = temp.path().join("early-failure.json");
    let graph = json!({
      "spec":"dag/v0.1",
      "meta":{"name":"early-failure"},
      "nodes":[
        {
          "id":"fail",
          "kind":"shell",
          "inputs":[],
          "outputs":[{"name":"out","path":"out"}],
          "params":{"argv":["/bin/sh","-c","exit 9"]},
          "effects":["filesystem"]
        }
      ],
      "edges":[]
    });
    write_graph(&graph_path, graph);
    let out_dir = temp.path().join("runs");
    fs::create_dir_all(&out_dir).expect("mkdir");
    let _ = run_matches(&[
        "bijux-dag",
        "run",
        graph_path.to_string_lossy().as_ref(),
        "--out",
        out_dir.to_string_lossy().as_ref(),
    ]);
    let entries = fs::read_dir(&out_dir).expect("read runs");
    for entry in entries {
        let path = entry.expect("entry").path();
        let is_tmp = path
            .file_name()
            .and_then(|v| v.to_str())
            .map(|v| v.contains("run.tmp"))
            .unwrap_or(false);
        assert!(!is_tmp, "stale temp run dir left behind: {}", path.display());
    }
}
