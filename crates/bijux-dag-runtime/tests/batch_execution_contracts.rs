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
    heartbeat_stale, restart_recovery_supported, retry_attempt, validate_batch_metadata,
    BatchHeartbeat, BatchJobMetadata,
};

#[test]
fn batch_metadata_requires_scheduler_and_attempt_identity() {
    let meta = BatchJobMetadata {
        scheduler_id: "slurm-a".to_string(),
        submission_time_unix_ms: 100,
        run_id: "run-1".to_string(),
        node_id: "node-a".to_string(),
        attempt_id: "1".to_string(),
        resource_request: "cpu=2,mem=4Gi".to_string(),
        status_mapping: "slurm-default".to_string(),
    };
    assert!(validate_batch_metadata(&meta).is_ok());
    assert!(
        validate_batch_metadata(&BatchJobMetadata { scheduler_id: String::new(), ..meta }).is_err()
    );
}

#[test]
fn retry_and_heartbeat_contracts_are_explicit() {
    let first = BatchJobMetadata {
        scheduler_id: "slurm-a".to_string(),
        submission_time_unix_ms: 100,
        run_id: "run-1".to_string(),
        node_id: "node-a".to_string(),
        attempt_id: "1".to_string(),
        resource_request: "cpu=2,mem=4Gi".to_string(),
        status_mapping: "slurm-default".to_string(),
    };
    let second = retry_attempt(&first, "2");
    assert_eq!(second.attempt_id, "2");
    assert!(second.submission_time_unix_ms > first.submission_time_unix_ms);

    let hb = BatchHeartbeat { scheduler_id: "slurm-a".to_string(), unix_ms: 1_000 };
    assert!(!heartbeat_stale(&hb, 1_250, 500));
    assert!(heartbeat_stale(&hb, 1_700, 500));
}

#[test]
fn restart_recovery_boundary_is_honest() {
    assert!(!restart_recovery_supported());
}
