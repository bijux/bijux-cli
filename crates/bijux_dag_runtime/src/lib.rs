mod adapter;

use adapter::{Adapter, AdapterId, EffectSet, NodeCtx};
use bijux_dag_artifacts::{
    now_unix_ms, write_outputs_index, AdapterInfo, ArtifactError, CacheProof, FailureInfo,
    Manifest, NodeCounts, NodeTrace, OutputSummary, OutputsIndex, Resources as TraceResources,
    RunDir,
};
use bijux_dag_core::{
    Effect, Graph, GraphError, Node, NodeKind, RetryPolicy, Severity, SPEC_VERSION,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("graph error: {0}")]
    Graph(#[from] GraphError),
    #[error("artifact error: {0}")]
    Artifact(#[from] ArtifactError),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
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

pub struct ExecutionContext {
    pub run_dir: Arc<RunDir>,
    pub graph_fingerprint: HashMap<String, String>,
    pub resolved_params: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub status: NodeStatus,
    pub stdout_path: String,
    pub stderr_path: String,
    pub outputs_dir: String,
    pub failure: Option<FailureInfo>,
    pub attempts: u32,
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

    fn required_effects(&self) -> EffectSet {
        EffectSet::default()
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let node_dir = exec.run_dir.node_dir(&node.id);
        let work_dir = exec.run_dir.node_work_dir(&node.id);
        fs::create_dir_all(exec.run_dir.node_outputs_dir(&node.id))?;
        fs::create_dir_all(&node_dir)?;
        fs::create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);
        let outputs_dir = exec.run_dir.node_outputs_dir(&node.id);

        let value = node.params.get("value").cloned().unwrap_or(Value::Null);
        fs::write(
            outputs_dir.join("value.json"),
            serde_json::to_vec_pretty(&value)?,
        )?;
        fs::write(&stdout_path, b"")?;
        fs::write(&stderr_path, b"")?;
        let fp = exec
            .graph_fingerprint
            .get(&node.id)
            .cloned()
            .unwrap_or_default();
        write_outputs_index(&outputs_dir, &node.id, &fp)?;

        Ok(NodeResult {
            status: NodeStatus::Success,
            stdout_path: stdout_path.display().to_string(),
            stderr_path: stderr_path.display().to_string(),
            outputs_dir: outputs_dir.display().to_string(),
            failure: None,
            attempts: 1,
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

    fn required_effects(&self) -> EffectSet {
        EffectSet {
            filesystem: true,
            env: false,
            network: false,
            clock: false,
        }
    }

    fn execute(&self, ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
        let node = ctx.node;
        let exec = ctx.exec;
        let argv = node
            .params
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
        fs::create_dir_all(&outputs_dir)?;
        fs::create_dir_all(&node_dir)?;
        fs::create_dir_all(&work_dir)?;
        let stdout_path = exec.run_dir.node_stdout_path(&node.id);
        let stderr_path = exec.run_dir.node_stderr_path(&node.id);

        let mut cmd = Command::new(&args[0]);
        cmd.args(&args[1..]);
        cmd.current_dir(&work_dir);
        cmd.env_clear();
        for key in &node.env_allowlist {
            if let Ok(val) = std::env::var(key) {
                cmd.env(key, val);
            }
        }

        let output = cmd.output()?;

        fs::write(&stdout_path, &output.stdout)?;
        fs::write(&stderr_path, &output.stderr)?;
        fs::write(outputs_dir.join("stdout.log"), &output.stdout)?;
        let fp = exec
            .graph_fingerprint
            .get(&node.id)
            .cloned()
            .unwrap_or_default();
        write_outputs_index(&outputs_dir, &node.id, &fp)?;

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

pub struct RuntimeOptions {
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub run_timeout_ms: Option<u64>,
    pub node_timeout_ms: Option<u64>,
    pub cache_mode: CacheMode,
    pub cache_dir: Option<PathBuf>,
    pub run_id: Option<String>,
    pub latest_symlink: Option<PathBuf>,
    pub policy: Policy,
    pub only_tag: Option<String>,
    pub skip_tag: Option<String>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            jobs: 1,
            cpu_budget: None,
            run_timeout_ms: None,
            node_timeout_ms: None,
            cache_mode: CacheMode::Off,
            cache_dir: None,
            run_id: None,
            latest_symlink: None,
            policy: Policy::default(),
            only_tag: None,
            skip_tag: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Policy {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
}

pub struct Runtime {
    adapters: HashMap<NodeKind, Arc<dyn Adapter>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut adapters: HashMap<NodeKind, Arc<dyn Adapter>> = HashMap::new();
        adapters.insert(NodeKind::Const, Arc::new(ConstAdapter));
        adapters.insert(NodeKind::Shell, Arc::new(ShellAdapter));
        Self { adapters }
    }

    pub fn run(
        &self,
        graph: &Graph,
        out_dir: impl AsRef<Path>,
        options: RuntimeOptions,
    ) -> Result<PathBuf, RuntimeError> {
        let diags = graph.validate_with_warnings();
        if diags.iter().any(|d| d.severity == Severity::Error) {
            return Err(GraphError::ValidationFailed.into());
        }

        let run_dir = if let Some(ref run_id) = options.run_id {
            RunDir::create_with_id(out_dir, run_id)?
        } else {
            RunDir::create(out_dir)?
        };
        let graph_fp = graph.graph_fingerprint()?;
        let graph_json = serde_json::json!({
            "graph": graph.canonicalize(),
            "graph_fingerprint": graph_fp,
        });
        run_dir.write_graph_snapshot(&serde_json::to_string_pretty(&graph_json)?)?;

        let run_id = options.run_id.clone().unwrap_or_else(|| {
            run_dir
                .final_path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        });

        let started_unix_ms = now_unix_ms();
        let effective_cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
        let mut manifest = Manifest {
            run_id,
            created_unix_ms: now_unix_ms(),
            started_unix_ms,
            finished_unix_ms: started_unix_ms,
            graph_snapshot: "graph.snapshot.json".to_string(),
            status: "success".to_string(),
            spec: SPEC_VERSION.to_string(),
            graph_fingerprint: graph_fp,
            tool_version: tool_version(),
            jobs: options.jobs.max(1),
            adapters: registered_adapters(),
            outputs: Vec::new(),
            node_counts: NodeCounts {
                success: 0,
                failed: 0,
                skipped: 0,
                cached: 0,
            },
            policy: bijux_dag_artifacts::PolicyInfo {
                deny_network: options.policy.deny_network,
                deny_env: options.policy.deny_env,
                deny_clock: options.policy.deny_clock,
            },
            cache_mode: cache_mode_string(&options.cache_mode),
            cache_dir: effective_cache_dir
                .as_ref()
                .map(|p| p.display().to_string()),
            run_timeout_ms: options.run_timeout_ms,
        };
        run_dir.write_manifest(&manifest)?;

        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_flag = cancel.clone();
        let _ = ctrlc::set_handler(move || {
            cancel_flag.store(true, Ordering::SeqCst);
        });

        let mut run_log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(run_dir.run_log_path())?;
        append_event(
            &mut run_log,
            serde_json::json!({
                "event": "run_started",
                "ts": started_unix_ms,
            }),
        )?;

        let mut node_fps = HashMap::new();
        for node in &graph.nodes {
            node_fps.insert(node.id.clone(), graph.node_fingerprint(node)?);
        }
        let resolved_params = graph.resolve_params()?;
        let resolved_params: HashMap<String, Value> = resolved_params.into_iter().collect();
        let ctx = ExecutionContext {
            run_dir: Arc::new(run_dir.clone()),
            graph_fingerprint: node_fps,
            resolved_params,
        };
        let start = Instant::now();
        let mut status_map: HashMap<String, NodeStatus> = HashMap::new();
        let mut cache_proofs: HashMap<String, CacheProof> = HashMap::new();
        let dep_map = build_dep_map(graph);
        let (mut indegree, adj) = build_graph_index(graph);
        let mut ready: BTreeSet<String> = indegree
            .iter()
            .filter_map(|(id, &deg)| if deg == 0 { Some(id.clone()) } else { None })
            .collect();

        let cpu_budget = options.cpu_budget.unwrap_or(options.jobs.max(1) as u32);
        while !ready.is_empty() {
            let mut batch: Vec<String> = Vec::new();
            let mut used_cpu: u32 = 0;
            let mut to_remove: Vec<String> = Vec::new();
            for id in ready.iter() {
                if batch.len() >= options.jobs.max(1) {
                    break;
                }
                let cpu = node_cpu(graph, id);
                if used_cpu + cpu > cpu_budget {
                    continue;
                }
                used_cpu += cpu;
                batch.push(id.clone());
                to_remove.push(id.clone());
            }
            if batch.is_empty() {
                if let Some(id) = ready.iter().next().cloned() {
                    batch.push(id.clone());
                    to_remove.push(id);
                }
            }
            for id in to_remove {
                ready.remove(&id);
            }

            let mut handles = Vec::new();
            let mut skipped: Vec<(String, String)> = Vec::new();
            let mut cached: Vec<(String, Node, CacheProof)> = Vec::new();
            let mut to_start: Vec<(String, Node)> = Vec::new();

            for node_id in &batch {
                if let Some(reason) = filter_reason(graph, node_id, &options) {
                    skipped.push((node_id.clone(), reason));
                    continue;
                }
                if cancel.load(Ordering::SeqCst) {
                    skipped.push((node_id.clone(), "cancelled".to_string()));
                    continue;
                }
                if let Some(limit) = options.run_timeout_ms {
                    if start.elapsed() > Duration::from_millis(limit) {
                        skipped.push((node_id.clone(), "run_timeout".to_string()));
                        continue;
                    }
                }

                if let Some(deps) = dep_map.get(node_id) {
                    if deps.iter().any(|d| {
                        matches!(
                            status_map.get(d),
                            Some(NodeStatus::Failed) | Some(NodeStatus::Skipped)
                        )
                    }) {
                        skipped.push((node_id.clone(), "upstream_failed".to_string()));
                        continue;
                    }
                }

                let mut node = graph
                    .nodes
                    .iter()
                    .find(|n| n.id == *node_id)
                    .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?
                    .clone();
                if let Some(r) = ctx.resolved_params.get(&node.id) {
                    node.params = r.clone();
                }

                if node.kind == NodeKind::Shell {
                    if !node.effects.contains(&Effect::Filesystem) {
                        return Err(RuntimeError::Executor(
                            "shell node missing filesystem effect".to_string(),
                        ));
                    }
                    if options.policy.deny_network && node.effects.contains(&Effect::Network) {
                        return Err(RuntimeError::Executor(
                            "network effect denied by policy".to_string(),
                        ));
                    }
                    if options.policy.deny_env && node.effects.contains(&Effect::Env) {
                        return Err(RuntimeError::Executor(
                            "env effect denied by policy".to_string(),
                        ));
                    }
                    if options.policy.deny_clock && node.effects.contains(&Effect::Clock) {
                        return Err(RuntimeError::Executor(
                            "clock effect denied by policy".to_string(),
                        ));
                    }
                }
                let adapter = self
                    .adapters
                    .get(&node.kind)
                    .ok_or_else(|| RuntimeError::Executor("missing adapter".to_string()))?;
                let required = adapter.required_effects();
                let declared = EffectSet::from_effects(&node.effects);
                if required.filesystem && !declared.filesystem
                    || required.env && !declared.env
                    || required.network && !declared.network
                    || required.clock && !declared.clock
                {
                    return Err(RuntimeError::Executor(
                        "missing required effects".to_string(),
                    ));
                }

                let cache_read = try_cache_read(&options, &node, &ctx, graph)?;
                if let Some(proof) = cache_read.proof.clone() {
                    if !cache_read.hit {
                        cache_proofs.insert(node_id.clone(), proof);
                    }
                }
                if cache_read.hit {
                    cached.push((node_id.clone(), node, cache_read.proof.unwrap()));
                    continue;
                }

                to_start.push((node_id.clone(), node));
            }

            skipped.sort_by(|a, b| a.0.cmp(&b.0));
            for (node_id, reason) in &skipped {
                status_map.insert(node_id.clone(), NodeStatus::Skipped);
                let node_kind = graph
                    .nodes
                    .iter()
                    .find(|n| n.id == *node_id)
                    .map(|n| n.kind.clone())
                    .unwrap_or(NodeKind::Const);
                let (aid, aver) = adapter_meta_for_kind(&node_kind);
                let started = now_unix_ms();
                write_trace(
                    &ctx,
                    graph,
                    node_id,
                    NodeStatus::Skipped,
                    None,
                    started,
                    started,
                    1,
                    None,
                    &aid,
                    &aver,
                )?;
                let _reason = reason;
            }

            let mut started_ids: Vec<String> = Vec::new();
            for (node_id, _) in &to_start {
                started_ids.push(node_id.clone());
            }
            for (node_id, _, _) in &cached {
                started_ids.push(node_id.clone());
            }
            started_ids.sort();
            for node_id in &started_ids {
                append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "node_started",
                        "ts": now_unix_ms(),
                        "node_id": node_id,
                    }),
                )?;
            }

            for (node_id, node, cache_proof) in &cached {
                status_map.insert(node_id.clone(), NodeStatus::Cached);
                let (aid, aver) = adapter_meta_for_kind(&node.kind);
                let started = now_unix_ms();
                write_trace(
                    &ctx,
                    graph,
                    node_id,
                    NodeStatus::Cached,
                    None,
                    started,
                    started,
                    1,
                    Some(cache_proof.clone()),
                    &aid,
                    &aver,
                )?;
                append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "node_finished",
                        "ts": now_unix_ms(),
                        "node_id": node_id,
                        "status": "cached",
                    }),
                )?;
                try_cache_write(&options, node, &ctx, graph)?;
            }

            for (node_id, node) in &to_start {
                let adapter = self
                    .adapters
                    .get(&node.kind)
                    .ok_or_else(|| RuntimeError::Executor("missing adapter".to_string()))?;
                let adapter = Arc::clone(adapter);
                let ctx_clone = ExecutionContext {
                    run_dir: Arc::clone(&ctx.run_dir),
                    graph_fingerprint: ctx.graph_fingerprint.clone(),
                    resolved_params: ctx.resolved_params.clone(),
                };
                let node_id_clone = node_id.clone();
                let node_for_thread = node.clone();
                let retry = node.retry.clone();
                handles.push((
                    node_id_clone,
                    node.clone(),
                    std::thread::spawn(move || {
                        let started = now_unix_ms();
                        let result = execute_with_retries(
                            adapter.as_ref(),
                            &node_for_thread,
                            &ctx_clone,
                            &retry,
                        );
                        let finished = now_unix_ms();
                        (started, finished, result)
                    }),
                ));
            }

            type ResultItem = (String, Node, u128, u128, Result<NodeResult, RuntimeError>);
            let mut results: Vec<ResultItem> = Vec::new();
            for (node_id, node, handle) in handles {
                let res = handle.join().unwrap_or_else(|_| {
                    (
                        now_unix_ms(),
                        now_unix_ms(),
                        Err(RuntimeError::Executor("thread panicked".to_string())),
                    )
                });
                results.push((node_id, node, res.0, res.1, res.2));
            }
            results.sort_by(|a, b| a.0.cmp(&b.0));
            for (node_id, node, started, finished, res) in results {
                match res {
                    Ok(result) => {
                        let (aid, aver) = adapter_meta_for_kind(&node.kind);
                        let trace_failure = result.failure.clone();
                        let cache_proof = cache_proofs.get(&node_id).cloned();
                        write_trace(
                            &ctx,
                            graph,
                            &node_id,
                            result.status.clone(),
                            trace_failure,
                            started,
                            finished,
                            result.attempts,
                            cache_proof,
                            &aid,
                            &aver,
                        )?;
                        append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_finished",
                                "ts": now_unix_ms(),
                                "node_id": node_id,
                                "status": status_string(&result.status),
                            }),
                        )?;
                        if result.status == NodeStatus::Failed {
                            status_map.insert(node_id.clone(), NodeStatus::Failed);
                        } else {
                            status_map.insert(node_id.clone(), result.status.clone());
                            try_cache_write(&options, &node, &ctx, graph)?;
                        }
                    }
                    Err(err) => {
                        let (aid, aver) = adapter_meta_for_kind(&node.kind);
                        status_map.insert(node_id.clone(), NodeStatus::Failed);
                        let cache_proof = cache_proofs.get(&node_id).cloned();
                        write_trace(
                            &ctx,
                            graph,
                            &node_id,
                            NodeStatus::Failed,
                            Some(FailureInfo {
                                kind: "Internal".to_string(),
                                code: "INTERNAL".to_string(),
                                message: err.to_string(),
                                details: None,
                            }),
                            started,
                            finished,
                            1,
                            cache_proof,
                            &aid,
                            &aver,
                        )?;
                        append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_finished",
                                "ts": now_unix_ms(),
                                "node_id": node_id,
                                "status": "failed",
                            }),
                        )?;
                    }
                }
            }

            for node_id in batch {
                if let Some(neighbors) = adj.get(&node_id) {
                    for n in neighbors {
                        if let Some(d) = indegree.get_mut(n) {
                            *d -= 1;
                            if *d == 0 {
                                ready.insert(n.clone());
                            }
                        }
                    }
                }
            }
        }

        if cancel.load(Ordering::SeqCst) {
            for node in &graph.nodes {
                if !status_map.contains_key(&node.id) {
                    status_map.insert(node.id.clone(), NodeStatus::Skipped);
                    let (aid, aver) = adapter_meta_for_kind(&node.kind);
                    let started = now_unix_ms();
                    write_trace(
                        &ctx,
                        graph,
                        &node.id,
                        NodeStatus::Skipped,
                        None,
                        started,
                        started,
                        1,
                        None,
                        &aid,
                        &aver,
                    )?;
                }
            }
        }

        let finished_unix_ms = now_unix_ms();
        if cancel.load(Ordering::SeqCst) {
            manifest.status = "cancelled".to_string();
        } else if status_map.values().any(|s| *s == NodeStatus::Failed) {
            manifest.status = "failed".to_string();
        }
        manifest.finished_unix_ms = finished_unix_ms;
        manifest.node_counts = count_nodes(&status_map);
        manifest.outputs = collect_outputs_summary(&ctx.run_dir)?;
        run_dir.write_manifest(&manifest)?;
        append_event(
            &mut run_log,
            serde_json::json!({
                "event": "run_finished",
                "ts": finished_unix_ms,
                "status": manifest.status,
            }),
        )?;

        let final_path = run_dir.finalize()?;
        if let Some(latest) = options.latest_symlink {
            let _ = fs::remove_file(&latest);
            let _ = std::os::unix::fs::symlink(&final_path, &latest);
        }
        Ok(final_path)
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_arguments)]
fn write_trace(
    ctx: &ExecutionContext,
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
) -> Result<(), RuntimeError> {
    let node = graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?;
    let node_dir = ctx.run_dir.node_dir(node_id);
    fs::create_dir_all(&node_dir)?;
    let trace = NodeTrace {
        node_id: node_id.to_string(),
        status: status_string(&status),
        started_unix_ms,
        finished_unix_ms,
        attempt,
        fingerprint: graph.node_fingerprint(node)?,
        adapter_id: adapter_id.to_string(),
        adapter_version: adapter_version.to_string(),
        resources: node.resources.as_ref().map(|r| TraceResources {
            cpu: r.cpu,
            mem_mb: r.mem_mb,
        }),
        resolved_params: ctx.resolved_params.get(node_id).cloned(),
        cache_proof,
        failure,
    };
    let data = serde_json::to_vec_pretty(&trace)?;
    fs::write(ctx.run_dir.node_trace_path(node_id), data)?;
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

#[allow(dead_code)]
fn node_timeout_ms(node: &Node, default_ms: Option<u64>) -> Option<Duration> {
    let param_timeout = node.params.get("timeout_ms").and_then(|v| v.as_u64());
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

fn filter_reason(graph: &Graph, node_id: &str, options: &RuntimeOptions) -> Option<String> {
    let node = graph.nodes.iter().find(|n| n.id == node_id)?;
    if let Some(ref only) = options.only_tag {
        if !node.tags.iter().any(|t| t == only) {
            return Some("filtered".to_string());
        }
    }
    if let Some(ref skip) = options.skip_tag {
        if node.tags.iter().any(|t| t == skip) {
            return Some("filtered".to_string());
        }
    }
    None
}

fn execute_with_retries(
    adapter: &dyn Adapter,
    node: &Node,
    ctx: &ExecutionContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let node_ctx = NodeCtx { node, exec: ctx };
        let mut result = adapter.execute(&node_ctx)?;
        result.attempts = attempt;
        if result.status != NodeStatus::Failed {
            return Ok(result);
        }
        if attempt > retry.max_attempts {
            return Ok(result);
        }
        if retry.backoff_ms > 0 {
            std::thread::sleep(Duration::from_millis(retry.backoff_ms));
        }
    }
}

fn append_event(file: &mut fs::File, value: serde_json::Value) -> Result<(), RuntimeError> {
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
    let adapters: Vec<Arc<dyn Adapter>> = vec![Arc::new(ConstAdapter), Arc::new(ShellAdapter)];
    let mut list = Vec::new();
    for a in adapters {
        let id = a.id();
        let req = a.required_effects();
        let mut effects = Vec::new();
        if req.filesystem {
            effects.push("filesystem".to_string());
        }
        if req.env {
            effects.push("env".to_string());
        }
        if req.network {
            effects.push("network".to_string());
        }
        if req.clock {
            effects.push("clock".to_string());
        }
        list.push(AdapterInfo {
            adapter_id: id.id,
            adapter_version: id.version,
            effects,
        });
    }
    list.sort_by(|a, b| a.adapter_id.cmp(&b.adapter_id));
    list
}

fn adapter_meta_for_kind(kind: &NodeKind) -> (String, String) {
    match kind {
        NodeKind::Const => ("const".to_string(), "0.1".to_string()),
        NodeKind::Shell => ("shell".to_string(), "0.1".to_string()),
    }
}

fn build_dep_map(graph: &Graph) -> HashMap<String, BTreeSet<String>> {
    let mut map: HashMap<String, BTreeSet<String>> = HashMap::new();
    for edge in &graph.edges {
        map.entry(edge.to.node_id.clone())
            .or_default()
            .insert(edge.from.node_id.clone());
    }
    map
}

fn build_graph_index(graph: &Graph) -> (HashMap<String, usize>, HashMap<String, Vec<String>>) {
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for node in &graph.nodes {
        indegree.insert(node.id.clone(), 0);
        adj.insert(node.id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        let from = edge.from.node_id.clone();
        let to = edge.to.node_id.clone();
        if let Some(v) = adj.get_mut(&from) {
            v.push(to.clone());
        }
        if let Some(d) = indegree.get_mut(&to) {
            *d += 1;
        }
    }
    (indegree, adj)
}

fn cache_dir_from_env() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

fn try_cache_read(
    options: &RuntimeOptions,
    node: &Node,
    ctx: &ExecutionContext,
    graph: &Graph,
) -> Result<CacheRead, RuntimeError> {
    if options.cache_mode == CacheMode::Off {
        return Ok(CacheRead {
            hit: false,
            proof: None,
        });
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let cache_dir = match cache_dir {
        Some(d) => d,
        None => {
            return Ok(CacheRead {
                hit: false,
                proof: None,
            })
        }
    };
    if options.cache_mode == CacheMode::Read || options.cache_mode == CacheMode::ReadWrite {
        let key = graph.node_fingerprint(node)?;
        let entry = cache_dir.join(&key);
        if entry.exists() {
            let (aid, aver) = adapter_meta_for_kind(&node.kind);
            if !verify_cache_entry(&entry, &key, &aid, &aver)? {
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
            let node_dir = ctx.run_dir.node_dir(&node.id);
            fs::create_dir_all(&node_dir)?;
            copy_dir_all(
                entry.join("outputs"),
                ctx.run_dir.node_outputs_dir(&node.id),
            )?;
            copy_dir_all(entry.join("logs"), node_dir.clone())?;
            return Ok(CacheRead {
                hit: true,
                proof: Some(CacheProof {
                    hit: true,
                    key,
                    source: "local".to_string(),
                    verified: true,
                    reason: "hit".to_string(),
                    corrupt_detected: false,
                }),
            });
        }
    }
    Ok(CacheRead {
        hit: false,
        proof: None,
    })
}

fn try_cache_write(
    options: &RuntimeOptions,
    node: &Node,
    ctx: &ExecutionContext,
    graph: &Graph,
) -> Result<(), RuntimeError> {
    if options.cache_mode != CacheMode::ReadWrite {
        return Ok(());
    }
    let cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let cache_dir = match cache_dir {
        Some(d) => d,
        None => return Ok(()),
    };
    let key = graph.node_fingerprint(node)?;
    let entry = cache_dir.join(&key);
    fs::create_dir_all(entry.join("outputs"))?;
    fs::create_dir_all(entry.join("logs"))?;
    let meta = serde_json::json!({
        "node_id": node.id,
        "node_fingerprint": key,
        "adapter_id": adapter_meta_for_kind(&node.kind).0,
        "adapter_version": adapter_meta_for_kind(&node.kind).1,
    });
    fs::write(entry.join("meta.json"), serde_json::to_vec_pretty(&meta)?)?;
    copy_dir_all(
        ctx.run_dir.node_outputs_dir(&node.id),
        entry.join("outputs"),
    )?;
    let node_dir = ctx.run_dir.node_dir(&node.id);
    let _ = fs::copy(
        node_dir.join("stdout.log"),
        entry.join("logs").join("stdout.log"),
    );
    let _ = fs::copy(
        node_dir.join("stderr.log"),
        entry.join("logs").join("stderr.log"),
    );
    let _ = fs::copy(
        node_dir.join("trace.json"),
        entry.join("logs").join("trace.json"),
    );
    Ok(())
}

fn verify_cache_entry(
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
) -> Result<bool, RuntimeError> {
    let index_path = entry.join("outputs").join("index.json");
    if !index_path.exists() {
        return Ok(false);
    }
    let meta_path = entry.join("meta.json");
    if !meta_path.exists() {
        return Ok(false);
    }
    let meta: serde_json::Value = serde_json::from_str(&fs::read_to_string(meta_path)?)?;
    if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(expected_key) {
        return Ok(false);
    }
    if meta.get("adapter_id").and_then(|v| v.as_str()) != Some(adapter_id) {
        return Ok(false);
    }
    if meta.get("adapter_version").and_then(|v| v.as_str()) != Some(adapter_version) {
        return Ok(false);
    }
    let data = fs::read_to_string(index_path)?;
    let index: OutputsIndex = serde_json::from_str(&data)?;
    for file in index.files {
        let path = entry.join("outputs").join(&file.path);
        if !path.exists() {
            return Ok(false);
        }
        let bytes = fs::read(path)?;
        let sha = sha256_bytes(&bytes);
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

fn collect_outputs_summary(run_dir: &RunDir) -> Result<Vec<OutputSummary>, RuntimeError> {
    let mut out = Vec::new();
    let nodes_dir = run_dir.staging_path().join("nodes");
    if nodes_dir.exists() {
        for entry in fs::read_dir(nodes_dir)? {
            let entry = entry?;
            let index_path = entry.path().join("outputs").join("index.json");
            if index_path.exists() {
                let data = fs::read_to_string(index_path)?;
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

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dag_core::{Edge, Effect, PortRef};

    fn sample_graph() -> Graph {
        Graph {
            spec: SPEC_VERSION.to_string(),
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec!["out".to_string()],
                    params: serde_json::json!({"value": 1}),
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec!["out_b".to_string()],
                    params: serde_json::json!({"argv": ["echo", "ok"]}),
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![Effect::Filesystem],
                    env_allowlist: vec![],
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "out".to_string(),
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
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeOptions::default())
            .unwrap();
        assert!(final_path.join("manifest.json").exists());
        assert!(final_path.join("graph.snapshot.json").exists());
        assert!(final_path
            .join("nodes")
            .join("a")
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
            .run(&sample_graph(), dir.path(), RuntimeOptions::default())
            .unwrap();
        let index = fs::read_to_string(
            final_path
                .join("nodes")
                .join("b")
                .join("outputs")
                .join("index.json"),
        )
        .unwrap();
        assert!(index.contains("stdout.log"));
    }

    #[test]
    fn artifact_tree_contains_expected_entries() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&sample_graph(), dir.path(), RuntimeOptions::default())
            .unwrap();
        let expected = vec![
            "manifest.json",
            "graph.snapshot.json",
            "run.log.jsonl",
            "nodes/a/trace.json",
            "nodes/a/stdout.log",
            "nodes/a/stderr.log",
            "nodes/a/outputs/index.json",
            "nodes/a/outputs/value.json",
            "nodes/b/trace.json",
            "nodes/b/stdout.log",
            "nodes/b/stderr.log",
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
        graph.nodes[1].params = serde_json::json!({"argv": ["false"]});
        let runtime = Runtime::new();
        let final_path = runtime
            .run(&graph, dir.path(), RuntimeOptions::default())
            .unwrap();
        let trace =
            fs::read_to_string(final_path.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"failure\""));
    }

    #[test]
    fn jobs_consistent() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt1 = RuntimeOptions {
            jobs: 1,
            ..RuntimeOptions::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt1).unwrap();
        let opt2 = RuntimeOptions {
            jobs: 4,
            ..RuntimeOptions::default()
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
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec!["out".to_string()],
                    params: serde_json::json!({"value": 1}),
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                },
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec!["out".to_string()],
                    params: serde_json::json!({"value": 2}),
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: bijux_dag_core::RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                },
            ],
            edges: vec![],
        };
        let order = graph.topo_order().unwrap();
        assert_eq!(order, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn cache_corruption_forces_recompute() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = tempfile::tempdir().unwrap();
        let runtime = Runtime::new();
        let opt = RuntimeOptions {
            cache_mode: CacheMode::ReadWrite,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            ..RuntimeOptions::default()
        };
        let run1 = runtime.run(&sample_graph(), dir.path(), opt).unwrap();

        // corrupt cache by deleting an output file
        let entries: Vec<_> = fs::read_dir(cache_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        let entry = entries[0].path();
        let out_file = entry.join("outputs").join("stdout.log");
        if out_file.exists() {
            fs::remove_file(out_file).unwrap();
        }

        let opt2 = RuntimeOptions {
            cache_mode: CacheMode::Read,
            cache_dir: Some(cache_dir.path().to_path_buf()),
            ..RuntimeOptions::default()
        };
        let run2 = runtime.run(&sample_graph(), dir.path(), opt2).unwrap();

        let trace = fs::read_to_string(run2.join("nodes").join("b").join("trace.json")).unwrap();
        assert!(trace.contains("\"cache_proof\""));
        assert!(trace.contains("\"corrupt_detected\""));

        // ensure outputs still exist
        assert!(run1
            .join("nodes")
            .join("b")
            .join("outputs")
            .join("index.json")
            .exists());
    }
}
