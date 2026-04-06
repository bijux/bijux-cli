use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteExecutionIdentity {
    pub run_id: String,
    pub node_id: String,
    pub attempt_id: String,
    pub backend_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteArtifactHandoff {
    pub upload_endpoint: String,
    pub download_endpoint: String,
    pub integrity_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteObservabilityHandoff {
    pub stream_mode: String,
    pub trace_forwarding: bool,
    pub retention_days_hint: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionModeStatus {
    Implemented,
    Simulated,
    NotImplemented,
}

pub fn validate_remote_identity(identity: &RemoteExecutionIdentity) -> Result<(), String> {
    for value in [
        &identity.run_id,
        &identity.node_id,
        &identity.attempt_id,
        &identity.backend_id,
    ] {
        if value.trim().is_empty() {
            return Err("remote identity fields must be non-empty".to_string());
        }
    }
    Ok(())
}

pub fn execution_mode_status(mode: &str) -> ExecutionModeStatus {
    match mode {
        "local" | "subprocess" => ExecutionModeStatus::Implemented,
        "container" | "remote-contract" | "kubernetes-contract" => ExecutionModeStatus::Simulated,
        "kubernetes" | "hpc" => ExecutionModeStatus::NotImplemented,
        _ => ExecutionModeStatus::NotImplemented,
    }
}

pub fn remote_handoff_valid(
    artifact: &RemoteArtifactHandoff,
    observability: &RemoteObservabilityHandoff,
) -> bool {
    !artifact.upload_endpoint.trim().is_empty()
        && !artifact.download_endpoint.trim().is_empty()
        && !observability.stream_mode.trim().is_empty()
}
