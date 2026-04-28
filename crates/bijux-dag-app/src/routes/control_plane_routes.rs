use crate::commands::{ControlPlaneCommands, DagCli};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{TypedControlPlaneRequest, TypedControlPlaneResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ApiSimulation {
    request: TypedControlPlaneRequest,
    #[serde(default)]
    replica_ids: Vec<String>,
    #[serde(default)]
    responses: Vec<TypedControlPlaneResponse>,
    load_balanced: bool,
    hidden_in_memory_authority: bool,
}

#[derive(Debug, Serialize)]
struct ApiReport {
    operation: String,
    replica_count: usize,
    load_balanced: bool,
    hidden_in_memory_authority: bool,
    response_consistent: bool,
    gaps: Vec<String>,
    api_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn api_payload(simulation: ApiSimulation) -> (serde_json::Value, bool) {
    let ApiSimulation { request, replica_ids, responses, load_balanced, hidden_in_memory_authority } =
        simulation;
    let first = responses.first();
    let response_consistent = first.is_some()
        && responses.iter().all(|response| {
            response.accepted == first.expect("first response").accepted
                && response.message == first.expect("first response").message
                && response.details == first.expect("first response").details
        });
    let mut gaps = Vec::new();
    if replica_ids.len() < 2 {
        gaps.push("stateless api evaluation should compare at least two replicas".to_string());
    }
    if responses.len() != replica_ids.len() {
        gaps.push("api simulation must provide one response per replica".to_string());
    }
    if !load_balanced {
        gaps.push("api tier should be exercised behind a load-balanced path".to_string());
    }
    if hidden_in_memory_authority {
        gaps.push("api replica depends on hidden in-memory authority".to_string());
    }
    if !response_consistent {
        gaps.push("replicas returned inconsistent control-plane responses".to_string());
    }
    let report = ApiReport {
        operation: format!("{:?}", request.operation).to_lowercase(),
        replica_count: replica_ids.len(),
        load_balanced,
        hidden_in_memory_authority,
        response_consistent,
        api_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.api_ready;
    (serde_json::to_value(report).expect("api report"), ok)
}

pub(crate) fn handle_control_plane_command(
    cli: &DagCli,
    command: &ControlPlaneCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        ControlPlaneCommands::Api { simulation } => {
            let simulation: ApiSimulation = parse_json_file(simulation)?;
            let (payload, ok) = api_payload(simulation);
            ("dag.control-plane.api", payload, ok)
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
            vec![json!({
                "message":"control-plane architecture posture is incomplete",
                "remediation":"fix the reported control-plane gaps before treating this surface as production-ready"
            })]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{api_payload, ApiSimulation};
    use bijux_dag_runtime::simulated_platform::{
        RunControlOperation, TypedControlPlaneRequest, TypedControlPlaneResponse,
    };
    use serde_json::json;

    #[test]
    fn api_accepts_load_balanced_consistent_replicas() {
        let request = TypedControlPlaneRequest {
            operation: RunControlOperation::Submit,
            dag_name: "atlas.load".to_string(),
            run_id: Some("run-1".to_string()),
            payload: json!({"graph_fingerprint":"g1"}),
        };
        let response = TypedControlPlaneResponse {
            accepted: true,
            message: "accepted".to_string(),
            details: json!({"run_id":"run-1"}),
        };
        let simulation = ApiSimulation {
            request,
            replica_ids: vec!["api-a".to_string(), "api-b".to_string()],
            responses: vec![response.clone(), response],
            load_balanced: true,
            hidden_in_memory_authority: false,
        };
        let (payload, ok) = api_payload(simulation);
        assert!(ok);
        assert_eq!(payload["api_ready"], true);
    }

    #[test]
    fn api_flags_sticky_or_inconsistent_replicas() {
        let simulation = ApiSimulation {
            request: TypedControlPlaneRequest {
                operation: RunControlOperation::Submit,
                dag_name: "atlas.load".to_string(),
                run_id: Some("run-1".to_string()),
                payload: json!({"graph_fingerprint":"g1"}),
            },
            replica_ids: vec!["api-a".to_string()],
            responses: vec![
                TypedControlPlaneResponse {
                    accepted: true,
                    message: "accepted".to_string(),
                    details: json!({"run_id":"run-1"}),
                },
                TypedControlPlaneResponse {
                    accepted: false,
                    message: "rejected".to_string(),
                    details: json!({"run_id":"run-1"}),
                },
            ],
            load_balanced: false,
            hidden_in_memory_authority: true,
        };
        let (payload, ok) = api_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }
}
