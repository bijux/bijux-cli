use crate::commands::{DagCli, IncidentCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    health_dashboard_score, IncidentClassification, IncidentSeverity, PlatformHealthDashboard,
    RunbookEntry, SupportabilityModel,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct IncidentModeSimulation {
    classification: IncidentClassification,
    dashboard: PlatformHealthDashboard,
    #[serde(default)]
    disabled_actions: Vec<String>,
    #[serde(default)]
    elevated_surfaces: Vec<String>,
    #[serde(default)]
    retained_actions: Vec<String>,
    #[serde(default)]
    runbooks: Vec<RunbookEntry>,
    supportability: SupportabilityModel,
}

#[derive(Debug, Serialize)]
struct IncidentModeReport {
    incident_type: String,
    severity: String,
    dashboard_score: f64,
    disabled_actions: Vec<String>,
    elevated_surfaces: Vec<String>,
    retained_actions: Vec<String>,
    required_evidence: Vec<String>,
    supported_backends: Vec<String>,
    official_plugins: Vec<String>,
    gaps: Vec<String>,
    incident_mode_ready: bool,
}

#[derive(Debug, Deserialize)]
struct BlastRadiusSimulation {
    classification: IncidentClassification,
    failing_components: Vec<String>,
    #[serde(default)]
    workflows: Vec<WorkflowImpact>,
    #[serde(default)]
    service_context: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkflowImpact {
    workflow_id: String,
    tenant_id: String,
    #[serde(default)]
    artifact_ids: Vec<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BlastRadiusReport {
    incident_type: String,
    severity: String,
    failing_components: Vec<String>,
    impacted_workflows: Vec<String>,
    impacted_tenants: Vec<String>,
    impacted_artifacts: Vec<String>,
    impacted_services: BTreeMap<String, Vec<String>>,
    gaps: Vec<String>,
    blast_radius_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn severity_name(severity: &IncidentSeverity) -> &'static str {
    match severity {
        IncidentSeverity::Critical => "critical",
        IncidentSeverity::High => "high",
        IncidentSeverity::Medium => "medium",
        IncidentSeverity::Low => "low",
    }
}

fn incident_mode_payload(simulation: IncidentModeSimulation) -> (serde_json::Value, bool) {
    let IncidentModeSimulation {
        classification,
        dashboard,
        disabled_actions,
        elevated_surfaces,
        retained_actions,
        runbooks,
        supportability,
    } = simulation;
    let dashboard_score = health_dashboard_score(&dashboard);
    let mut gaps = Vec::new();
    if disabled_actions.is_empty() {
        gaps.push("incident mode should suppress at least one nonessential action".to_string());
    }
    if elevated_surfaces.is_empty() {
        gaps.push("incident mode should elevate critical visibility surfaces".to_string());
    }
    if retained_actions.is_empty() {
        gaps.push("incident mode should preserve a minimal safe operator action set".to_string());
    }
    if runbooks.is_empty() {
        gaps.push("incident mode should point to at least one runbook".to_string());
    }
    if supportability.supported_backends.is_empty() {
        gaps.push("incident mode should name supported execution backends".to_string());
    }
    if dashboard_score < 0.60 {
        gaps.push("dashboard health is too weak for confident incident operation".to_string());
    }
    let required_evidence = runbooks
        .iter()
        .flat_map(|entry| entry.required_evidence.iter().cloned())
        .collect::<Vec<_>>();
    let supported_backends = supportability.supported_backends.into_iter().collect::<Vec<_>>();
    let official_plugins = supportability.official_plugins.into_iter().collect::<Vec<_>>();
    let report = IncidentModeReport {
        incident_type: classification.incident_type,
        severity: severity_name(&classification.severity).to_string(),
        dashboard_score,
        disabled_actions,
        elevated_surfaces,
        retained_actions,
        required_evidence,
        supported_backends,
        official_plugins,
        incident_mode_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.incident_mode_ready;
    (serde_json::to_value(report).expect("incident mode report"), ok)
}

fn blast_radius_payload(simulation: BlastRadiusSimulation) -> (serde_json::Value, bool) {
    let BlastRadiusSimulation { classification, failing_components, workflows, service_context } =
        simulation;
    let failing_set = failing_components.iter().cloned().collect::<BTreeSet<_>>();
    let impacted_workflows = workflows
        .iter()
        .filter(|workflow| workflow.dependencies.iter().any(|dependency| failing_set.contains(dependency)))
        .collect::<Vec<_>>();
    let impacted_workflow_ids = impacted_workflows
        .iter()
        .map(|workflow| workflow.workflow_id.clone())
        .collect::<Vec<_>>();
    let impacted_tenants = impacted_workflows
        .iter()
        .map(|workflow| workflow.tenant_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let impacted_artifacts = impacted_workflows
        .iter()
        .flat_map(|workflow| workflow.artifact_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let impacted_services = service_context
        .into_iter()
        .filter(|(component, _)| failing_set.contains(component))
        .collect::<BTreeMap<_, _>>();
    let mut gaps = Vec::new();
    if failing_components.is_empty() {
        gaps.push("blast-radius analysis requires at least one failing component".to_string());
    }
    if impacted_workflow_ids.is_empty() {
        gaps.push("blast-radius analysis did not resolve any impacted workflows".to_string());
    }
    if impacted_tenants.is_empty() {
        gaps.push("blast-radius analysis should resolve at least one impacted tenant".to_string());
    }
    if impacted_services.len() != failing_set.len() {
        gaps.push("service context is incomplete for one or more failing components".to_string());
    }
    let report = BlastRadiusReport {
        incident_type: classification.incident_type,
        severity: severity_name(&classification.severity).to_string(),
        failing_components,
        impacted_workflows: impacted_workflow_ids,
        impacted_tenants,
        impacted_artifacts,
        impacted_services,
        blast_radius_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.blast_radius_ready;
    (serde_json::to_value(report).expect("blast radius report"), ok)
}

pub(crate) fn handle_incident_command(
    cli: &DagCli,
    command: &IncidentCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        IncidentCommands::Mode { simulation } => {
            let simulation: IncidentModeSimulation = parse_json_file(simulation)?;
            let (payload, ok) = incident_mode_payload(simulation);
            ("dag.incident.mode", payload, ok)
        }
        IncidentCommands::BlastRadius { simulation } => {
            let simulation: BlastRadiusSimulation = parse_json_file(simulation)?;
            let (payload, ok) = blast_radius_payload(simulation);
            ("dag.incident.blast-radius", payload, ok)
        }
    };
    emit_json(
        cli,
        surface,
        ok,
        payload,
        if ok {
            Vec::new()
        } else {
            vec![json!({"message":"incident posture is incomplete","remediation":"fill the reported incident-mode gaps before treating the workflow family as incident-ready"})]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{blast_radius_payload, incident_mode_payload};
    use bijux_dag_runtime::simulated_platform::{
        IncidentClassification, IncidentSeverity, PlatformHealthDashboard, RunbookEntry,
        SupportabilityModel,
    };
    use std::collections::{BTreeMap, BTreeSet};

    use super::{BlastRadiusSimulation, IncidentModeSimulation, WorkflowImpact};

    #[test]
    fn incident_mode_accepts_reduced_action_surface_with_runbooks() {
        let simulation = IncidentModeSimulation {
            classification: IncidentClassification {
                incident_type: "scheduler-partition".to_string(),
                severity: IncidentSeverity::High,
                routing: "platform-oncall".to_string(),
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.78,
                scheduler_health: 0.82,
                artifact_store_health: 0.75,
                auth_health: 0.88,
                policy_health: 0.84,
            },
            disabled_actions: vec!["bulk-backfill".to_string()],
            elevated_surfaces: vec!["scheduler-health".to_string(), "run-failures".to_string()],
            retained_actions: vec!["cancel".to_string(), "pause".to_string()],
            runbooks: vec![RunbookEntry {
                name: "scheduler outage".to_string(),
                trigger: "leader election instability".to_string(),
                required_evidence: vec!["timeline".to_string(), "queue snapshot".to_string()],
            }],
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::from(["builtin-scheduler".to_string()]),
                supported_backends: BTreeSet::from(["local".to_string(), "remote".to_string()]),
            },
        };
        let (payload, ok) = incident_mode_payload(simulation);
        assert!(ok);
        assert_eq!(payload["incident_mode_ready"], true);
    }

    #[test]
    fn incident_mode_flags_missing_controls() {
        let simulation = IncidentModeSimulation {
            classification: IncidentClassification {
                incident_type: "artifact-store-outage".to_string(),
                severity: IncidentSeverity::Critical,
                routing: "platform-incident".to_string(),
            },
            dashboard: PlatformHealthDashboard {
                engine_health: 0.50,
                scheduler_health: 0.45,
                artifact_store_health: 0.30,
                auth_health: 0.70,
                policy_health: 0.68,
            },
            disabled_actions: Vec::new(),
            elevated_surfaces: Vec::new(),
            retained_actions: Vec::new(),
            runbooks: Vec::new(),
            supportability: SupportabilityModel {
                official_plugins: BTreeSet::new(),
                supported_backends: BTreeSet::new(),
            },
        };
        let (payload, ok) = incident_mode_payload(simulation);
        assert!(!ok);
        assert_eq!(payload["incident_mode_ready"], false);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn blast_radius_resolves_impacted_workflows_and_tenants() {
        let simulation = BlastRadiusSimulation {
            classification: IncidentClassification {
                incident_type: "policy-store-outage".to_string(),
                severity: IncidentSeverity::High,
                routing: "platform-oncall".to_string(),
            },
            failing_components: vec!["policy-store".to_string()],
            workflows: vec![
                WorkflowImpact {
                    workflow_id: "tenant-a/regulated-backfill".to_string(),
                    tenant_id: "tenant-a".to_string(),
                    artifact_ids: vec!["artifact-1".to_string(), "artifact-2".to_string()],
                    dependencies: vec!["policy-store".to_string(), "artifact-store".to_string()],
                },
                WorkflowImpact {
                    workflow_id: "tenant-b/analytics-refresh".to_string(),
                    tenant_id: "tenant-b".to_string(),
                    artifact_ids: vec!["artifact-3".to_string()],
                    dependencies: vec!["scheduler".to_string()],
                },
            ],
            service_context: BTreeMap::from([(
                "policy-store".to_string(),
                vec!["authz".to_string(), "release-gates".to_string()],
            )]),
        };
        let (payload, ok) = blast_radius_payload(simulation);
        assert!(ok);
        assert_eq!(payload["impacted_workflows"].as_array().expect("workflows").len(), 1);
        assert_eq!(payload["impacted_tenants"][0], "tenant-a");
    }

    #[test]
    fn blast_radius_flags_missing_context_or_impacts() {
        let simulation = BlastRadiusSimulation {
            classification: IncidentClassification {
                incident_type: "scheduler-drift".to_string(),
                severity: IncidentSeverity::Medium,
                routing: "platform-review".to_string(),
            },
            failing_components: vec!["scheduler".to_string(), "artifact-store".to_string()],
            workflows: vec![WorkflowImpact {
                workflow_id: "tenant-a/no-hit".to_string(),
                tenant_id: "tenant-a".to_string(),
                artifact_ids: Vec::new(),
                dependencies: vec!["authz".to_string()],
            }],
            service_context: BTreeMap::from([("scheduler".to_string(), vec!["planner".to_string()])]),
        };
        let (payload, ok) = blast_radius_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 2);
    }
}
