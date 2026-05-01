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

/// Container image identity policy decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerImageIdentityDecisionV1 {
    pub image_reference: String,
    pub production_mode: bool,
    pub advisory_mode: bool,
    pub accepted: bool,
    pub reason: String,
}

/// Enforce strict image identity for production container runs.
pub fn enforce_container_image_identity(
    image_reference: &str,
    production_mode: bool,
    advisory_mode: bool,
) -> ContainerImageIdentityDecisionV1 {
    let has_digest = image_reference.contains("@sha256:");
    let accepted = if production_mode {
        has_digest || advisory_mode
    } else {
        true
    };
    let reason = if production_mode && !has_digest && !advisory_mode {
        "tag-only image reference is refused in production mode".to_string()
    } else if production_mode && !has_digest && advisory_mode {
        "tag-only image reference accepted in explicit advisory mode".to_string()
    } else {
        "image identity satisfies active policy".to_string()
    };
    ContainerImageIdentityDecisionV1 {
        image_reference: image_reference.to_string(),
        production_mode,
        advisory_mode,
        accepted,
        reason,
    }
}

/// Apptainer support boundary status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApptainerSupportStateV1 {
    Supported,
    Refused,
    Advisory,
}

/// Apptainer execution boundary report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApptainerBoundaryReportV1 {
    pub state: ApptainerSupportStateV1,
    pub engine: String,
    pub reason: String,
    pub smoke_behavior: String,
}

/// Evaluate explicit Apptainer/Singularity support boundary.
pub fn evaluate_apptainer_boundary(
    engine: &str,
    binary_available: bool,
    production_mode: bool,
) -> ApptainerBoundaryReportV1 {
    if !binary_available {
        return ApptainerBoundaryReportV1 {
            state: ApptainerSupportStateV1::Refused,
            engine: engine.to_string(),
            reason: "apptainer/singularity binary is unavailable".to_string(),
            smoke_behavior: "refused".to_string(),
        };
    }
    if production_mode {
        ApptainerBoundaryReportV1 {
            state: ApptainerSupportStateV1::Advisory,
            engine: engine.to_string(),
            reason: "apptainer backend remains advisory until full runtime parity is proven".to_string(),
            smoke_behavior: "advisory-smoke-only".to_string(),
        }
    } else {
        ApptainerBoundaryReportV1 {
            state: ApptainerSupportStateV1::Supported,
            engine: engine.to_string(),
            reason: "apptainer descriptor is accepted for non-production smoke execution".to_string(),
            smoke_behavior: "smoke-enabled".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_apptainer_boundary, ApptainerSupportStateV1,
        enforce_container_image_identity, validate_docker_smoke_execution,
        DockerSmokeExecutionRecordV1,
    };

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

    #[test]
    fn g142_container_image_identity_refuses_tag_only_in_production() {
        let refused = enforce_container_image_identity("ghcr.io/bijux/tool:latest", true, false);
        assert!(!refused.accepted);
        assert!(refused.reason.contains("tag-only image reference is refused"));

        let advisory = enforce_container_image_identity("ghcr.io/bijux/tool:latest", true, true);
        assert!(advisory.accepted);
        assert!(advisory.reason.contains("advisory mode"));

        let strict = enforce_container_image_identity("ghcr.io/bijux/tool@sha256:abc123", true, false);
        assert!(strict.accepted);
    }

    #[test]
    fn g143_apptainer_boundary_reports_explicit_support_or_refusal() {
        let refused = evaluate_apptainer_boundary("apptainer", false, false);
        assert_eq!(refused.state, ApptainerSupportStateV1::Refused);
        assert_eq!(refused.smoke_behavior, "refused");

        let advisory = evaluate_apptainer_boundary("apptainer", true, true);
        assert_eq!(advisory.state, ApptainerSupportStateV1::Advisory);
        assert!(advisory.reason.contains("advisory"));

        let supported = evaluate_apptainer_boundary("apptainer", true, false);
        assert_eq!(supported.state, ApptainerSupportStateV1::Supported);
    }
}
