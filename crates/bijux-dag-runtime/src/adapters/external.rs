use crate::{
    adapter::{AdapterDescriptor, AdapterOrigin, EffectSet},
    adapter_conformance, Adapter, AdapterId, FailureClass, FailureInfo, NodeCtx, NodeResult,
    RuntimeError,
};
use bijux_dag_artifacts::write_outputs_index;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Output;
use std::sync::Arc;

const MAX_NODE_SPEC_BYTES: usize = 256 * 1024;
const MAX_INFO_HANDSHAKE_BYTES: usize = 64 * 1024;
const MAX_FAILURE_ENVELOPE_BYTES: u64 = 64 * 1024;
const EXTERNAL_ADAPTER_FAILURE_FILE: &str = "adapter-failure.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterInfo {
    pub protocol_version: String,
    #[serde(alias = "id")]
    pub adapter_id: String,
    #[serde(alias = "version")]
    pub adapter_version: String,
    pub required_effects: ExternalEffectSet,
    pub supported_kinds: Vec<String>,
    #[serde(alias = "produces_outputs_schema_version", alias = "output_schema")]
    pub output_schema: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalEffectSet {
    pub filesystem: bool,
    pub env: bool,
    pub network: bool,
    pub clock: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalAdapterFailureEnvelope {
    #[serde(default)]
    class: Option<FailureClass>,
    kind: String,
    code: String,
    message: String,
    #[serde(default)]
    details: Option<Value>,
}

#[derive(Clone)]
pub struct ExternalAdapter {
    path: PathBuf,
    info: ExternalAdapterInfo,
    binary_hash: Option<String>,
}

impl ExternalAdapter {
    pub fn new(path: PathBuf, info: ExternalAdapterInfo, binary_hash: Option<String>) -> Self {
        Self { path, info, binary_hash }
    }
}

impl TryFrom<ExternalAdapterFailureEnvelope> for FailureInfo {
    type Error = String;

    fn try_from(value: ExternalAdapterFailureEnvelope) -> Result<Self, Self::Error> {
        if value.kind.trim().is_empty() {
            return Err("failure envelope kind must not be empty".to_string());
        }
        if value.code.trim().is_empty() {
            return Err("failure envelope code must not be empty".to_string());
        }
        if value.message.trim().is_empty() {
            return Err("failure envelope message must not be empty".to_string());
        }
        Ok(FailureInfo {
            class: value.class,
            kind: value.kind,
            code: value.code,
            message: value.message,
            details: value.details,
        })
    }
}

fn failure_path(work_dir: &Path) -> PathBuf {
    work_dir.join(EXTERNAL_ADAPTER_FAILURE_FILE)
}

fn failure_details_with_runtime_context(
    details: Option<Value>,
    exit_code: Option<i32>,
    failure_path: &Path,
) -> Option<Value> {
    let mut object = match details {
        Some(Value::Object(map)) => map,
        Some(value) => {
            let mut map = serde_json::Map::new();
            map.insert("adapter_details".to_string(), value);
            map
        }
        None => serde_json::Map::new(),
    };
    object.entry("exit_code".to_string()).or_insert_with(|| json!(exit_code));
    object
        .entry("failure_path".to_string())
        .or_insert_with(|| json!(failure_path.display().to_string()));
    Some(Value::Object(object))
}

fn invalid_failure_envelope(
    message: impl Into<String>,
    failure_path: &Path,
    details: Value,
) -> FailureInfo {
    FailureInfo::new(
        FailureClass::Execution,
        "Execution",
        "ADAPTER_FAILURE_SCHEMA_INVALID",
        message.into(),
        Some(
            failure_details_with_runtime_context(Some(details), None, failure_path)
                .unwrap_or(Value::Null),
        ),
    )
}

fn structured_failure_from_file(
    fs: &dyn crate::Fs,
    failure_path: &Path,
    exit_code: Option<i32>,
) -> Result<Option<FailureInfo>, RuntimeError> {
    let metadata = match fs.metadata(failure_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.len() > MAX_FAILURE_ENVELOPE_BYTES {
        return Ok(Some(invalid_failure_envelope(
            "external adapter failure envelope exceeds the runtime size limit",
            failure_path,
            json!({
                "reason": "payload_too_large",
                "size_bytes": metadata.len(),
                "max_bytes": MAX_FAILURE_ENVELOPE_BYTES,
            }),
        )));
    }

    let raw = fs.read_to_string(failure_path)?;
    let envelope: ExternalAdapterFailureEnvelope = match serde_json::from_str(&raw) {
        Ok(envelope) => envelope,
        Err(error) => {
            return Ok(Some(invalid_failure_envelope(
                "external adapter failure envelope is not valid JSON",
                failure_path,
                json!({
                    "reason": "invalid_json",
                    "details": error.to_string(),
                }),
            )));
        }
    };
    let mut failure = match FailureInfo::try_from(envelope) {
        Ok(failure) => failure,
        Err(error) => {
            return Ok(Some(invalid_failure_envelope(
                "external adapter failure envelope is missing required fields",
                failure_path,
                json!({
                    "reason": "invalid_shape",
                    "details": error,
                }),
            )));
        }
    };
    failure.details =
        failure_details_with_runtime_context(failure.details, exit_code, failure_path);
    Ok(Some(failure))
}

impl Adapter for ExternalAdapter {
    fn id(&self) -> AdapterId {
        AdapterId { id: self.info.adapter_id.clone(), version: self.info.adapter_version.clone() }
    }

    fn supported_kinds(&self) -> Vec<String> {
        self.info.supported_kinds.clone()
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: self.info.required_effects.filesystem,
            env: self.info.required_effects.env,
            network: self.info.required_effects.network,
            clock: self.info.required_effects.clock,
        }
    }

    fn produces_outputs_schema_version(&self) -> String {
        self.info.output_schema.clone()
    }

    fn protocol_version(&self) -> String {
        self.info.protocol_version.clone()
    }

    fn origin(&self) -> AdapterOrigin {
        AdapterOrigin::External
    }

    fn binary_hash(&self) -> Option<String> {
        self.binary_hash.clone()
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
            exec.fs.write(&stdout_path, b"")?;
            exec.fs.write(&stderr_path, failure.message.as_bytes())?;
            return Ok(NodeResult {
                status: crate::NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: Vec::new(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: self.binary_hash.clone(),
            });
        }

        let mut node_spec_value = serde_json::to_value(node)?;
        if let serde_json::Value::Object(map) = &mut node_spec_value {
            map.insert("params".to_string(), ctx.params.clone());
        }
        let node_spec = serde_json::to_string(&node_spec_value)?;
        if node_spec.len() > MAX_NODE_SPEC_BYTES {
            return Err(RuntimeError::Executor(format!(
                "node spec payload exceeds {} bytes",
                MAX_NODE_SPEC_BYTES
            )));
        }
        let mut cmd = Command::new(&self.path);
        let failure_path = failure_path(&work_dir);
        match exec.fs.remove_file(&failure_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        cmd.args([
            "execute",
            "--node-spec",
            &node_spec,
            "--workdir",
            &work_dir.display().to_string(),
            "--outdir",
            &outputs_dir.display().to_string(),
            "--failure-path",
            &failure_path.display().to_string(),
        ]);
        cmd.current_dir(&work_dir);
        let env_allowlist = crate::effective_env_allowlist(node);
        cmd.env_clear();
        for (key, value) in crate::shaped_environment(exec.policy.clean_env, &env_allowlist, &[]) {
            cmd.env(key, value);
        }
        crate::apply_temp_env(&mut cmd, &exec.run_dir.node_temp_dir(&node.id));
        let output = crate::command_output_with_controls(
            &mut cmd,
            node.timeout_ms.or_else(|| ctx.params.get("timeout_ms").and_then(|v| v.as_u64())),
            Some(exec.cancellation_requested.as_ref()),
        )?;

        output.persist_streams(exec.fs.as_ref(), &stdout_path, &stderr_path)?;
        let output = match output {
            crate::ControlledCommandResult::Exited(output) => output,
            crate::ControlledCommandResult::TimedOut(output) => {
                let quarantined_outputs_dir =
                    quarantine_partial_outputs(exec, &outputs_dir, &node_dir, "timeout")?;
                return Ok(NodeResult {
                    status: crate::NodeStatus::Failed,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(crate::FailureInfo::new(
                        bijux_dag_artifacts::FailureClass::Timeout,
                        "Timeout",
                        "EXEC_TIMEOUT",
                        "execution timed out after configured node timeout",
                        Some(json!({
                            "timeout_class": "external_adapter_process",
                            "quarantined_outputs_dir": quarantined_outputs_dir,
                            "exit_code": output.exit_code(),
                        })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: self.binary_hash.clone(),
                });
            }
            crate::ControlledCommandResult::Cancelled(output) => {
                let quarantined_outputs_dir =
                    quarantine_partial_outputs(exec, &outputs_dir, &node_dir, "cancelled")?;
                return Ok(NodeResult {
                    status: crate::NodeStatus::Cancelled,
                    stdout_path: stdout_path.display().to_string(),
                    stderr_path: stderr_path.display().to_string(),
                    outputs_dir: outputs_dir.display().to_string(),
                    output_evidence: Vec::new(),
                    failure: Some(crate::FailureInfo::new(
                        bijux_dag_artifacts::FailureClass::Execution,
                        "Execution",
                        "EXEC_CANCELLED",
                        "execution cancelled by operator",
                        Some(json!({
                            "quarantined_outputs_dir": quarantined_outputs_dir,
                            "exit_code": output.exit_code(),
                        })),
                    )),
                    attempts: 1,
                    attempt_events: Vec::new(),
                    container_meta: None,
                    adapter_binary_sha256: self.binary_hash.clone(),
                });
            }
        };
        let success = output.status.success();
        if !success {
            let failure =
                structured_failure_from_file(exec.fs.as_ref(), &failure_path, output.exit_code())?
                    .unwrap_or_else(|| {
                        FailureInfo::new(
                            bijux_dag_artifacts::FailureClass::Execution,
                            "Execution",
                            "EXEC_FAIL",
                            "adapter command failed",
                            Some(json!({
                                "exit_code": output.exit_code(),
                            })),
                        )
                    });
            return Ok(NodeResult {
                status: crate::NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: Vec::new(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: self.binary_hash.clone(),
            });
        }

        let output_report = crate::inspect_declared_outputs(&outputs_dir, &node.outputs);
        if let Some(failure) = output_report.failure {
            return Ok(NodeResult {
                status: crate::NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                output_evidence: output_report.output_evidence,
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: self.binary_hash.clone(),
            });
        }
        let fp = crate::node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_report.present_outputs)?;

        Ok(NodeResult {
            status: crate::NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            output_evidence: output_report.output_evidence,
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: self.binary_hash.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExternalAdapterHandshakeStatus {
    Ok,
    Rejected,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExternalAdapterHandshakeReport {
    pub path: String,
    pub status: ExternalAdapterHandshakeStatus,
    pub adapter_id: Option<String>,
    pub adapter_version: Option<String>,
    pub descriptor: Option<AdapterDescriptor>,
    pub violations: Vec<String>,
    pub reason: Option<String>,
}

pub fn probe_external_adapters() -> Result<Vec<ExternalAdapterHandshakeReport>, RuntimeError> {
    let dir = match std::env::var("BIJUX_DAG_ADAPTERS_DIR") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => return Ok(Vec::new()),
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    let mut reports = Vec::new();
    for entry in entries {
        let original_path = entry.path();
        if !original_path.is_file() {
            continue;
        }
        let Some(path) = canonicalize_external_adapter_path(&original_path) else {
            continue;
        };
        let (report, _) = handshake_report_for_path(&path);
        reports.push(report);
    }
    Ok(reports)
}

pub fn discover_external_adapters() -> Result<Vec<Arc<dyn Adapter>>, RuntimeError> {
    let mut adapters: Vec<Arc<dyn Adapter>> = Vec::new();
    let dir = match std::env::var("BIJUX_DAG_ADAPTERS_DIR") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => return Ok(adapters),
    };
    if !dir.exists() {
        return Ok(adapters);
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let original_path = entry.path();
        if !original_path.is_file() {
            continue;
        }
        let Some(path) = canonicalize_external_adapter_path(&original_path) else {
            continue;
        };
        let (_, adapter) = handshake_report_for_path(&path);
        if let Some(adapter) = adapter {
            adapters.push(adapter);
        }
    }
    Ok(adapters)
}

fn handshake_report_for_path(
    path: &PathBuf,
) -> (ExternalAdapterHandshakeReport, Option<Arc<dyn Adapter>>) {
    let handshake = Command::new(path).args(["info", "--json"]).output();
    match handshake {
        Ok(out) => match validate_info_handshake_output(&out) {
            Ok(()) => match serde_json::from_slice::<ExternalAdapterInfo>(&out.stdout) {
                Ok(info) => {
                    let binary_hash = std::fs::read(path).ok().map(|b| crate::sha256_bytes(&b));
                    let adapter = ExternalAdapter::new(path.clone(), info, binary_hash);
                    let descriptor = adapter.descriptor();
                    let conformance = adapter_conformance::validate_descriptor(&descriptor);
                    let status = if conformance.passed {
                        ExternalAdapterHandshakeStatus::Ok
                    } else {
                        ExternalAdapterHandshakeStatus::Rejected
                    };
                    (
                        ExternalAdapterHandshakeReport {
                            path: path.display().to_string(),
                            status,
                            adapter_id: Some(descriptor.id.clone()),
                            adapter_version: Some(descriptor.version.clone()),
                            descriptor: Some(descriptor),
                            violations: conformance.violations.clone(),
                            reason: if conformance.passed {
                                None
                            } else {
                                Some("descriptor validation failed".to_string())
                            },
                        },
                        if conformance.passed {
                            Some(Arc::new(adapter) as Arc<dyn Adapter>)
                        } else {
                            None
                        },
                    )
                }
                Err(error) => (
                    ExternalAdapterHandshakeReport {
                        path: path.display().to_string(),
                        status: ExternalAdapterHandshakeStatus::Rejected,
                        adapter_id: None,
                        adapter_version: None,
                        descriptor: None,
                        violations: Vec::new(),
                        reason: Some(format!("invalid adapter manifest: {error}")),
                    },
                    None,
                ),
            },
            Err(reason) => (
                ExternalAdapterHandshakeReport {
                    path: path.display().to_string(),
                    status: ExternalAdapterHandshakeStatus::Rejected,
                    adapter_id: None,
                    adapter_version: None,
                    descriptor: None,
                    violations: Vec::new(),
                    reason: Some(reason),
                },
                None,
            ),
        },
        Err(error) => (
            ExternalAdapterHandshakeReport {
                path: path.display().to_string(),
                status: ExternalAdapterHandshakeStatus::Rejected,
                adapter_id: None,
                adapter_version: None,
                descriptor: None,
                violations: Vec::new(),
                reason: Some(format!("failed to launch adapter info handshake: {error}")),
            },
            None,
        ),
    }
}

fn validate_info_handshake_output(output: &Output) -> Result<(), String> {
    if !output.status.success() {
        return Err(format!(
            "info handshake failed with exit code {}",
            output.status.code().unwrap_or(-1)
        ));
    }
    if !output.stderr.is_empty() {
        return Err(format!(
            "info handshake must write protocol JSON to stdout only; stderr was: {}",
            preview_bytes(&output.stderr)
        ));
    }
    if output.stdout.len() > MAX_INFO_HANDSHAKE_BYTES {
        return Err(format!("info handshake payload exceeds {} bytes", MAX_INFO_HANDSHAKE_BYTES));
    }
    Ok(())
}

fn preview_bytes(bytes: &[u8]) -> String {
    let limit = bytes.len().min(120);
    String::from_utf8_lossy(&bytes[..limit]).trim().to_string()
}

fn quarantine_partial_outputs(
    exec: &crate::RunContext,
    outputs_dir: &PathBuf,
    node_dir: &PathBuf,
    reason: &str,
) -> Result<Option<String>, RuntimeError> {
    if exec.fs.metadata(outputs_dir).is_err() {
        return Ok(None);
    }
    let entries = exec.fs.read_dir(outputs_dir)?;
    if entries.is_empty() {
        return Ok(None);
    }
    let quarantine_root = node_dir.join("quarantine");
    exec.fs.create_dir_all(&quarantine_root)?;
    let quarantine_dir =
        quarantine_root.join(format!("{}-outputs-{}", reason, exec.clock.now_unix_ms()));
    exec.fs.rename(outputs_dir, &quarantine_dir)?;
    exec.fs.create_dir_all(outputs_dir)?;
    let relative = quarantine_dir
        .strip_prefix(node_dir)
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| quarantine_dir.display().to_string());
    Ok(Some(relative))
}

fn canonicalize_external_adapter_path(path: &PathBuf) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    std::fs::canonicalize(path).ok()
}
