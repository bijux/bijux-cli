use bijux_dag_runtime::{
    execution_mode_status, remote_handoff_valid, validate_remote_identity, ExecutionModeStatus,
    RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
};

#[test]
fn remote_identity_requires_run_node_attempt_backend_fields() {
    let identity = RemoteExecutionIdentity {
        run_id: "run-1".to_string(),
        node_id: "node-a".to_string(),
        attempt_id: "1".to_string(),
        backend_id: "remote-sim".to_string(),
    };
    assert!(validate_remote_identity(&identity).is_ok());

    let missing_backend = RemoteExecutionIdentity {
        backend_id: String::new(),
        ..identity
    };
    assert!(validate_remote_identity(&missing_backend).is_err());
}

#[test]
fn remote_handoff_requires_artifact_and_observability_fields() {
    let artifact = RemoteArtifactHandoff {
        upload_endpoint: "s3://bucket/upload".to_string(),
        download_endpoint: "s3://bucket/download".to_string(),
        integrity_required: true,
    };
    let observability = RemoteObservabilityHandoff {
        stream_mode: "line-buffered".to_string(),
        trace_forwarding: true,
        retention_days_hint: 14,
    };
    assert!(remote_handoff_valid(&artifact, &observability));
}

#[test]
fn execution_mode_status_is_explicit_for_container_and_kubernetes() {
    assert_eq!(execution_mode_status("local"), ExecutionModeStatus::Implemented);
    assert_eq!(
        execution_mode_status("container"),
        ExecutionModeStatus::Simulated
    );
    assert_eq!(
        execution_mode_status("kubernetes"),
        ExecutionModeStatus::NotImplemented
    );
    assert_eq!(execution_mode_status("hpc"), ExecutionModeStatus::NotImplemented);
}
