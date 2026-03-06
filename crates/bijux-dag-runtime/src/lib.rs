mod adapter;
mod clock;
mod engine;
mod external_adapter;
mod io;
mod planner;
mod registry;
mod store;

use adapter::{Adapter, AdapterId, EffectSet, NodeCtx};
use bijux_dag_artifacts::{
    write_inputs_index, write_outputs_index, AdapterInfo, ArtifactError, CacheProof,
    ContainerTrace, FailureInfo, InputFile, InputsIndex, NodeCounts, NodeTrace, OutputSummary,
    OutputsIndex, Resources as TraceResources, RunDir, RunOutputFile, RunOutputsIndex,
};
use bijux_dag_core::{
    Effect, FileOutput, Graph, GraphError, Node, NodeKind, RetryPolicy, Severity,
};
use clock::{Clock, SystemClock};
use io::{Fs, StdFs};
use planner::build_plan;
use registry::{build_registry, AdapterRegistry};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap};
use std::io::{self as std_io, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use store::{ArtifactStore, CacheStore};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("artifact error: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("io error: {0}")]
    Io(#[from] std_io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("executor error: {0}")]
    Executor(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeStatus {
    Success,
    Failed,
    Skipped,
    Cached,
}

pub struct RunContext {
    pub run_dir: Arc<RunDir>,
    pub graph_fingerprint: Arc<Mutex<HashMap<String, String>>>,
    pub resolved_params: HashMap<String, Value>,
    pub fs: Arc<dyn Fs>,
    pub clock: Arc<dyn Clock>,
    pub store: ArtifactStore,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub status: NodeStatus,
    pub stdout_path: String,
    pub stderr_path: String,
    pub outputs_dir: String,
    pub failure: Option<FailureInfo>,
    pub attempts: u32,
    pub attempt_events: Vec<AttemptEvent>,
    pub container_meta: Option<bijux_dag_artifacts::ContainerTrace>,
    pub adapter_binary_sha256: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttemptEvent {
    pub attempt: u32,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub status: NodeStatus,
}

#[derive(Clone)]
pub struct ConstAdapter;

impl Adapter for ConstAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "const".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["const".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet::default()
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let params = ctx.params;
        let node_dir = exec.run_dir.node_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs
            .create_dir_all(exec.run_dir.node_outputs_dir(&node.id).as_path())?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);

        let value = params.get("value").cloned().unwrap_or(Value::Null);
        let target = node
            .outputs
            .iter()
            .find(|o| o.name == "value")
            .or_else(|| node.outputs.first())
            .ok_or_else(|| RuntimeError::Executor("no outputs declared".to_string()))?;
        let out_path = outputs_dir.join(&target.path);
        if let Some(parent) = out_path.parent() {
            exec.fs.create_dir_all(parent)?;
        }
        exec.fs
            .write(&out_path, &serde_json::to_vec_pretty(&value)?)?;
        exec.fs.write(&stdout_path, b"")?;
        exec.fs.write(&stderr_path, b"")?;
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        let output_paths = declared_output_paths(node);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure: None,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Clone)]
pub struct ShellAdapter;

impl Adapter for ShellAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "shell".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["shell".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: true,
            env: false,
            network: false,
            clock: false,
        }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let params = ctx.params;
        let argv = params
            .get("argv")
            .and_then(|v| v.as_array())
            .ok_or_else(|| RuntimeError::Executor("missing argv".to_string()))?;
        if argv.is_empty() {
            return Err(RuntimeError::Executor("empty argv".to_string()));
        }
        let mut args: Vec<String> = Vec::new();
        for v in argv {
            let s = v
                .as_str()
                .ok_or_else(|| RuntimeError::Executor("argv must be strings".to_string()))?;
            args.push(s.to_string());
        }

        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let mut cmd = Command::new(&args[0]);
        cmd.args(&args[1..]);
        cmd.current_dir(&work_dir);
        if exec.policy.clean_env {
            cmd.env_clear();
        }
        for key in &node.env_allowlist {
            if let Ok(val) = std::env::var(key) {
                cmd.env(key, val);
            }
        }

        let output = cmd.output()?;

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_paths = declared_output_paths(node);
        if let Some(failure) = validate_outputs_dir(&outputs_dir, &node.outputs) {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        let success = output.status.success();
        let failure = if success {
            None
        } else {
            Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "command failed".to_string(),
                details: None,
            })
        };

        Ok(NodeResult {
            status: if success {
                NodeStatus::Success
            } else {
                NodeStatus::Failed
            },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Clone)]
pub struct ContainerAdapter;

impl Adapter for ContainerAdapter {
    fn id(&self) -> AdapterId {
        AdapterId {
            id: "container".to_string(),
            version: "0.1".to_string(),
        }
    }

    fn supported_kinds(&self) -> Vec<String> {
        vec!["container".to_string()]
    }

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: true,
            env: false,
            network: false,
            clock: false,
        }
    }

    fn produces_outputs_schema_version(&self) -> String {
        "v0.1".to_string()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let spec = node
            .container
            .as_ref()
            .ok_or_else(|| RuntimeError::Executor("missing container spec".to_string()))?;

        let node_dir = exec.run_dir.node_dir(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        exec.fs.create_dir_all(&outputs_dir)?;
        exec.fs.create_dir_all(&node_dir)?;
        exec.fs.create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let engine = spec.engine.as_str();
        let engine_version = engine_version(engine);
        if engine_version.is_none() {
            exec.fs.write(&stdout_path, b"")?;
            exec.fs.write(
                &stderr_path,
                format!("container engine not available: {}", engine).as_bytes(),
            )?;
            return Ok(NodeResult {
                status: NodeStatus::Skipped,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: None,
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(spec, engine, None, engine_version)),
                adapter_binary_sha256: None,
            });
        }

        let mut cmd = Command::new(engine);
        cmd.arg("run").arg("--rm");

        if !node.effects.contains(&Effect::Network) || exec.policy.deny_network {
            cmd.args(["--network", "none"]);
        }

        cmd.args(["-v", &format!("{}:/bijux/node", node_dir.display())]);

        let workdir = spec
            .workdir
            .clone()
            .unwrap_or_else(|| "/bijux/node/work".to_string());
        cmd.args(["--workdir", &workdir]);

        for key in &spec.env_allowlist {
            if let Ok(val) = std::env::var(key) {
                cmd.arg("-e").arg(format!("{}={}", key, val));
            }
        }

        cmd.arg(&spec.image);
        for part in &spec.argv {
            cmd.arg(part);
        }

        let output = cmd.output()?;
        let exit_code = output.status.code();

        exec.fs.write(&stdout_path, &output.stdout)?;
        exec.fs.write(&stderr_path, &output.stderr)?;
        let output_paths = declared_output_paths(node);
        if let Some(failure) = validate_outputs_dir(&outputs_dir, &node.outputs) {
            return Ok(NodeResult {
                status: NodeStatus::Failed,
                stdout_path: stdout_path.display().to_string(),
                stderr_path: stderr_path.display().to_string(),
                outputs_dir: outputs_dir.display().to_string(),
                failure: Some(failure),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: Some(container_trace(spec, engine, exit_code, engine_version.clone())),
                adapter_binary_sha256: None,
            });
        }
        let fp = node_fingerprint_from_ctx(exec, &node.id);
        write_outputs_index(&outputs_dir, &node.id, &fp, &output_paths)?;

        let success = output.status.success();
        let failure = if success {
            None
        } else {
            Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "EXEC_FAIL".to_string(),
                message: "container command failed".to_string(),
                details: Some(serde_json::json!({ "exit_code": exit_code })),
            })
        };

        Ok(NodeResult {
            status: if success {
                NodeStatus::Success
            } else {
                NodeStatus::Failed
            },
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: Some(container_trace(spec, engine, exit_code, engine_version)),
            adapter_binary_sha256: None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheMode {
    Off,
    Read,
    ReadWrite,
}

struct CacheRead {
    hit: bool,
    proof: Option<CacheProof>,
}

pub struct RuntimeConfig {
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub run_timeout_ms: Option<u64>,
    pub node_timeout_ms: Option<u64>,
    pub materialize_inputs: MaterializeMode,
    pub cache_mode: CacheMode,
    pub cache_dir: Option<PathBuf>,
    pub remote_cache_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub latest_symlink: Option<PathBuf>,
    pub policy: PolicyConfig,
    pub selectors: SelectorSet,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            jobs: 1,
            cpu_budget: None,
            run_timeout_ms: None,
            node_timeout_ms: None,
            materialize_inputs: MaterializeMode::Copy,
            cache_mode: CacheMode::Off,
            cache_dir: None,
            remote_cache_dir: None,
            run_id: None,
            latest_symlink: None,
            policy: PolicyConfig::default(),
            selectors: SelectorSet::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectorSet {
    pub include: Vec<Selector>,
    pub exclude: Vec<Selector>,
}

#[derive(Debug, Clone)]
pub enum Selector {
    IdPrefix(String),
    Tag(String),
    Kind(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterializeMode {
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: true,
        }
    }
}

pub struct Runtime {
    registry: AdapterRegistry,
    fs: Arc<dyn Fs>,
    clock: Arc<dyn Clock>,
    init_error: Option<String>,
}

impl Runtime {
    pub fn new() -> Self {
        let registry_result = build_registry(vec![
            Arc::new(ConstAdapter),
            Arc::new(ShellAdapter),
            Arc::new(ContainerAdapter),
        ]);
        let (registry, init_error) = match registry_result {
            Ok(reg) => (reg, None),
            Err(err) => (AdapterRegistry::new(), Some(err.to_string())),
        };
        Self {
            registry,
            fs: Arc::new(StdFs),
            clock: Arc::new(SystemClock),
            init_error,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_io(fs: Arc<dyn Fs>, clock: Arc<dyn Clock>) -> Self {
        let mut runtime = Self::new();
        runtime.fs = fs;
        runtime.clock = clock;
        runtime
    }

    fn adapter_for_kind(&self, kind: &NodeKind) -> Result<Arc<dyn Adapter>, RuntimeError> {
        self.registry.resolve(kind.as_str())
    }

    fn adapter_meta_for_kind(&self, kind: &NodeKind) -> (String, String) {
        self.registry
            .resolve(kind.as_str())
            .map(|a| {
                let id = a.id();
                (id.id, id.version)
            })
            .unwrap_or_else(|_| ("unknown".to_string(), "unknown".to_string()))
    }

    fn adapter_schema_for_kind(&self, kind: &NodeKind) -> String {
        self.registry
            .resolve(kind.as_str())
            .map(|a| a.produces_outputs_schema_version())
            .unwrap_or_else(|_| "unknown".to_string())
    }

    pub fn run(
        &self,
        graph: &Graph,
        out_dir: impl AsRef<Path>,
        options: RuntimeConfig,
    ) -> Result<PathBuf, RuntimeError> {
        if let Some(err) = &self.init_error {
            return Err(RuntimeError::Executor(err.clone()));
        }
        let diags = graph.validate_with_warnings();
        if diags.iter().any(|d| d.severity == Severity::Error) {
            return Err(GraphError::ValidationFailed.into());
        }
        let plan = build_plan(graph, &options);
        engine::execute(self, graph, plan, out_dir, options)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn write_trace(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    status: NodeStatus,
    failure: Option<FailureInfo>,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    attempt: u32,
    cache_proof: Option<CacheProof>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
    container_meta: Option<ContainerTrace>,
    adapter_binary_sha256: Option<String>,
    skip_reason: Option<bijux_dag_artifacts::SkipReason>,
) -> Result<(), RuntimeError> {
    let node = graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?;
    ctx.store.ensure_node_dir(node_id)?;
    write_resolved_params(ctx, node_id)?;
    let inputs_index = if ctx
        .fs
        .metadata(ctx.run_dir.node_inputs_index_path(node_id).as_path())
        .is_ok()
    {
        Some("inputs/index.json".to_string())
    } else {
        None
    };
    let trace = NodeTrace {
        node_id: node_id.to_string(),
        status: status_string(&status),
        started_unix_ms,
        finished_unix_ms,
        attempt,
        fingerprint: node_fingerprint_from_ctx(ctx, node_id),
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        adapter_outputs_schema_version: adapter_outputs_schema_version.to_string(),
        adapter_binary_sha256,
        resources: node.resources.as_ref().map(|r| TraceResources {
            cpu: r.cpu,
            mem_mb: r.mem_mb,
        }),
        inputs_index,
        resolved_params: ctx.resolved_params.get(node_id).cloned(),
        container: container_meta,
        cache_proof,
        skip_reason,
        failure,
    };
    let data = serde_json::to_vec_pretty(&trace)?;
    ctx.store.write_trace(node_id, &data)?;
    Ok(())
}

fn status_string(status: &NodeStatus) -> String {
    match status {
        NodeStatus::Success => "success".to_string(),
        NodeStatus::Failed => "failed".to_string(),
        NodeStatus::Skipped => "skipped".to_string(),
        NodeStatus::Cached => "cached".to_string(),
    }
}

fn write_resolved_params(ctx: &RunContext, node_id: &str) -> Result<(), RuntimeError> {
    let mut params = ctx
        .resolved_params
        .get(node_id)
        .cloned()
        .unwrap_or(Value::Null);
    sort_value_maps(&mut params);
    let data = serde_json::to_vec_pretty(&params)?;
    ctx.store.write_resolved_params(node_id, &data)?;
    Ok(())
}

#[allow(dead_code)]
fn node_timeout_ms(
    node: &Node,
    resolved_params: &Value,
    default_ms: Option<u64>,
) -> Option<Duration> {
    let param_timeout = resolved_params.get("timeout_ms").and_then(|v| v.as_u64());
    let ms = node.timeout_ms.or(param_timeout).or(default_ms);
    ms.map(Duration::from_millis)
}

fn node_cpu(graph: &Graph, node_id: &str) -> u32 {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .and_then(|n| n.resources.as_ref().map(|r| r.cpu))
        .unwrap_or(1)
        .max(1)
}

fn execute_with_retries(
    adapter: &dyn Adapter,
    node: &Node,
    params: &Value,
    ctx: &RunContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    let mut attempt = 0u32;
    let max = retry.max_attempts;
    let mut attempt_events = Vec::new();
    loop {
        attempt += 1;
        let started = ctx.clock.now_unix_ms();
        let node_ctx = NodeCtx {
            node,
            exec: ctx,
            params,
        };
        let mut result = adapter.execute(&node_ctx)?;
        let finished = ctx.clock.now_unix_ms();
        attempt_events.push(AttemptEvent {
            attempt,
            started_unix_ms: started,
            finished_unix_ms: finished,
            status: result.status.clone(),
        });
        result.attempts = attempt;
        if result.status != NodeStatus::Failed {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        if attempt > max {
            result.attempt_events = attempt_events;
            return Ok(result);
        }
        if retry.backoff_ms > 0 {
            let wait = retry
                .backoff_ms
                .saturating_mul(attempt.saturating_sub(1) as u64);
            if wait > 0 {
                std::thread::sleep(Duration::from_millis(wait));
            }
        }
    }
}

fn append_event(file: &mut std::fs::File, value: serde_json::Value) -> Result<(), RuntimeError> {
    let line = serde_json::to_string(&value)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

fn cache_mode_string(mode: &CacheMode) -> Option<String> {
    match mode {
        CacheMode::Off => None,
        CacheMode::Read => Some("read".to_string()),
        CacheMode::ReadWrite => Some("readwrite".to_string()),
    }
}

fn tool_version() -> String {
    let base = env!("CARGO_PKG_VERSION");
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if out.status.success() {
            let commit = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !commit.is_empty() {
                return format!("{}+{}", base, commit);
            }
        }
    }
    base.to_string()
}

pub fn registered_adapters() -> Vec<AdapterInfo> {
    let registry = build_registry(vec![
        Arc::new(ConstAdapter),
        Arc::new(ShellAdapter),
        Arc::new(ContainerAdapter),
    ])
    .unwrap_or_else(|_| AdapterRegistry::new());
    registry.list()
}

fn materialize_inputs(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    mode: MaterializeMode,
) -> Result<InputsIndex, RuntimeError> {
    let inputs_dir = ctx.run_dir.node_inputs_dir(node_id);
    ctx.fs.create_dir_all(&inputs_dir)?;
    let mut files = Vec::new();
    for edge in &graph.edges {
        if edge.to.node_id != node_id {
            continue;
        }
        let from_node = graph
            .nodes
            .iter()
            .find(|n| n.id == edge.from.node_id)
            .ok_or_else(|| RuntimeError::Executor("missing source node".to_string()))?;
        let out = from_node
            .outputs
            .iter()
            .find(|o| o.name == edge.from.port)
            .ok_or_else(|| RuntimeError::Executor("missing output port".to_string()))?;
        let src_path = ctx
            .run_dir
            .node_outputs_dir(&edge.from.node_id)
            .join(&out.path);
        let dst_dir = inputs_dir.join(&edge.from.node_id);
        ctx.fs.create_dir_all(&dst_dir)?;
        let dst_path = dst_dir.join(&edge.to.port);
        if let Some(parent) = dst_path.parent() {
            ctx.fs.create_dir_all(parent)?;
        }
        if ctx.fs.metadata(&src_path).is_ok() {
            materialize_file(ctx.fs.as_ref(), &src_path, &dst_path, mode)?;
            let data = ctx.fs.read(&dst_path)?;
            let sha = sha256_bytes(&data);
            let rel = dst_path.strip_prefix(&inputs_dir).unwrap_or(&dst_path);
            let rel_str = rel.to_string_lossy().to_string();
            let from_fp = node_fingerprint_from_ctx(ctx, &edge.from.node_id);
            files.push(InputFile {
                path: rel_str,
                sha256: sha,
                from_node: edge.from.node_id.clone(),
                from_node_fingerprint: from_fp,
                from_output: edge.from.port.clone(),
            });
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let index = InputsIndex { files };
    write_inputs_index(&inputs_dir, &index)?;
    Ok(index)
}

fn cache_dir_from_env() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

pub(crate) fn declared_output_paths(node: &Node) -> Vec<String> {
    node.outputs.iter().map(|o| o.path.clone()).collect()
}

#[allow(clippy::too_many_arguments)]
fn try_cache_read(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<CacheRead, RuntimeError> {
    if options.cache_mode == CacheMode::Off {
        return Ok(CacheRead {
            hit: false,
            proof: None,
        });
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let cache_store = match cache_dir {
        Some(d) => Some(CacheStore::new(d, Arc::clone(&fs))),
        None => {
            return Ok(CacheRead {
                hit: false,
                proof: None,
            })
        }
    };
    if options.cache_mode == CacheMode::Read || options.cache_mode == CacheMode::ReadWrite {
        let key = node_fingerprint.to_string();
        let store = cache_store.as_ref().unwrap();
        let entry = store.entry(&key);
        if store.fs().metadata(&entry).is_ok() {
            if !verify_cache_entry(
                store.fs(),
                &entry,
                &key,
                adapter_id,
                adapter_version,
                adapter_outputs_schema_version,
            )? {
                return Ok(CacheRead {
                    hit: false,
                    proof: Some(CacheProof {
                        hit: false,
                        key,
                        source: "local".to_string(),
                        verified: false,
                        reason: "corrupt".to_string(),
                        corrupt_detected: true,
                    }),
                });
            }
            let source = cache_source_from_meta(store.fs(), &entry)
                .unwrap_or_else(|| "local".to_string());
            let node_dir = ctx.run_dir.node_dir(&node.id);
            store.fs().create_dir_all(&node_dir)?;
            copy_dir_all(
                store.fs(),
                entry.join("outputs"),
                ctx.run_dir.node_outputs_dir(&node.id),
            )?;
            copy_dir_all(store.fs(), entry.join("logs"), node_dir.clone())?;
            return Ok(CacheRead {
                hit: true,
                proof: Some(CacheProof {
                    hit: true,
                    key,
                    source,
                    verified: true,
                    reason: "hit".to_string(),
                    corrupt_detected: false,
                }),
            });
        }
        if let Some(remote_dir) = options.remote_cache_dir.as_ref() {
            let remote_entry = remote_dir.join(&key);
            if store.fs().metadata(&remote_entry).is_ok() {
                if !verify_cache_entry(
                    store.fs(),
                    &remote_entry,
                    &key,
                    adapter_id,
                    adapter_version,
                    adapter_outputs_schema_version,
                )? {
                    return Ok(CacheRead {
                        hit: false,
                        proof: Some(CacheProof {
                            hit: false,
                            key,
                            source: "remote".to_string(),
                            verified: false,
                            reason: "remote_corrupt".to_string(),
                            corrupt_detected: true,
                        }),
                    });
                }
                let node_dir = ctx.run_dir.node_dir(&node.id);
                store.fs().create_dir_all(&node_dir)?;
                copy_dir_all(
                    store.fs(),
                    remote_entry.join("outputs"),
                    ctx.run_dir.node_outputs_dir(&node.id),
                )?;
                copy_dir_all(store.fs(), remote_entry.join("logs"), node_dir.clone())?;
                if let Some(local_dir) = options.cache_dir.as_ref() {
                    let local_entry = local_dir.join(&key);
                    let _ = copy_dir_all(store.fs(), &remote_entry, &local_entry);
                }
                return Ok(CacheRead {
                    hit: true,
                    proof: Some(CacheProof {
                        hit: true,
                        key,
                        source: "remote".to_string(),
                        verified: true,
                        reason: format!("fetched:{}", cache_dir_id(remote_dir)),
                        corrupt_detected: false,
                    }),
                });
            }
        }
    }
    Ok(CacheRead {
        hit: false,
        proof: None,
    })
}

#[allow(clippy::too_many_arguments)]
fn try_cache_write(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<(), RuntimeError> {
    if options.cache_mode != CacheMode::ReadWrite {
        return Ok(());
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let store = match cache_dir {
        Some(d) => CacheStore::new(d, Arc::clone(&fs)),
        None => return Ok(()),
    };
    let key = node_fingerprint.to_string();
    let entry = store.entry(&key);
    store.fs().create_dir_all(entry.join("outputs").as_path())?;
    store.fs().create_dir_all(entry.join("logs").as_path())?;
    let meta = serde_json::json!({
        "node_id": node.id,
        "node_fingerprint": key,
        "adapter_id": adapter_id,
        "adapter_version": adapter_version,
        "produces_outputs_schema_version": adapter_outputs_schema_version,
        "created_unix_ms": ctx.clock.now_unix_ms(),
        "cache_source": "local",
        "schema_version": "v0.1",
    });
    store.fs().write(
        entry.join("meta.json").as_path(),
        &serde_json::to_vec_pretty(&meta)?,
    )?;
    copy_dir_all(
        store.fs(),
        ctx.run_dir.node_outputs_dir(&node.id),
        entry.join("outputs"),
    )?;
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let _ = store.fs().copy(
        node_dir.join("stdout.log").as_path(),
        entry.join("logs").join("stdout.log").as_path(),
    );
    let _ = store.fs().copy(
        node_dir.join("stderr.log").as_path(),
        entry.join("logs").join("stderr.log").as_path(),
    );
    let _ = store.fs().copy(
        node_dir.join("trace.json").as_path(),
        entry.join("logs").join("trace.json").as_path(),
    );
    Ok(())
}

fn verify_cache_entry(
    fs: &dyn Fs,
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<bool, RuntimeError> {
    let index_path = entry.join("outputs").join("index.json");
    if fs.metadata(&index_path).is_err() {
        return Ok(false);
    }
    let meta_path = entry.join("meta.json");
    if fs.metadata(&meta_path).is_err() {
        return Ok(false);
    }
    let meta: serde_json::Value = serde_json::from_str(&fs.read_to_string(&meta_path)?)?;
    if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(expected_key) {
        return Ok(false);
    }
    if meta.get("adapter_id").and_then(|v| v.as_str()) != Some(adapter_id) {
        return Ok(false);
    }
    if meta.get("adapter_version").and_then(|v| v.as_str()) != Some(adapter_version) {
        return Ok(false);
    }
    if meta
        .get("produces_outputs_schema_version")
        .and_then(|v| v.as_str())
        != Some(adapter_outputs_schema_version)
    {
        return Ok(false);
    }
    let data = fs.read_to_string(&index_path)?;
    let index: OutputsIndex = serde_json::from_str(&data)?;
    for file in index.files {
        let path = entry.join("outputs").join(&file.path);
        if fs.metadata(&path).is_err() {
            return Ok(false);
        }
        let bytes = fs.read(&path)?;
        let sha = sha256_bytes(&bytes);
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

fn node_fingerprint_from_ctx(ctx: &RunContext, node_id: &str) -> String {
    ctx.graph_fingerprint
        .lock()
        .ok()
        .and_then(|map| map.get(node_id).cloned())
        .unwrap_or_default()
}

fn set_node_fingerprint(ctx: &RunContext, node_id: &str, fp: String) {
    if let Ok(mut map) = ctx.graph_fingerprint.lock() {
        map.insert(node_id.to_string(), fp);
    }
}

fn node_fingerprint_with_inputs(base_fp: &str, inputs: &InputsIndex) -> Result<String, RuntimeError> {
    let value = serde_json::json!({
        "base": base_fp,
        "inputs": &inputs.files,
    });
    Ok(sha256_bytes(&serde_json::to_vec_pretty(&value)?))
}

fn cache_dir_id(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn cache_source_from_meta(fs: &dyn Fs, entry: &Path) -> Option<String> {
    let meta_path = entry.join("meta.json");
    let data = fs.read_to_string(&meta_path).ok()?;
    let meta: serde_json::Value = serde_json::from_str(&data).ok()?;
    meta.get("cache_source")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn sort_value_maps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut sorted: BTreeMap<String, Value> = BTreeMap::new();
            let entries = std::mem::take(map);
            for (k, mut v) in entries {
                sort_value_maps(&mut v);
                sorted.insert(k, v);
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in sorted {
                new_map.insert(k, v);
            }
            *map = new_map;
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                sort_value_maps(v);
            }
        }
        _ => {}
    }
}

pub(crate) fn validate_outputs_dir(dir: &Path, outputs: &[FileOutput]) -> Option<FailureInfo> {
    for out in outputs {
        if out.path.contains("..") || out.path.starts_with('/') || out.path.starts_with('\\') {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_PATH_INVALID".to_string(),
                message: "invalid output path".to_string(),
                details: None,
            });
        }
        let path = dir.join(&out.path);
        if !path.exists() {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_MISSING".to_string(),
                message: format!("missing output file: {}", out.path),
                details: None,
            });
        }
        if path.is_dir() || path.is_symlink() {
            return Some(FailureInfo {
                kind: "Execution".to_string(),
                code: "OUTPUT_PATH_INVALID".to_string(),
                message: format!("output must be a file: {}", out.path),
                details: None,
            });
        }
    }
    None
}

fn container_trace(
    spec: &bijux_dag_core::ContainerSpec,
    engine: &str,
    exit_code: Option<i32>,
    engine_version: Option<String>,
) -> ContainerTrace {
    let image_digest = Command::new(engine)
        .args(["image", "inspect", "--format", "{{.Id}}", &spec.image])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        });
    ContainerTrace {
        image: spec.image.clone(),
        image_digest,
        engine: engine.to_string(),
        engine_version,
        exit_code,
    }
}

fn engine_version(engine: &str) -> Option<String> {
    Command::new(engine)
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            } else {
                None
            }
        })
}

fn collect_outputs_summary(
    fs: &dyn Fs,
    run_dir: &RunDir,
) -> Result<Vec<OutputSummary>, RuntimeError> {
    let mut out = Vec::new();
    let nodes_dir = run_dir.staging_path().join("nodes");
    if fs.metadata(&nodes_dir).is_ok() {
        for entry in fs.read_dir(&nodes_dir)? {
            let index_path = entry.path().join("outputs").join("index.json");
            if fs.metadata(&index_path).is_ok() {
                let data = fs.read_to_string(&index_path)?;
                let index: OutputsIndex = serde_json::from_str(&data)?;
                for f in index.files {
                    out.push(OutputSummary {
                        node_id: f.node_id,
                        node_fingerprint: f.node_fingerprint,
                        file: f.path,
                        sha256: f.sha256,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| {
        (a.node_id.clone(), a.file.clone()).cmp(&(b.node_id.clone(), b.file.clone()))
    });
    Ok(out)
}

fn build_run_outputs_index(
    run_dir: &RunDir,
    outputs: &[OutputSummary],
) -> Result<RunOutputsIndex, RuntimeError> {
    let mut files = Vec::new();
    for out in outputs {
        let rel = run_dir.node_output_relpath(&out.node_id, &out.file);
        files.push(RunOutputFile {
            node_id: out.node_id.clone(),
            node_fingerprint: out.node_fingerprint.clone(),
            sha256: out.sha256.clone(),
            path: rel,
        });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(RunOutputsIndex { files })
}

fn rustc_version() -> String {
    if let Ok(out) = Command::new("rustc").arg("--version").output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

fn count_nodes(status_map: &HashMap<String, NodeStatus>) -> NodeCounts {
    let mut counts = NodeCounts {
        success: 0,
        failed: 0,
        skipped: 0,
        cached: 0,
    };
    for status in status_map.values() {
        match status {
            NodeStatus::Success => counts.success += 1,
            NodeStatus::Failed => counts.failed += 1,
            NodeStatus::Skipped => counts.skipped += 1,
            NodeStatus::Cached => counts.cached += 1,
        }
    }
    counts
}

fn copy_dir_all(fs: &dyn Fs, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std_io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs.create_dir_all(dst)?;
    for entry in fs.read_dir(src)? {
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(fs, entry.path(), dst_path)?;
        } else {
            let _ = fs.copy(entry.path().as_path(), dst_path.as_path())?;
        }
    }
    Ok(())
}

fn materialize_file(
    fs: &dyn Fs,
    src: &Path,
    dst: &Path,
    mode: MaterializeMode,
) -> std_io::Result<()> {
    if fs.metadata(dst).is_ok() {
        let _ = fs.remove_file(dst);
    }
    match mode {
        MaterializeMode::Copy => {
            let _ = fs.copy(src, dst)?;
        }
        MaterializeMode::Hardlink => {
            if fs.hard_link(src, dst).is_err() {
                let _ = fs.copy(src, dst)?;
            }
        }
        MaterializeMode::Symlink => {
            if fs.symlink(src, dst).is_err() {
                let _ = fs.copy(src, dst)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::FixedClock;
    use bijux_dag_core::{ContainerSpec, Edge, Effect, ParamValue, PortRef, Severity, SPEC_VERSION};
    use std::collections::BTreeMap;
    use std::fs;

    fn param_object(items: Vec<(&str, Value)>) -> ParamValue {
        let mut map = BTreeMap::new();
        for (k, v) in items {
            map.insert(k.to_string(), ParamValue::Literal(v));
        }
        ParamValue::Object(map)
    }

    fn docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn sample_graph() -> Graph {
        Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("echo ok > ../outputs/out_b"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "out_a".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        }
    }

    #[test]
    fn run_produces_artifacts() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let diags = sample_graph().validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        assert!(final_path.join("manifest.json").exists());
        assert!(final_path.join("graph.snapshot.json").exists());
        assert!(final_path
            .join("nodes")
            .join("a")
            .join("resolved_params.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("resolved_params.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("trace.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("trace.json")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("stdout.log")
            .exists());
        assert!(final_path
            .join("nodes")
            .join("b")
            .join("outputs")
            .join("index.json")
            .exists());
    }

    #[test]
    fn shell_outputs_index_contains_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(index.contains("out_b"));
    }

    #[test]
    fn artifact_tree_contains_expected_entries() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let expected = vec![
            "manifest.json",
            "provenance.json",
            "graph.snapshot.json",
            "run.log.jsonl",
            "outputs/index.json",
            "nodes/a/trace.json",
            "nodes/a/stdout.log",
            "nodes/a/stderr.log",
            "nodes/a/resolved_params.json",
            "nodes/a/inputs/index.json",
            "nodes/a/outputs/index.json",
            "nodes/a/outputs/out_a",
            "nodes/b/trace.json",
            "nodes/b/stdout.log",
            "nodes/b/stderr.log",
            "nodes/b/resolved_params.json",
            "nodes/b/inputs/index.json",
            "nodes/b/outputs/index.json",
        ];
        for e in expected {
            assert!(final_path.join(e).exists(), "missing {}", e);
        }
    }

    #[test]
    fn failing_node_writes_failure() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].params =
            param_object(vec![("argv", Value::Array(vec![Value::from("false")]))]);
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        if let Err(err) = &result {
            panic!("{:?}", err);
        }
        let final_path = result.unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"failure\""));
    }

    #[test]
    fn jobs_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt1 = RuntimeConfig {
            jobs: 1,
            ..RuntimeConfig::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt1).unwrap();
        let opt2 = RuntimeConfig {
            jobs: 4,
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let snap1 = fs::read_to_string(run1.join("graph.snapshot.json")).unwrap();
        let snap2 = fs::read_to_string(run2.join("graph.snapshot.json")).unwrap();
        assert_eq!(snap1, snap2);
    }

    #[test]
    fn scheduler_order_stable() {
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };
        let order = graph.topo_order().unwrap();
        assert_eq!(order, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn selector_filters_inclusion_and_exclusion() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let graph = sample_graph();
        let include = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![Selector::Tag("etl".to_string())],
                exclude: vec![],
            },
            ..RuntimeConfig::default()
        };
        let run = runtime.run(&graph, dir.path(), include).unwrap();
        let trace_a = std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("\"status\": \"success\""));

        let dir = tempfile::tempdir().unwrap();
        let exclude = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![],
                exclude: vec![Selector::Tag("etl".to_string())],
            },
            ..RuntimeConfig::default()
        };
        let run = runtime.run(&graph, dir.path(), exclude).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("b").join("trace.json")).unwrap();
        let trace_a = std::fs::read_to_string(run.join("nodes").join("a").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("\"status\": \"success\""));
    }

    #[test]
    fn replay_run_outputs_are_deterministic() {
        let graph = sample_graph();
        let clock = Arc::new(clock::FixedClock::new(999));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);

        let run1 = tempfile::tempdir().unwrap();
        let path_1 = runtime
            .run(&graph, run1.path(), RuntimeConfig::default())
            .unwrap();
        let out1 = std::fs::read_to_string(path_1.join("outputs").join("index.json")).unwrap();

        let run2 = tempfile::tempdir().unwrap();
        let path_2 = runtime
            .run(&graph, run2.path(), RuntimeConfig::default())
            .unwrap();
        let out2 = std::fs::read_to_string(path_2.join("outputs").join("index.json")).unwrap();

        let log1 = std::fs::read_to_string(path_1.join("run.log.jsonl")).unwrap();
        let log2 = std::fs::read_to_string(path_2.join("run.log.jsonl")).unwrap();
        assert!(log1.contains("run_started"));
        assert_eq!(out1, out2);
    }

    #[test]
    fn run_timeout_skips_after_budget_is_reached() {
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "long_a".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![
                        (
                            "argv",
                            serde_json::json!(
                                ["/bin/sh", "-c", "sleep 0.05; echo done > ../outputs/out_a"]
                            ),
                        ),
                    ]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "long_b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![
                        (
                            "argv",
                            serde_json::json!(
                                ["/bin/sh", "-c", "echo skipped > ../outputs/out_b"]
                            ),
                        ),
                    ]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["timeout".to_string()],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let run = runtime
            .run(
                &graph,
                dir.path(),
                RuntimeConfig {
                    run_timeout_ms: Some(10),
                    jobs: 1,
                    ..RuntimeConfig::default()
                },
            )
            .unwrap();

        let trace_a = std::fs::read_to_string(run.join("nodes").join("long_a").join("trace.json")).unwrap();
        let trace_b = std::fs::read_to_string(run.join("nodes").join("long_b").join("trace.json")).unwrap();
        assert!(trace_a.contains("\"status\": \"success\""));
        assert!(trace_b.contains("\"status\": \"skipped\""));
        assert!(trace_b.contains("run_timeout"));
    }

    #[test]
    fn cache_corruption_forces_recompute() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        // corrupt cache by deleting an output file
        let entries: Vec<_> = fs::read_dir(cache_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let entry = entries[0].path();
        let index_path = entry.join("outputs").join("index.json");
        if let Ok(data) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<OutputsIndex>(&data) {
                if let Some(file) = index.files.first() {
                    let out_file = entry.join("outputs").join(&file.path);
                    if out_file.exists() {
                        fs::remove_file(out_file).unwrap();
                    }
                }
            }
        }

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();

        let trace_a = fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap();
        let trace_b = fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap();
        let has_corrupt =
            trace_a.contains("\"corrupt_detected\"") || trace_b.contains("\"corrupt_detected\"");
        assert!(has_corrupt);

        // ensure outputs still exist
        assert!(run1
            .join("nodes")
            .join("b")
            .join("outputs")
            .join("index.json")
            .exists());
    }

    #[test]
    fn remote_cache_hit() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::Read,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        let trace_b: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap(),
        )
        .unwrap();
        let src_a = trace_a
            .get("cache_proof")
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str());
        let src_b = trace_b
            .get("cache_proof")
            .and_then(|v| v.get("source"))
            .and_then(|v| v.as_str());
        assert!(src_a == Some("remote") || src_b == Some("remote"));
    }

    #[test]
    fn remote_cache_corruption_reexecutes() {
        let dir = tempfile::tempdir().unwrap();
        let local_cache = tempfile::tempdir().unwrap();
        let remote_cache = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();

        let opt = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(remote_cache.path().to_path_buf()),
            remote_cache_dir: None,
            ..RuntimeConfig::default()
        };
        let _ = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        let entries: Vec<_> = fs::read_dir(remote_cache.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let entry = entries[0].path();
        let index_path = entry.join("outputs").join("index.json");
        if let Ok(data) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<OutputsIndex>(&data) {
                if let Some(file) = index.files.first() {
                    let out_file = entry.join("outputs").join(&file.path);
                    if out_file.exists() {
                        fs::remove_file(out_file).unwrap();
                    }
                }
            }
        }

        let opt2 = RuntimeConfig {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(local_cache.path().to_path_buf()),
            remote_cache_dir: Some(remote_cache.path().to_path_buf()),
            ..RuntimeConfig::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();
        let trace_a: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("a").join("trace.json")).unwrap(),
        )
        .unwrap();
        let trace_b: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap(),
        )
        .unwrap();
        let bad_a = trace_a
            .get("cache_proof")
            .and_then(|v| v.get("corrupt_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let bad_b = trace_b
            .get("cache_proof")
            .and_then(|v| v.get("corrupt_detected"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        assert!(bad_a || bad_b);
    }

    #[test]
    fn fixed_clock_produces_stable_event_ts() {
        let dir = tempfile::tempdir().unwrap();
        let clock = Arc::new(FixedClock::new(123));
        let runtime = Runtime::with_io(Arc::new(StdFs), clock);
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeConfig::default())
            .unwrap();
        let log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        for line in log.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            assert_eq!(v.get("ts").and_then(|v| v.as_u64()), Some(123));
        }
    }

    #[test]
    fn downstream_reads_upstream_file() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from("hello"))]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("cat ../inputs/a/in > ../outputs/out_b"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "out_a".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        };
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let result = runtime.run(&graph, dir.path(), RuntimeConfig::default());
        if let Err(err) = &result {
            panic!("{:?}", err);
        }
        let final_path = result.unwrap();
        let out = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("out_b"),
        )
        .unwrap();
        assert!(out.contains("hello"));
        let inputs_index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("inputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(inputs_index.contains("a/in"));
    }

    #[test]
    fn file_wiring_only_materializes_bound_output() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![
                        FileOutput {
                            name: "a".to_string(),
                            path: "a.txt".to_string(),
                        },
                        FileOutput {
                            name: "b".to_string(),
                            path: "b.txt".to_string(),
                        },
                    ],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("echo a > ../outputs/a.txt; echo b > ../outputs/b.txt"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![(
                        "argv",
                        Value::Array(vec![
                            Value::from("/bin/sh"),
                            Value::from("-c"),
                            Value::from("if [ ! -f ../inputs/a/in ]; then exit 1; fi; if [ -e ../inputs/a/b ]; then exit 1; fi; cat ../inputs/a/in > ../outputs/out_b"),
                        ]),
                    )]),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "a".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        };
        let runtime = Runtime::new();
        let diags = graph.validate_with_warnings();
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{:?}",
            diags
        );
        graph.resolve_graph().unwrap();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("out_b"),
        )
        .unwrap();
        assert!(out.contains("a"));
        let inputs_index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("inputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(inputs_index.contains("a/in"));
        assert!(!inputs_index.contains("a/b"));
    }

    #[test]
    fn retry_succeeds_on_second_attempt() {
        let dir = tempfile::tempdir().unwrap();
        let mut graph = sample_graph();
        graph.nodes[1].retry = bijux_dag_core::RetryPolicy {
            max_attempts: 1,
            backoff_ms: 0,
        };
        graph.nodes[1].params = param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from(
                    "if [ ! -f marker ]; then touch marker; exit 1; fi; echo ok > ../outputs/out_b",
                ),
            ]),
        )]);
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"attempt\": 2"));
        assert!(trace.contains("\"status\": \"success\""));
    }

    #[test]
    fn cpu_budget_schedules_deterministically() {
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_a".to_string(),
                        path: "out_a".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(1))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_b".to_string(),
                        path: "out_b".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(2))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "c".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out_c".to_string(),
                        path: "out_c".to_string(),
                    }],
                    params: param_object(vec![("value", Value::from(3))]),
                    container: None,
                    timeout_ms: None,
                    resources: Some(bijux_dag_core::Resources { cpu: 2, mem_mb: 0 }),
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let opt = RuntimeConfig {
            jobs: 3,
            cpu_budget: Some(2),
            ..RuntimeConfig::default()
        };
        let final_path = runtime.run(&graph, dir.path(), opt).unwrap();
        let log = fs::read_to_string(final_path.join("run.log.jsonl")).unwrap();
        let mut scheduled = Vec::new();
        for line in log.lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            if v.get("event") == Some(&Value::String("node_scheduled".to_string())) {
                if let Some(id) = v.get("node_id").and_then(|v| v.as_str()) {
                    scheduled.push(id.to_string());
                }
            }
        }
        assert_eq!(scheduled, vec!["a", "b", "c"]);
    }

    #[test]
    fn container_node_writes_output() {
        if !docker_available() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "c1".to_string(),
                kind: NodeKind::Container,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out_c".to_string(),
                    path: "out_c".to_string(),
                }],
                params: ParamValue::default(),
                container: Some(ContainerSpec {
                    image: "alpine:3.19".to_string(),
                    argv: vec![
                        "sh".to_string(),
                        "-c".to_string(),
                        "echo hi > /bijux/node/outputs/out_c".to_string(),
                    ],
                    env_allowlist: vec![],
                    workdir: Some("/bijux/node/work".to_string()),
                    engine: "docker".to_string(),
                }),
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = final_path
            .join("nodes")
            .join("c1")
            .join("outputs")
            .join("out_c");
        assert!(out.exists());
    }

    #[test]
    fn shell_env_is_clean_except_allowlist() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("BIJUX_TEST_FOO", "allowed");
        std::env::set_var("BIJUX_TEST_BAR", "blocked");
        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "env".to_string(),
                kind: NodeKind::Shell,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("env"),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem, Effect::Env],
                env_allowlist: vec!["BIJUX_TEST_FOO".to_string()],
                group: None,
            }],
            edges: vec![],
        };
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let stdout =
            fs::read_to_string(final_path.join("nodes").join("env").join("stdout.log")).unwrap();
        assert!(stdout.contains("BIJUX_TEST_FOO=allowed"));
        assert!(!stdout.contains("BIJUX_TEST_BAR=blocked"));
    }

    #[test]
    fn external_adapter_executes() {
        let dir = tempfile::tempdir().unwrap();
        let adapter_dir = dir.path().join("adapters");
        fs::create_dir_all(&adapter_dir).unwrap();
        let adapter_path = adapter_dir.join("fake-adapter");
        fs::write(&adapter_path, include_str!("../tests/bin/fake_adapter.sh")).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&adapter_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&adapter_path, perms).unwrap();
        }
        std::env::set_var("BIJUX_DAG_ADAPTERS_DIR", &adapter_dir);

        let graph = Graph {
            spec: SPEC_VERSION.to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![Node {
                id: "n1".to_string(),
                kind: NodeKind::External("fake".to_string()),
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out".to_string(),
                    path: "out".to_string(),
                }],
                params: ParamValue::default(),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            }],
            edges: vec![],
        };

        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeConfig::default())
            .unwrap();
        let out = final_path
            .join("nodes")
            .join("n1")
            .join("outputs")
            .join("out");
        assert!(out.exists());
        std::env::remove_var("BIJUX_DAG_ADAPTERS_DIR");
    }
}
