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

use bijux_dag_runtime::{
    cancel_batch_attempt, duplicate_status_delivery_detected, execution_mode_report,
    BatchAttemptState, BatchJobMetadata, BatchLifecycleEvent,
};

#[test]
fn fake_batch_state_handles_cancel_and_duplicate_status_detection() {
    let mut state = BatchAttemptState {
        metadata: BatchJobMetadata {
            scheduler_id: "scheduler-sim".to_string(),
            submission_time_unix_ms: 10,
            run_id: "run-1".to_string(),
            node_id: "node-a".to_string(),
            attempt_id: "1".to_string(),
            resource_request: "cpu=1".to_string(),
            status_mapping: "sim".to_string(),
        },
        events: vec![BatchLifecycleEvent {
            scheduler_id: "scheduler-sim".to_string(),
            status: "submitted".to_string(),
            unix_ms: 10,
        }],
        cancelled: false,
    };
    cancel_batch_attempt(&mut state);
    assert!(state.cancelled);
    assert!(state.events.iter().any(|e| e.status == "cancel-requested"));

    let events = vec![
        BatchLifecycleEvent {
            scheduler_id: "scheduler-sim".to_string(),
            status: "running".to_string(),
            unix_ms: 20,
        },
        BatchLifecycleEvent {
            scheduler_id: "scheduler-sim".to_string(),
            status: "running".to_string(),
            unix_ms: 20,
        },
    ];
    assert!(duplicate_status_delivery_detected(&events));
}

#[test]
fn mode_report_separates_implemented_simulated_and_aspirational() {
    let report = execution_mode_report();
    assert!(report.implemented.contains(&"local".to_string()));
    assert!(report.implemented.contains(&"container".to_string()));
    assert!(report.simulated.contains(&"fake-batch-backend".to_string()));
    assert!(report.simulated.contains(&"slurm-backend".to_string()));
    assert!(!report.simulated.contains(&"container-contract".to_string()));
    assert!(!report.aspirational.contains(&"slurm-backend".to_string()));
}
