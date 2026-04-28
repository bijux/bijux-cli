use crate::commands::{DagCli, FleetCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    check_worker_version_compatibility, validate_worker_identity, worker_alive, LivenessPolicy,
    WorkerHeartbeat, WorkerRegistration, WorkerVersionCompatibilityRule,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RegistrationSimulation {
    registration: WorkerRegistration,
    heartbeat: WorkerHeartbeat,
    liveness_policy: LivenessPolicy,
    version_rule: WorkerVersionCompatibilityRule,
    now_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct RegistrationReport {
    worker_id: String,
    backend_kind: String,
    labels: std::collections::BTreeMap<String, String>,
    inflight_nodes: Vec<String>,
    alive: bool,
    version_compatible: bool,
    gaps: Vec<String>,
    registration_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn registration_payload(simulation: RegistrationSimulation) -> (serde_json::Value, bool) {
    let RegistrationSimulation {
        registration,
        heartbeat,
        liveness_policy,
        version_rule,
        now_unix_ms,
    } = simulation;
    let identity_valid = validate_worker_identity(&registration.identity).is_ok();
    let alive = worker_alive(&heartbeat, now_unix_ms, &liveness_policy);
    let version_compatible =
        check_worker_version_compatibility(&registration.identity.worker_version, &version_rule);
    let mut gaps = Vec::new();
    if !identity_valid {
        gaps.push("worker registration requires stable non-empty identity fields".to_string());
    }
    if registration.registered_unix_ms == 0 {
        gaps.push("worker registration must record a registration timestamp".to_string());
    }
    if heartbeat.worker_id != registration.identity.worker_id {
        gaps.push("worker heartbeat does not match the registered worker identity".to_string());
    }
    if !alive {
        gaps.push("worker liveness is stale under the declared heartbeat policy".to_string());
    }
    if !version_compatible {
        gaps.push("worker version is below the planner compatibility floor".to_string());
    }
    if registration.capabilities.cpu_capacity == 0 || registration.capabilities.memory_mb == 0 {
        gaps.push("worker registration must declare non-zero cpu and memory capacity".to_string());
    }
    let report = RegistrationReport {
        worker_id: registration.identity.worker_id,
        backend_kind: registration.identity.backend_kind,
        labels: registration.identity.labels,
        inflight_nodes: heartbeat.inflight_nodes,
        alive,
        version_compatible,
        registration_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.registration_ready;
    (serde_json::to_value(report).expect("registration report"), ok)
}

pub(crate) fn handle_fleet_command(
    cli: &DagCli,
    command: &FleetCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        FleetCommands::Register { simulation } => {
            let simulation: RegistrationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = registration_payload(simulation);
            ("dag.fleet.register", payload, ok)
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
                "message":"worker fleet posture is incomplete",
                "remediation":"fix the reported fleet gaps before treating this worker path as production-ready"
            })]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{registration_payload, RegistrationSimulation};
    use bijux_dag_runtime::simulated_platform::{
        LivenessPolicy, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity, WorkerRegistration,
        WorkerVersionCompatibilityRule,
    };
    use std::collections::BTreeMap;

    #[test]
    fn registration_accepts_live_compatible_worker() {
        let simulation = RegistrationSimulation {
            registration: WorkerRegistration {
                identity: WorkerIdentity {
                    worker_id: "worker-a".to_string(),
                    worker_version: "1.4.0".to_string(),
                    backend_kind: "kubernetes".to_string(),
                    labels: BTreeMap::from([
                        ("region".to_string(), "eu-north".to_string()),
                        ("pool".to_string(), "general".to_string()),
                    ]),
                },
                capabilities: WorkerCapabilities {
                    cpu_capacity: 16,
                    memory_mb: 65_536,
                    supports_gpu: false,
                    supports_container: true,
                    supports_sandbox_profiles: vec!["strict".to_string()],
                },
                registered_unix_ms: 1_700_000_000_000,
            },
            heartbeat: WorkerHeartbeat {
                worker_id: "worker-a".to_string(),
                unix_ms: 1_700_000_000_100,
                inflight_nodes: vec!["load".to_string()],
            },
            liveness_policy: LivenessPolicy { heartbeat_timeout_ms: 5_000, grace_retries: 2 },
            version_rule: WorkerVersionCompatibilityRule {
                planner_version: "1.4.0".to_string(),
                minimum_worker_version: "1.3.0".to_string(),
            },
            now_unix_ms: 1_700_000_001_000,
        };
        let (payload, ok) = registration_payload(simulation);
        assert!(ok);
        assert_eq!(payload["registration_ready"], true);
    }

    #[test]
    fn registration_flags_stale_or_anonymous_worker() {
        let simulation = RegistrationSimulation {
            registration: WorkerRegistration {
                identity: WorkerIdentity {
                    worker_id: String::new(),
                    worker_version: "0.8.0".to_string(),
                    backend_kind: String::new(),
                    labels: BTreeMap::new(),
                },
                capabilities: WorkerCapabilities {
                    cpu_capacity: 0,
                    memory_mb: 0,
                    supports_gpu: false,
                    supports_container: false,
                    supports_sandbox_profiles: Vec::new(),
                },
                registered_unix_ms: 0,
            },
            heartbeat: WorkerHeartbeat {
                worker_id: "other-worker".to_string(),
                unix_ms: 10,
                inflight_nodes: Vec::new(),
            },
            liveness_policy: LivenessPolicy { heartbeat_timeout_ms: 100, grace_retries: 1 },
            version_rule: WorkerVersionCompatibilityRule {
                planner_version: "1.4.0".to_string(),
                minimum_worker_version: "1.3.0".to_string(),
            },
            now_unix_ms: 1_000,
        };
        let (payload, ok) = registration_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }
}
