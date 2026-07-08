use crate::{
    effective_env_allowlist, Adapter, AdapterId, ControlledCommandResult, FailureClass,
    FailureInfo, NodeCtx, NodeResult, NodeStatus, RuntimeError,
};
use bijux_dag_artifacts::write_outputs_index;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PYTHON_INVOCATION_FILE: &str = "python-invocation.json";
const PYTHON_FUNCTION_RUNNER: &str = r#"
import importlib
import json
import pathlib
import sys
import traceback


def fail(phase, exc):
    payload = {
        "phase": phase,
        "exception_type": exc.__class__.__name__,
        "message": str(exc),
        "traceback": traceback.format_exc().splitlines(),
    }
    sys.stderr.write(json.dumps(payload, sort_keys=True))
    sys.exit(2)


def write_json(path_text, value):
    path = pathlib.Path(path_text)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(value, handle, sort_keys=True)


def main():
    with open(sys.argv[1], "r", encoding="utf-8") as handle:
        request = json.load(handle)
    module_name = request["module"]
    function_name = request["function"]
    payload = request["payload"]
    outputs = request["outputs"]

    try:
        module = importlib.import_module(module_name)
    except Exception as exc:
        fail("import_module", exc)

    try:
        target = getattr(module, function_name)
    except Exception as exc:
        fail("resolve_function", exc)

    if not callable(target):
        fail("resolve_function", TypeError(f"{module_name}.{function_name} is not callable"))

    try:
        result = target(payload)
    except Exception as exc:
        fail("call_function", exc)

    try:
        if len(outputs) == 1:
            write_json(outputs[0]["path"], result)
            return

        if not isinstance(result, dict):
            raise TypeError(
                "python adapter must return an object keyed by output name when multiple outputs are declared"
            )

        declared_names = {entry["name"] for entry in outputs}
        extra_names = sorted(set(result.keys()) - declared_names)
        if extra_names:
            raise KeyError(
                "python adapter returned undeclared outputs: " + ", ".join(extra_names)
            )

        missing_names = sorted(
            entry["name"] for entry in outputs if entry.get("required", True) and entry["name"] not in result
        )
        if missing_names:
            raise KeyError(
                "python adapter did not return required outputs: " + ", ".join(missing_names)
            )

        for entry in outputs:
            name = entry["name"]
            if name in result:
                write_json(entry["path"], result[name])
    except Exception as exc:
        fail("write_outputs", exc)


if __name__ == "__main__":
    main()
"#;

#[derive(Clone)]
pub struct PythonFunctionAdapter;

struct PythonInvocationParams {
    module: String,
    function: String,
    python_bin: Option<String>,
    payload: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct PythonInvocationRequest {
    module: String,
    function: String,
    payload: BTreeMap<String, Value>,
    outputs: Vec<PythonOutputTarget>,
}

#[derive(Debug, Serialize)]
struct PythonOutputTarget {
    name: String,
    path: String,
    required: bool,
}

#[derive(Debug, Deserialize)]
struct PythonFailurePayload {
    phase: String,
    exception_type: String,
    message: String,
    traceback: Vec<String>,
}

fn python_failure(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<Value>,
) -> FailureInfo {
    FailureInfo::new(FailureClass::Execution, "Execution", code.into(), message.into(), details)
}

fn python_user_failure(message: impl Into<String>, details: Option<Value>) -> FailureInfo {
    FailureInfo::new(FailureClass::User, "User", "EXEC_ERROR", message.into(), details)
}

fn failure_result(
    exec: &crate::RunContext,
    node_id: &str,
    status: NodeStatus,
    failure: FailureInfo,
    stderr_contents: &[u8],
) -> Result<NodeResult, RuntimeError> {
    let stdout_path = exec.run_dir.node_stdout_path(node_id);
    let stderr_path = exec.run_dir.node_stderr_path(node_id);
    let outputs_dir = exec.run_dir.node_outputs_dir(node_id);
    exec.fs.write(&stdout_path, b"")?;
    exec.fs.write(&stderr_path, stderr_contents)?;
    Ok(NodeResult {
        status,
        stdout_path: stdout_path.display().to_string(),
        stderr_path: stderr_path.display().to_string(),
        outputs_dir: outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure: Some(failure),
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
    })
}

fn python_invocation_params(params: &Value) -> Result<PythonInvocationParams, FailureInfo> {
    let Some(params) = params.as_object() else {
        return Err(python_user_failure(
            "python params must be an object",
            Some(json!({
                "field": "params",
                "reason": "expected_object",
            })),
        ));
    };

    let module = match params.get("module") {
        Some(Value::String(module)) if !module.trim().is_empty() => module.clone(),
        Some(Value::String(_)) => {
            return Err(python_user_failure(
                "python module must not be empty",
                Some(json!({
                    "field": "module",
                    "reason": "empty",
                })),
            ));
        }
        Some(_) => {
            return Err(python_user_failure(
                "python module must be a string",
                Some(json!({
                    "field": "module",
                    "reason": "expected_string",
                })),
            ));
        }
        None => {
            return Err(python_user_failure(
                "python module is required",
                Some(json!({
                    "field": "module",
                    "reason": "missing",
                })),
            ));
        }
    };
    let function = match params.get("function") {
        Some(Value::String(function)) if !function.trim().is_empty() => function.clone(),
        Some(Value::String(_)) => {
            return Err(python_user_failure(
                "python function must not be empty",
                Some(json!({
                    "field": "function",
                    "reason": "empty",
                })),
            ));
        }
        Some(_) => {
            return Err(python_user_failure(
                "python function must be a string",
                Some(json!({
                    "field": "function",
                    "reason": "expected_string",
                })),
            ));
        }
        None => {
            return Err(python_user_failure(
                "python function is required",
                Some(json!({
                    "field": "function",
                    "reason": "missing",
                })),
            ));
        }
    };

    let python_bin = match params.get("python_bin") {
        Some(Value::String(value)) if value.trim().is_empty() => {
            return Err(python_user_failure(
                "python_bin must not be empty when provided",
                Some(json!({
                    "field": "python_bin",
                    "reason": "empty",
                })),
            ));
        }
        Some(Value::String(value)) => Some(value.clone()),
        Some(_) => {
            return Err(python_user_failure(
                "python_bin must be a string when provided",
                Some(json!({
                    "field": "python_bin",
                    "reason": "expected_string",
                })),
            ));
        }
        None => None,
    };

    let payload = params
        .iter()
        .filter(|(key, _)| !matches!(key.as_str(), "module" | "function" | "python_bin"))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();

    Ok(PythonInvocationParams { module, function, python_bin, payload })
}

fn resolve_python_executable(explicit: Option<&str>) -> Result<PathBuf, FailureInfo> {
    let mut candidates = Vec::new();
    if let Some(explicit) = explicit {
        candidates.push(explicit.to_string());
    } else {
        candidates.push("python3".to_string());
        candidates.push("python".to_string());
    }

    for candidate in &candidates {
        if let Some(path) = resolve_ambient_executable(candidate) {
            return Ok(path);
        }
    }

    Err(FailureInfo::new(
        FailureClass::Infrastructure,
        "Infrastructure",
        "MISSING_EXECUTABLE",
        "python interpreter could not be resolved",
        Some(json!({
            "executable_candidates": candidates,
        })),
    ))
}

fn resolve_ambient_executable(candidate: &str) -> Option<PathBuf> {
    let candidate_path = Path::new(candidate);
    if candidate_path.components().count() > 1 || candidate_path.is_absolute() {
        return candidate_path.is_file().then(|| {
            candidate_path.canonicalize().unwrap_or_else(|_| candidate_path.to_path_buf())
        });
    }

    let path_value = std::env::var_os("PATH")?;
    let mut names = vec![candidate.to_string()];
    if cfg!(windows) && candidate_path.extension().is_none() {
        names.push(format!("{candidate}.exe"));
    }

    for directory in std::env::split_paths(&path_value) {
        for name in &names {
            let path = directory.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}

fn python_output_targets(
    node: &bijux_dag_core::Node,
    outputs_dir: &Path,
) -> Result<Vec<PythonOutputTarget>, FailureInfo> {
    let mut targets = Vec::with_capacity(node.outputs.len());
    for output in &node.outputs {
        if output.expects_directory() {
            return Err(FailureInfo::new(
                FailureClass::User,
                "User",
                "OUTPUT_PATH_INVALID",
                format!("python adapter cannot materialize directory output: {}", output.path),
                Some(json!({
                    "output": output.name,
                    "path": output.path,
                })),
            ));
        }
        let authorized = crate::authorized_declared_output_path(outputs_dir, output)?;
        targets.push(PythonOutputTarget {
            name: output.name.clone(),
            path: authorized.display().to_string(),
            required: output.required,
        });
    }
    Ok(targets)
}

fn python_failure_from_stderr(stderr: &[u8], exit_code: Option<i32>) -> Option<FailureInfo> {
    let payload: PythonFailurePayload = serde_json::from_slice(stderr).ok()?;
    Some(python_failure(
        "PYTHON_EXCEPTION",
        format!("python {} failed: {}", payload.phase, payload.message),
        Some(json!({
            "phase": payload.phase,
            "exception_type": payload.exception_type,
            "message": payload.message,
            "traceback": payload.traceback,
            "exit_code": exit_code,
        })),
    ))
}

const PYTHON_FAILURE_STDERR_READ_BYTES: u64 = 256 * 1024;

impl Adapter for PythonFunctionAdapter {
    fn id(&self) -> AdapterId {
        AdapterId { id: "python".to_string(), version: "0.1".to_string() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["python".to_string()]
    }

    fn required_effects(&self) -> crate::EffectSet {
        crate::EffectSet { filesystem: true, env: false, network: false, clock: false }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;

        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        if let Err(failure) = crate::preflight_declared_output_targets(&outputs_dir, &node.outputs)
        {
            let stderr_message = failure.message.clone();
            return failure_result(
                exec,
                &node.id,
                NodeStatus::Failed,
                failure,
                stderr_message.as_bytes(),
            );
        }

        let invocation = match python_invocation_params(ctx.params) {
            Ok(invocation) => invocation,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };
        let interpreter = match resolve_python_executable(invocation.python_bin.as_deref()) {
            Ok(interpreter) => interpreter,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };
        let outputs = match python_output_targets(node, &outputs_dir) {
            Ok(outputs) => outputs,
            Err(failure) => {
                let stderr_message = failure.message.clone();
                return failure_result(
                    exec,
                    &node.id,
                    NodeStatus::Failed,
                    failure,
                    stderr_message.as_bytes(),
                );
            }
        };

        let request = PythonInvocationRequest {
            module: invocation.module,
            function: invocation.function,
            payload: invocation.payload,
            outputs,
        };
        let request_path = work_dir.join(PYTHON_INVOCATION_FILE);
        exec.fs.write(&request_path, &serde_json::to_vec_pretty(&request)?)?;

        let mut cmd = std::process::Command::new(&interpreter);
        cmd.arg("-c").arg(PYTHON_FUNCTION_RUNNER).arg(&request_path);
        cmd.current_dir(&work_dir);
        let env_allowlist = effective_env_allowlist(node);
        crate::apply_shaped_env(&mut cmd, exec.policy.clean_env, &env_allowlist, &[]);
        crate::apply_temp_env(&mut cmd, &exec.run_dir.node_temp_dir(&node.id));

        let output = crate::command_output_with_controls(
            &mut cmd,
            crate::effective_node_timeout_ms(node, ctx.params),
            Some(exec.cancellation_requested.as_ref()),
        )?;

        output.persist_streams(exec.fs.as_ref(), &stdout_path, &stderr_path)?;
        match output {
            ControlledCommandResult::TimedOut(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Failed,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(FailureInfo::new(
                        FailureClass::Timeout,
                        "Timeout",
                        "EXEC_TIMEOUT",
                        "python function timed out after configured node timeout",
                        Some(json!({
                            "interpreter": interpreter.display().to_string(),
                            "exit_code": output.status.code(),
                        })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Cancelled(output) => {
                return Ok(NodeResult {
                    status: NodeStatus::Cancelled,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(FailureInfo::new(
                        FailureClass::Execution,
                        "Execution",
                        "EXEC_CANCELLED",
                        "python function execution cancelled by operator",
                        Some(json!({
                            "interpreter": interpreter.display().to_string(),
                            "exit_code": output.status.code(),
                        })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: None,
                });
            }
            ControlledCommandResult::Exited(output) => {
                if !output.status.success() {
                    let stderr = output.read_tail_bytes(PYTHON_FAILURE_STDERR_READ_BYTES)?;
                    let failure = python_failure_from_stderr(&stderr, output.status.code())
                        .unwrap_or_else(|| {
                            python_failure(
                                "EXEC_FAIL",
                                "python adapter command failed",
                                Some(json!({
                                    "interpreter": interpreter.display().to_string(),
                                    "exit_code": output.status.code(),
                                })),
                            )
                        });
                    return Ok(NodeResult {
                        status: NodeStatus::Failed,
                        stdout_path: stdout_path.display().to_string(),
                        stderr_path: stderr_path.display().to_string(),
                        outputs_dir: outputs_dir.display().to_string(),
                        output_evidence: Vec::new(),
                        failure: Some(failure),
                        attempts: 1,
                        attempt_events: Vec::new(),
                        container_meta: None,
                        adapter_binary_sha256: None,
                    });
                }
            }
        }

        let output_report = crate::inspect_declared_outputs(&outputs_dir, &node.outputs);
        if let Some(failure) = output_report.failure {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: output_report.output_evidence,
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            });
        }
        let fingerprint = crate::node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fingerprint, &output_report.present_outputs)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}
