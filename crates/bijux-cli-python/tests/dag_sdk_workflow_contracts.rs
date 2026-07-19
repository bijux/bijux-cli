#![forbid(unsafe_code)]
//! Rust-owned release-lane coverage for Python DAG SDK workflow helpers.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn python_root() -> PathBuf {
    workspace_root().join("crates/bijux-cli-python/python")
}

fn python_interpreter() -> String {
    if let Ok(explicit) = std::env::var("PYTHON") {
        if !explicit.trim().is_empty() {
            return explicit;
        }
    }
    for candidate in ["python3.12", "python3.11", "python3", "python"] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return candidate.to_string();
        }
    }
    panic!("python interpreter not found");
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[cfg(unix)]
fn write_dag_wrapper(path: &Path, workspace_root: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let script = format!(
        "#!/bin/sh\ncd {workspace:?}\nexec {cargo_bin:?} run -q -p bijux-dag-cli --bin bijux-dag -- \"$@\"\n",
        workspace = workspace_root.display().to_string(),
    );
    fs::write(path, script).expect("write dag wrapper");
    let mut permissions = fs::metadata(path).expect("wrapper metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod dag wrapper");
}

#[cfg(windows)]
fn write_dag_wrapper(path: &Path, workspace_root: &Path) {
    let cargo_bin = std::env::var("CARGO")
        .ok()
        .or_else(|| option_env!("CARGO").map(ToOwned::to_owned))
        .unwrap_or_else(|| "cargo".to_string());
    let script = format!(
        "@echo off\r\ncd /d {workspace}\r\n{cargo_bin} run -q -p bijux-dag-cli --bin bijux-dag -- %*\r\n",
        workspace = workspace_root.display(),
    );
    fs::write(path, script).expect("write dag wrapper");
}

fn run_python_json(script: &str, envs: &[(&str, String)]) -> Value {
    let mut command = Command::new(python_interpreter());
    command.arg("-c").arg(script);
    command.env("PYTHONPATH", python_root());
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("python workflow script should execute");
    assert!(
        output.status.success(),
        "python workflow script failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("python workflow script should emit json")
}

#[test]
fn python_dag_sdk_workflows_match_direct_runtime_for_hello_graph() {
    let workspace = workspace_root();
    let wrapper_root = temp_dir("bijux-dag-sdk-workflow-wrapper");
    let runtime = if let Ok(explicit) = std::env::var("BIJUX_DAG_BIN") {
        if !explicit.trim().is_empty() {
            PathBuf::from(explicit)
        } else {
            let path = wrapper_root.join(if cfg!(windows) { "bijux-dag.bat" } else { "bijux-dag" });
            write_dag_wrapper(&path, &workspace);
            path
        }
    } else {
        let path = wrapper_root.join(if cfg!(windows) { "bijux-dag.bat" } else { "bijux-dag" });
        write_dag_wrapper(&path, &workspace);
        path
    };

    let payload = run_python_json(
        r###"
import json
import os
import subprocess
import tempfile
from pathlib import Path

from bijux_cli_py import (
    inspect_dag_run,
    plan_dag_graph,
    query_dag_artifacts,
    run_dag_graph,
    validate_dag_graph,
)


def direct_json(runtime: Path, *args: str) -> dict[str, object]:
    completed = subprocess.run(
        [str(runtime), "--json", *args],
        capture_output=True,
        text=True,
        check=False,
    )
    payload = json.loads((completed.stdout or completed.stderr).strip())
    assert isinstance(payload, dict)
    return payload


workspace = Path(os.environ["BIJUX_WORKSPACE_ROOT"])
runtime = Path(os.environ["BIJUX_DAG_BIN"])
hello_graph = workspace / "evidence" / "dag" / "authoring" / "examples" / "hello.dag.json"
out = Path(tempfile.mkdtemp(prefix="bijux-dag-sdk-workflows-")) / "runs"

sdk_validate = validate_dag_graph(hello_graph)
direct_validate = direct_json(runtime, "validate", str(hello_graph))

sdk_plan = plan_dag_graph(hello_graph, out=out, run_id="sdk-plan")
direct_plan = direct_json(
    runtime,
    "plan",
    "explain",
    str(hello_graph),
    "--out",
    str(out),
    "--run-id",
    "sdk-plan",
)

sdk_run = run_dag_graph(hello_graph, out=out, run_id="sdk-run")
direct_run_dir = out / "run-sdk-run"
sdk_inspect = inspect_dag_run(run_id="sdk-run", root=out)
direct_inspect = direct_json(runtime, "runs", "inspect", "sdk-run", "--root", str(out))
sdk_artifacts = query_dag_artifacts(direct_run_dir)
direct_artifacts = direct_json(runtime, "artifact", "registry", str(direct_run_dir))

print(
    json.dumps(
        {
            "validate_equal": sdk_validate == direct_validate,
            "plan_command_equal": sdk_plan["command"] == direct_plan["command"] == "dag.plan.explain",
            "plan_selected_nodes_equal": (
                sdk_plan["data"]["selection"]["selected_nodes"]
                == direct_plan["data"]["selection"]["selected_nodes"]
            ),
            "plan_run_layout_equal": sdk_plan["data"]["run_layout"] == direct_plan["data"]["run_layout"],
            "run_succeeded": sdk_run["ok"] and sdk_run["data"]["summary"]["status"] == "success",
            "inspect_equal": sdk_inspect == direct_inspect,
            "artifacts_equal": sdk_artifacts == direct_artifacts,
        }
    )
)
"###,
        &[
            ("BIJUX_WORKSPACE_ROOT", workspace.display().to_string()),
            ("BIJUX_DAG_BIN", runtime.display().to_string()),
        ],
    );

    assert_eq!(payload["validate_equal"], true);
    assert_eq!(payload["plan_command_equal"], true);
    assert_eq!(payload["plan_selected_nodes_equal"], true);
    assert_eq!(payload["plan_run_layout_equal"], true);
    assert_eq!(payload["run_succeeded"], true);
    assert_eq!(payload["inspect_equal"], true);
    assert_eq!(payload["artifacts_equal"], true);
}
