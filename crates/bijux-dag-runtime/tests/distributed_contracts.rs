use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    check_worker_version_compatibility, should_reassign, worker_alive, DistributedExecutionRequest,
    DistributedReadinessChecklist, LivenessPolicy, MockRemoteBackend, RemoteExecutionRequest,
    RemoteExecutorSubmitter, WorkerHeartbeat, WorkerVersionCompatibilityRule, WorkLease,
};
use std::collections::BTreeMap;

#[test]
fn worker_liveness_and_reassignment_follow_contract() {
    let heartbeat = WorkerHeartbeat {
        worker_id: "worker-a".to_string(),
        unix_ms: 1000,
        inflight_nodes: vec!["n1".to_string()],
    };
    let policy = LivenessPolicy {
        heartbeat_timeout_ms: 500,
        grace_retries: 2,
    };
    assert!(worker_alive(&heartbeat, 1300, &policy));
    assert!(!worker_alive(&heartbeat, 2000, &policy));

    let lease = WorkLease {
        lease_id: "lease-1".to_string(),
        run_id: "run-1".to_string(),
        node_id: "n1".to_string(),
        worker_id: "worker-a".to_string(),
        expires_unix_ms: 1200,
    };
    assert!(!should_reassign(&lease, 1100));
    assert!(should_reassign(&lease, 1300));
}

#[test]
fn mock_backend_accepts_typed_submissions() {
    let backend = MockRemoteBackend::default();
    let distributed = DistributedExecutionRequest {
        run_id: "run-7".to_string(),
        node_id: "n7".to_string(),
        worker_pool: "default".to_string(),
        backend_hint: "mock".to_string(),
        command: vec!["echo".to_string(), "ok".to_string()],
        env: BTreeMap::new(),
        attempt: 1,
    };
    let result = backend
        .submit_distributed(distributed)
        .expect("distributed submission should succeed");
    assert_eq!(result.status, "accepted");

    let legacy = backend
        .submit(RemoteExecutionRequest {
            run_id: "run-8".to_string(),
            node_id: "n8".to_string(),
            contract_digest: "abc".to_string(),
        })
        .expect("legacy submission should succeed");
    assert!(legacy.accepted);
}

#[test]
fn worker_version_and_readiness_checklist_are_explicit() {
    let rule = WorkerVersionCompatibilityRule {
        planner_version: "1.0.0".to_string(),
        minimum_worker_version: "1.0.0".to_string(),
    };
    assert!(check_worker_version_compatibility("1.0.1", &rule));
    let readiness = DistributedReadinessChecklist {
        typed_transport_contracts: true,
        worker_liveness_contracts: true,
        retry_lineage_contracts: true,
        security_model_documented: true,
        conformance_fixtures_present: true,
    };
    assert!(readiness.typed_transport_contracts && readiness.security_model_documented);
}
