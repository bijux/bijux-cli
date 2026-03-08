use serde_json::json;

pub(crate) fn backend_capability_payload(name: &str) -> Option<serde_json::Value> {
    match name {
        "k8s" | "kubernetes" => {
            let version = bijux_dag_runtime::K8sBackendVersionMetadata {
                k8s_version: "simulated-v1.30".to_string(),
                api_server: "simulated-control-plane".to_string(),
                cluster_uid: "simulated-cluster".to_string(),
            };
            let caps = bijux_dag_runtime::k8s_capability_declaration();
            Some(json!({
                "format": "capabilities/v1",
                "backend": "kubernetes",
                "status": "simulated",
                "capabilities": {
                    "node_selector": caps.supports_node_selector,
                    "node_affinity": caps.supports_node_affinity,
                    "pod_affinity": caps.supports_pod_affinity
                },
                "version_metadata": version,
                "notes": [
                    "kubernetes execution remains simulated in this repository",
                    "capability declaration is contract-level and evidence-backed"
                ]
            }))
        }
        "hpc" | "slurm" => {
            let version = bijux_dag_runtime::capture_hpc_scheduler_version("slurm", "23.11.5");
            let retry = bijux_dag_runtime::effective_hpc_retry_policy(true, true);
            Some(json!({
                "format": "capabilities/v1",
                "backend": "hpc",
                "status": "simulated",
                "capabilities": {
                    "queue_partition_mapping": true,
                    "walltime_mapping": true,
                    "scheduler_retry_precedence": retry.effective_retry_owner
                },
                "version_metadata": version,
                "notes": [
                    "hpc execution remains simulated in this repository",
                    "slurm contract semantics are evidence-backed"
                ]
            }))
        }
        "remote" | "distributed" => {
            let lease = bijux_dag_runtime::TaskLeaseSemantics {
                lease_duration_ms: 30_000,
                renew_before_expiry_ms: 5_000,
                max_renewals: 10,
                recovery_grace_ms: 10_000,
            };
            let heartbeat = bijux_dag_runtime::HeartbeatSemantics {
                interval_ms: 1_000,
                timeout_ms: 5_000,
                delayed_threshold_ms: 2_500,
            };
            Some(json!({
                "format": "capabilities/v1",
                "backend": "remote",
                "status": "simulated",
                "capabilities": {
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
                },
                "notes": [
                    "remote execution remains simulated in this repository",
                    "worker protocol contract semantics are evidence-backed"
                ]
            }))
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
    }

    #[test]
    fn capability_query_output_is_stable_for_hpc() {
        let first = backend_capability_payload("hpc").expect("hpc payload");
        let second = backend_capability_payload("hpc").expect("hpc payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "hpc");
        assert_eq!(first["status"], "simulated");
    }

    #[test]
    fn capability_query_output_is_stable_for_remote() {
        let first = backend_capability_payload("remote").expect("remote payload");
        let second = backend_capability_payload("remote").expect("remote payload");
        assert_eq!(first, second);
        assert_eq!(first["format"], "capabilities/v1");
        assert_eq!(first["backend"], "remote");
        assert_eq!(first["status"], "simulated");
    }

    #[test]
    fn unknown_backend_query_is_rejected_by_surface() {
        assert!(backend_capability_payload("unknown-backend").is_none());
    }
}
