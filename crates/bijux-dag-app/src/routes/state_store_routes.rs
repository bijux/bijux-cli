use crate::commands::{DagCli, StateStoreCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_artifacts::hash::sha256_hex;
use bijux_dag_artifacts::NodeCounts;
use bijux_dag_runtime::simulated_platform::{clock_within_assumption, SchedulerClockAssumption};
use bijux_dag_runtime::{
    build_cost_model, check_run_consistency, event_names_emitted_once, forecast_storage_growth,
    reconstruct_timeline_from_events, required_event_fields_present,
    validate_and_repair_run_metadata, validate_required_event_names,
    validate_required_timeline_labels, validate_storage_relative_path, EventRecord, NodeState,
    PersistedRunSnapshotRef, RunCompactionPolicy, RunId, RunState, RunSummaryV2,
    StorageHealthReport,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
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
    required_timeline_labels_present: bool,
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

#[derive(Debug, Deserialize)]
struct ArchiveSimulation {
    archived_run_count: usize,
    searchable_manifest_entries: usize,
    reconstructible_run_count: usize,
    daily_gb: f64,
    hot_store_gb: f64,
    cold_store_gb: f64,
    hot_cost_per_gb: f64,
    cold_cost_per_gb: f64,
}

#[derive(Debug, Serialize)]
struct ArchiveReport {
    monthly_gb: f64,
    annual_gb: f64,
    searchable_after_archive: bool,
    reconstructible_after_archive: bool,
    cold_storage_cheaper: bool,
    gaps: Vec<String>,
    archive_ready: bool,
}

#[derive(Debug, Serialize)]
struct ChecksumFileReport {
    relative_path: String,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct ChecksumReport {
    run_dir: String,
    health: StorageHealthReport,
    validated_paths: Vec<String>,
    files: Vec<ChecksumFileReport>,
    gaps: Vec<String>,
    checksum_ready: bool,
}

#[derive(Debug, Deserialize)]
struct AmplificationSimulation {
    logical_output_gb: f64,
    journal_gb: f64,
    snapshot_gb: f64,
    index_gb: f64,
    replicated_artifact_gb: f64,
    max_write_amplification_ratio: f64,
}

#[derive(Debug, Serialize)]
struct AmplificationReport {
    logical_output_gb: f64,
    persisted_gb: f64,
    write_amplification_ratio: f64,
    within_budget: bool,
    gaps: Vec<String>,
    amplification_ready: bool,
}

#[derive(Debug, Deserialize)]
struct RetentionSimulation {
    hot_partition_days: u64,
    archive_partition_days: u64,
    delete_partition_days: u64,
    overlap_detected: bool,
    unpartitioned_event_count: usize,
    oldest_hot_age_days: u64,
    oldest_archive_age_days: u64,
}

#[derive(Debug, Serialize)]
struct RetentionReport {
    tiers_strictly_ordered: bool,
    partitions_non_overlapping: bool,
    all_events_partitioned: bool,
    old_data_moved_out_of_hot_tier: bool,
    gaps: Vec<String>,
    retention_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ConsistencySimulation {
    manifest_exists: bool,
    index_exists: bool,
    repair_allowed: bool,
    object_store_lag_s: u64,
    search_index_lag_s: u64,
    max_allowed_lag_s: u64,
    manifest_run_id: String,
    index_run_id: String,
    lineage_run_id: String,
}

#[derive(Debug, Serialize)]
struct ConsistencyReport {
    metadata_valid: bool,
    repair_possible: bool,
    lag_within_budget: bool,
    run_identity_consistent: bool,
    gaps: Vec<String>,
    consistency_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ClockSimulation {
    planner_unix_ms: u128,
    scheduler_unix_ms: u128,
    reference_unix_ms: u128,
    persisted_unix_ms: u128,
    max_clock_skew_ms: u64,
    tick_grace_ms: u64,
    source_tags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ClockReport {
    planner_within_skew: bool,
    scheduler_within_skew: bool,
    persisted_after_reference: bool,
    source_tags_present: bool,
    gaps: Vec<String>,
    clock_ready: bool,
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
    let state_pairs = node_states
        .iter()
        .map(|record| (record.node_id.clone(), record.state.clone()))
        .collect::<Vec<_>>();
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
    let atomic_visible_state =
        (all_components_written && consistency.summary_matches_node_states) || rollback_recorded;
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
    let timeline = reconstruct_timeline_from_events(&events);
    let required_timeline_labels_present =
        validate_required_timeline_labels(&events, &timeline).is_empty();
    let append_only = !rewrite_detected;
    let monotonic_timestamps = events.windows(2).all(|pair| pair[0].unix_ms <= pair[1].unix_ms);
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
    if !required_timeline_labels_present {
        gaps.push("journal is missing required timeline lifecycle labels".to_string());
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
        required_timeline_labels_present,
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
        Some(snapshot) => {
            !snapshot.run_id.trim().is_empty()
                && !snapshot.snapshot_path.trim().is_empty()
                && snapshot.persisted_unix_ms > 0
        }
        None => !compaction_due,
    };
    let keep_latest_attempts_respected =
        latest_attempts_kept >= compaction_policy.keep_latest_attempts;
    let mut gaps = Vec::new();
    if compaction_due && !snapshot_present {
        gaps.push(
            "snapshot is missing even though compaction threshold has been crossed".to_string(),
        );
    }
    if !persisted_after_threshold {
        gaps.push("persisted snapshot reference is incomplete or not durable".to_string());
    }
    if !keep_latest_attempts_respected {
        gaps.push(
            "snapshot retention does not preserve the configured latest attempts".to_string(),
        );
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
    let high_cardinality_ready = owner_cardinality > 0
        && tag_cardinality > 0
        && partition_cardinality > 0
        && failure_class_cardinality > 0;
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

fn archive_payload(simulation: ArchiveSimulation) -> (serde_json::Value, bool) {
    let ArchiveSimulation {
        archived_run_count,
        searchable_manifest_entries,
        reconstructible_run_count,
        daily_gb,
        hot_store_gb,
        cold_store_gb,
        hot_cost_per_gb,
        cold_cost_per_gb,
    } = simulation;
    let growth = forecast_storage_growth(daily_gb);
    let costs =
        build_cost_model(hot_store_gb, cold_store_gb, 0.0, hot_cost_per_gb, cold_cost_per_gb, 0.0);
    let searchable_after_archive = searchable_manifest_entries >= archived_run_count;
    let reconstructible_after_archive = reconstructible_run_count >= archived_run_count;
    let cold_storage_cheaper = costs.object_store_monthly_cost < costs.local_store_monthly_cost;
    let mut gaps = Vec::new();
    if archived_run_count == 0 {
        gaps.push("archive audit requires archived runs to evaluate".to_string());
    }
    if !searchable_after_archive {
        gaps.push("archived runs are not fully searchable after export".to_string());
    }
    if !reconstructible_after_archive {
        gaps.push("archived runs are not reconstructible from persisted export state".to_string());
    }
    if !cold_storage_cheaper {
        gaps.push("cold storage does not improve monthly storage economics".to_string());
    }
    let report = ArchiveReport {
        monthly_gb: growth.monthly_gb,
        annual_gb: growth.annual_gb,
        searchable_after_archive,
        reconstructible_after_archive,
        cold_storage_cheaper,
        archive_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.archive_ready;
    (serde_json::to_value(report).expect("archive report"), ok)
}

fn checksum_payload(run_dir: &Path) -> (serde_json::Value, bool) {
    let required = ["manifest.json", "graph.snapshot.json", "outputs/index.json"];
    let mut files = Vec::new();
    let mut validated_paths = Vec::new();
    let mut anomalies = Vec::new();
    for relative_path in required {
        if validate_storage_relative_path(relative_path).is_ok() {
            validated_paths.push(relative_path.to_string());
        } else {
            anomalies.push(format!("invalid storage path contract: {relative_path}"));
            continue;
        }
        let path = run_dir.join(relative_path);
        match fs::read(&path) {
            Ok(bytes) => {
                files.push(ChecksumFileReport {
                    relative_path: relative_path.to_string(),
                    sha256: sha256_hex(&bytes),
                });
            }
            Err(_) => anomalies.push(format!("missing persisted file: {relative_path}")),
        }
    }
    let health = StorageHealthReport {
        run_dir: run_dir.display().to_string(),
        healthy: anomalies.is_empty(),
        anomalies: anomalies.clone(),
    };
    let mut gaps = Vec::new();
    if !health.healthy {
        gaps.extend(health.anomalies.iter().cloned());
    }
    if files.len() != required.len() {
        gaps.push("required persisted files are not all checksummed".to_string());
    }
    let report = ChecksumReport {
        run_dir: run_dir.display().to_string(),
        health,
        validated_paths,
        files,
        checksum_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.checksum_ready;
    (serde_json::to_value(report).expect("checksum report"), ok)
}

fn amplification_payload(simulation: AmplificationSimulation) -> (serde_json::Value, bool) {
    let AmplificationSimulation {
        logical_output_gb,
        journal_gb,
        snapshot_gb,
        index_gb,
        replicated_artifact_gb,
        max_write_amplification_ratio,
    } = simulation;
    let persisted_gb = journal_gb + snapshot_gb + index_gb + replicated_artifact_gb;
    let write_amplification_ratio =
        if logical_output_gb > 0.0 { persisted_gb / logical_output_gb } else { f64::INFINITY };
    let within_budget = write_amplification_ratio <= max_write_amplification_ratio;
    let mut gaps = Vec::new();
    if logical_output_gb <= 0.0 {
        gaps.push("amplification audit requires positive logical output volume".to_string());
    }
    if !within_budget {
        gaps.push("persisted bytes exceed the declared write amplification budget".to_string());
    }
    let report = AmplificationReport {
        logical_output_gb,
        persisted_gb,
        write_amplification_ratio,
        within_budget,
        amplification_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.amplification_ready;
    (serde_json::to_value(report).expect("amplification report"), ok)
}

fn retention_payload(simulation: RetentionSimulation) -> (serde_json::Value, bool) {
    let RetentionSimulation {
        hot_partition_days,
        archive_partition_days,
        delete_partition_days,
        overlap_detected,
        unpartitioned_event_count,
        oldest_hot_age_days,
        oldest_archive_age_days,
    } = simulation;
    let tiers_strictly_ordered = hot_partition_days < archive_partition_days
        && archive_partition_days < delete_partition_days;
    let partitions_non_overlapping = !overlap_detected;
    let all_events_partitioned = unpartitioned_event_count == 0;
    let old_data_moved_out_of_hot_tier = oldest_hot_age_days <= hot_partition_days
        && oldest_archive_age_days <= archive_partition_days;
    let mut gaps = Vec::new();
    if !tiers_strictly_ordered {
        gaps.push("retention tiers are not strictly ordered from hot to delete".to_string());
    }
    if !partitions_non_overlapping {
        gaps.push("retention partitions overlap and can double-own event history".to_string());
    }
    if !all_events_partitioned {
        gaps.push("some events are not assigned to a retention partition".to_string());
    }
    if !old_data_moved_out_of_hot_tier {
        gaps.push(
            "aged data remains in a hotter tier than the retention policy allows".to_string(),
        );
    }
    let report = RetentionReport {
        tiers_strictly_ordered,
        partitions_non_overlapping,
        all_events_partitioned,
        old_data_moved_out_of_hot_tier,
        retention_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.retention_ready;
    (serde_json::to_value(report).expect("retention report"), ok)
}

fn consistency_payload(simulation: ConsistencySimulation) -> (serde_json::Value, bool) {
    let ConsistencySimulation {
        manifest_exists,
        index_exists,
        repair_allowed,
        object_store_lag_s,
        search_index_lag_s,
        max_allowed_lag_s,
        manifest_run_id,
        index_run_id,
        lineage_run_id,
    } = simulation;
    let repair = validate_and_repair_run_metadata(manifest_exists, index_exists, repair_allowed);
    let metadata_valid = repair.manifest_valid && repair.index_valid;
    let lag_within_budget =
        object_store_lag_s <= max_allowed_lag_s && search_index_lag_s <= max_allowed_lag_s;
    let run_identity_consistent = !manifest_run_id.trim().is_empty()
        && manifest_run_id == index_run_id
        && manifest_run_id == lineage_run_id;
    let mut gaps = Vec::new();
    if !metadata_valid {
        gaps.push("manifest and index state are not durably consistent".to_string());
    }
    if !lag_within_budget {
        gaps.push("cross-store lag exceeds the declared reconciliation budget".to_string());
    }
    if !run_identity_consistent {
        gaps.push("manifest, index, and lineage stores disagree on run identity".to_string());
    }
    let report = ConsistencyReport {
        metadata_valid,
        repair_possible: repair.repaired_manifest || repair.repaired_index,
        lag_within_budget,
        run_identity_consistent,
        consistency_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.consistency_ready;
    (serde_json::to_value(report).expect("consistency report"), ok)
}

fn clock_payload(simulation: ClockSimulation) -> (serde_json::Value, bool) {
    let ClockSimulation {
        planner_unix_ms,
        scheduler_unix_ms,
        reference_unix_ms,
        persisted_unix_ms,
        max_clock_skew_ms,
        tick_grace_ms,
        source_tags,
    } = simulation;
    let assumption = SchedulerClockAssumption { max_clock_skew_ms, tick_grace_ms };
    let planner_within_skew =
        clock_within_assumption(planner_unix_ms, reference_unix_ms, &assumption);
    let scheduler_within_skew =
        clock_within_assumption(scheduler_unix_ms, reference_unix_ms, &assumption);
    let persisted_after_reference =
        persisted_unix_ms + assumption.tick_grace_ms as u128 >= reference_unix_ms;
    let source_tags_present = source_tags.iter().any(|tag| tag == "planner")
        && source_tags.iter().any(|tag| tag == "scheduler")
        && source_tags.iter().any(|tag| tag == "persistence");
    let mut gaps = Vec::new();
    if !planner_within_skew {
        gaps.push("planner clock exceeds the declared skew assumption".to_string());
    }
    if !scheduler_within_skew {
        gaps.push("scheduler clock exceeds the declared skew assumption".to_string());
    }
    if !persisted_after_reference {
        gaps.push(
            "persisted timestamps fall behind the authoritative schedule reference".to_string(),
        );
    }
    if !source_tags_present {
        gaps.push(
            "clock records are missing planner, scheduler, or persistence source tags".to_string(),
        );
    }
    let report = ClockReport {
        planner_within_skew,
        scheduler_within_skew,
        persisted_after_reference,
        source_tags_present,
        clock_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.clock_ready;
    (serde_json::to_value(report).expect("clock report"), ok)
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
        StateStoreCommands::Archive { simulation } => {
            let simulation: ArchiveSimulation = parse_json_file(simulation)?;
            let (payload, ok) = archive_payload(simulation);
            ("dag.state-store.archive", payload, ok)
        }
        StateStoreCommands::Checksum { run_dir } => {
            let (payload, ok) = checksum_payload(run_dir);
            ("dag.state-store.checksum", payload, ok)
        }
        StateStoreCommands::Amplification { simulation } => {
            let simulation: AmplificationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = amplification_payload(simulation);
            ("dag.state-store.amplification", payload, ok)
        }
        StateStoreCommands::Retention { simulation } => {
            let simulation: RetentionSimulation = parse_json_file(simulation)?;
            let (payload, ok) = retention_payload(simulation);
            ("dag.state-store.retention", payload, ok)
        }
        StateStoreCommands::Consistency { simulation } => {
            let simulation: ConsistencySimulation = parse_json_file(simulation)?;
            let (payload, ok) = consistency_payload(simulation);
            ("dag.state-store.consistency", payload, ok)
        }
        StateStoreCommands::Clock { simulation } => {
            let simulation: ClockSimulation = parse_json_file(simulation)?;
            let (payload, ok) = clock_payload(simulation);
            ("dag.state-store.clock", payload, ok)
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
        amplification_payload, archive_payload, checksum_payload, clock_payload,
        consistency_payload, index_payload, journal_payload, retention_payload, snapshot_payload,
        transaction_payload, AmplificationSimulation, ArchiveSimulation, ClockSimulation,
        ConsistencySimulation, IndexSimulation, JournalSimulation, NodeStateRecord,
        RetentionSimulation, SnapshotSimulation, TransactionSimulation,
    };
    use bijux_dag_artifacts::NodeCounts;
    use bijux_dag_artifacts::RunDir;
    use bijux_dag_runtime::{
        EventCategory, EventRecord, NodeState, PersistedRunSnapshotRef, RunCompactionPolicy,
        RunState,
    };
    use serde_json::json;
    use std::fs;

    #[test]
    fn transaction_accepts_consistent_atomic_visible_state() {
        let simulation = TransactionSimulation {
            run_id: "run-1".to_string(),
            run_state: RunState::Succeeded,
            counts: NodeCounts { success: 1, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
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
            counts: NodeCounts { success: 1, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
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
                    category: EventCategory::Verify,
                    name: "node_finished".to_string(),
                    unix_ms: 7,
                    node_id: Some("extract".to_string()),
                    run_id: Some("run-1".to_string()),
                    details: json!({ "status": "success" }),
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
        assert_eq!(payload["required_timeline_labels_present"], true);
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
        assert_eq!(payload["required_timeline_labels_present"], true);
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

    #[test]
    fn archive_accepts_searchable_reconstructible_cold_exports() {
        let simulation = ArchiveSimulation {
            archived_run_count: 100,
            searchable_manifest_entries: 100,
            reconstructible_run_count: 100,
            daily_gb: 50.0,
            hot_store_gb: 5_000.0,
            cold_store_gb: 5_000.0,
            hot_cost_per_gb: 0.08,
            cold_cost_per_gb: 0.02,
        };
        let (payload, ok) = archive_payload(simulation);
        assert!(ok);
        assert_eq!(payload["archive_ready"], true);
    }

    #[test]
    fn archive_flags_unsearchable_or_cost_negative_exports() {
        let simulation = ArchiveSimulation {
            archived_run_count: 100,
            searchable_manifest_entries: 50,
            reconstructible_run_count: 40,
            daily_gb: 50.0,
            hot_store_gb: 5_000.0,
            cold_store_gb: 5_000.0,
            hot_cost_per_gb: 0.02,
            cold_cost_per_gb: 0.03,
        };
        let (payload, ok) = archive_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn checksum_accepts_required_persisted_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let run_dir = RunDir::create(temp.path()).expect("run dir");
        fs::write(
            run_dir.staging_path().join("manifest.json"),
            br#"{"run_id":"run-1","status":"success"}"#,
        )
        .expect("manifest");
        run_dir.write_graph_snapshot("{\"nodes\":[]}").expect("graph snapshot");
        let outputs_index = run_dir.run_outputs_index_path();
        let outputs_dir = outputs_index.parent().expect("outputs parent");
        fs::create_dir_all(outputs_dir).expect("outputs dir");
        fs::write(outputs_index, br#"{"files":[]}"#).expect("outputs index");
        let (payload, ok) = checksum_payload(run_dir.staging_path());
        assert!(ok);
        assert_eq!(payload["checksum_ready"], true);
        assert_eq!(payload["files"].as_array().expect("files").len(), 3);
    }

    #[test]
    fn checksum_flags_missing_required_persisted_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let run_dir = RunDir::create(temp.path()).expect("run dir");
        let (payload, ok) = checksum_payload(run_dir.staging_path());
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 2);
    }

    #[test]
    fn amplification_accepts_bounded_persisted_write_ratio() {
        let simulation = AmplificationSimulation {
            logical_output_gb: 100.0,
            journal_gb: 10.0,
            snapshot_gb: 5.0,
            index_gb: 3.0,
            replicated_artifact_gb: 20.0,
            max_write_amplification_ratio: 0.5,
        };
        let (payload, ok) = amplification_payload(simulation);
        assert!(ok);
        assert_eq!(payload["amplification_ready"], true);
    }

    #[test]
    fn amplification_flags_overbudget_persisted_write_ratio() {
        let simulation = AmplificationSimulation {
            logical_output_gb: 10.0,
            journal_gb: 8.0,
            snapshot_gb: 4.0,
            index_gb: 3.0,
            replicated_artifact_gb: 6.0,
            max_write_amplification_ratio: 1.0,
        };
        let (payload, ok) = amplification_payload(simulation);
        assert!(!ok);
        assert!(!payload["gaps"].as_array().expect("gaps").is_empty());
    }

    #[test]
    fn retention_accepts_strict_non_overlapping_tiers() {
        let simulation = RetentionSimulation {
            hot_partition_days: 7,
            archive_partition_days: 30,
            delete_partition_days: 365,
            overlap_detected: false,
            unpartitioned_event_count: 0,
            oldest_hot_age_days: 7,
            oldest_archive_age_days: 30,
        };
        let (payload, ok) = retention_payload(simulation);
        assert!(ok);
        assert_eq!(payload["retention_ready"], true);
    }

    #[test]
    fn retention_flags_overlap_or_unpartitioned_history() {
        let simulation = RetentionSimulation {
            hot_partition_days: 30,
            archive_partition_days: 20,
            delete_partition_days: 10,
            overlap_detected: true,
            unpartitioned_event_count: 4,
            oldest_hot_age_days: 50,
            oldest_archive_age_days: 40,
        };
        let (payload, ok) = retention_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn consistency_accepts_reconciled_metadata_and_store_identity() {
        let simulation = ConsistencySimulation {
            manifest_exists: true,
            index_exists: true,
            repair_allowed: false,
            object_store_lag_s: 5,
            search_index_lag_s: 10,
            max_allowed_lag_s: 15,
            manifest_run_id: "run-1".to_string(),
            index_run_id: "run-1".to_string(),
            lineage_run_id: "run-1".to_string(),
        };
        let (payload, ok) = consistency_payload(simulation);
        assert!(ok);
        assert_eq!(payload["consistency_ready"], true);
    }

    #[test]
    fn consistency_flags_divergent_ids_or_excessive_lag() {
        let simulation = ConsistencySimulation {
            manifest_exists: false,
            index_exists: false,
            repair_allowed: false,
            object_store_lag_s: 50,
            search_index_lag_s: 75,
            max_allowed_lag_s: 20,
            manifest_run_id: "run-1".to_string(),
            index_run_id: "run-2".to_string(),
            lineage_run_id: String::new(),
        };
        let (payload, ok) = consistency_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn clock_accepts_bounded_skew_and_tagged_sources() {
        let simulation = ClockSimulation {
            planner_unix_ms: 1_000,
            scheduler_unix_ms: 1_010,
            reference_unix_ms: 1_005,
            persisted_unix_ms: 1_007,
            max_clock_skew_ms: 20,
            tick_grace_ms: 5,
            source_tags: vec![
                "planner".to_string(),
                "scheduler".to_string(),
                "persistence".to_string(),
            ],
        };
        let (payload, ok) = clock_payload(simulation);
        assert!(ok);
        assert_eq!(payload["clock_ready"], true);
    }

    #[test]
    fn clock_flags_skew_drift_and_missing_sources() {
        let simulation = ClockSimulation {
            planner_unix_ms: 1_000,
            scheduler_unix_ms: 1_100,
            reference_unix_ms: 1_010,
            persisted_unix_ms: 900,
            max_clock_skew_ms: 20,
            tick_grace_ms: 5,
            source_tags: vec!["planner".to_string()],
        };
        let (payload, ok) = clock_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }
}
