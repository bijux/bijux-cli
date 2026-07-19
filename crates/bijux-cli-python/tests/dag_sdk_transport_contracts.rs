#![forbid(unsafe_code)]
//! Rust-owned release-lane coverage for Python DAG SDK transport helpers.

use std::path::PathBuf;
use std::process::Command;

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

fn run_python_json(script: &str, envs: &[(&str, String)]) -> Value {
    let mut command = Command::new(python_interpreter());
    command.arg("-c").arg(script);
    command.env("PYTHONPATH", python_root());
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("python transport script should execute");
    assert!(
        output.status.success(),
        "python transport script failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("python transport script should emit json")
}

#[test]
fn python_dag_sdk_transport_preserves_runtime_override_and_input_staging() {
    let payload = run_python_json(
        r###"
import json
import os
import shlex
import tempfile
from pathlib import Path

from bijux_cli_py import dag_command_json, dag_post_install_diagnostics, load_dag_graph, run_dag_graph

workspace = Path(os.environ["BIJUX_WORKSPACE_ROOT"])
temp_root = Path(tempfile.mkdtemp(prefix="bijux-dag-sdk-transport-"))
capture = temp_root / "capture.txt"
runtime = temp_root / "bijux-dag"
runtime.write_text(
    "#!/bin/sh\n"
    f"capture={shlex.quote(str(capture))}\n"
    "printf '%s\\n' \"$@\" > \"$capture\"\n"
    "while [ \"$#\" -gt 0 ]; do\n"
    "  if [ \"$1\" = \"--inputs-file\" ]; then\n"
    "    shift\n"
    "    printf 'inputs_path=%s\\n' \"$1\" >> \"$capture\"\n"
    "    if [ -f \"$1\" ]; then\n"
    "      printf 'inputs_present=1\\n' >> \"$capture\"\n"
    "    fi\n"
    "    continue\n"
    "  fi\n"
    "  shift\n"
    "done\n"
    "printf '%s\\n' '{\"ok\":true,\"status\":\"ok\",\"command\":\"dag.stub\",\"data\":{\"transport\":\"stub\"},\"diagnostics\":[],\"error\":null}'\n",
    encoding="utf-8",
)
runtime.chmod(0o755)
os.environ["BIJUX_DAG_BIN"] = str(runtime)

graph = load_dag_graph(
    workspace / "evidence" / "dag" / "authoring" / "examples" / "hello.dag.json"
)
diagnostics = dag_post_install_diagnostics()
command_payload = dag_command_json(["validate", "graph.json"])
run_payload = run_dag_graph(
    "graph.json",
    out=temp_root / "runs",
    run_id="transport-run",
    graph_inputs={"scheduled_at_unix_ms": 42},
)
capture_lines = capture.read_text(encoding="utf-8").splitlines()
inputs_path = next(
    line.split("=", 1)[1] for line in capture_lines if line.startswith("inputs_path=")
)
print(
    json.dumps(
        {
            "graph_spec": graph["spec"],
            "first_node": graph["nodes"][0]["id"],
            "command_ok": command_payload["ok"],
            "run_transport": run_payload["data"]["transport"],
            "runtime_binary": diagnostics["runtime_binary"],
            "warnings": diagnostics["warnings"],
            "inputs_present_during_call": "inputs_present=1" in capture_lines,
            "inputs_removed_after_call": not Path(inputs_path).exists(),
        }
    )
)
"###,
        &[("BIJUX_WORKSPACE_ROOT", workspace_root().display().to_string())],
    );

    assert_eq!(payload["graph_spec"], "bijux-dag/v0.1");
    assert_eq!(payload["first_node"], "const1");
    assert_eq!(payload["command_ok"], true);
    assert_eq!(payload["run_transport"], "stub");
    assert!(payload["runtime_binary"].as_str().is_some_and(|value| value.ends_with("bijux-dag")));
    assert_eq!(payload["warnings"], serde_json::json!([]));
    assert_eq!(payload["inputs_present_during_call"], true);
    assert_eq!(payload["inputs_removed_after_call"], true);
}

#[test]
fn python_dag_sdk_transport_rejects_invalid_payloads_and_graph_shapes() {
    let payload = run_python_json(
        r###"
import json
import os
import tempfile
from pathlib import Path

from bijux_cli_py import dag_command_json, load_dag_graph
from bijux_cli_py._exceptions import InternalError, ValidationError

temp_root = Path(tempfile.mkdtemp(prefix="bijux-dag-sdk-transport-errors-"))
bad_graph = temp_root / "bad-graph.json"
bad_graph.write_text('["not","a","graph"]\n', encoding="utf-8")
graph_error = None
try:
    load_dag_graph(bad_graph)
except ValidationError as exc:
    graph_error = str(exc)

runtime = temp_root / "bijux-dag"
runtime.write_text("#!/bin/sh\nprintf '%s\\n' 'not-json'\n", encoding="utf-8")
runtime.chmod(0o755)
os.environ["BIJUX_DAG_BIN"] = str(runtime)
runtime_error = None
try:
    dag_command_json(["validate", "graph.json"])
except InternalError as exc:
    runtime_error = str(exc)

duplicate_error = None
try:
    from bijux_cli_py import run_dag_graph

    run_dag_graph(
        "graph.json",
        out=temp_root / "runs",
        graph_inputs={"x": 1},
        inputs_file="inputs.json",
    )
except ValueError as exc:
    duplicate_error = str(exc)

print(
    json.dumps(
        {
            "graph_error": graph_error,
            "runtime_error": runtime_error,
            "duplicate_error": duplicate_error,
        }
    )
)
"###,
        &[],
    );

    assert!(payload["graph_error"]
        .as_str()
        .is_some_and(|value| value.contains("must decode to a JSON object")));
    assert!(payload["runtime_error"]
        .as_str()
        .is_some_and(|value| value.contains("invalid JSON payload from bijux-dag runtime")));
    assert_eq!(payload["duplicate_error"], "graph_inputs and inputs_file are mutually exclusive");
}
