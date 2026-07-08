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
    negotiate_backend_capabilities, BackendCapabilityRequirement, BackendExecutionCompletion,
    BackendExecutionRequest, ExecutorBackend, InfrastructureBackendCapabilities,
};
use std::collections::BTreeMap;

#[test]
fn rejects_backend_when_required_capability_is_missing() {
    let capabilities = InfrastructureBackendCapabilities {
        supports_container: false,
        supports_network_isolation: true,
        supports_env_allowlist: true,
        supports_artifact_mounts: true,
        supports_remote_logs: true,
        supports_gpu: false,
    };
    let requirements = BackendCapabilityRequirement {
        container_required: true,
        network_isolation_required: true,
        env_allowlist_required: false,
        artifact_mount_required: false,
        remote_logs_required: false,
        gpu_required: false,
    };
    let decision = negotiate_backend_capabilities(&capabilities, &requirements);
    assert!(!decision.accepted);
    assert!(decision.reason.contains("container execution"));
}

#[test]
fn backend_request_and_completion_have_stable_serialization_shape() {
    let request = BackendExecutionRequest {
        backend: ExecutorBackend::Kubernetes,
        run_id: "run-20260306".to_string(),
        node_id: "train-model".to_string(),
        command: vec!["python".to_string(), "train.py".to_string()],
        environment: BTreeMap::from([("MODEL".to_string(), "v1".to_string())]),
    };
    let completion = BackendExecutionCompletion {
        backend: ExecutorBackend::Kubernetes,
        run_id: "run-20260306".to_string(),
        node_id: "train-model".to_string(),
        status: "success".to_string(),
        exit_code: Some(0),
        diagnostics: vec![],
    };
    let request_json = serde_json::to_value(&request).expect("request should serialize");
    let completion_json = serde_json::to_value(&completion).expect("completion should serialize");
    assert_eq!(request_json.get("run_id").and_then(|v| v.as_str()), Some("run-20260306"));
    assert_eq!(completion_json.get("status").and_then(|v| v.as_str()), Some("success"));
}
