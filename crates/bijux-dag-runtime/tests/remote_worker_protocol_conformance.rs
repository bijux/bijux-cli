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

use bijux_dag_runtime::simulated_platform::{
    artifact_upload_can_commit, classify_heartbeat, is_duplicate_dispatch, normalize_status_events,
    reject_worker_version_mismatch, worker_pool_satisfies_capability_request, HeartbeatClass,
    HeartbeatSemantics, RemoteArtifactCommitContract, RemoteArtifactUploadContract,
    RemoteStatusEvent, WorkerCapabilities, WorkerHeartbeat, WorkerPoolCapabilityRequest,
    WorkerVersionCompatibilityRule,
};
use std::collections::BTreeSet;

#[test]
fn conformance_heartbeat_classification_is_stable() {
    let policy =
        HeartbeatSemantics { interval_ms: 1_000, timeout_ms: 5_000, delayed_threshold_ms: 2_500 };
    let hb = WorkerHeartbeat {
        worker_id: "worker-x".to_string(),
        unix_ms: 10_000,
        inflight_nodes: vec!["n1".to_string()],
    };

    assert_eq!(classify_heartbeat(&hb, 11_000, &policy), HeartbeatClass::Healthy);
    assert_eq!(classify_heartbeat(&hb, 13_000, &policy), HeartbeatClass::Delayed);
    assert_eq!(classify_heartbeat(&hb, 16_000, &policy), HeartbeatClass::Lost);
}

#[test]
fn conformance_duplicate_dispatch_and_event_dedup_hold() {
    let mut seen = BTreeSet::new();
    assert!(!is_duplicate_dispatch(&mut seen, "run-c1", "node-a"));
    assert!(is_duplicate_dispatch(&mut seen, "run-c1", "node-a"));

    let events = vec![
        RemoteStatusEvent {
            run_id: "run-c1".to_string(),
            node_id: "node-a".to_string(),
            sequence: 2,
            status: "running".to_string(),
            unix_ms: 200,
        },
        RemoteStatusEvent {
            run_id: "run-c1".to_string(),
            node_id: "node-a".to_string(),
            sequence: 1,
            status: "started".to_string(),
            unix_ms: 100,
        },
        RemoteStatusEvent {
            run_id: "run-c1".to_string(),
            node_id: "node-a".to_string(),
            sequence: 2,
            status: "running".to_string(),
            unix_ms: 210,
        },
    ];
    let (ordered, duplicates) = normalize_status_events(&events);
    assert_eq!(ordered.len(), 2);
    assert_eq!(duplicates.len(), 1);
}

#[test]
fn conformance_upload_commit_version_gate_and_capability_negotiation_hold() {
    let upload = RemoteArtifactUploadContract {
        run_id: "run-c2".to_string(),
        node_id: "node-b".to_string(),
        artifact_path: "outputs/result.bin".to_string(),
        target_store: "object://store".to_string(),
        checksum: "abc".to_string(),
    };
    let committed = RemoteArtifactCommitContract {
        run_id: "run-c2".to_string(),
        node_id: "node-b".to_string(),
        attempt: 1,
        upload_id: "upload-1".to_string(),
        committed: true,
    };
    assert!(artifact_upload_can_commit(&upload, &committed));

    let rule = WorkerVersionCompatibilityRule {
        planner_version: "1.4.0".to_string(),
        minimum_worker_version: "1.4.0".to_string(),
    };
    assert!(reject_worker_version_mismatch("1.4.1", &rule).is_ok());
    assert!(reject_worker_version_mismatch("1.3.9", &rule).is_err());

    let caps = WorkerCapabilities {
        cpu_capacity: 8,
        memory_mb: 16_384,
        supports_gpu: false,
        supports_container: true,
        supports_sandbox_profiles: vec!["strict".to_string(), "balanced".to_string()],
    };
    let request = WorkerPoolCapabilityRequest {
        required_min_cpu_capacity: 4,
        required_min_memory_mb: 8_192,
        require_gpu: false,
        require_container_support: true,
        required_sandbox_profile: Some("strict".to_string()),
    };
    assert!(worker_pool_satisfies_capability_request(&caps, &request));
}
