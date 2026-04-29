use crate::{
    adapter::{AdapterDescriptor, AdapterOrigin, EffectSet},
    adapter_conformance, Adapter, AdapterId, NodeCtx, NodeResult, RuntimeError,
};
use bijux_dag_artifacts::write_outputs_index;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

const MAX_NODE_SPEC_BYTES: usize = 256 * 1024;

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

        let node_spec = serde_json::to_string(node)?;
        if node_spec.len() > MAX_NODE_SPEC_BYTES {
            return Err(RuntimeError::Executor(format!(
                "node spec payload exceeds {} bytes",
                MAX_NODE_SPEC_BYTES
            )));
        }
        let mut cmd = Command::new(&self.path);
        cmd.args([
            "execute",
            "--node-spec",
            &node_spec,
            "--workdir",
            &work_dir.display().to_string(),
            "--outdir",
            &outputs_dir.display().to_string(),
        ]);
        cmd.current_dir(&work_dir);
        cmd.env_clear();
        for (key, value) in
            crate::shaped_environment(exec.policy.clean_env, &node.env_allowlist, &[])
        {
            cmd.env(key, value);
        }
        let output = crate::command_output_with_timeout(
            &mut cmd,
            node.timeout_ms.or_else(|| ctx.params.get("timeout_ms").and_then(|v| v.as_u64())),
        )?;

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;

        let output_paths = crate::declared_output_paths(node);
        if let Some(failure) = crate::validate_outputs_dir(&outputs_dir, &node.outputs) {
            return Ok(NodeResult {
                status: crate::NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: self.binary_hash.clone(),
            });
        }
        let fp = crate::node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        let success = output.status.success();
        let failure = if success {
            None
        } else {
            Some(crate::FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "adapter command failed".to_string(),
                details: None,
            })
        };

        Ok(NodeResult {
            status: if success { crate::NodeStatus::Success } else { crate::NodeStatus::Failed },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure,
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
        let handshake = Command::new(&path).args(["info", "--json"]).output();
        let (report, _) = match handshake {
            Ok(out) if out.status.success() => {
                match serde_json::from_slice::<ExternalAdapterInfo>(&out.stdout) {
                    Ok(info) => {
                        let binary_hash =
                            std::fs::read(&path).ok().map(|b| crate::sha256_bytes(&b));
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
                            Some(Arc::new(adapter) as Arc<dyn Adapter>),
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
                }
            }
            Ok(out) => (
                ExternalAdapterHandshakeReport {
                    path: path.display().to_string(),
                    status: ExternalAdapterHandshakeStatus::Rejected,
                    adapter_id: None,
                    adapter_version: None,
                    descriptor: None,
                    violations: Vec::new(),
                    reason: Some(format!(
                        "info handshake failed with exit code {}",
                        out.status.code().unwrap_or(-1)
                    )),
                },
                None,
            ),
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
        };
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
        let Ok(out) = Command::new(&path).args(["info", "--json"]).output() else {
            continue;
        };
        if !out.status.success() {
            continue;
        }
        let Ok(info) = serde_json::from_slice::<ExternalAdapterInfo>(&out.stdout) else {
            continue;
        };
        let binary_hash = std::fs::read(&path).ok().map(|b| crate::sha256_bytes(&b));
        let adapter = ExternalAdapter::new(path.clone(), info, binary_hash);
        let descriptor = adapter.descriptor();
        if !adapter_conformance::validate_descriptor(&descriptor).passed {
            continue;
        }
        adapters.push(Arc::new(adapter));
    }
    Ok(adapters)
}

fn canonicalize_external_adapter_path(path: &PathBuf) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    std::fs::canonicalize(path).ok()
}
