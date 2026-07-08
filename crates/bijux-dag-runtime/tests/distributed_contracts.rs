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

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::simulated_platform::{
    artifact_upload_can_commit, cancellation_delivered_in_time, check_worker_version_compatibility,
    classify_heartbeat, classify_status_reporting, is_duplicate_dispatch, normalize_status_events,
    recover_lost_lease, reject_worker_version_mismatch, should_reassign,
    validate_task_lease_semantics, validate_worker_identity, verify_remote_artifact_integrity,
    worker_alive, worker_pool_satisfies_capability_request, DistributedExecutionRequest,
    DistributedReadinessChecklist, HeartbeatClass, HeartbeatSemantics, LivenessPolicy,
    MockRemoteBackend, RemoteArtifactCommitContract, RemoteArtifactUploadContract,
    RemoteStatusEvent, StatusReportingClass, TaskLeaseSemantics, WorkLease, WorkerCapabilities,
    WorkerHeartbeat, WorkerIdentity, WorkerPoolCapabilityRequest, WorkerVersionCompatibilityRule,
};
use bijux_dag_runtime::{
    AbsolutePathPolicy, NodeStatus, PolicyConfig, RemoteExecutionFingerprintSet,
    RemoteExecutionIdentity, RemoteExecutionRequest, RemoteExecutionWorkspace,
    RemoteExecutorSubmitter, RemoteNodeExecutionPayload,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn worker_liveness_and_reassignment_follow_contract() {
    let heartbeat = WorkerHeartbeat {
        worker_id: "worker-a".to_string(),
        unix_ms: 1000,
        inflight_nodes: vec!["n1".to_string()],
    };
    let policy = LivenessPolicy { heartbeat_timeout_ms: 500, grace_retries: 2 };
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
    let result =
        backend.submit_distributed(distributed).expect("distributed submission should succeed");
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
fn mock_backend_executes_remote_payloads_through_shared_worker_model() {
    let backend = MockRemoteBackend::default();
    let temp = tempfile::tempdir().expect("temp dir");
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "const-node",
              "kind": "const",
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"value": "hello"}
            }
          ],
          "edges": []
        }"#,
    )
    .expect("graph");
    let node = graph.nodes[0].clone();
    let payload = RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: "run-remote-worker".to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: "mock".to_string(),
        },
        graph,
        node,
        params: json!({"value": "hello"}),
        input_artifacts: Vec::new(),
        workspace: RemoteExecutionWorkspace {
            out_base: temp.path().display().to_string(),
            cache_dir: None,
        },
        policy: PolicyConfig::default(),
        absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        fingerprints: RemoteExecutionFingerprintSet {
            node_fingerprint: "node-fp".to_string(),
            node_definition_fingerprint: "node-def-fp".to_string(),
            declared_environment_fingerprint: "env-fp".to_string(),
            params_fingerprint: "params-fp".to_string(),
            command_fingerprint: Some("command-fp".to_string()),
            execution_fingerprint: "execution-fp".to_string(),
            evidence_fingerprint: "evidence-fp".to_string(),
            execution_contract_fingerprint: "execution-contract-fp".to_string(),
        },
    };

    let result = backend
        .execute_remote_payload(payload.clone())
        .expect("remote payload execution should succeed");
    let recorded = backend.payload_executions();
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].identity.run_id, payload.identity.run_id);
    assert_eq!(recorded[0].identity.node_id, payload.identity.node_id);
    assert_eq!(result.node_result.status, NodeStatus::Success);
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

#[test]
fn task_lease_and_heartbeat_semantics_are_typed_and_enforced() {
    let lease_semantics = TaskLeaseSemantics {
        lease_duration_ms: 30_000,
        renew_before_expiry_ms: 5_000,
        max_renewals: 5,
        recovery_grace_ms: 10_000,
    };
    assert!(validate_task_lease_semantics(&lease_semantics).is_ok());

    let bad = TaskLeaseSemantics {
        lease_duration_ms: 1_000,
        renew_before_expiry_ms: 1_000,
        max_renewals: 1,
        recovery_grace_ms: 100,
    };
    assert!(validate_task_lease_semantics(&bad).is_err());

    let heartbeat = WorkerHeartbeat {
        worker_id: "worker-a".to_string(),
        unix_ms: 10_000,
        inflight_nodes: vec!["n1".to_string()],
    };
    let heartbeat_semantics =
        HeartbeatSemantics { interval_ms: 500, timeout_ms: 2_000, delayed_threshold_ms: 1_000 };
    assert_eq!(
        classify_heartbeat(&heartbeat, 10_700, &heartbeat_semantics),
        HeartbeatClass::Healthy
    );
    assert_eq!(
        classify_heartbeat(&heartbeat, 11_500, &heartbeat_semantics),
        HeartbeatClass::Delayed
    );
    assert_eq!(classify_heartbeat(&heartbeat, 12_500, &heartbeat_semantics), HeartbeatClass::Lost);
}

#[test]
fn worker_identity_and_version_mismatch_are_validated() {
    let identity = WorkerIdentity {
        worker_id: "worker-b".to_string(),
        worker_version: "1.1.0".to_string(),
        backend_kind: "remote".to_string(),
        labels: BTreeMap::new(),
    };
    assert!(validate_worker_identity(&identity).is_ok());
    let invalid = WorkerIdentity { worker_id: String::new(), ..identity };
    assert!(validate_worker_identity(&invalid).is_err());

    let rule = WorkerVersionCompatibilityRule {
        planner_version: "1.2.0".to_string(),
        minimum_worker_version: "1.2.0".to_string(),
    };
    assert!(reject_worker_version_mismatch("1.2.1", &rule).is_ok());
    assert!(reject_worker_version_mismatch("1.1.9", &rule).is_err());
}

#[test]
fn duplicate_dispatch_and_lost_lease_recovery_contracts_hold() {
    let mut dispatched = BTreeSet::new();
    assert!(!is_duplicate_dispatch(&mut dispatched, "run-1", "node-1"));
    assert!(is_duplicate_dispatch(&mut dispatched, "run-1", "node-1"));

    let lease = WorkLease {
        lease_id: "lease-2".to_string(),
        run_id: "run-2".to_string(),
        node_id: "node-2".to_string(),
        worker_id: "worker-z".to_string(),
        expires_unix_ms: 1_000,
    };
    let semantics = TaskLeaseSemantics {
        lease_duration_ms: 2_000,
        renew_before_expiry_ms: 500,
        max_renewals: 2,
        recovery_grace_ms: 1_500,
    };
    assert!(recover_lost_lease(&lease, 2_000, &semantics));
    assert!(!recover_lost_lease(&lease, 2_700, &semantics));
}

#[test]
fn worker_crash_paths_and_network_partition_are_classified() {
    let upload = RemoteArtifactUploadContract {
        run_id: "run-3".to_string(),
        node_id: "node-3".to_string(),
        artifact_path: "out/model.bin".to_string(),
        target_store: "object://primary".to_string(),
        checksum: "abc123".to_string(),
    };
    let uncommitted = RemoteArtifactCommitContract {
        run_id: "run-3".to_string(),
        node_id: "node-3".to_string(),
        attempt: 1,
        upload_id: "u-1".to_string(),
        committed: false,
    };
    let committed = RemoteArtifactCommitContract { committed: true, ..uncommitted.clone() };

    assert!(
        !artifact_upload_can_commit(&upload, &uncommitted),
        "worker crash after upload but before commit must remain uncommitted"
    );
    assert!(artifact_upload_can_commit(&upload, &committed));

    assert!(verify_remote_artifact_integrity("abc123", "abc123"));
    assert!(!verify_remote_artifact_integrity("abc123", "def456"));

    assert_eq!(classify_status_reporting(1_000, 1_300, 500), StatusReportingClass::Healthy);
    assert_eq!(classify_status_reporting(1_000, 2_000, 500), StatusReportingClass::Partitioned);
}

#[test]
fn status_event_ordering_and_duplicate_ack_resilience_are_explicit() {
    let events = vec![
        RemoteStatusEvent {
            run_id: "run-4".to_string(),
            node_id: "node-4".to_string(),
            sequence: 2,
            status: "running".to_string(),
            unix_ms: 2_000,
        },
        RemoteStatusEvent {
            run_id: "run-4".to_string(),
            node_id: "node-4".to_string(),
            sequence: 1,
            status: "started".to_string(),
            unix_ms: 1_000,
        },
        RemoteStatusEvent {
            run_id: "run-4".to_string(),
            node_id: "node-4".to_string(),
            sequence: 2,
            status: "running".to_string(),
            unix_ms: 2_100,
        },
    ];
    let (ordered, duplicates) = normalize_status_events(&events);
    assert_eq!(ordered.len(), 2);
    assert_eq!(duplicates.len(), 1);
    assert_eq!(ordered[0].sequence, 1);
    assert_eq!(ordered[1].sequence, 2);
}

#[test]
fn cancellation_delivery_and_pool_capability_negotiation_are_checked() {
    assert!(cancellation_delivered_in_time(1_000, 1_200, 500));
    assert!(!cancellation_delivered_in_time(1_000, 1_800, 500));

    let caps = WorkerCapabilities {
        cpu_capacity: 16,
        memory_mb: 32_768,
        supports_gpu: true,
        supports_container: true,
        supports_sandbox_profiles: vec!["strict".to_string()],
    };
    let request = WorkerPoolCapabilityRequest {
        required_min_cpu_capacity: 8,
        required_min_memory_mb: 16_384,
        require_gpu: true,
        require_container_support: true,
        required_sandbox_profile: Some("strict".to_string()),
    };
    assert!(worker_pool_satisfies_capability_request(&caps, &request));

    let impossible = WorkerPoolCapabilityRequest {
        require_gpu: false,
        required_sandbox_profile: Some("unavailable".to_string()),
        ..request
    };
    assert!(!worker_pool_satisfies_capability_request(&caps, &impossible));
}
