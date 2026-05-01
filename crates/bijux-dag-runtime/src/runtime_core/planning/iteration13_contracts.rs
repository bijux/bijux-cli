use crate::container_engine_discovery;
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

    let production_ready =
        probe.enabled && runtime_available && image_digest_verified && probe.image_reference.trim().len() > 0;
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
    let runtime_available = if probe.enabled {
        container_engine_discovery(&probe.engine).is_ok()
    } else {
        false
    };
    evaluate_container_capability_negotiation(probe, runtime_available)
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_container_capability_negotiation, ContainerCapabilityNegotiationReportV1,
        ContainerCapabilityProbeV1,
    };

    fn require_advisory(report: &ContainerCapabilityNegotiationReportV1) {
        assert_eq!(report.status, "advisory");
        assert!(!report.production_ready);
    }

    #[test]
    fn g124_container_capability_requires_backend_and_digest_before_production_ready() {
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
}
