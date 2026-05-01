use serde::{Deserialize, Serialize};

/// Docker smoke execution contract record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerSmokeExecutionRecordV1 {
    pub workflow_id: String,
    pub engine: String,
    pub image_reference: String,
    pub mount_count: usize,
    pub workdir_recorded: bool,
    pub user_recorded: bool,
    pub network_recorded: bool,
    pub stdout_recorded: bool,
    pub stderr_recorded: bool,
    pub artifacts_recorded: bool,
    pub declared_output_verified: bool,
}

/// Validate Docker smoke execution evidence when backend is available.
pub fn validate_docker_smoke_execution(
    record: &DockerSmokeExecutionRecordV1,
    engine_available: bool,
) -> Result<(), String> {
    if record.workflow_id.trim().is_empty() {
        return Err("docker smoke execution must include workflow_id".to_string());
    }
    if record.image_reference.trim().is_empty() {
        return Err("docker smoke execution must include image_reference".to_string());
    }
    if !engine_available {
        return Err(format!("docker smoke execution unavailable: engine '{}' is not ready", record.engine));
    }
    if record.mount_count == 0 {
        return Err("docker smoke execution requires at least one mount".to_string());
    }
    if !record.workdir_recorded
        || !record.user_recorded
        || !record.network_recorded
        || !record.stdout_recorded
        || !record.stderr_recorded
        || !record.artifacts_recorded
        || !record.declared_output_verified
    {
        return Err("docker smoke execution evidence is incomplete".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_docker_smoke_execution, DockerSmokeExecutionRecordV1};

    #[test]
    fn g141_docker_smoke_contract_requires_recorded_execution_evidence() {
        let record = DockerSmokeExecutionRecordV1 {
            workflow_id: "docker-smoke".to_string(),
            engine: "docker".to_string(),
            image_reference: "ghcr.io/bijux/smoke@sha256:abc123".to_string(),
            mount_count: 3,
            workdir_recorded: true,
            user_recorded: true,
            network_recorded: true,
            stdout_recorded: true,
            stderr_recorded: true,
            artifacts_recorded: true,
            declared_output_verified: true,
        };
        validate_docker_smoke_execution(&record, true).expect("docker smoke should validate");

        let mut incomplete = record;
        incomplete.artifacts_recorded = false;
        let error =
            validate_docker_smoke_execution(&incomplete, true).expect_err("must reject incomplete evidence");
        assert!(error.contains("evidence is incomplete"));
    }
}
