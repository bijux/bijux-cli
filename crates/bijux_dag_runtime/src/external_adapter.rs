use crate::{adapter::EffectSet, Adapter, AdapterId, NodeCtx, NodeResult, RuntimeError};
use bijux_dag_artifacts::write_outputs_index;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalAdapterInfo {
    pub id: String,
    pub version: String,
    pub required_effects: ExternalEffectSet,
    pub supported_kinds: Vec<String>,
    #[serde(default = "default_outputs_schema_version")]
    pub produces_outputs_schema_version: String,
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
        Self {
            path,
            info,
            binary_hash,
        }
    }
}

impl Adapter for ExternalAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: self.info.id.clone(),
            version: self.info.version.clone(),
        }
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
        self.info.produces_outputs_schema_version.clone()
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
        for key in &node.env_allowlist {
            if let Ok(val) = std::env::var(key) {
                cmd.env(key, val);
            }
        }
        let output = cmd.output()?;

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
        let fp = exec
            .graph_fingerprint
            .get(&node.id)
            .cloned()
            .unwrap_or_default();
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
            status: if success {
                crate::NodeStatus::Success
            } else {
                crate::NodeStatus::Failed
            },
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

pub fn discover_external_adapters() -> Result<Vec<Arc<dyn Adapter>>, RuntimeError> {
    let dir = match std::env::var("BIJUX_DAG_ADAPTERS_DIR") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => return Ok(Vec::new()),
    };
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    let mut adapters: Vec<Arc<dyn Adapter>> = Vec::new();
    for entry in entries {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let info = match Command::new(&path).args(["info", "--json"]).output() {
            Ok(out) if out.status.success() => {
                serde_json::from_slice::<ExternalAdapterInfo>(&out.stdout).ok()
            }
            _ => None,
        };
        let info = match info {
            Some(i) => i,
            None => continue,
        };
        let binary_hash = std::fs::read(&path).ok().map(|b| crate::sha256_bytes(&b));
        let adapter = ExternalAdapter::new(path.clone(), info, binary_hash);
        adapters.push(Arc::new(adapter));
    }
    Ok(adapters)
}

fn default_outputs_schema_version() -> String {
    "v0.1".to_string()
}
