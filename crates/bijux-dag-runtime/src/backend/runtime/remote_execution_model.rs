use crate::adapter::Adapter;
use crate::clock::SystemClock;
use crate::io::{Fs, StdFs};
use crate::store::ArtifactStore;
use crate::{
    failed_node_result_from_runtime_error, AbsolutePathPolicy, ConstAdapter, NodeResult,
    PolicyConfig, RunContext, ShellAdapter,
};
use bijux_dag_artifacts::{is_normalized_relative_path, RunDir};
use bijux_dag_core::{Graph, Node, NodeKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionIdentity {
    pub run_id: String,
    pub node_id: String,
    pub attempt_id: String,
    pub backend_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteArtifactHandoff {
    pub upload_endpoint: String,
    pub download_endpoint: String,
    pub integrity_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteObservabilityHandoff {
    pub stream_mode: String,
    pub trace_forwarding: bool,
    pub retention_days_hint: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionWorkspace {
    pub out_base: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionFingerprintSet {
    pub node_fingerprint: String,
    pub node_definition_fingerprint: String,
    pub declared_environment_fingerprint: String,
    pub params_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_fingerprint: Option<String>,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub execution_contract_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteInputArtifact {
    pub relative_path: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteNodeExecutionPayload {
    pub identity: RemoteExecutionIdentity,
    pub graph: Graph,
    pub node: Node,
    pub params: Value,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_artifacts: Vec<RemoteInputArtifact>,
    pub workspace: RemoteExecutionWorkspace,
    pub policy: PolicyConfig,
    pub absolute_path_policy: AbsolutePathPolicy,
    pub planner_contract_version: String,
    pub fingerprints: RemoteExecutionFingerprintSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteNodeExecutionResult {
    pub identity: RemoteExecutionIdentity,
    pub node_result: NodeResult,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
}

pub trait RemoteWorkerExecutor: Send + Sync {
    fn execute_payload(
        &self,
        payload: RemoteNodeExecutionPayload,
    ) -> Result<RemoteNodeExecutionResult, String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MockRemoteWorker;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionModeStatus {
    Implemented,
    Simulated,
    NotImplemented,
}

pub fn validate_remote_identity(identity: &RemoteExecutionIdentity) -> Result<(), String> {
    for value in [&identity.run_id, &identity.node_id, &identity.attempt_id, &identity.backend_id] {
        if value.trim().is_empty() {
            return Err("remote identity fields must be non-empty".to_string());
        }
    }
    Ok(())
}

pub fn validate_remote_execution_workspace(
    workspace: &RemoteExecutionWorkspace,
) -> Result<(), String> {
    if Path::new(&workspace.out_base).as_os_str().is_empty() {
        return Err("remote workspace out_base must be non-empty".to_string());
    }
    if workspace.cache_dir.as_ref().is_some_and(|cache_dir| cache_dir.trim().is_empty()) {
        return Err("remote workspace cache_dir must be non-empty when provided".to_string());
    }
    Ok(())
}

pub fn remote_input_artifact_digest_matches(artifact: &RemoteInputArtifact) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(&artifact.bytes);
    let digest = format!("{:x}", hasher.finalize());
    digest == artifact.sha256
}

pub fn validate_remote_input_artifact(artifact: &RemoteInputArtifact) -> Result<(), String> {
    if artifact.relative_path.trim().is_empty() {
        return Err("remote input artifact path must be non-empty".to_string());
    }
    if !is_normalized_relative_path(&artifact.relative_path) {
        return Err(format!(
            "remote input artifact path must be normalized and relative: {}",
            artifact.relative_path
        ));
    }
    if artifact.sha256.trim().is_empty() {
        return Err(format!(
            "remote input artifact '{}' must include sha256",
            artifact.relative_path
        ));
    }
    if !remote_input_artifact_digest_matches(artifact) {
        return Err(format!(
            "remote input artifact '{}' sha256 does not match payload bytes",
            artifact.relative_path
        ));
    }
    Ok(())
}

pub fn validate_remote_execution_fingerprint_set(
    fingerprints: &RemoteExecutionFingerprintSet,
) -> Result<(), String> {
    for (label, value) in [
        ("node_fingerprint", &fingerprints.node_fingerprint),
        ("node_definition_fingerprint", &fingerprints.node_definition_fingerprint),
        ("declared_environment_fingerprint", &fingerprints.declared_environment_fingerprint),
        ("params_fingerprint", &fingerprints.params_fingerprint),
        ("execution_fingerprint", &fingerprints.execution_fingerprint),
        ("evidence_fingerprint", &fingerprints.evidence_fingerprint),
        ("execution_contract_fingerprint", &fingerprints.execution_contract_fingerprint),
    ] {
        if value.trim().is_empty() {
            return Err(format!("remote execution fingerprint '{label}' must be non-empty"));
        }
    }
    Ok(())
}

pub fn validate_remote_execution_payload(
    payload: &RemoteNodeExecutionPayload,
) -> Result<(), String> {
    validate_remote_identity(&payload.identity)?;
    validate_remote_execution_workspace(&payload.workspace)?;
    validate_remote_execution_fingerprint_set(&payload.fingerprints)?;
    if payload.planner_contract_version.trim().is_empty() {
        return Err("remote planner contract version must be non-empty".to_string());
    }
    if payload.node.id != payload.identity.node_id {
        return Err(format!(
            "remote payload node '{}' does not match identity node '{}'",
            payload.node.id, payload.identity.node_id
        ));
    }
    if !payload.graph.nodes.iter().any(|node| node.id == payload.node.id) {
        return Err(format!("remote payload graph does not contain node '{}'", payload.node.id));
    }
    for artifact in &payload.input_artifacts {
        validate_remote_input_artifact(artifact)?;
    }
    Ok(())
}

pub fn execution_mode_status(mode: &str) -> ExecutionModeStatus {
    match mode {
        "local" | "subprocess" => ExecutionModeStatus::Implemented,
        "container"
        | "remote-contract"
        | "remote-worker"
        | "k8s"
        | "kubernetes-contract"
        | "kubernetes"
        | "kubernetes-job"
        | "hpc"
        | "slurm" => ExecutionModeStatus::Simulated,
        _ => ExecutionModeStatus::NotImplemented,
    }
}

pub fn remote_handoff_valid(
    artifact: &RemoteArtifactHandoff,
    observability: &RemoteObservabilityHandoff,
) -> bool {
    !artifact.upload_endpoint.trim().is_empty()
        && !artifact.download_endpoint.trim().is_empty()
        && !observability.stream_mode.trim().is_empty()
}

pub fn serialize_node_result_payload(result: &NodeResult) -> Result<Value, String> {
    serde_json::to_value(result).map_err(|error| format!("serialize node result payload: {error}"))
}

impl RemoteWorkerExecutor for MockRemoteWorker {
    fn execute_payload(
        &self,
        payload: RemoteNodeExecutionPayload,
    ) -> Result<RemoteNodeExecutionResult, String> {
        validate_remote_execution_payload(&payload)?;
        let adapter = remote_worker_adapter(&payload.node.kind)?;
        execute_modeled_payload(payload, adapter)
    }
}

pub fn execute_remote_payload_in_place(
    payload: RemoteNodeExecutionPayload,
) -> Result<RemoteNodeExecutionResult, String> {
    validate_remote_execution_payload(&payload)?;
    let adapter = remote_worker_adapter(&payload.node.kind)?;
    execute_modeled_payload_with_workspace(payload, adapter, RemoteWorkspaceMode::ReuseExisting)
}

pub(crate) fn execute_modeled_payload(
    payload: RemoteNodeExecutionPayload,
    adapter: Box<dyn Adapter>,
) -> Result<RemoteNodeExecutionResult, String> {
    execute_modeled_payload_with_workspace(payload, adapter, RemoteWorkspaceMode::CreateFresh)
}

enum RemoteWorkspaceMode {
    CreateFresh,
    ReuseExisting,
}

fn execute_modeled_payload_with_workspace(
    payload: RemoteNodeExecutionPayload,
    adapter: Box<dyn Adapter>,
    workspace_mode: RemoteWorkspaceMode,
) -> Result<RemoteNodeExecutionResult, String> {
    let run_dir = match workspace_mode {
        RemoteWorkspaceMode::CreateFresh => {
            RunDir::create_with_id(&payload.workspace.out_base, &payload.identity.run_id)
                .map_err(|error| format!("create remote run dir: {error}"))?
        }
        RemoteWorkspaceMode::ReuseExisting => {
            RunDir::resume_with_id(&payload.workspace.out_base, &payload.identity.run_id)
                .map_err(|error| format!("reuse remote run dir: {error}"))?
        }
    };
    let run_dir = Arc::new(run_dir);
    let fs: Arc<dyn Fs> = Arc::new(StdFs);
    if matches!(workspace_mode, RemoteWorkspaceMode::CreateFresh) {
        materialize_remote_inputs(
            fs.as_ref(),
            run_dir.as_ref(),
            &payload.node.id,
            &payload.input_artifacts,
        )?;
    }

    let node_id = payload.node.id.clone();
    let mut graph_fingerprint = HashMap::new();
    graph_fingerprint.insert(node_id.clone(), payload.fingerprints.node_fingerprint.clone());

    let mut node_definition_fingerprints = HashMap::new();
    node_definition_fingerprints
        .insert(node_id.clone(), payload.fingerprints.node_definition_fingerprint.clone());

    let mut declared_environment_fingerprints = HashMap::new();
    declared_environment_fingerprints
        .insert(node_id.clone(), payload.fingerprints.declared_environment_fingerprint.clone());

    let mut params_fingerprints = HashMap::new();
    params_fingerprints.insert(node_id.clone(), payload.fingerprints.params_fingerprint.clone());

    let mut command_fingerprints = HashMap::new();
    command_fingerprints.insert(node_id.clone(), payload.fingerprints.command_fingerprint.clone());

    let mut resolved_params = HashMap::new();
    resolved_params.insert(node_id.clone(), payload.params.clone());

    let ctx = RunContext {
        run_dir: Arc::clone(&run_dir),
        replay_source_run_dir: None,
        graph_fingerprint: Arc::new(Mutex::new(graph_fingerprint)),
        node_definition_fingerprints: Arc::new(node_definition_fingerprints),
        declared_environment_fingerprints: Arc::new(declared_environment_fingerprints),
        params_fingerprints: Arc::new(params_fingerprints),
        command_fingerprints: Arc::new(command_fingerprints),
        planner_contract_version: payload.planner_contract_version.clone(),
        execution_fingerprint: payload.fingerprints.execution_fingerprint.clone(),
        evidence_fingerprint: payload.fingerprints.evidence_fingerprint.clone(),
        execution_contract_fingerprint: payload.fingerprints.execution_contract_fingerprint.clone(),
        resolved_params,
        effective_cache_dir: payload.workspace.cache_dir.as_ref().map(PathBuf::from),
        fs: Arc::clone(&fs),
        clock: Arc::new(SystemClock),
        store: ArtifactStore::new(Arc::clone(&run_dir), Arc::clone(&fs)),
        policy: payload.policy.clone(),
        absolute_path_policy: payload.absolute_path_policy,
        cancellation_requested: Arc::new(AtomicBool::new(false)),
    };

    let node_ctx = crate::NodeCtx {
        graph: &payload.graph,
        node: &payload.node,
        exec: &ctx,
        params: &payload.params,
    };
    let started_unix_ms = ctx.clock.now_unix_ms();
    let node_result = match adapter.execute(&node_ctx) {
        Ok(result) => result,
        Err(error) => failed_node_result_from_runtime_error(&ctx, &payload.node, error),
    };
    let finished_unix_ms = ctx.clock.now_unix_ms();
    Ok(RemoteNodeExecutionResult {
        identity: payload.identity,
        node_result,
        started_unix_ms,
        finished_unix_ms,
    })
}

fn materialize_remote_inputs(
    fs: &dyn Fs,
    run_dir: &RunDir,
    node_id: &str,
    input_artifacts: &[RemoteInputArtifact],
) -> Result<(), String> {
    let inputs_dir = run_dir.node_inputs_dir(node_id);
    fs.create_dir_all(&inputs_dir).map_err(|error| format!("create remote inputs dir: {error}"))?;
    for artifact in input_artifacts {
        validate_remote_input_artifact(artifact)?;
        let target = inputs_dir.join(&artifact.relative_path);
        if let Some(parent) = target.parent() {
            fs.create_dir_all(parent)
                .map_err(|error| format!("create remote input parent dir: {error}"))?;
        }
        fs.write(&target, &artifact.bytes).map_err(|error| {
            format!("write remote input artifact '{}': {error}", artifact.relative_path)
        })?;
    }
    Ok(())
}

fn remote_worker_adapter(kind: &NodeKind) -> Result<Box<dyn Adapter>, String> {
    match kind {
        NodeKind::Const => Ok(Box::new(ConstAdapter)),
        NodeKind::Http => Ok(Box::new(crate::http_adapter::HttpRequestAdapter)),
        NodeKind::FileTransform => {
            Ok(Box::new(crate::file_transform_adapter::FileTransformAdapter))
        }
        NodeKind::Shell => Ok(Box::new(ShellAdapter)),
        NodeKind::Python => Ok(Box::new(crate::python_adapter::PythonFunctionAdapter)),
        NodeKind::Container => Err(
            "remote worker model currently supports non-container built-in adapters; container remains local-only"
                .to_string(),
        ),
        NodeKind::External(kind) => Err(format!(
            "remote worker model does not yet execute external adapter kind '{kind}'"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::Adapter;
    use crate::{AbsolutePathPolicy, NodeCtx, PolicyConfig, RunContext};
    use bijux_dag_artifacts::RunDir;
    use bijux_dag_core::parse_graph_strict;
    use serde_json::json;

    fn remote_payload(
        out_base: &Path,
        run_id: &str,
        graph: Graph,
        node: Node,
        params: Value,
    ) -> RemoteNodeExecutionPayload {
        RemoteNodeExecutionPayload {
            identity: RemoteExecutionIdentity {
                run_id: run_id.to_string(),
                node_id: node.id.clone(),
                attempt_id: "1".to_string(),
                backend_id: "remote-worker".to_string(),
            },
            graph,
            node,
            params,
            input_artifacts: Vec::new(),
            workspace: RemoteExecutionWorkspace {
                out_base: out_base.display().to_string(),
                cache_dir: None,
            },
            policy: PolicyConfig::default(),
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            planner_contract_version: "bijux-dag-planner/v1".to_string(),
            fingerprints: RemoteExecutionFingerprintSet {
                node_fingerprint: "node-fp".to_string(),
                node_definition_fingerprint: "node-def-fp".to_string(),
                declared_environment_fingerprint: "env-fp".to_string(),
                params_fingerprint: "params-fp".to_string(),
                command_fingerprint: Some("command-fp".to_string()),
                execution_fingerprint: "execution-fp".to_string(),
                evidence_fingerprint: "evidence-fp".to_string(),
                execution_contract_fingerprint: "execution-contract-fp".to_string(),
            },
        }
    }

    fn local_const_result(out_base: &Path, run_id: &str) -> NodeResult {
        let graph = parse_graph_strict(
            r#"{
              "spec": "bijux-dag/v0.1",
              "nodes": [
                {
                  "id": "const-node",
                  "kind": "const",
                  "outputs": [{"name": "value", "path": "value.txt"}],
                  "params": {"value": "hello"}
                }
              ],
              "edges": []
            }"#,
        )
        .expect("parse graph");
        let node = graph.nodes[0].clone();
        let run_dir = Arc::new(RunDir::create_with_id(out_base, run_id).expect("create run dir"));
        let fs: Arc<dyn Fs> = Arc::new(StdFs);
        let mut graph_fingerprint = HashMap::new();
        graph_fingerprint.insert(node.id.clone(), "node-fp".to_string());
        let ctx = RunContext {
            run_dir: Arc::clone(&run_dir),
            replay_source_run_dir: None,
            graph_fingerprint: Arc::new(Mutex::new(graph_fingerprint)),
            node_definition_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                "node-def-fp".to_string(),
            )])),
            declared_environment_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                "env-fp".to_string(),
            )])),
            params_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                "params-fp".to_string(),
            )])),
            command_fingerprints: Arc::new(HashMap::from([(
                node.id.clone(),
                Some("command-fp".to_string()),
            )])),
            planner_contract_version: "bijux-dag-planner/v1".to_string(),
            execution_fingerprint: "execution-fp".to_string(),
            evidence_fingerprint: "evidence-fp".to_string(),
            execution_contract_fingerprint: "execution-contract-fp".to_string(),
            resolved_params: HashMap::from([(node.id.clone(), json!({"value": "hello"}))]),
            effective_cache_dir: None,
            fs: Arc::clone(&fs),
            clock: Arc::new(SystemClock),
            store: ArtifactStore::new(Arc::clone(&run_dir), Arc::clone(&fs)),
            policy: PolicyConfig::default(),
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            cancellation_requested: Arc::new(AtomicBool::new(false)),
        };
        ConstAdapter
            .execute(&NodeCtx {
                graph: &graph,
                node: &node,
                exec: &ctx,
                params: &json!({"value": "hello"}),
            })
            .expect("local const execute")
    }

    fn shape(value: &Value) -> Value {
        match value {
            Value::Null => Value::String("null".to_string()),
            Value::Bool(_) => Value::String("bool".to_string()),
            Value::Number(_) => Value::String("number".to_string()),
            Value::String(_) => Value::String("string".to_string()),
            Value::Array(items) => Value::Array(items.iter().map(shape).collect()),
            Value::Object(map) => {
                let shaped = map.iter().map(|(key, entry)| (key.clone(), shape(entry))).collect();
                Value::Object(shaped)
            }
        }
    }

    #[test]
    fn remote_worker_const_result_uses_same_node_result_schema_as_local_execution() {
        let graph = parse_graph_strict(
            r#"{
              "spec": "bijux-dag/v0.1",
              "nodes": [
                {
                  "id": "const-node",
                  "kind": "const",
                  "outputs": [{"name": "value", "path": "value.txt"}],
                  "params": {"value": "hello"}
                }
              ],
              "edges": []
            }"#,
        )
        .expect("parse graph");
        let node = graph.nodes[0].clone();
        let temp = tempfile::tempdir().expect("temp dir");
        let payload =
            remote_payload(temp.path(), "remote-const", graph, node, json!({"value": "hello"}));

        let remote = MockRemoteWorker.execute_payload(payload).expect("remote execute");
        let local = local_const_result(temp.path(), "local-const");

        let remote_shape =
            shape(&serialize_node_result_payload(&remote.node_result).expect("remote value"));
        let local_shape = shape(&serialize_node_result_payload(&local).expect("local value"));
        assert_eq!(remote_shape, local_shape);
    }
}
