use crate::{
    build_run_outputs_index, cache_dir_from_env, cache_mode_string,
    category_from_runtime_event_name, collect_outputs_summary, current_process_memory_bytes,
    node_fingerprint_from_ctx, node_fingerprint_with_inputs, registered_adapters, sacred_execution,
    set_node_fingerprint, summarize_failure_root_causes, write_timeline_export, CacheProof,
    EffectSet, EventRecord, ExecutionCheckpoint, InMemoryMetricsRegistry, MetricsRegistry,
    NodeMetrics, NodeResult, NodeStatus, ReplayNodeAction, RunAttempt, RunContext, RunId,
    RunSnapshot, Runtime, RuntimeConfig, RuntimeError, SchedulerEventHook, TimelineEntry,
    TimelineExport,
};
#[path = "engine_dispatch.rs"]
mod engine_dispatch;
#[path = "engine_finalize.rs"]
mod engine_finalize;
#[path = "engine_metrics.rs"]
mod engine_metrics;
#[path = "engine_observe.rs"]
mod engine_observe;
#[path = "engine_record.rs"]
mod engine_record;
use bijux_dag_artifacts::{
    write_provenance, write_run_outputs_index, FailureInfo, Manifest, NodeCounts, Provenance,
    ReplayProvenance, RunDir, RunMetadata,
};
use bijux_dag_core::{Effect, Graph, Node, NodeKind, SPEC_VERSION};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::{Duration, Instant};

pub fn execute(
    runtime: &Runtime,
    graph: &Graph,
    plan: crate::ExecutionPlan,
    out_dir: impl AsRef<Path>,
    options: RuntimeConfig,
) -> Result<PathBuf, RuntimeError> {
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
        let dir_name = run_dir
            .final_path()
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        dir_name.strip_prefix("run-").unwrap_or(dir_name.as_str()).to_string()
    });

    let started_unix_ms = runtime.clock.now_unix_ms();
    let effective_cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let mut manifest = Manifest {
        manifest_version: "run-manifest/v0.1".to_string(),
        run_id,
        created_unix_ms: runtime.clock.now_unix_ms(),
        started_unix_ms,
        finished_unix_ms: started_unix_ms,
        graph_snapshot: "graph.snapshot.json".to_string(),
        status: "success".to_string(),
        spec: SPEC_VERSION.to_string(),
        graph_fingerprint: graph_fp,
        planner_fingerprint: Some(plan.planner_fingerprint.clone()),
        execution_fingerprint: Some(plan.execution_fingerprint.clone()),
        evidence_fingerprint: Some(plan.evidence_fingerprint.clone()),
        tool_version: crate::tool_version(),
        jobs: options.jobs.max(1),
        adapters: registered_adapters(),
        outputs: Vec::new(),
        node_counts: NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0 },
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: options.policy.deny_network,
            deny_env: options.policy.deny_env,
            deny_clock: options.policy.deny_clock,
            clean_env: options.policy.clean_env,
        },
        cache_mode: cache_mode_string(&options.cache_mode),
        cache_dir: effective_cache_dir.as_ref().map(|p| p.display().to_string()),
        run_timeout_ms: options.run_timeout_ms,
        run_metadata: None,
        run_summary: None,
    };
    manifest.run_metadata = Some(RunMetadata {
        submission_source: options.submission_source.clone(),
        trigger_source: options.trigger_source.clone(),
        operator: options.operator.clone(),
        labels: options.labels.clone(),
        parent_run_id: options.parent_run_id.clone(),
        source_run_id: options.parent_run_id.clone(),
    });
    run_dir.write_manifest(&manifest)?;

    let registered = registered_adapters();
    let prov = Provenance {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rustc: crate::rustc_version(),
        tool_version: crate::tool_version(),
        graph_fingerprint: Some(manifest.graph_fingerprint.clone()),
        planner_fingerprint: manifest.planner_fingerprint.clone(),
        execution_fingerprint: manifest.execution_fingerprint.clone(),
        evidence_fingerprint: manifest.evidence_fingerprint.clone(),
        runtime_fingerprint: Some(crate::runtime_fingerprint(&registered)),
        policy_fingerprint: Some(crate::policy_fingerprint(&options.policy)),
        adapters: registered,
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: options.policy.deny_network,
            deny_env: options.policy.deny_env,
            deny_clock: options.policy.deny_clock,
            clean_env: options.policy.clean_env,
        },
        time_source: "system_clock".to_string(),
    };
    write_provenance(run_dir.provenance_path(), &prov)?;

    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_flag = cancel.clone();
    let _ = ctrlc::set_handler(move || {
        cancel_flag.store(true, Ordering::SeqCst);
    });

    let run_dir_arc = Arc::new(run_dir.clone());
    let store = crate::store::ArtifactStore::new(Arc::clone(&run_dir_arc), Arc::clone(&runtime.fs));
    let mut run_log = store.open_run_log()?;
    let mut run_log_index: Vec<serde_json::Value> = Vec::new();
    let mut run_audit_events: Vec<serde_json::Value> = Vec::new();
    let mut failure_propagation_records: Vec<serde_json::Value> = Vec::new();
    let mut node_metric_rows: Vec<NodeMetrics> = Vec::new();
    let mut metrics_registry = InMemoryMetricsRegistry::default();
    engine_record::append_indexed_event(
        &mut run_log,
        &mut run_log_index,
        serde_json::json!({
            "event": "run_started",
            "ts": started_unix_ms,
        }),
    )?;
    run_audit_events.push(serde_json::json!({
        "action": "start",
        "ts": started_unix_ms,
        "run_id": manifest.run_id.clone(),
    }));

    let resolved = graph.resolve_graph()?;
    let mut base_fps = HashMap::new();
    for node in &graph.nodes {
        let params = resolved.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        base_fps.insert(node.id.clone(), graph.node_fingerprint_with_params(node, &params)?);
    }
    let resolved_params: HashMap<String, Value> = resolved.resolved_params.into_iter().collect();
    let graph_fingerprint = Arc::new(Mutex::new(base_fps.clone()));
    let ctx = RunContext {
        run_dir: Arc::clone(&run_dir_arc),
        graph_fingerprint: Arc::clone(&graph_fingerprint),
        resolved_params,
        fs: Arc::clone(&runtime.fs),
        clock: Arc::clone(&runtime.clock),
        store,
        policy: options.policy.clone(),
    };
    let selected_nodes = options
        .selectors
        .include
        .iter()
        .map(|selector| match selector {
            crate::Selector::IdPrefix(v) => format!("id_prefix:{v}"),
            crate::Selector::Tag(v) => format!("tag:{v}"),
            crate::Selector::Kind(v) => format!("kind:{v}"),
        })
        .collect();
    let run_snapshot = RunSnapshot {
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        graph_snapshot_path: "graph.snapshot.json".to_string(),
        planner_config: "default".to_string(),
        scheduler_config: "local".to_string(),
        policy_config: "runtime-policy-v0.1".to_string(),
        provenance: "provenance.json".to_string(),
        submission_source: options.submission_source.clone(),
        trigger_source: options.trigger_source.clone(),
        operator: options.operator.clone(),
        labels: options.labels.clone(),
        parent_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        selected_nodes,
        dependency_closure_enabled: options.partial_rerun_dependency_closure,
        replay_source_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
    };
    let run_snapshot_path = ctx.run_dir.staging_path().join("run.snapshot.json");
    let _ = ctx.fs.write(&run_snapshot_path, &serde_json::to_vec_pretty(&run_snapshot)?);
    let run_attempt = RunAttempt {
        attempt_index: 1,
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        parent_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        reason: if options.parent_run_id.is_some() {
            "replay_or_retry".to_string()
        } else {
            "initial_submission".to_string()
        },
    };
    let run_attempts_path = ctx.run_dir.staging_path().join("run.attempts.json");
    let _ = ctx.fs.write(&run_attempts_path, &serde_json::to_vec_pretty(&vec![run_attempt])?);
    let start = Instant::now();
    let mut status_map: HashMap<String, NodeStatus> = HashMap::new();
    let mut cache_proofs: HashMap<String, CacheProof> = HashMap::new();
    let dep_map = plan.dep_map.clone();
    let mut dependency_counter = sacred_execution::resolve_dependencies(&plan);
    let mut ready_queue = sacred_execution::ready_queue_from_dependencies(&dependency_counter);
    let mut scheduler = crate::build_scheduler(&options.scheduler_policy);
    let scheduler_hook = crate::NoopSchedulerEventHook;
    let mut loop_index: u64 = 0;
    engine_record::append_indexed_event(
        &mut run_log,
        &mut run_log_index,
        serde_json::json!({
            "event": "plan_built",
            "ts": ctx.clock.now_unix_ms(),
            "nodes": graph.nodes.len(),
        }),
    )?;
    while !ready_queue.is_empty() {
        loop_index = loop_index.saturating_add(1);
        let ready_vec: Vec<String> = ready_queue.snapshot_sorted();
        for node_id in &ready_vec {
            scheduler_hook.on_node_eligible(node_id);
            let events = engine_observe::node_eligible_events(
                std::slice::from_ref(node_id),
                ctx.clock.now_unix_ms(),
            );
            for event in events {
                crate::append_event(&mut run_log, event)?;
            }
        }
        let decision = engine_dispatch::next_scheduler_decision(
            scheduler.as_mut(),
            graph,
            &mut ready_queue,
            &options,
            start,
            cancel.load(Ordering::SeqCst),
        );
        if decision.cancelled {
            break;
        }
        if decision.timed_out {
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "run_timeout",
                    "ts": ctx.clock.now_unix_ms(),
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "run_timeout",
                "ts": ctx.clock.now_unix_ms(),
            }));
            break;
        }
        let batch = decision.batch;
        let mut blocked_by_budget = decision.blocked_by_budget;
        blocked_by_budget.sort();
        for node_id in &blocked_by_budget {
            scheduler_hook.on_node_blocked_by_budget(node_id);
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "blocked_by_budget",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }),
            )?;
        }
        let forced_batch = batch.len() == 1;

        let mut handles = Vec::new();
        let mut skipped: Vec<(String, String)> = Vec::new();
        let mut cached: Vec<(String, Node, CacheProof)> = Vec::new();
        let mut to_start: Vec<(String, Node, Value)> = Vec::new();
        let mut preflight_failures: Vec<(String, Node, FailureInfo, String)> = Vec::new();

        for node_id in &batch {
            if let Some(reason) = plan.filter_reasons.get(node_id) {
                skipped.push((node_id.clone(), reason.clone()));
                continue;
            }
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?
                .clone();
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
                    preflight_failures.push((
                        node_id.clone(),
                        node,
                        FailureInfo {
                            kind: "Dependency".to_string(),
                            code: "UPSTREAM_FAILED".to_string(),
                            message: "upstream dependency did not complete successfully".to_string(),
                            details: Some(serde_json::json!({ "dependencies": deps })),
                        },
                        "DependencyFailed".to_string(),
                    ));
                    continue;
                }
            }
            let resolved_params = ctx.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);

            if node.retry.max_attempts > 0
                && (node.effects.contains(&Effect::Clock)
                    || node.effects.contains(&Effect::Network))
                && !graph.inputs.contains_key("random_seed")
                && !graph.nondeterminism_allowed
            {
                return Err(RuntimeError::Executor(
                    "retry not allowed for nondeterministic node".to_string(),
                ));
            }
            if options.policy.deny_network && node.effects.contains(&Effect::Network) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "network",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "network effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "network" })),
                    },
                    "PolicyDenied".to_string(),
                ));
                continue;
            }
            if options.policy.deny_env && node.effects.contains(&Effect::Env) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "env",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "env effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "env" })),
                    },
                    "PolicyDenied".to_string(),
                ));
                continue;
            }
            if options.policy.deny_clock && node.effects.contains(&Effect::Clock) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "clock",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "clock effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "clock" })),
                    },
                    "PolicyDenied".to_string(),
                ));
                continue;
            }
            let adapter = runtime.adapter_for_kind(&node.kind)?;
            let required = adapter.required_effects();
            let declared = EffectSet::from_effects(&node.effects);
            if required.filesystem && !declared.filesystem
                || required.env && !declared.env
                || required.network && !declared.network
                || required.clock && !declared.clock
            {
                return Err(RuntimeError::Executor("missing required effects".to_string()));
            }

            let adapter_id = adapter.id();
            let adapter_schema = adapter.produces_outputs_schema_version();
            let inputs_index = sacred_execution::run_materialize_inputs(
                &ctx,
                graph,
                node_id,
                options.materialize_inputs,
            )?;
            let base_fp = base_fps.get(&node.id).cloned().unwrap_or_default();
            let node_fp = node_fingerprint_with_inputs(&base_fp, &inputs_index)?;
            set_node_fingerprint(&ctx, &node.id, node_fp.clone());
            let cache_read = sacred_execution::run_cache_lookup(
                &options,
                &node,
                &node_fp,
                &ctx,
                Arc::clone(&ctx.fs),
                &adapter_id.id,
                &adapter_id.version,
                &adapter_schema,
            )?;
            if let Some(proof) = cache_read.proof.clone() {
                if !cache_read.hit {
                    cache_proofs.insert(node_id.clone(), proof);
                }
            }
            if cache_read.hit {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "cache_hit",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                    }),
                )?;
                run_log_index.push(serde_json::json!({
                    "event": "cache_hit",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }));
                cached.push((node_id.clone(), node, cache_read.proof.unwrap()));
                continue;
            }
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "cache_miss",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "cache_miss",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
            }));

            to_start.push((node_id.clone(), node, resolved_params));
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
            let (aid, aver) = runtime.adapter_meta_for_kind(&node_kind);
            let aschema = runtime.adapter_schema_for_kind(&node_kind);
            let adapter_hash =
                runtime.adapter_for_kind(&node_kind).ok().and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
            sacred_execution::run_write_trace(
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
                &aschema,
                None,
                adapter_hash,
                Some(bijux_dag_artifacts::SkipReason { reason: reason.clone() }),
                Some(crate::transition_cause_for_skip_reason(reason).to_string()),
                Some(ReplayProvenance {
                    node_action: "skipped".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node_id,
                "status": "skipped",
                "cause": crate::transition_cause_for_skip_reason(reason).to_lowercase(),
            }));
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_skipped",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "reason": reason,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_skipped",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "reason": reason,
            }));
        }
        preflight_failures.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, node, failure, transition_cause) in &preflight_failures {
            sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
            status_map.insert(node_id.clone(), NodeStatus::Failed);
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let adapter_hash =
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
            sacred_execution::run_write_trace(
                &ctx,
                graph,
                node_id,
                NodeStatus::Failed,
                Some(failure.clone()),
                started,
                started,
                1,
                None,
                &aid,
                &aver,
                &aschema,
                None,
                adapter_hash,
                None,
                Some(transition_cause.clone()),
                Some(ReplayProvenance {
                    node_action: "reexecuted".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node_id,
                "status": "failed",
                "cause": crate::failure_propagation_cause(Some(failure)),
            }));
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_finished",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": "failed",
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_finished",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "status": "failed",
            }));
        }

        let mut started_ids: Vec<String> = Vec::new();
        for (node_id, _, _) in &to_start {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _) in &cached {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _, _) in &preflight_failures {
            started_ids.push(node_id.clone());
        }
        started_ids.sort();
        let schedule_reason = if forced_batch { "ready" } else { "budget_available" };
        for node_id in &started_ids {
            scheduler_hook.on_node_scheduled(node_id);
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_scheduled",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "reason": schedule_reason,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_scheduled",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "reason": schedule_reason,
            }));
        }

        let checkpoint = ExecutionCheckpoint {
            loop_index,
            ready_queue_depth: ready_queue.len(),
            scheduled: started_ids.clone(),
            blocked_by_budget: blocked_by_budget.clone(),
            generated_unix_ms: ctx.clock.now_unix_ms(),
        };
        let checkpoint_path = ctx.run_dir.staging_path().join("scheduler.checkpoint.json");
        let _ = ctx
            .fs
            .write(&checkpoint_path, &serde_json::to_vec_pretty(&checkpoint).unwrap_or_default());
        for node_id in &started_ids {
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_started",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_started",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
            }));
        }

        for (node_id, node, cache_proof) in &cached {
            sacred_execution::guard_terminal_node_status(&NodeStatus::Cached)?;
            status_map.insert(node_id.clone(), NodeStatus::Cached);
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let adapter_hash =
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
            sacred_execution::run_write_trace(
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
                &aschema,
                None,
                adapter_hash,
                None,
                Some("CachedReuse".to_string()),
                Some(ReplayProvenance {
                    node_action: "reused".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_finished",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": "cached",
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_finished",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "status": "cached",
            }));
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let node_fp = node_fingerprint_from_ctx(&ctx, &node.id);
            sacred_execution::run_cache_write(
                &options,
                node,
                &node_fp,
                &ctx,
                Arc::clone(&ctx.fs),
                &aid,
                &aver,
                &aschema,
            )?;
        }

        for (node_id, node, params) in &to_start {
            let adapter = runtime.adapter_for_kind(&node.kind)?;
            let ctx_clone = RunContext {
                run_dir: Arc::clone(&ctx.run_dir),
                graph_fingerprint: ctx.graph_fingerprint.clone(),
                resolved_params: ctx.resolved_params.clone(),
                fs: Arc::clone(&ctx.fs),
                clock: Arc::clone(&ctx.clock),
                store: ctx.store.clone(),
                policy: ctx.policy.clone(),
            };
            let node_id_clone = node_id.clone();
            let node_for_thread = node.clone();
            let params_for_thread = params.clone();
            let retry = node.retry.clone();
            handles.push((
                node_id_clone,
                node.clone(),
                std::thread::spawn(move || {
                    let started = ctx_clone.clock.now_unix_ms();
                    let result = sacred_execution::run_retry_logic(
                        adapter.as_ref(),
                        &node_for_thread,
                        &params_for_thread,
                        &ctx_clone,
                        &retry,
                    );
                    let finished = ctx_clone.clock.now_unix_ms();
                    (started, finished, result)
                }),
            ));
        }

        type ResultItem = (String, Node, u128, u128, Result<NodeResult, RuntimeError>);
        let mut results: Vec<ResultItem> = Vec::new();
        for (node_id, node, handle) in handles {
            let res = handle.join().unwrap_or_else(|_| {
                (
                    ctx.clock.now_unix_ms(),
                    ctx.clock.now_unix_ms(),
                    Err(RuntimeError::Executor("thread panicked".to_string())),
                )
            });
            results.push((node_id, node, res.0, res.1, res.2));
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, node, started, finished, res) in results {
            match res {
                Ok(result) => {
                    sacred_execution::guard_terminal_node_status(&result.status)?;
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    let adapter_hash =
                        runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
                    let trace_failure = result.failure.clone();
                    let cache_proof = cache_proofs.get(&node_id).cloned();
                    for attempt in &result.attempt_events {
                        crate::append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_attempt_started",
                                "ts": attempt.started_unix_ms,
                                "node_id": node_id,
                                "attempt": attempt.attempt,
                            }),
                        )?;
                        run_log_index.push(serde_json::json!({
                            "event": "node_attempt_started",
                            "ts": attempt.started_unix_ms,
                            "node_id": node_id,
                            "attempt": attempt.attempt,
                        }));
                        crate::append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_attempt_finished",
                                "ts": attempt.finished_unix_ms,
                                "node_id": node_id,
                                "attempt": attempt.attempt,
                                "status": crate::status_string(&attempt.status),
                            }),
                        )?;
                        run_log_index.push(serde_json::json!({
                            "event": "node_attempt_finished",
                            "ts": attempt.finished_unix_ms,
                            "node_id": node_id,
                            "attempt": attempt.attempt,
                            "status": crate::status_string(&attempt.status),
                        }));
                    }
                    crate::write_attempt_events(&ctx, &node_id, &result.attempt_events)?;
                    let replay_action = match result.status {
                        NodeStatus::Cached => ReplayNodeAction::Reused,
                        NodeStatus::Skipped => ReplayNodeAction::Skipped,
                        _ => ReplayNodeAction::Reexecuted,
                    };
                    let replay_event = match replay_action {
                        ReplayNodeAction::Reused => "replay_reused",
                        ReplayNodeAction::Reexecuted => "replay_reexecuted",
                        ReplayNodeAction::Skipped => "replay_reused",
                        ReplayNodeAction::Restored => "replay_reused",
                    };
                    run_log_index.push(serde_json::json!({
                        "event": replay_event,
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                    }));
                    sacred_execution::run_write_trace(
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
                        &aschema,
                        result.container_meta.clone(),
                        adapter_hash,
                        None,
                        Some(
                            if result.status == NodeStatus::Failed {
                                crate::transition_cause_for_failure(result.failure.as_ref())
                            } else {
                                crate::transition_cause_for_status(&result.status)
                            }
                            .to_string(),
                        ),
                        Some(ReplayProvenance {
                            node_action: match replay_action {
                                ReplayNodeAction::Reexecuted => "reexecuted",
                                ReplayNodeAction::Reused => "reused",
                                ReplayNodeAction::Skipped => "skipped",
                                ReplayNodeAction::Restored => "restored",
                            }
                            .to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "node_finished",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "status": crate::status_string(&result.status),
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": crate::status_string(&result.status),
                    }));
                    if result.status == NodeStatus::Failed {
                        status_map.insert(node_id.clone(), NodeStatus::Failed);
                        failure_propagation_records.push(serde_json::json!({
                            "node_id": node_id,
                            "status": "failed",
                            "cause": crate::failure_propagation_cause(result.failure.as_ref()),
                        }));
                    } else {
                        status_map.insert(node_id.clone(), result.status.clone());
                        let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                        let aschema = runtime.adapter_schema_for_kind(&node.kind);
                        let node_fp = node_fingerprint_from_ctx(&ctx, &node.id);
                        sacred_execution::run_cache_write(
                            &options,
                            &node,
                            &node_fp,
                            &ctx,
                            Arc::clone(&ctx.fs),
                            &aid,
                            &aver,
                            &aschema,
                        )?;
                    }
                    let output_bytes =
                        match ctx.fs.metadata(&ctx.run_dir.node_outputs_dir(&node_id)) {
                            Ok(meta) => meta.len(),
                            Err(_) => 0,
                        };
                    node_metric_rows.push(NodeMetrics {
                        node_id: node_id.clone(),
                        queue_delay_ms: 0,
                        execution_time_ms: finished.saturating_sub(started),
                        retries: result.attempts.saturating_sub(1),
                        output_bytes,
                        cache_status: crate::status_string(&result.status),
                        effect_usage: node
                            .effects
                            .iter()
                            .map(|e| format!("{e:?}").to_lowercase())
                            .collect(),
                    });
                }
                Err(err) => {
                    sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    status_map.insert(node_id.clone(), NodeStatus::Failed);
                    let cache_proof = cache_proofs.get(&node_id).cloned();
                    let adapter_hash =
                        runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
                    let failure = FailureInfo {
                        kind: "Internal".to_string(),
                        code: "INTERNAL".to_string(),
                        message: err.to_string(),
                        details: None,
                    };
                    sacred_execution::run_write_trace(
                        &ctx,
                        graph,
                        &node_id,
                        NodeStatus::Failed,
                        Some(failure.clone()),
                        started,
                        finished,
                        1,
                        cache_proof,
                        &aid,
                        &aver,
                        &aschema,
                        None,
                        adapter_hash,
                        None,
                        Some(crate::transition_cause_for_failure(Some(&failure)).to_string()),
                        Some(ReplayProvenance {
                            node_action: "reexecuted".to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    failure_propagation_records.push(serde_json::json!({
                        "node_id": node_id,
                        "status": "failed",
                        "cause": crate::failure_propagation_cause(Some(&failure)),
                    }));
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "node_finished",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "status": "failed",
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": "failed",
                    }));
                    node_metric_rows.push(NodeMetrics {
                        node_id: node_id.clone(),
                        queue_delay_ms: 0,
                        execution_time_ms: finished.saturating_sub(started),
                        retries: 0,
                        output_bytes: 0,
                        cache_status: "failed".to_string(),
                        effect_usage: node
                            .effects
                            .iter()
                            .map(|e| format!("{e:?}").to_lowercase())
                            .collect(),
                    });
                }
            }
        }

        for node_id in batch {
            for newly_ready in dependency_counter.mark_completed(&node_id) {
                ready_queue.insert(newly_ready);
            }
        }

        if matches!(options.failure_propagation, crate::FailurePropagationMode::FailFast)
            && status_map.values().any(|s| *s == NodeStatus::Failed)
        {
            break;
        }
    }

    if cancel.load(Ordering::SeqCst) {
        run_audit_events.push(serde_json::json!({
            "action": "cancel",
            "ts": ctx.clock.now_unix_ms(),
            "run_id": manifest.run_id.clone(),
        }));
        for node in &graph.nodes {
            if !status_map.contains_key(&node.id) {
                status_map.insert(node.id.clone(), NodeStatus::Skipped);
                let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                let aschema = runtime.adapter_schema_for_kind(&node.kind);
                let started = ctx.clock.now_unix_ms();
                sacred_execution::run_write_trace(
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
                    &aschema,
                    None,
                    runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash()),
                    Some(bijux_dag_artifacts::SkipReason { reason: "cancelled".to_string() }),
                    Some("CancelRequested".to_string()),
                    Some(ReplayProvenance {
                        node_action: "skipped".to_string(),
                        source_run_id: options.parent_run_id.clone(),
                    }),
                )?;
                failure_propagation_records.push(serde_json::json!({
                    "node_id": node.id,
                    "status": "skipped",
                    "cause": "cancel_requested",
                }));
            }
        }
    }

    let finished_unix_ms = ctx.clock.now_unix_ms();
    let memory_before_materialization = current_process_memory_bytes().unwrap_or(0);
    if cancel.load(Ordering::SeqCst) {
        manifest.status = "cancelled".to_string();
    } else if status_map.values().any(|s| *s == NodeStatus::Failed) {
        manifest.status = "failed".to_string();
    }
    manifest.finished_unix_ms = finished_unix_ms;
    manifest.node_counts = sacred_execution::count_terminal_nodes(&status_map);
    let trace_statuses: Vec<NodeStatus> = status_map.values().cloned().collect();
    let invariant_counts = crate::invariants::RunNodeCounts {
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
    };
    if !crate::invariants::run_summary_invariant_ok(invariant_counts, &trace_statuses) {
        return Err(RuntimeError::Executor(
            "run summary invariant violated: manifest totals do not match trace totals".to_string(),
        ));
    }
    manifest.run_summary = Some(engine_finalize::summarize_counts(&manifest.node_counts));
    manifest.outputs = collect_outputs_summary(ctx.fs.as_ref(), &ctx.run_dir)?;
    let memory_after_materialization = current_process_memory_bytes().unwrap_or(0);
    let run_index = build_run_outputs_index(&ctx.run_dir, &manifest.outputs)?;
    let lineage_edges = manifest
        .outputs
        .iter()
        .map(|out| bijux_dag_artifacts::lineage::ArtifactLineageEdge {
            artifact_id: format!("{}:{}", out.node_id, out.file),
            producer_node_id: out.node_id.clone(),
            upstream_artifact_ids: dep_map
                .get(&out.node_id)
                .map(|deps| deps.iter().map(|d| format!("{d}:*")).collect())
                .unwrap_or_default(),
        })
        .collect();
    let lineage_snapshot = bijux_dag_artifacts::lineage::ArtifactLineageSnapshot {
        schema_version: "v0.1".to_string(),
        edges: lineage_edges,
    };
    let _ = bijux_dag_artifacts::lineage::write_lineage_snapshot(
        ctx.run_dir.staging_path().join("lineage.snapshot.json"),
        &lineage_snapshot,
    );
    let _ = bijux_dag_artifacts::lineage::export_lineage_visualization(
        ctx.run_dir.staging_path().join("observability.lineage-visualization.json"),
        &lineage_snapshot,
    );
    write_run_outputs_index(ctx.run_dir.staging_path().join("outputs"), &run_index)?;
    run_dir.write_manifest(&manifest)?;
    crate::append_event(
        &mut run_log,
        serde_json::json!({
            "event": "run_finished",
            "ts": finished_unix_ms,
            "status": manifest.status,
        }),
    )?;
    run_log_index.push(serde_json::json!({
        "event": "run_finished",
        "ts": finished_unix_ms,
        "status": manifest.status,
    }));
    run_audit_events.push(serde_json::json!({
        "action": "finish",
        "ts": finished_unix_ms,
        "run_id": manifest.run_id.clone(),
        "status": manifest.status.clone(),
    }));
    let mut structured_events: Vec<EventRecord> = Vec::new();
    for entry in &run_log_index {
        let name = entry.get("event").and_then(|v| v.as_str()).unwrap_or("unknown");
        let unix_ms = entry.get("ts").and_then(|v| v.as_u64()).unwrap_or(0) as u128;
        let node_id = entry.get("node_id").and_then(|v| v.as_str()).map(ToString::to_string);
        let details = entry.clone();
        structured_events.push(EventRecord {
            category: category_from_runtime_event_name(name),
            name: name.to_string(),
            unix_ms,
            node_id,
            run_id: Some(manifest.run_id.clone()),
            details,
        });
    }
    let cache_hits = engine_metrics::count_cache_hits(&status_map);
    let run_metrics = engine_metrics::build_run_metrics(
        &manifest.node_counts,
        graph.nodes.len(),
        &options,
        finished_unix_ms,
        started_unix_ms,
        cache_hits,
        manifest.outputs.len(),
    );
    let scheduler_metrics = engine_metrics::build_scheduler_metrics(
        &manifest.node_counts,
        &run_log_index,
        &options,
        &failure_propagation_records,
    );
    for row in node_metric_rows {
        metrics_registry.record_node(row);
    }
    metrics_registry.record_run(run_metrics);
    metrics_registry.record_scheduler(scheduler_metrics);
    let timeline = TimelineExport {
        schema_version: "v0.1".to_string(),
        entries: structured_events
            .iter()
            .map(|event| TimelineEntry {
                unix_ms: event.unix_ms,
                category: format!("{:?}", event.category).to_lowercase(),
                label: event.name.clone(),
                node_id: event.node_id.clone(),
            })
            .collect(),
    };
    let _ = write_timeline_export(
        ctx.run_dir.staging_path().join("observability.timeline.json"),
        &timeline,
    );
    let root_causes = summarize_failure_root_causes(&structured_events);
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.root-causes.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({ "roots": root_causes }))?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.events.json"),
        &serde_json::to_vec_pretty(&structured_events)?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.metrics.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "node": metrics_registry.node_metrics,
            "run": metrics_registry.run_metrics,
            "scheduler": metrics_registry.scheduler_metrics,
            "memory": {
                "before_materialization_bytes": memory_before_materialization,
                "after_materialization_bytes": memory_after_materialization
            }
        }))?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.graph-visualization.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "nodes": graph.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "status": status_map.get(&n.id).map(crate::status_string).unwrap_or_else(|| "unknown".to_string()),
                })
            }).collect::<Vec<_>>(),
            "edges": graph.edges.iter().map(|e| {
                serde_json::json!({"from": e.from.node_id, "to": e.to.node_id})
            }).collect::<Vec<_>>(),
            "lineage_snapshot": "lineage.snapshot.json",
            "timeline": "observability.timeline.json"
        }))?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("run-log.index.json"),
        &serde_json::to_vec_pretty(&run_log_index)?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("run.audit.json"),
        &serde_json::to_vec_pretty(&run_audit_events)?,
    );
    let _ = ctx.fs.write(
        &ctx.run_dir.staging_path().join("failure-propagation.json"),
        &serde_json::to_vec_pretty(&failure_propagation_records)?,
    );

    let final_path = run_dir.finalize()?;
    if let Some(latest) = options.latest_symlink {
        let _ = runtime.fs.remove_file(&latest);
        let _ = runtime.fs.symlink(&final_path, &latest);
    }
    Ok(final_path)
}
