use crate::commands::{DagCli, StateStoreCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_artifacts::NodeCounts;
use bijux_dag_runtime::{check_run_consistency, NodeState, RunId, RunState, RunSummaryV2};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct TransactionSimulation {
    run_id: String,
    run_state: RunState,
    counts: NodeCounts,
    node_states: Vec<NodeStateRecord>,
    artifact_nodes: Vec<String>,
    manifest_written: bool,
    journal_written: bool,
    index_written: bool,
    rollback_recorded: bool,
}

#[derive(Debug, Deserialize)]
struct NodeStateRecord {
    node_id: String,
    state: NodeState,
}

#[derive(Debug, Serialize)]
struct TransactionReport {
    run_id: String,
    summary_matches_node_states: bool,
    all_success_nodes_have_artifacts: bool,
    materialized_components: Vec<String>,
    rollback_recorded: bool,
    gaps: Vec<String>,
    transaction_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn transaction_payload(simulation: TransactionSimulation) -> (serde_json::Value, bool) {
    let TransactionSimulation {
        run_id,
        run_state,
        counts,
        node_states,
        artifact_nodes,
        manifest_written,
        journal_written,
        index_written,
        rollback_recorded,
    } = simulation;
    let summary = RunSummaryV2 {
        run_id: RunId::parse(&run_id).unwrap_or_else(|_| RunId("invalid-run-id".to_string())),
        state: run_state,
        counts,
    };
    let state_pairs =
        node_states.iter().map(|record| (record.node_id.clone(), record.state.clone())).collect::<Vec<_>>();
    let consistency = check_run_consistency(&state_pairs, &artifact_nodes, &summary);
    let mut materialized_components = Vec::new();
    if manifest_written {
        materialized_components.push("manifest".to_string());
    }
    if journal_written {
        materialized_components.push("journal".to_string());
    }
    if index_written {
        materialized_components.push("index".to_string());
    }
    let all_components_written = manifest_written && journal_written && index_written;
    let atomic_visible_state = (all_components_written && consistency.summary_matches_node_states)
        || rollback_recorded;
    let mut gaps = Vec::new();
    if run_id.trim().is_empty() {
        gaps.push("transaction audit requires a stable run id".to_string());
    }
    if !consistency.summary_matches_node_states {
        gaps.push("run summary does not match materialized node states".to_string());
    }
    if !consistency.all_success_nodes_have_artifacts {
        gaps.push("successful nodes are missing persisted artifacts".to_string());
    }
    if !all_components_written && !rollback_recorded {
        gaps.push("partial state write became visible without a rollback record".to_string());
    }
    if !atomic_visible_state {
        gaps.push("state mutation is not provably atomic from the visible components".to_string());
    }
    let report = TransactionReport {
        run_id,
        summary_matches_node_states: consistency.summary_matches_node_states,
        all_success_nodes_have_artifacts: consistency.all_success_nodes_have_artifacts,
        materialized_components,
        rollback_recorded,
        transaction_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.transaction_ready;
    (serde_json::to_value(report).expect("transaction report"), ok)
}

pub(crate) fn handle_state_store_command(
    cli: &DagCli,
    command: &StateStoreCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        StateStoreCommands::Transaction { simulation } => {
            let simulation: TransactionSimulation = parse_json_file(simulation)?;
            let (payload, ok) = transaction_payload(simulation);
            ("dag.state-store.transaction", payload, ok)
        }
    };
    emit_json(
        cli,
        surface,
        ok,
        payload,
        if ok {
            Vec::new()
        } else {
            vec![json!({
                "message":"state-store posture is incomplete",
                "remediation":"fix the reported state-store gaps before treating this persistence surface as production-ready"
            })]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{transaction_payload, NodeStateRecord, TransactionSimulation};
    use bijux_dag_artifacts::NodeCounts;
    use bijux_dag_runtime::{NodeState, RunState};

    #[test]
    fn transaction_accepts_consistent_atomic_visible_state() {
        let simulation = TransactionSimulation {
            run_id: "run-1".to_string(),
            run_state: RunState::Succeeded,
            counts: NodeCounts { success: 1, failed: 0, skipped: 0, cached: 0 },
            node_states: vec![NodeStateRecord {
                node_id: "extract".to_string(),
                state: NodeState::Success,
            }],
            artifact_nodes: vec!["extract".to_string()],
            manifest_written: true,
            journal_written: true,
            index_written: true,
            rollback_recorded: false,
        };
        let (payload, ok) = transaction_payload(simulation);
        assert!(ok);
        assert_eq!(payload["transaction_ready"], true);
    }

    #[test]
    fn transaction_flags_partial_visible_state_without_rollback() {
        let simulation = TransactionSimulation {
            run_id: String::new(),
            run_state: RunState::Succeeded,
            counts: NodeCounts { success: 1, failed: 0, skipped: 0, cached: 0 },
            node_states: vec![NodeStateRecord {
                node_id: "extract".to_string(),
                state: NodeState::Success,
            }],
            artifact_nodes: Vec::new(),
            manifest_written: true,
            journal_written: false,
            index_written: true,
            rollback_recorded: false,
        };
        let (payload, ok) = transaction_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }
}
