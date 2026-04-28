use crate::simulated_platform::{is_duplicate_dispatch, normalize_status_events, RemoteStatusEvent};
use crate::{
    default_forced_cleanup, duplicate_status_delivery_detected, validate_task_contracts,
    BatchLifecycleEvent, ForcedCancellationCleanup, Graph, RuntimeConfig, RuntimeError,
    TaskIsolationMode,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIsolationNodeReport {
    pub node_id: String,
    pub isolation_mode: TaskIsolationMode,
    pub forced_cleanup: ForcedCancellationCleanup,
    pub idempotency_mode: String,
    pub executor_surface: String,
    pub side_effects: Vec<String>,
    pub sandbox_guards: Vec<String>,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIsolationReport {
    pub total_nodes: usize,
    pub isolation_counts: BTreeMap<String, usize>,
    pub executor_surfaces: BTreeSet<String>,
    pub nodes: Vec<ExecutionIsolationNodeReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchKeyRecord {
    pub run_id: String,
    pub node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchAuditReport {
    pub submitted_dispatches: usize,
    pub duplicate_dispatch_keys: Vec<String>,
    pub remote_status_duplicates: usize,
    pub normalized_remote_statuses: usize,
    pub duplicate_batch_delivery_detected: bool,
    pub idempotent_dispatch_guarantee: bool,
}

pub fn build_execution_isolation_report(
    graph: &Graph,
    options: &RuntimeConfig,
) -> Result<ExecutionIsolationReport, RuntimeError> {
    let mut isolation_counts = BTreeMap::new();
    let mut executor_surfaces = BTreeSet::new();
    let mut nodes = validate_task_contracts(graph, options)?
        .into_iter()
        .map(|contract| {
            let isolation_label = match contract.isolation_mode {
                TaskIsolationMode::InProcess => "in_process",
                TaskIsolationMode::Subprocess => "subprocess",
                TaskIsolationMode::Container => "container",
                TaskIsolationMode::ExternalAdapter => "external_adapter",
            };
            *isolation_counts.entry(isolation_label.to_string()).or_insert(0) += 1;

            let executor_surface = match contract.isolation_mode {
                TaskIsolationMode::InProcess => "inline-kernel",
                TaskIsolationMode::Subprocess => "local-subprocess",
                TaskIsolationMode::Container => "container-engine",
                TaskIsolationMode::ExternalAdapter => "remote-adapter",
            }
            .to_string();
            executor_surfaces.insert(executor_surface.clone());

            let mut sandbox_guards = Vec::new();
            if contract.sandbox_policy.deny_network {
                sandbox_guards.push("deny-network".to_string());
            }
            if contract.sandbox_policy.deny_env {
                sandbox_guards.push("deny-env".to_string());
            }
            if contract.sandbox_policy.deny_clock {
                sandbox_guards.push("deny-clock".to_string());
            }
            if contract.sandbox_policy.clean_env {
                sandbox_guards.push("clean-env".to_string());
            }

            let mut risk_flags = Vec::new();
            if contract.nondeterministic_allowed {
                risk_flags.push("nondeterministic".to_string());
            }
            if matches!(contract.isolation_mode, TaskIsolationMode::InProcess)
                && !contract.effects.is_empty()
            {
                risk_flags.push("side-effects-without-process-boundary".to_string());
            }
            if matches!(contract.isolation_mode, TaskIsolationMode::ExternalAdapter) {
                risk_flags.push("adapter-boundary".to_string());
            }

            ExecutionIsolationNodeReport {
                node_id: contract.node_id,
                isolation_mode: contract.isolation_mode.clone(),
                forced_cleanup: default_forced_cleanup(&contract.isolation_mode),
                idempotency_mode: format!("{:?}", contract.idempotency_mode).to_lowercase(),
                executor_surface,
                side_effects: contract
                    .effects
                    .iter()
                    .map(|effect| format!("{:?}", effect.effect).to_lowercase())
                    .collect(),
                sandbox_guards,
                risk_flags,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    Ok(ExecutionIsolationReport {
        total_nodes: nodes.len(),
        isolation_counts,
        executor_surfaces,
        nodes,
    })
}

pub fn audit_dispatch_discipline(
    dispatches: &[DispatchKeyRecord],
    remote_status_events: &[RemoteStatusEvent],
    batch_events: &[BatchLifecycleEvent],
) -> DispatchAuditReport {
    let mut seen = BTreeSet::new();
    let mut duplicate_dispatch_keys = Vec::new();
    for dispatch in dispatches {
        if is_duplicate_dispatch(&mut seen, &dispatch.run_id, &dispatch.node_id) {
            duplicate_dispatch_keys.push(format!("{}:{}", dispatch.run_id, dispatch.node_id));
        }
    }

    let (normalized_remote_statuses, remote_duplicates) = normalize_status_events(remote_status_events);
    let duplicate_batch_delivery_detected = duplicate_status_delivery_detected(batch_events);
    let idempotent_dispatch_guarantee =
        duplicate_dispatch_keys.is_empty() && !duplicate_batch_delivery_detected;

    DispatchAuditReport {
        submitted_dispatches: dispatches.len(),
        duplicate_dispatch_keys,
        remote_status_duplicates: remote_duplicates.len(),
        normalized_remote_statuses: normalized_remote_statuses.len(),
        duplicate_batch_delivery_detected,
        idempotent_dispatch_guarantee,
    }
}

#[cfg(test)]
mod tests {
    use super::{audit_dispatch_discipline, build_execution_isolation_report, DispatchKeyRecord};
    use crate::{BatchLifecycleEvent, RuntimeConfig};
    use bijux_dag_core::{Edge, FileOutput, Graph, GraphMeta, Node, NodeKind, ParamValue, PortRef};

    fn graph_fixture() -> Graph {
        Graph {
            spec: "bijux-dag/v0.1".to_string(),
            meta: Some(GraphMeta {
                name: "runtime".to_string(),
                description: None,
                owners: Vec::new(),
                tags: Vec::new(),
            }),
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "const1".to_string(),
                    kind: NodeKind::Const,
                    inputs: Vec::new(),
                    outputs: vec![FileOutput { name: "out".to_string(), path: "a/out".to_string() }],
                    params: ParamValue::Literal(serde_json::json!({"value":"1"})),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: Vec::new(),
                    retry: Default::default(),
                    effects: Vec::new(),
                    env_allowlist: Vec::new(),
                    group: None,
                },
                Node {
                    id: "shell1".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput { name: "out".to_string(), path: "b/out".to_string() }],
                    params: ParamValue::Literal(serde_json::json!({"argv":["/bin/sh","-c","true"]})),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: Vec::new(),
                    retry: Default::default(),
                    effects: vec![bijux_dag_core::Effect::Filesystem],
                    env_allowlist: Vec::new(),
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef { node_id: "const1".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "shell1".to_string(), port: "in".to_string() },
            }],
        }
    }

    #[test]
    fn isolation_report_distinguishes_inline_and_subprocess_nodes() {
        let report =
            build_execution_isolation_report(&graph_fixture(), &RuntimeConfig::default()).expect("report");
        assert_eq!(report.total_nodes, 2);
        assert!(report.isolation_counts.contains_key("in_process"));
        assert!(report.isolation_counts.contains_key("subprocess"));
    }

    #[test]
    fn dispatch_audit_flags_duplicate_dispatch_keys() {
        let report = audit_dispatch_discipline(
            &[
                DispatchKeyRecord { run_id: "run-1".to_string(), node_id: "a".to_string() },
                DispatchKeyRecord { run_id: "run-1".to_string(), node_id: "a".to_string() },
            ],
            &[],
            &[BatchLifecycleEvent {
                scheduler_id: "scheduler".to_string(),
                status: "submitted".to_string(),
                unix_ms: 1,
            }],
        );
        assert!(!report.idempotent_dispatch_guarantee);
        assert_eq!(report.duplicate_dispatch_keys, vec!["run-1:a".to_string()]);
    }
}
