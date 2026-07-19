use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_artifacts::NodeCounts;
use bijux_dag_runtime::{
    check_run_consistency, detect_stuck_run, evaluate_pause_state, should_quarantine_run,
    validate_and_repair_run_metadata, NodeState, RunId, RunPauseMode, RunPausePolicy, RunState,
    RunSummaryV2, StuckRunPolicy,
};

#[test]
fn pause_state_contract_freezes_ready_and_dispatch_in_full_pause_mode() {
    let policy =
        RunPausePolicy { mode: RunPauseMode::PauseAllNewDispatch, preserve_running_nodes: true };
    let state = evaluate_pause_state(&policy, 2, 3, 1);
    assert_eq!(state.get("freeze_dispatch"), Some(&true));
    assert_eq!(state.get("freeze_ready_queue"), Some(&true));
    assert_eq!(state.get("has_running"), Some(&true));
}

#[test]
fn stuck_detection_uses_progress_or_heartbeat_gap() {
    let policy = StuckRunPolicy { max_without_progress_ms: 10, max_without_heartbeat_ms: 20 };
    assert!(detect_stuck_run(100, 80, 90, &policy));
    assert!(!detect_stuck_run(100, 95, 90, &policy));
}

#[test]
fn repair_outcome_marks_repair_when_enabled() {
    let outcome = validate_and_repair_run_metadata(false, false, true);
    assert!(outcome.manifest_valid);
    assert!(outcome.index_valid);
    assert!(outcome.repaired_manifest);
    assert!(outcome.repaired_index);
}

#[test]
fn consistency_and_quarantine_contracts_detect_inconsistent_terminal_run() {
    let summary = RunSummaryV2 {
        run_id: RunId("run_01".to_string()),
        state: RunState::Failed,
        counts: NodeCounts { success: 2, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
    };
    let node_states =
        vec![("n1".to_string(), NodeState::Success), ("n2".to_string(), NodeState::Failed)];
    let artifacts = vec!["n1".to_string()];
    let consistency = check_run_consistency(&node_states, &artifacts, &summary);
    assert!(!consistency.summary_matches_node_states);
    let quarantine = should_quarantine_run(&RunState::Failed, &consistency);
    assert!(quarantine.is_some());
}
