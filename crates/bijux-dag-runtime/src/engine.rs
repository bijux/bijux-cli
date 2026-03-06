use crate::{
    build_run_outputs_index, cache_dir_from_env, cache_mode_string, collect_outputs_summary,
    count_nodes, execute_with_retries, materialize_inputs, node_fingerprint_from_ctx,
    node_fingerprint_with_inputs, registered_adapters, set_node_fingerprint, try_cache_read,
    try_cache_write, write_trace, CacheProof, DependencyCounter, EffectSet, ExecutionCheckpoint,
    NodeResult, NodeStatus, ReadyQueue, ReplayNodeAction, RunAttempt, RunId, RunSnapshot, Runtime,
    RuntimeConfig, RuntimeError, SchedulerEventHook,
};
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
    plan: crate::planner::ExecutionPlan,
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
        run_dir
            .final_path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
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
        tool_version: crate::tool_version(),
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
            clean_env: options.policy.clean_env,
        },
        cache_mode: cache_mode_string(&options.cache_mode),
        cache_dir: effective_cache_dir
            .as_ref()
            .map(|p| p.display().to_string()),
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
    });
    run_dir.write_manifest(&manifest)?;

    let prov = Provenance {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rustc: crate::rustc_version(),
        tool_version: crate::tool_version(),
        adapters: registered_adapters(),
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
    crate::append_event(
        &mut run_log,
        serde_json::json!({
            "event": "run_started",
            "ts": started_unix_ms,
        }),
    )?;
    run_log_index.push(serde_json::json!({
        "event": "run_started",
        "ts": started_unix_ms,
    }));
    run_audit_events.push(serde_json::json!({
        "action": "start",
        "ts": started_unix_ms,
        "run_id": manifest.run_id.clone(),
    }));

    let resolved = graph.resolve_graph()?;
    let mut base_fps = HashMap::new();
    for node in &graph.nodes {
        let params = resolved
            .resolved_params
            .get(&node.id)
            .cloned()
            .unwrap_or(Value::Null);
        base_fps.insert(
            node.id.clone(),
            graph.node_fingerprint_with_params(node, &params)?,
        );
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
        parent_run_id: options
            .parent_run_id
            .as_deref()
            .and_then(|v| RunId::parse(v).ok()),
        selected_nodes,
        dependency_closure_enabled: options.partial_rerun_dependency_closure,
        replay_source_run_id: options
            .parent_run_id
            .as_deref()
            .and_then(|v| RunId::parse(v).ok()),
    };
    let run_snapshot_path = ctx.run_dir.staging_path().join("run.snapshot.json");
    let _ = ctx
        .fs
        .write(&run_snapshot_path, &serde_json::to_vec_pretty(&run_snapshot)?);
    let run_attempt = RunAttempt {
        attempt_index: 1,
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        parent_run_id: options
            .parent_run_id
            .as_deref()
            .and_then(|v| RunId::parse(v).ok()),
        reason: if options.parent_run_id.is_some() {
            "replay_or_retry".to_string()
        } else {
            "initial_submission".to_string()
        },
    };
    let run_attempts_path = ctx.run_dir.staging_path().join("run.attempts.json");
    let _ = ctx
        .fs
        .write(&run_attempts_path, &serde_json::to_vec_pretty(&vec![run_attempt])?);
    let start = Instant::now();
    let mut status_map: HashMap<String, NodeStatus> = HashMap::new();
    let mut cache_proofs: HashMap<String, CacheProof> = HashMap::new();
    let dep_map = plan.dep_map.clone();
    let mut dependency_counter = DependencyCounter::from_plan(&plan);
    let mut ready_queue = ReadyQueue::from_indegree(dependency_counter.indegree_map());
    let mut scheduler = crate::build_scheduler(&options.scheduler_policy);
    let scheduler_hook = crate::NoopSchedulerEventHook;
    let mut loop_index: u64 = 0;
    while !ready_queue.is_empty() {
        loop_index = loop_index.saturating_add(1);
        let ready_vec: Vec<String> = ready_queue.snapshot_sorted();
        for node_id in &ready_vec {
            scheduler_hook.on_node_eligible(node_id);
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_eligible",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }),
            )?;
        }
        let decision = scheduler.next_batch(
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

        for node_id in &batch {
            if let Some(reason) = plan.filter_reasons.get(node_id) {
                skipped.push((node_id.clone(), reason.clone()));
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

            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?
                .clone();
            let resolved_params = ctx
                .resolved_params
                .get(&node.id)
                .cloned()
                .unwrap_or(Value::Null);

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
                return Err(RuntimeError::Executor(
                    "network effect denied by policy".to_string(),
                ));
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
                return Err(RuntimeError::Executor(
                    "env effect denied by policy".to_string(),
                ));
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
                return Err(RuntimeError::Executor(
                    "clock effect denied by policy".to_string(),
                ));
            }
            let adapter = runtime.adapter_for_kind(&node.kind)?;
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

            let adapter_id = adapter.id();
            let adapter_schema = adapter.produces_outputs_schema_version();
            let inputs_index =
                materialize_inputs(&ctx, graph, node_id, options.materialize_inputs)?;
            let base_fp = base_fps.get(&node.id).cloned().unwrap_or_default();
            let node_fp = node_fingerprint_with_inputs(&base_fp, &inputs_index)?;
            set_node_fingerprint(&ctx, &node.id, node_fp.clone());
            let cache_read = try_cache_read(
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
                cached.push((node_id.clone(), node, cache_read.proof.unwrap()));
                continue;
            }

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
            let adapter_hash = runtime
                .adapter_for_kind(&node_kind)
                .ok()
                .and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
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
                &aschema,
                None,
                adapter_hash,
                Some(bijux_dag_artifacts::SkipReason {
                    reason: reason.clone(),
                }),
                Some("SelectionFiltered".to_string()),
                Some(ReplayProvenance {
                    node_action: "skipped".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node_id,
                "status": "skipped",
                "cause": reason,
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

        let mut started_ids: Vec<String> = Vec::new();
        for (node_id, _, _) in &to_start {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _) in &cached {
            started_ids.push(node_id.clone());
        }
        started_ids.sort();
        let schedule_reason = if forced_batch {
            "ready"
        } else {
            "budget_available"
        };
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
        let _ = ctx.fs.write(
            &checkpoint_path,
            &serde_json::to_vec_pretty(&checkpoint).unwrap_or_default(),
        );
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
            status_map.insert(node_id.clone(), NodeStatus::Cached);
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let adapter_hash = runtime
                .adapter_for_kind(&node.kind)
                .ok()
                .and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
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
            try_cache_write(
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
                    let result = execute_with_retries(
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
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    let adapter_hash = runtime
                        .adapter_for_kind(&node.kind)
                        .ok()
                        .and_then(|a| a.binary_hash());
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
                    let replay_action = match result.status {
                        NodeStatus::Cached => ReplayNodeAction::Reused,
                        NodeStatus::Skipped => ReplayNodeAction::Skipped,
                        _ => ReplayNodeAction::Reexecuted,
                    };
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
                        &aschema,
                        result.container_meta.clone(),
                        adapter_hash,
                        None,
                        Some(crate::transition_cause_for_status(&result.status).to_string()),
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
                            "cause": "execution_failed",
                        }));
                    } else {
                        status_map.insert(node_id.clone(), result.status.clone());
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    let node_fp = node_fingerprint_from_ctx(&ctx, &node.id);
                    try_cache_write(
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
                }
                Err(err) => {
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    status_map.insert(node_id.clone(), NodeStatus::Failed);
                    let cache_proof = cache_proofs.get(&node_id).cloned();
                    let adapter_hash = runtime
                        .adapter_for_kind(&node.kind)
                        .ok()
                        .and_then(|a| a.binary_hash());
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
                        &aschema,
                        None,
                        adapter_hash,
                        None,
                        Some("ExecutionFailed".to_string()),
                        Some(ReplayProvenance {
                            node_action: "reexecuted".to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    failure_propagation_records.push(serde_json::json!({
                        "node_id": node_id,
                        "status": "failed",
                        "cause": "internal_error",
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
            }
        }

        for node_id in batch {
            for newly_ready in dependency_counter.mark_completed(&node_id) {
                ready_queue.insert(newly_ready);
            }
        }

        if matches!(
            options.failure_propagation,
            crate::FailurePropagationMode::FailFast
        ) && status_map.values().any(|s| *s == NodeStatus::Failed)
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
                    &aschema,
                    None,
                    runtime
                        .adapter_for_kind(&node.kind)
                        .ok()
                        .and_then(|a| a.binary_hash()),
                    Some(bijux_dag_artifacts::SkipReason {
                        reason: "cancelled".to_string(),
                    }),
                    Some("CancelRequested".to_string()),
                    Some(ReplayProvenance {
                        node_action: "skipped".to_string(),
                        source_run_id: options.parent_run_id.clone(),
                    }),
                )?;
                failure_propagation_records.push(serde_json::json!({
                    "node_id": node.id,
                    "status": "skipped",
                    "cause": "cancelled",
                }));
            }
        }
    }

    let finished_unix_ms = ctx.clock.now_unix_ms();
    if cancel.load(Ordering::SeqCst) {
        manifest.status = "cancelled".to_string();
    } else if status_map.values().any(|s| *s == NodeStatus::Failed) {
        manifest.status = "failed".to_string();
    }
    manifest.finished_unix_ms = finished_unix_ms;
    manifest.node_counts = count_nodes(&status_map);
    manifest.run_summary = Some(bijux_dag_artifacts::RunSummary {
        total_nodes: manifest.node_counts.success
            + manifest.node_counts.failed
            + manifest.node_counts.skipped
            + manifest.node_counts.cached,
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
    });
    manifest.outputs = collect_outputs_summary(ctx.fs.as_ref(), &ctx.run_dir)?;
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
