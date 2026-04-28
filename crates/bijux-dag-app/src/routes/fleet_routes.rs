use crate::commands::{DagCli, FleetCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    check_worker_version_compatibility, validate_worker_identity, worker_alive,
    worker_pool_satisfies_capability_request, LivenessPolicy, PlacementHint, WorkerCapabilities,
    WorkerHeartbeat, WorkerPool, WorkerPoolCapabilityRequest, WorkerRegistration,
    WorkerVersionCompatibilityRule,
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

#[derive(Debug, Deserialize)]
struct CapabilitySimulation {
    worker_id: String,
    capabilities: WorkerCapabilities,
    request: WorkerPoolCapabilityRequest,
    pool: WorkerPool,
    placement_hint: PlacementHint,
}

#[derive(Debug, Serialize)]
struct CapabilityReport {
    worker_id: String,
    pool_id: String,
    pool_class: String,
    worker_count: usize,
    capability_match: bool,
    preferred_pool_match: bool,
    preferred_label_match: bool,
    gaps: Vec<String>,
    capability_ready: bool,
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

fn capability_payload(simulation: CapabilitySimulation) -> (serde_json::Value, bool) {
    let CapabilitySimulation { worker_id, capabilities, request, pool, placement_hint } =
        simulation;
    let capability_match = worker_pool_satisfies_capability_request(&capabilities, &request);
    let preferred_pool_match = placement_hint.preferred_pool == pool.pool_id;
    let preferred_label_match = placement_hint.preferred_worker_labels.iter().all(|(key, value)| {
        match key.as_str() {
            "backend_kind" => value == "any",
            "supports_container" => {
                value == if capabilities.supports_container { "true" } else { "false" }
            }
            "supports_gpu" => value == if capabilities.supports_gpu { "true" } else { "false" },
            "sandbox_profile" => {
                capabilities.supports_sandbox_profiles.iter().any(|profile| profile == value)
            }
            _ => false,
        }
    });
    let mut gaps = Vec::new();
    if worker_id.trim().is_empty() {
        gaps.push("capability advertisement requires a worker identifier".to_string());
    }
    if pool.pool_id.trim().is_empty() || pool.class.trim().is_empty() {
        gaps.push("worker pool advertisement must declare a stable pool id and class".to_string());
    }
    if pool.workers.is_empty() || !pool.workers.iter().any(|id| id == &worker_id) {
        gaps.push("worker must appear in the declared worker pool membership".to_string());
    }
    if !capability_match {
        gaps.push("declared worker capabilities do not satisfy the requested placement contract".to_string());
    }
    if !preferred_pool_match {
        gaps.push("placement hint points to a different worker pool".to_string());
    }
    if !preferred_label_match {
        gaps.push("worker capabilities do not satisfy the preferred placement labels".to_string());
    }
    let report = CapabilityReport {
        worker_id,
        pool_id: pool.pool_id,
        pool_class: pool.class,
        worker_count: pool.workers.len(),
        capability_match,
        preferred_pool_match,
        preferred_label_match,
        capability_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.capability_ready;
    (serde_json::to_value(report).expect("capability report"), ok)
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
        FleetCommands::Capabilities { simulation } => {
            let simulation: CapabilitySimulation = parse_json_file(simulation)?;
            let (payload, ok) = capability_payload(simulation);
            ("dag.fleet.capabilities", payload, ok)
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
    use super::{capability_payload, registration_payload, CapabilitySimulation, RegistrationSimulation};
    use bijux_dag_runtime::simulated_platform::{
        LivenessPolicy, PlacementHint, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity,
        WorkerPool, WorkerPoolCapabilityRequest, WorkerRegistration, WorkerVersionCompatibilityRule,
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

    #[test]
    fn capability_accepts_worker_that_satisfies_requested_pool_contract() {
        let simulation = CapabilitySimulation {
            worker_id: "worker-a".to_string(),
            capabilities: WorkerCapabilities {
                cpu_capacity: 32,
                memory_mb: 131_072,
                supports_gpu: true,
                supports_container: true,
                supports_sandbox_profiles: vec!["strict".to_string(), "gpu".to_string()],
            },
            request: WorkerPoolCapabilityRequest {
                required_min_cpu_capacity: 16,
                required_min_memory_mb: 65_536,
                require_gpu: true,
                require_container_support: true,
                required_sandbox_profile: Some("strict".to_string()),
            },
            pool: WorkerPool {
                pool_id: "gpu-prod".to_string(),
                class: "gpu".to_string(),
                workers: vec!["worker-a".to_string(), "worker-b".to_string()],
            },
            placement_hint: PlacementHint {
                node_id: "train".to_string(),
                preferred_pool: "gpu-prod".to_string(),
                preferred_worker_labels: BTreeMap::from([
                    ("supports_gpu".to_string(), "true".to_string()),
                    ("sandbox_profile".to_string(), "strict".to_string()),
                ]),
            },
        };
        let (payload, ok) = capability_payload(simulation);
        assert!(ok);
        assert_eq!(payload["capability_ready"], true);
    }

    #[test]
    fn capability_flags_pool_or_feature_mismatch() {
        let simulation = CapabilitySimulation {
            worker_id: "worker-a".to_string(),
            capabilities: WorkerCapabilities {
                cpu_capacity: 4,
                memory_mb: 8_192,
                supports_gpu: false,
                supports_container: false,
                supports_sandbox_profiles: Vec::new(),
            },
            request: WorkerPoolCapabilityRequest {
                required_min_cpu_capacity: 16,
                required_min_memory_mb: 65_536,
                require_gpu: true,
                require_container_support: true,
                required_sandbox_profile: Some("strict".to_string()),
            },
            pool: WorkerPool {
                pool_id: "general".to_string(),
                class: String::new(),
                workers: vec!["worker-b".to_string()],
            },
            placement_hint: PlacementHint {
                node_id: "train".to_string(),
                preferred_pool: "gpu-prod".to_string(),
                preferred_worker_labels: BTreeMap::from([
                    ("supports_gpu".to_string(), "true".to_string()),
                    ("sandbox_profile".to_string(), "strict".to_string()),
                ]),
            },
        };
        let (payload, ok) = capability_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }
}
