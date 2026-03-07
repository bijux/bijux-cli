//! Centralized sacred execution flow hooks.

use crate::adapter::Adapter;
use crate::{
    materialize_inputs, try_cache_read, try_cache_write, write_trace, CacheProof, CacheRead,
    DependencyCounter, NodeResult, NodeStatus, ReadyQueue, RetryPolicy, RunContext, RuntimeConfig,
    RuntimeError,
};
use bijux_dag_artifacts::{ContainerTrace, FailureInfo, InputsIndex, NodeCounts, ReplayProvenance};
use bijux_dag_core::{Graph, Node};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) fn run_retry_logic(
    adapter: &dyn Adapter,
    node: &Node,
    params: &Value,
    ctx: &RunContext,
    retry: &RetryPolicy,
) -> Result<NodeResult, RuntimeError> {
    crate::execute_with_retries(adapter, node, params, ctx, retry)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_write_trace(
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
    transition_cause: Option<String>,
    replay_provenance: Option<ReplayProvenance>,
) -> Result<(), RuntimeError> {
    write_trace(
        ctx,
        graph,
        node_id,
        status,
        failure,
        started_unix_ms,
        finished_unix_ms,
        attempt,
        cache_proof,
        adapter_id,
        adapter_version,
        adapter_outputs_schema_version,
        container_meta,
        adapter_binary_sha256,
        skip_reason,
        transition_cause,
        replay_provenance,
    )
}

pub(crate) fn run_materialize_inputs(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    mode: crate::MaterializeMode,
) -> Result<InputsIndex, RuntimeError> {
    materialize_inputs(ctx, graph, node_id, mode)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_cache_lookup(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn crate::Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<CacheRead, RuntimeError> {
    try_cache_read(
        options,
        node,
        node_fingerprint,
        ctx,
        fs,
        adapter_id,
        adapter_version,
        adapter_outputs_schema_version,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_cache_write(
    options: &RuntimeConfig,
    node: &Node,
    node_fingerprint: &str,
    ctx: &RunContext,
    fs: Arc<dyn crate::Fs>,
    adapter_id: &str,
    adapter_version: &str,
    adapter_outputs_schema_version: &str,
) -> Result<(), RuntimeError> {
    try_cache_write(
        options,
        node,
        node_fingerprint,
        ctx,
        fs,
        adapter_id,
        adapter_version,
        adapter_outputs_schema_version,
    )
}

pub(crate) fn ready_queue_from_dependencies(counter: &DependencyCounter) -> ReadyQueue {
    ReadyQueue::from_indegree(counter.indegree_map())
}

pub(crate) fn resolve_dependencies(plan: &crate::ExecutionPlan) -> DependencyCounter {
    DependencyCounter::from_plan(plan)
}

pub(crate) fn count_terminal_nodes(status_map: &HashMap<String, NodeStatus>) -> NodeCounts {
    crate::count_nodes(status_map)
}

pub(crate) fn guard_terminal_node_status(to: &NodeStatus) -> Result<(), RuntimeError> {
    use crate::state_machine::{node_transition_allowed, NodeLifecycleState as S};
    let target = match to {
        NodeStatus::Success => S::Succeeded,
        NodeStatus::Failed => S::Failed,
        NodeStatus::Skipped => S::Skipped,
        NodeStatus::Cached => S::Cached,
    };
    if node_transition_allowed(S::Running, target) {
        Ok(())
    } else {
        Err(RuntimeError::Executor(format!(
            "illegal node terminal transition from running to {:?}",
            to
        )))
    }
}
