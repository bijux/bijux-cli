use crate::commands::{DagCli, StateStoreCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_artifacts::NodeCounts;
use bijux_dag_runtime::{
    check_run_consistency, event_names_emitted_once, required_event_fields_present,
    validate_required_event_names, EventRecord, NodeState, PersistedRunSnapshotRef, RunCompactionPolicy,
    RunId, RunState, RunSummaryV2,
};
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

#[derive(Debug, Deserialize)]
struct JournalSimulation {
    events: Vec<EventRecord>,
    rewrite_detected: bool,
}

#[derive(Debug, Serialize)]
struct JournalReport {
    event_count: usize,
    required_names_present: bool,
    append_only: bool,
    monotonic_timestamps: bool,
    singleton_boundaries_ok: bool,
    gaps: Vec<String>,
    journal_ready: bool,
}

#[derive(Debug, Deserialize)]
struct SnapshotSimulation {
    snapshot: Option<PersistedRunSnapshotRef>,
    compaction_policy: RunCompactionPolicy,
    event_count: usize,
    latest_attempts_kept: usize,
    rebuildable_from_journal: bool,
}

#[derive(Debug, Serialize)]
struct SnapshotReport {
    snapshot_present: bool,
    compaction_due: bool,
    persisted_after_threshold: bool,
    keep_latest_attempts_respected: bool,
    rebuildable_from_journal: bool,
    gaps: Vec<String>,
    snapshot_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IndexSimulation {
    indexed_dimensions: Vec<String>,
    run_count: usize,
    owner_cardinality: usize,
    tag_cardinality: usize,
    partition_cardinality: usize,
    failure_class_cardinality: usize,
    p95_lookup_ms: u64,
    max_lookup_ms: u64,
}

#[derive(Debug, Serialize)]
struct IndexReport {
    dimension_count: usize,
    all_required_dimensions_indexed: bool,
    high_cardinality_ready: bool,
    lookup_within_limit: bool,
    gaps: Vec<String>,
    index_ready: bool,
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

fn journal_payload(simulation: JournalSimulation) -> (serde_json::Value, bool) {
    let JournalSimulation { events, rewrite_detected } = simulation;
    let required_names_present = validate_required_event_names(&events).is_empty();
    let append_only = !rewrite_detected;
    let monotonic_timestamps = events
        .windows(2)
        .all(|pair| pair[0].unix_ms <= pair[1].unix_ms);
    let singleton_boundaries_ok =
        event_names_emitted_once(&events, &["run_started", "run_finished"]);
    let all_fields_present = events.iter().all(required_event_fields_present);
    let mut gaps = Vec::new();
    if events.is_empty() {
        gaps.push("journal audit requires at least one persisted event".to_string());
    }
    if !all_fields_present {
        gaps.push("persisted events are missing required journal fields".to_string());
    }
    if !required_names_present {
        gaps.push("journal is missing required lifecycle event names".to_string());
    }
    if !append_only {
        gaps.push("journal rewrite was detected on an append-only surface".to_string());
    }
    if !monotonic_timestamps {
        gaps.push("journal event timestamps are not monotonic".to_string());
    }
    if !singleton_boundaries_ok {
        gaps.push("run boundary events are duplicated or missing".to_string());
    }
    let report = JournalReport {
        event_count: events.len(),
        required_names_present,
        append_only,
        monotonic_timestamps,
        singleton_boundaries_ok,
        journal_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.journal_ready;
    (serde_json::to_value(report).expect("journal report"), ok)
}

fn snapshot_payload(simulation: SnapshotSimulation) -> (serde_json::Value, bool) {
    let SnapshotSimulation {
        snapshot,
        compaction_policy,
        event_count,
        latest_attempts_kept,
        rebuildable_from_journal,
    } = simulation;
    let snapshot_present = snapshot.is_some();
    let compaction_due = event_count >= compaction_policy.max_event_count_before_compaction;
    let persisted_after_threshold = match snapshot.as_ref() {
        Some(snapshot) => !snapshot.run_id.trim().is_empty()
            && !snapshot.snapshot_path.trim().is_empty()
            && snapshot.persisted_unix_ms > 0,
        None => !compaction_due,
    };
    let keep_latest_attempts_respected =
        latest_attempts_kept >= compaction_policy.keep_latest_attempts;
    let mut gaps = Vec::new();
    if compaction_due && !snapshot_present {
        gaps.push("snapshot is missing even though compaction threshold has been crossed".to_string());
    }
    if !persisted_after_threshold {
        gaps.push("persisted snapshot reference is incomplete or not durable".to_string());
    }
    if !keep_latest_attempts_respected {
        gaps.push("snapshot retention does not preserve the configured latest attempts".to_string());
    }
    if !rebuildable_from_journal {
        gaps.push("snapshot cannot be rebuilt from the append-only journal".to_string());
    }
    let report = SnapshotReport {
        snapshot_present,
        compaction_due,
        persisted_after_threshold,
        keep_latest_attempts_respected,
        rebuildable_from_journal,
        snapshot_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.snapshot_ready;
    (serde_json::to_value(report).expect("snapshot report"), ok)
}

fn index_payload(simulation: IndexSimulation) -> (serde_json::Value, bool) {
    let IndexSimulation {
        indexed_dimensions,
        run_count,
        owner_cardinality,
        tag_cardinality,
        partition_cardinality,
        failure_class_cardinality,
        p95_lookup_ms,
        max_lookup_ms,
    } = simulation;
    let required = ["owner", "tag", "partition", "failure_class"];
    let all_required_dimensions_indexed =
        required.iter().all(|dimension| indexed_dimensions.iter().any(|value| value == dimension));
    let high_cardinality_ready =
        owner_cardinality > 0 && tag_cardinality > 0 && partition_cardinality > 0 && failure_class_cardinality > 0;
    let lookup_within_limit = p95_lookup_ms <= max_lookup_ms;
    let mut gaps = Vec::new();
    if run_count == 0 {
        gaps.push("index audit requires non-zero history volume".to_string());
    }
    if !all_required_dimensions_indexed {
        gaps.push("required query dimensions are missing from the run index".to_string());
    }
    if !high_cardinality_ready {
        gaps.push("index does not cover the declared high-cardinality dimensions".to_string());
    }
    if !lookup_within_limit {
        gaps.push("lookup latency exceeds the declared index budget".to_string());
    }
    let report = IndexReport {
        dimension_count: indexed_dimensions.len(),
        all_required_dimensions_indexed,
        high_cardinality_ready,
        lookup_within_limit,
        index_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.index_ready;
    (serde_json::to_value(report).expect("index report"), ok)
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
        StateStoreCommands::Journal { simulation } => {
            let simulation: JournalSimulation = parse_json_file(simulation)?;
            let (payload, ok) = journal_payload(simulation);
            ("dag.state-store.journal", payload, ok)
        }
        StateStoreCommands::Snapshot { simulation } => {
            let simulation: SnapshotSimulation = parse_json_file(simulation)?;
            let (payload, ok) = snapshot_payload(simulation);
            ("dag.state-store.snapshot", payload, ok)
        }
        StateStoreCommands::Index { simulation } => {
            let simulation: IndexSimulation = parse_json_file(simulation)?;
            let (payload, ok) = index_payload(simulation);
            ("dag.state-store.index", payload, ok)
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
    use super::{
        index_payload, journal_payload, snapshot_payload, transaction_payload, IndexSimulation,
        JournalSimulation, NodeStateRecord, SnapshotSimulation, TransactionSimulation,
    };
    use bijux_dag_artifacts::NodeCounts;
    use bijux_dag_runtime::{
        EventCategory, EventRecord, NodeState, PersistedRunSnapshotRef, RunCompactionPolicy,
        RunState,
    };
    use serde_json::json;

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

    #[test]
    fn journal_accepts_append_only_required_event_sequence() {
        let simulation = JournalSimulation {
            events: vec![
                EventRecord {
                    category: EventCategory::Plan,
                    name: "run_started".to_string(),
                    unix_ms: 1,
                    node_id: None,
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Dispatch,
                    name: "node_ready".to_string(),
                    unix_ms: 2,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Start,
                    name: "node_started".to_string(),
                    unix_ms: 3,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Verify,
                    name: "node_attempt_started".to_string(),
                    unix_ms: 4,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Verify,
                    name: "node_attempt_finished".to_string(),
                    unix_ms: 5,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Schedule,
                    name: "node_scheduled".to_string(),
                    unix_ms: 6,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Failure,
                    name: "node_failed".to_string(),
                    unix_ms: 7,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Verify,
                    name: "run_finished".to_string(),
                    unix_ms: 8,
                    node_id: None,
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
            ],
            rewrite_detected: false,
        };
        let (payload, ok) = journal_payload(simulation);
        assert!(ok);
        assert_eq!(payload["journal_ready"], true);
    }

    #[test]
    fn journal_flags_missing_names_and_rewrite_behavior() {
        let simulation = JournalSimulation {
            events: vec![
                EventRecord {
                    category: EventCategory::Plan,
                    name: String::new(),
                    unix_ms: 2,
                    node_id: None,
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
                EventRecord {
                    category: EventCategory::Verify,
                    name: "run_finished".to_string(),
                    unix_ms: 1,
                    node_id: None,
                    run_id: Some("run-1".to_string()),
                    details: json!({}),
                },
            ],
            rewrite_detected: true,
        };
        let (payload, ok) = journal_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn snapshot_accepts_persisted_rebuildable_state_ref() {
        let simulation = SnapshotSimulation {
            snapshot: Some(PersistedRunSnapshotRef {
                run_id: "run-1".to_string(),
                snapshot_path: "snapshots/run-1.json".to_string(),
                persisted_unix_ms: 100,
            }),
            compaction_policy: RunCompactionPolicy {
                max_event_count_before_compaction: 10,
                keep_latest_attempts: 3,
            },
            event_count: 12,
            latest_attempts_kept: 3,
            rebuildable_from_journal: true,
        };
        let (payload, ok) = snapshot_payload(simulation);
        assert!(ok);
        assert_eq!(payload["snapshot_ready"], true);
    }

    #[test]
    fn snapshot_flags_missing_or_nonrebuildable_state_ref() {
        let simulation = SnapshotSimulation {
            snapshot: None,
            compaction_policy: RunCompactionPolicy {
                max_event_count_before_compaction: 10,
                keep_latest_attempts: 4,
            },
            event_count: 20,
            latest_attempts_kept: 2,
            rebuildable_from_journal: false,
        };
        let (payload, ok) = snapshot_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn index_accepts_covering_dimensions_with_bounded_lookup() {
        let simulation = IndexSimulation {
            indexed_dimensions: vec![
                "owner".to_string(),
                "tag".to_string(),
                "partition".to_string(),
                "failure_class".to_string(),
            ],
            run_count: 1_000_000,
            owner_cardinality: 100,
            tag_cardinality: 400,
            partition_cardinality: 10_000,
            failure_class_cardinality: 12,
            p95_lookup_ms: 80,
            max_lookup_ms: 100,
        };
        let (payload, ok) = index_payload(simulation);
        assert!(ok);
        assert_eq!(payload["index_ready"], true);
    }

    #[test]
    fn index_flags_missing_dimensions_or_slow_lookup() {
        let simulation = IndexSimulation {
            indexed_dimensions: vec!["owner".to_string()],
            run_count: 0,
            owner_cardinality: 10,
            tag_cardinality: 0,
            partition_cardinality: 0,
            failure_class_cardinality: 0,
            p95_lookup_ms: 250,
            max_lookup_ms: 100,
        };
        let (payload, ok) = index_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }
}
