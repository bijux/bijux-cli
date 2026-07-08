use serde_json::json;

const LANE_ENFORCED: &str = "ENFORCED";
const LANE_SIMULATED: &str = "SIMULATED";

fn execution_lane(status: &str) -> &'static str {
    match status {
        "implemented" => LANE_ENFORCED,
        _ => LANE_SIMULATED,
    }
}

fn base_payload(backend: &str, status: &str) -> serde_json::Value {
    let lane = execution_lane(status);
    base_payload_with_lane(backend, status, lane, lane == LANE_ENFORCED)
}

fn base_payload_with_lane(
    backend: &str,
    status: &str,
    lane: &str,
    production_ready: bool,
) -> serde_json::Value {
    json!({
        "format": "capabilities/v1",
        "backend": backend,
        "status": status,
        "execution_lane": lane,
        "production_ready": production_ready
    })
}

pub(crate) fn backend_capability_payload(name: &str) -> Option<serde_json::Value> {
    match name {
        "local" => {
            let mut payload = base_payload("local", "implemented");
            payload["capabilities"] = json!({
                "runtime_execution": true,
                "replay_support": true,
                "stream_capture": true
            });
            payload["notes"] = json!([
                "local execution is implemented in this repository",
                "local capability contract is evidence-backed"
            ]);
            Some(payload)
        }
        "k8s" | "kubernetes" => {
            let version = bijux_dag_runtime::K8sBackendVersionMetadata {
                k8s_version: "simulated-v1.30".to_string(),
                api_server: "simulated-control-plane".to_string(),
                cluster_uid: "simulated-cluster".to_string(),
            };
            let caps = bijux_dag_runtime::k8s_capability_declaration();
            let mut payload = base_payload("kubernetes", "simulated");
            payload["capabilities"] = json!({
                "job_submission": true,
                "resource_mapping": true,
                "active_deadline_mapping": true,
                "pod_phase_mapping": true,
                "workspace_transfer": true,
                "log_capture": true,
                "node_selector": caps.supports_node_selector,
                "node_affinity": caps.supports_node_affinity,
                "pod_affinity": caps.supports_pod_affinity
            });
            payload["version_metadata"] = json!(version);
            payload["notes"] = json!([
                "kubernetes job execution is modeled and exercised through the shared runtime lane",
                "cluster semantics remain simulated rather than control-plane-backed in this repository"
            ]);
            Some(payload)
        }
        "slurm" => {
            let version = bijux_dag_runtime::capture_hpc_scheduler_version("slurm", "23.11.5");
            let retry = bijux_dag_runtime::effective_hpc_retry_policy(true, true);
            let mut payload =
                base_payload_with_lane("slurm", "implemented", LANE_ENFORCED, false);
            payload["capabilities"] = json!({
                "job_submission": true,
                "job_id_capture": true,
                "queue_partition_mapping": true,
                "walltime_mapping": true,
                "status_mapping": true,
                "log_capture": true,
                "shared_run_directory": true,
                "result_payload_handoff": true,
                "scheduler_retry_precedence": retry.effective_retry_owner
            });
            payload["version_metadata"] = json!(version);
            payload["notes"] = json!([
                "slurm execution submits nodes through sbatch and polls sacct through the stable run surface",
                "scheduled workers must reopen the same retained run directory on a shared filesystem; this is not a generic hpc abstraction or a public scheduler service"
            ]);
            Some(payload)
        }
        "hpc" => {
            let version = bijux_dag_runtime::capture_hpc_scheduler_version("slurm", "23.11.5");
            let retry = bijux_dag_runtime::effective_hpc_retry_policy(true, true);
            let mut payload = base_payload("hpc", "simulated");
            payload["capabilities"] = json!({
                "job_submission": true,
                "job_id_capture": true,
                "queue_partition_mapping": true,
                "walltime_mapping": true,
                "status_mapping": true,
                "log_capture": true,
                "scheduler_retry_precedence": retry.effective_retry_owner
            });
            payload["version_metadata"] = json!(version);
            payload["notes"] = json!([
                "generic hpc capability reporting remains modeled rather than tied to one concrete scheduler lane",
                "the separate slurm backend reports the implemented shared-filesystem sbatch and sacct path"
            ]);
            Some(payload)
        }
        "remote" | "distributed" => {
            let lease = bijux_dag_runtime::simulated_platform::TaskLeaseSemantics {
                lease_duration_ms: 30_000,
                renew_before_expiry_ms: 5_000,
                max_renewals: 10,
                recovery_grace_ms: 10_000,
            };
            let heartbeat = bijux_dag_runtime::simulated_platform::HeartbeatSemantics {
                interval_ms: 1_000,
                timeout_ms: 5_000,
                delayed_threshold_ms: 2_500,
            };
            let mut payload = base_payload("remote", "simulated");
            payload["capabilities"] = json!({
                "task_lease_semantics": {
                    "lease_duration_ms": lease.lease_duration_ms,
                    "renew_before_expiry_ms": lease.renew_before_expiry_ms,
                    "max_renewals": lease.max_renewals,
                    "recovery_grace_ms": lease.recovery_grace_ms
                },
                "heartbeat_semantics": {
                    "interval_ms": heartbeat.interval_ms,
                    "timeout_ms": heartbeat.timeout_ms,
                    "delayed_threshold_ms": heartbeat.delayed_threshold_ms
                },
                "duplicate_dispatch_prevention": true,
                "artifact_upload_commit_contract": true,
                "status_event_ordering_contract": true,
                "version_mismatch_rejection": true,
                "worker_pool_capability_negotiation": true
            });
            payload["notes"] = json!([
                "remote execution remains simulated in this repository",
                "worker protocol contract semantics are evidence-backed"
            ]);
            Some(payload)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::backend_capability_payload;

    #[test]
    fn capability_query_output_is_stable_for_kubernetes() {
        let first = backend_capability_payload("kubernetes").expect("kubernetes payload");
        let second = backend_capability_payload("kubernetes").expect("kubernetes payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "kubernetes");
        assert_eq!(first["status"], "simulated");
        assert_eq!(first["execution_lane"], "SIMULATED");
        assert_eq!(first["production_ready"], false);
        assert_eq!(first["capabilities"]["job_submission"], true);
        assert_eq!(first["capabilities"]["pod_phase_mapping"], true);
        assert_eq!(first["capabilities"]["workspace_transfer"], true);
    }

    #[test]
    fn capability_query_output_is_stable_for_local() {
        let first = backend_capability_payload("local").expect("local payload");
        let second = backend_capability_payload("local").expect("local payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "local");
        assert_eq!(first["status"], "implemented");
        assert_eq!(first["execution_lane"], "ENFORCED");
        assert_eq!(first["production_ready"], true);
    }

    #[test]
    fn capability_query_output_is_stable_for_hpc() {
        let first = backend_capability_payload("hpc").expect("hpc payload");
        let second = backend_capability_payload("hpc").expect("hpc payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "hpc");
        assert_eq!(first["status"], "simulated");
        assert_eq!(first["execution_lane"], "SIMULATED");
        assert_eq!(first["production_ready"], false);
        assert_eq!(first["capabilities"]["job_id_capture"], true);
        assert_eq!(first["capabilities"]["status_mapping"], true);
        assert_eq!(first["capabilities"]["log_capture"], true);
    }

    #[test]
    fn capability_query_output_is_stable_for_slurm() {
        let first = backend_capability_payload("slurm").expect("slurm payload");
        let second = backend_capability_payload("slurm").expect("slurm payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "slurm");
        assert_eq!(first["status"], "implemented");
        assert_eq!(first["execution_lane"], "ENFORCED");
        assert_eq!(first["production_ready"], false);
        assert_eq!(first["capabilities"]["job_submission"], true);
        assert_eq!(first["capabilities"]["shared_run_directory"], true);
        assert_eq!(first["capabilities"]["result_payload_handoff"], true);
    }

    #[test]
    fn capability_query_output_is_stable_for_remote() {
        let first = backend_capability_payload("remote").expect("remote payload");
        let second = backend_capability_payload("remote").expect("remote payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "remote");
        assert_eq!(first["status"], "simulated");
        assert_eq!(first["execution_lane"], "SIMULATED");
        assert_eq!(first["production_ready"], false);
    }

    #[test]
    fn unknown_backend_query_is_rejected_by_surface() {
        assert!(backend_capability_payload("unknown-backend").is_none());
    }

    #[test]
    fn capability_payload_excludes_modeled_only_runtime_surface_tokens() {
        let rendered = serde_json::to_string(&backend_capability_payload("remote").unwrap())
            .expect("payload json")
            .to_lowercase();
        for forbidden in [
            "federated_scheduling",
            "geo_federation",
            "ha_scheduler",
            "control_plane_api",
            "workflow_product",
            "ai_operator_assist",
            "dataset_semantics",
            "cost_optimization",
        ] {
            assert!(
                !rendered.contains(forbidden),
                "capability payload should exclude modeled-only token: {forbidden}"
            );
        }
    }

    #[test]
    fn simulated_backends_are_never_reported_as_enforced() {
        for backend in ["kubernetes", "hpc", "remote"] {
            let payload = backend_capability_payload(backend).expect("payload");
            assert_eq!(payload["status"], "simulated");
            assert_eq!(payload["execution_lane"], "SIMULATED");
            assert_eq!(payload["production_ready"], false);
        }
    }
}
