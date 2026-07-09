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
        return Err(format!(
            "docker smoke execution unavailable: engine '{}' is not ready",
            record.engine
        ));
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
    let accepted = if production_mode { has_digest || advisory_mode } else { true };
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
            reason: "apptainer backend remains advisory until full runtime parity is proven"
                .to_string(),
            smoke_behavior: "advisory-smoke-only".to_string(),
        }
    } else {
        ApptainerBoundaryReportV1 {
            state: ApptainerSupportStateV1::Supported,
            engine: engine.to_string(),
            reason: "apptainer descriptor is accepted for non-production smoke execution"
                .to_string(),
            smoke_behavior: "smoke-enabled".to_string(),
        }
    }
}

/// Batch script export descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchScriptExportDescriptorV1 {
    pub scheduler: String,
    pub job_name: String,
    pub cpus: u32,
    pub memory_mb: u32,
    pub walltime: String,
    pub log_path: String,
    pub scratch_path: String,
    pub artifacts_path: String,
    pub cleanup_command: String,
}

/// Batch script export output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchScriptExportOutputV1 {
    pub script: String,
    pub submitted: bool,
}

/// Render scheduler export script without claiming execution submission.
pub fn render_batch_script_export(
    descriptor: &BatchScriptExportDescriptorV1,
) -> Result<BatchScriptExportOutputV1, String> {
    for required in [
        descriptor.scheduler.as_str(),
        descriptor.job_name.as_str(),
        descriptor.walltime.as_str(),
        descriptor.log_path.as_str(),
        descriptor.scratch_path.as_str(),
        descriptor.artifacts_path.as_str(),
        descriptor.cleanup_command.as_str(),
    ] {
        if required.trim().is_empty() {
            return Err("batch script export descriptor contains empty required fields".to_string());
        }
    }
    if descriptor.cpus == 0 || descriptor.memory_mb == 0 {
        return Err(
            "batch script export descriptor requires positive cpus and memory_mb".to_string()
        );
    }
    let script = format!(
        "#!/usr/bin/env bash\n# scheduler: {scheduler}\n# job_name: {job}\n# cpus: {cpus}\n# memory_mb: {memory}\n# walltime: {walltime}\n# logs: {logs}\n# scratch: {scratch}\n# artifacts: {artifacts}\nset -euo pipefail\nmkdir -p {scratch}\nmkdir -p {artifacts}\n# run workload command here\necho \"collecting artifacts\"\n{cleanup}\n",
        scheduler = descriptor.scheduler,
        job = descriptor.job_name,
        cpus = descriptor.cpus,
        memory = descriptor.memory_mb,
        walltime = descriptor.walltime,
        logs = descriptor.log_path,
        scratch = descriptor.scratch_path,
        artifacts = descriptor.artifacts_path,
        cleanup = descriptor.cleanup_command,
    );
    Ok(BatchScriptExportOutputV1 { script, submitted: false })
}

/// Mocked batch lifecycle event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockBatchLifecycleEventV1 {
    pub action: String,
    pub runtime_state: String,
}

/// Validate mocked batch lifecycle completeness and state mapping.
pub fn validate_mock_batch_lifecycle(events: &[MockBatchLifecycleEventV1]) -> Result<(), String> {
    if events.is_empty() {
        return Err("mock batch lifecycle requires at least one event".to_string());
    }
    let actions = events.iter().map(|event| event.action.as_str()).collect::<Vec<_>>();
    for required in ["submit", "poll", "cancel", "fail", "complete", "collect_logs"] {
        if !actions.iter().any(|action| action == &required) {
            return Err(format!("mock batch lifecycle is missing required action '{}'", required));
        }
    }
    for event in events {
        let expected = match event.action.as_str() {
            "submit" => "queued",
            "poll" => "running",
            "cancel" => "cancelled",
            "fail" => "failed",
            "complete" => "succeeded",
            "collect_logs" => "succeeded",
            _ => return Err(format!("unknown mock batch action '{}'", event.action)),
        };
        if event.runtime_state != expected {
            return Err(format!(
                "mock batch action '{}' must map to runtime state '{}', got '{}'",
                event.action, expected, event.runtime_state
            ));
        }
    }
    Ok(())
}

/// Batch backend promotion evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchBackendPromotionEvidenceV1 {
    pub export_only: bool,
    pub has_job_id: bool,
    pub has_state_polling: bool,
    pub has_exit_code: bool,
    pub has_artifact_collection: bool,
}

/// Batch backend promotion decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchBackendPromotionDecisionV1 {
    pub production_ready: bool,
    pub reason: String,
}

/// Evaluate whether batch backend can be promoted from export-only to real execution.
pub fn evaluate_batch_backend_promotion(
    evidence: &BatchBackendPromotionEvidenceV1,
) -> BatchBackendPromotionDecisionV1 {
    if evidence.export_only {
        return BatchBackendPromotionDecisionV1 {
            production_ready: false,
            reason: "export-only mode cannot be promoted to real batch execution".to_string(),
        };
    }
    let production_ready = evidence.has_job_id
        && evidence.has_state_polling
        && evidence.has_exit_code
        && evidence.has_artifact_collection;
    let reason = if production_ready {
        "batch backend promotion requirements are satisfied".to_string()
    } else {
        "batch backend promotion requires job_id, state polling, exit code, and artifact collection"
            .to_string()
    };
    BatchBackendPromotionDecisionV1 { production_ready, reason }
}

/// Remote worker protocol event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteWorkerProtocolEventV1 {
    pub event: String,
    pub worker_id: String,
}

/// Validate remote worker protocol conformance sequence.
pub fn validate_remote_worker_protocol_trace(
    events: &[RemoteWorkerProtocolEventV1],
) -> Result<(), String> {
    let expected =
        ["register", "lease", "heartbeat", "artifact_upload", "log_stream", "result_submit"];
    if events.len() < expected.len() {
        return Err("remote worker protocol trace is incomplete".to_string());
    }
    for (idx, step) in expected.iter().enumerate() {
        let event = events
            .get(idx)
            .ok_or_else(|| "remote worker protocol trace is incomplete".to_string())?;
        if event.worker_id.trim().is_empty() {
            return Err("remote worker protocol event must include worker_id".to_string());
        }
        if event.event != *step {
            return Err(format!(
                "remote worker protocol step {} must be '{}', got '{}'",
                idx, step, event.event
            ));
        }
    }
    Ok(())
}

/// External adapter SDK descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalAdapterSdkDescriptorV1 {
    pub adapter_id: String,
    pub capabilities: Vec<String>,
    pub effects: Vec<String>,
    pub outputs: Vec<String>,
    pub error_codes: Vec<String>,
}

/// Validate external adapter SDK descriptor for runtime conformance.
pub fn validate_external_adapter_sdk_descriptor(
    descriptor: &ExternalAdapterSdkDescriptorV1,
) -> Result<(), String> {
    if descriptor.adapter_id.trim().is_empty() {
        return Err("external adapter descriptor must include adapter_id".to_string());
    }
    if descriptor.capabilities.is_empty()
        || descriptor.effects.is_empty()
        || descriptor.outputs.is_empty()
        || descriptor.error_codes.is_empty()
    {
        return Err(
            "external adapter descriptor must declare capabilities, effects, outputs, and error codes"
                .to_string(),
        );
    }
    for collection in [
        &descriptor.capabilities,
        &descriptor.effects,
        &descriptor.outputs,
        &descriptor.error_codes,
    ] {
        if collection.iter().any(|value| value.trim().is_empty()) {
            return Err("external adapter descriptor contains empty contract fields".to_string());
        }
    }
    Ok(())
}

/// Executor fallback request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorFallbackRequestV1 {
    pub from_backend: String,
    pub to_backend: String,
    pub output_semantics_compatible: bool,
    pub evidence_obligations_compatible: bool,
}

/// Executor fallback decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutorFallbackDecisionV1 {
    pub allowed: bool,
    pub reason: String,
}

/// Evaluate safe fallback policy across execution backends.
pub fn evaluate_executor_fallback(
    request: &ExecutorFallbackRequestV1,
) -> ExecutorFallbackDecisionV1 {
    if !request.output_semantics_compatible {
        return ExecutorFallbackDecisionV1 {
            allowed: false,
            reason: format!(
                "unsafe fallback refused: output semantics differ between '{}' and '{}'",
                request.from_backend, request.to_backend
            ),
        };
    }
    if !request.evidence_obligations_compatible {
        return ExecutorFallbackDecisionV1 {
            allowed: false,
            reason: format!(
                "unsafe fallback refused: evidence obligations differ between '{}' and '{}'",
                request.from_backend, request.to_backend
            ),
        };
    }
    ExecutorFallbackDecisionV1 {
        allowed: true,
        reason: "fallback is safe under output and evidence compatibility policy".to_string(),
    }
}

/// Backend execution fingerprint used for comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendExecutionFingerprintV1 {
    pub backend: String,
    pub env_fingerprint: String,
    pub artifact_fingerprint: String,
    pub runtime_fingerprint: String,
}

/// Backend comparison report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendComparisonReportV1 {
    pub differing_env_backends: Vec<String>,
    pub differing_artifact_backends: Vec<String>,
    pub differing_runtime_backends: Vec<String>,
}

/// Compare execution behavior across backends and expose differences.
pub fn build_backend_comparison_report(
    fingerprints: &[BackendExecutionFingerprintV1],
) -> BackendComparisonReportV1 {
    let baseline = fingerprints.first();
    let mut differing_env_backends = Vec::new();
    let mut differing_artifact_backends = Vec::new();
    let mut differing_runtime_backends = Vec::new();

    if let Some(base) = baseline {
        for entry in fingerprints.iter().skip(1) {
            if entry.env_fingerprint != base.env_fingerprint {
                differing_env_backends.push(entry.backend.clone());
            }
            if entry.artifact_fingerprint != base.artifact_fingerprint {
                differing_artifact_backends.push(entry.backend.clone());
            }
            if entry.runtime_fingerprint != base.runtime_fingerprint {
                differing_runtime_backends.push(entry.backend.clone());
            }
        }
    }

    BackendComparisonReportV1 {
        differing_env_backends,
        differing_artifact_backends,
        differing_runtime_backends,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_backend_comparison_report, enforce_container_image_identity,
        evaluate_apptainer_boundary, evaluate_batch_backend_promotion, evaluate_executor_fallback,
        render_batch_script_export, validate_docker_smoke_execution,
        validate_external_adapter_sdk_descriptor, validate_mock_batch_lifecycle,
        validate_remote_worker_protocol_trace, ApptainerSupportStateV1,
        BackendExecutionFingerprintV1, BatchBackendPromotionEvidenceV1,
        BatchScriptExportDescriptorV1, DockerSmokeExecutionRecordV1, ExecutorFallbackRequestV1,
        ExternalAdapterSdkDescriptorV1, MockBatchLifecycleEventV1, RemoteWorkerProtocolEventV1,
    };

    #[test]
    fn docker_smoke_contract_requires_recorded_execution_evidence() {
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
        let error = validate_docker_smoke_execution(&incomplete, true)
            .expect_err("must reject incomplete evidence");
        assert!(error.contains("evidence is incomplete"));
    }

    #[test]
    fn container_image_identity_refuses_tag_only_in_production() {
        let refused = enforce_container_image_identity("ghcr.io/bijux/tool:latest", true, false);
        assert!(!refused.accepted);
        assert!(refused.reason.contains("tag-only image reference is refused"));

        let advisory = enforce_container_image_identity("ghcr.io/bijux/tool:latest", true, true);
        assert!(advisory.accepted);
        assert!(advisory.reason.contains("advisory mode"));

        let strict =
            enforce_container_image_identity("ghcr.io/bijux/tool@sha256:abc123", true, false);
        assert!(strict.accepted);
    }

    #[test]
    fn apptainer_boundary_reports_explicit_support_or_refusal() {
        let refused = evaluate_apptainer_boundary("apptainer", false, false);
        assert_eq!(refused.state, ApptainerSupportStateV1::Refused);
        assert_eq!(refused.smoke_behavior, "refused");

        let advisory = evaluate_apptainer_boundary("apptainer", true, true);
        assert_eq!(advisory.state, ApptainerSupportStateV1::Advisory);
        assert!(advisory.reason.contains("advisory"));

        let supported = evaluate_apptainer_boundary("apptainer", true, false);
        assert_eq!(supported.state, ApptainerSupportStateV1::Supported);
    }

    #[test]
    fn batch_script_export_includes_resources_logs_scratch_artifacts_and_cleanup() {
        let output = render_batch_script_export(&BatchScriptExportDescriptorV1 {
            scheduler: "slurm".to_string(),
            job_name: "align-smoke".to_string(),
            cpus: 8,
            memory_mb: 16384,
            walltime: "02:00:00".to_string(),
            log_path: "/tmp/run/logs".to_string(),
            scratch_path: "/tmp/run/scratch".to_string(),
            artifacts_path: "/tmp/run/artifacts".to_string(),
            cleanup_command: "rm -rf /tmp/run/scratch".to_string(),
        })
        .expect("batch script export");
        assert!(!output.submitted);
        assert!(output.script.contains("cpus: 8"));
        assert!(output.script.contains("memory_mb: 16384"));
        assert!(output.script.contains("scratch: /tmp/run/scratch"));
        assert!(output.script.contains("artifacts: /tmp/run/artifacts"));
        assert!(output.script.contains("rm -rf /tmp/run/scratch"));
    }

    #[test]
    fn mock_batch_lifecycle_covers_submit_poll_cancel_fail_complete_and_logs() {
        let events = vec![
            MockBatchLifecycleEventV1 {
                action: "submit".to_string(),
                runtime_state: "queued".to_string(),
            },
            MockBatchLifecycleEventV1 {
                action: "poll".to_string(),
                runtime_state: "running".to_string(),
            },
            MockBatchLifecycleEventV1 {
                action: "cancel".to_string(),
                runtime_state: "cancelled".to_string(),
            },
            MockBatchLifecycleEventV1 {
                action: "fail".to_string(),
                runtime_state: "failed".to_string(),
            },
            MockBatchLifecycleEventV1 {
                action: "complete".to_string(),
                runtime_state: "succeeded".to_string(),
            },
            MockBatchLifecycleEventV1 {
                action: "collect_logs".to_string(),
                runtime_state: "succeeded".to_string(),
            },
        ];
        validate_mock_batch_lifecycle(&events).expect("mock batch lifecycle should validate");
    }

    #[test]
    fn batch_backend_promotion_refuses_export_only_and_requires_execution_evidence() {
        let export_only = evaluate_batch_backend_promotion(&BatchBackendPromotionEvidenceV1 {
            export_only: true,
            has_job_id: false,
            has_state_polling: false,
            has_exit_code: false,
            has_artifact_collection: false,
        });
        assert!(!export_only.production_ready);
        assert!(export_only.reason.contains("export-only"));

        let promoted = evaluate_batch_backend_promotion(&BatchBackendPromotionEvidenceV1 {
            export_only: false,
            has_job_id: true,
            has_state_polling: true,
            has_exit_code: true,
            has_artifact_collection: true,
        });
        assert!(promoted.production_ready);
    }

    #[test]
    fn remote_worker_protocol_trace_requires_concrete_lifecycle_order() {
        let events = vec![
            RemoteWorkerProtocolEventV1 {
                event: "register".to_string(),
                worker_id: "w1".to_string(),
            },
            RemoteWorkerProtocolEventV1 { event: "lease".to_string(), worker_id: "w1".to_string() },
            RemoteWorkerProtocolEventV1 {
                event: "heartbeat".to_string(),
                worker_id: "w1".to_string(),
            },
            RemoteWorkerProtocolEventV1 {
                event: "artifact_upload".to_string(),
                worker_id: "w1".to_string(),
            },
            RemoteWorkerProtocolEventV1 {
                event: "log_stream".to_string(),
                worker_id: "w1".to_string(),
            },
            RemoteWorkerProtocolEventV1 {
                event: "result_submit".to_string(),
                worker_id: "w1".to_string(),
            },
        ];
        validate_remote_worker_protocol_trace(&events).expect("remote worker protocol trace");
    }

    #[test]
    fn external_adapter_sdk_descriptor_requires_capabilities_effects_outputs_and_errors() {
        let descriptor = ExternalAdapterSdkDescriptorV1 {
            adapter_id: "ext.aligner".to_string(),
            capabilities: vec!["streaming".to_string(), "typed-output".to_string()],
            effects: vec!["filesystem".to_string()],
            outputs: vec!["bam".to_string()],
            error_codes: vec!["EXT_TIMEOUT".to_string(), "EXT_SCHEMA".to_string()],
        };
        validate_external_adapter_sdk_descriptor(&descriptor).expect("external adapter descriptor");
    }

    #[test]
    fn executor_fallback_refuses_incompatible_semantics_or_evidence() {
        let semantics_fail = evaluate_executor_fallback(&ExecutorFallbackRequestV1 {
            from_backend: "container".to_string(),
            to_backend: "shell".to_string(),
            output_semantics_compatible: false,
            evidence_obligations_compatible: true,
        });
        assert!(!semantics_fail.allowed);
        assert!(semantics_fail.reason.contains("output semantics differ"));

        let evidence_fail = evaluate_executor_fallback(&ExecutorFallbackRequestV1 {
            from_backend: "batch".to_string(),
            to_backend: "local".to_string(),
            output_semantics_compatible: true,
            evidence_obligations_compatible: false,
        });
        assert!(!evidence_fail.allowed);
        assert!(evidence_fail.reason.contains("evidence obligations differ"));
    }

    #[test]
    fn backend_comparison_report_makes_env_artifact_runtime_differences_visible() {
        let report = build_backend_comparison_report(&[
            BackendExecutionFingerprintV1 {
                backend: "local".to_string(),
                env_fingerprint: "env-a".to_string(),
                artifact_fingerprint: "artifact-a".to_string(),
                runtime_fingerprint: "rt-a".to_string(),
            },
            BackendExecutionFingerprintV1 {
                backend: "shell".to_string(),
                env_fingerprint: "env-b".to_string(),
                artifact_fingerprint: "artifact-a".to_string(),
                runtime_fingerprint: "rt-a".to_string(),
            },
            BackendExecutionFingerprintV1 {
                backend: "container".to_string(),
                env_fingerprint: "env-a".to_string(),
                artifact_fingerprint: "artifact-b".to_string(),
                runtime_fingerprint: "rt-b".to_string(),
            },
        ]);
        assert_eq!(report.differing_env_backends, vec!["shell".to_string()]);
        assert_eq!(report.differing_artifact_backends, vec!["container".to_string()]);
        assert_eq!(report.differing_runtime_backends, vec!["container".to_string()]);
    }
}
