use crate::container_engine_discovery;
use bijux_dag_core::experimental::resource_capabilities::{
    PoolPlacementReportV1, ResourcePreflightReportV1,
};
use serde::{Deserialize, Serialize};

/// Runtime probe used for container capability negotiation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerCapabilityProbeV1 {
    pub enabled: bool,
    pub engine: String,
    pub image_reference: String,
    pub image_digest: Option<String>,
}

/// Container capability preflight report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerCapabilityNegotiationReportV1 {
    pub engine: String,
    pub runtime_available: bool,
    pub image_reference: String,
    pub image_digest_verified: bool,
    pub production_ready: bool,
    pub status: String,
    pub diagnostics: Vec<String>,
}

/// Capability maturity class for remote execution surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityMaturityV1 {
    Real,
    Simulated,
    Advisory,
}

/// Capability status row for remote execution planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCapabilityStatusV1 {
    pub capability: String,
    pub maturity: CapabilityMaturityV1,
    pub backend: String,
}

/// Remote capability honesty report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteCapabilityHonestyReportV1 {
    pub statuses: Vec<RemoteCapabilityStatusV1>,
    pub production_profile_ready: bool,
    pub diagnostics: Vec<String>,
}

/// Resource admission decision consumed by runtime queueing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceAdmissionDecisionV1 {
    pub admitted: bool,
    pub refusal_reasons: Vec<String>,
    pub refused_nodes: Vec<String>,
}

/// Evaluate container capability negotiation from a probe and discovered engine availability.
pub fn evaluate_container_capability_negotiation(
    probe: &ContainerCapabilityProbeV1,
    runtime_available: bool,
) -> ContainerCapabilityNegotiationReportV1 {
    let mut diagnostics = Vec::new();
    let image_digest_verified = probe
        .image_digest
        .as_deref()
        .is_some_and(|digest| digest.starts_with("sha256:") && digest.len() > "sha256:".len());

    if !probe.enabled {
        diagnostics.push("container backend is disabled by configuration".to_string());
    }
    if probe.image_reference.trim().is_empty() {
        diagnostics.push("container image reference must not be empty".to_string());
    }
    if probe.image_digest.is_none() {
        diagnostics.push("container image digest is missing".to_string());
    } else if !image_digest_verified {
        diagnostics.push("container image digest must be sha256-qualified".to_string());
    }
    if !runtime_available {
        diagnostics.push(format!("container engine '{}' is unavailable", probe.engine));
    }

    let production_ready = probe.enabled
        && runtime_available
        && image_digest_verified
        && !probe.image_reference.trim().is_empty();
    let status = if production_ready {
        "implemented".to_string()
    } else if probe.enabled {
        "advisory".to_string()
    } else {
        "disabled".to_string()
    };
    ContainerCapabilityNegotiationReportV1 {
        engine: probe.engine.clone(),
        runtime_available,
        image_reference: probe.image_reference.clone(),
        image_digest_verified,
        production_ready,
        status,
        diagnostics,
    }
}

/// Run real container capability preflight against local engine discovery.
pub fn preflight_container_capability(
    probe: &ContainerCapabilityProbeV1,
) -> ContainerCapabilityNegotiationReportV1 {
    let runtime_available =
        if probe.enabled { container_engine_discovery(&probe.engine).is_ok() } else { false };
    evaluate_container_capability_negotiation(probe, runtime_available)
}

/// Build remote capability status report with production-profile honesty rules.
pub fn build_remote_capability_honesty_report(
    statuses: Vec<RemoteCapabilityStatusV1>,
) -> RemoteCapabilityHonestyReportV1 {
    let mut diagnostics = Vec::new();
    for status in &statuses {
        if status.maturity != CapabilityMaturityV1::Real {
            diagnostics.push(format!(
                "capability '{}' on backend '{}' is {:?}",
                status.capability, status.backend, status.maturity
            ));
        }
    }
    let production_profile_ready = diagnostics.is_empty();
    RemoteCapabilityHonestyReportV1 { statuses, production_profile_ready, diagnostics }
}

/// Consume planner outputs and decide whether execution may be admitted.
pub fn admit_run_from_planner_outputs(
    resource_report: &ResourcePreflightReportV1,
    pool_report: &PoolPlacementReportV1,
    container_report: &ContainerCapabilityNegotiationReportV1,
    remote_report: &RemoteCapabilityHonestyReportV1,
) -> ResourceAdmissionDecisionV1 {
    let mut refusal_reasons = Vec::new();
    let mut refused_nodes = Vec::new();

    if !resource_report.admitted {
        refusal_reasons.push("resource preflight rejected one or more nodes".to_string());
        for refusal in &resource_report.refusals {
            refused_nodes.push(refusal.node_id.clone());
        }
    }
    if !pool_report.diagnostics.is_empty() {
        refusal_reasons.push("pool placement has unavailable pools".to_string());
        for placement in &pool_report.placements {
            if placement.assigned_pool.is_none() {
                refused_nodes.push(placement.node_id.clone());
            }
        }
    }
    if !container_report.production_ready && container_report.status != "disabled" {
        refusal_reasons.push("container capability is not production-ready".to_string());
    }
    if !remote_report.production_profile_ready {
        refusal_reasons.push("remote capability maturity is not production-ready".to_string());
    }

    refused_nodes.sort();
    refused_nodes.dedup();
    refusal_reasons.sort();
    refusal_reasons.dedup();

    ResourceAdmissionDecisionV1 {
        admitted: refusal_reasons.is_empty(),
        refusal_reasons,
        refused_nodes,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        admit_run_from_planner_outputs, build_remote_capability_honesty_report,
        evaluate_container_capability_negotiation, CapabilityMaturityV1,
        ContainerCapabilityNegotiationReportV1, ContainerCapabilityProbeV1,
        RemoteCapabilityStatusV1, ResourceAdmissionDecisionV1,
    };
    use bijux_dag_core::experimental::resource_capabilities::{
        ExecutionPoolV1, PoolPlacementDecisionV1, PoolPlacementReportV1,
        ResourcePreflightRefusalV1, ResourcePreflightReportV1, ResourceRequirementV1,
    };

    fn require_advisory(report: &ContainerCapabilityNegotiationReportV1) {
        assert_eq!(report.status, "advisory");
        assert!(!report.production_ready);
    }

    #[test]
    fn container_capability_requires_backend_and_digest_before_production_ready() {
        let report = evaluate_container_capability_negotiation(
            &ContainerCapabilityProbeV1 {
                enabled: true,
                engine: "docker".to_string(),
                image_reference: "ghcr.io/acme/workflow:1.0.0".to_string(),
                image_digest: Some("sha256:abc123".to_string()),
            },
            false,
        );
        require_advisory(&report);
        assert!(report
            .diagnostics
            .iter()
            .any(|entry| entry == "container engine 'docker' is unavailable"));
    }

    #[test]
    fn remote_capability_status_blocks_production_when_not_real() {
        let report = build_remote_capability_honesty_report(vec![
            RemoteCapabilityStatusV1 {
                capability: "remote".to_string(),
                maturity: CapabilityMaturityV1::Simulated,
                backend: "distributed".to_string(),
            },
            RemoteCapabilityStatusV1 {
                capability: "batch".to_string(),
                maturity: CapabilityMaturityV1::Advisory,
                backend: "slurm".to_string(),
            },
            RemoteCapabilityStatusV1 {
                capability: "high_availability".to_string(),
                maturity: CapabilityMaturityV1::Real,
                backend: "local-ha".to_string(),
            },
            RemoteCapabilityStatusV1 {
                capability: "federated".to_string(),
                maturity: CapabilityMaturityV1::Simulated,
                backend: "federated".to_string(),
            },
            RemoteCapabilityStatusV1 {
                capability: "distributed".to_string(),
                maturity: CapabilityMaturityV1::Advisory,
                backend: "distributed".to_string(),
            },
        ]);
        assert!(!report.production_profile_ready);
        assert!(report.diagnostics.iter().any(|entry| entry.contains("capability 'remote'")));
        assert!(report.diagnostics.iter().any(|entry| entry.contains("capability 'federated'")));
    }

    #[test]
    fn admission_refuses_oversized_or_unavailable_planner_outputs_before_execution() {
        let admission = admit_run_from_planner_outputs(
            &ResourcePreflightReportV1 {
                requirements: vec![ResourceRequirementV1 {
                    node_id: "align".to_string(),
                    cpu_cores: 128,
                    memory_mb: 524_288,
                    disk_mb: 10_240,
                    scratch_mb: 20_480,
                    network_required: true,
                    walltime_ms: 7_200_000,
                    accelerator: Some("gpu".to_string()),
                }],
                refusals: vec![ResourcePreflightRefusalV1 {
                    node_id: "align".to_string(),
                    code: "R121_MEMORY_UNAVAILABLE".to_string(),
                    message: "requested memory exceeds available".to_string(),
                }],
                admitted: false,
            },
            &PoolPlacementReportV1 {
                placements: vec![PoolPlacementDecisionV1 {
                    node_id: "align".to_string(),
                    requested_pool: ExecutionPoolV1::Gpu,
                    assigned_pool: None,
                    diagnostic: Some("node 'align' requested unavailable pool 'gpu'".to_string()),
                }],
                diagnostics: vec!["node 'align' requested unavailable pool 'gpu'".to_string()],
            },
            &ContainerCapabilityNegotiationReportV1 {
                engine: "docker".to_string(),
                runtime_available: false,
                image_reference: "ghcr.io/acme/workflow:1.0.0".to_string(),
                image_digest_verified: false,
                production_ready: false,
                status: "advisory".to_string(),
                diagnostics: vec!["container engine 'docker' is unavailable".to_string()],
            },
            &build_remote_capability_honesty_report(vec![RemoteCapabilityStatusV1 {
                capability: "distributed".to_string(),
                maturity: CapabilityMaturityV1::Simulated,
                backend: "distributed".to_string(),
            }]),
        );
        assert_eq!(
            admission,
            ResourceAdmissionDecisionV1 {
                admitted: false,
                refusal_reasons: vec![
                    "container capability is not production-ready".to_string(),
                    "pool placement has unavailable pools".to_string(),
                    "remote capability maturity is not production-ready".to_string(),
                    "resource preflight rejected one or more nodes".to_string(),
                ],
                refused_nodes: vec!["align".to_string()],
            }
        );
    }
}
