use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::simulated_platform::{
    ArtifactStoreBackend, HighAvailabilitySchedulerPlan, KubernetesExecutorContract,
    MultiTenantIdentity, QueuePartition, RegistryPersistenceBackend, RuntimeSecretContract,
    SchedulerScalingPlan,
};
use std::collections::BTreeMap;

#[test]
fn modeled_infrastructure_types_live_under_simulated_platform_surface() {
    let contract = KubernetesExecutorContract {
        namespace: "training".to_string(),
        pod_template: "cpu-default".to_string(),
        image_resolution_policy: "digest-only".to_string(),
        artifact_mount_strategy: "readonly-volume".to_string(),
        log_collection: "watch".to_string(),
    };
    let identity = MultiTenantIdentity {
        tenant_id: "tenant-a".to_string(),
        namespace: "training".to_string(),
        labels: BTreeMap::from([("owner".to_string(), "ml".to_string())]),
    };
    let queue = QueuePartition {
        queue_name: "priority".to_string(),
        tenant_id: Some("tenant-a".to_string()),
        max_concurrency: 4,
    };
    let scaling = SchedulerScalingPlan { worker_count: 3, sharding_key: "tenant".to_string() };
    let ha = HighAvailabilitySchedulerPlan {
        enabled: true,
        leader_election: "lease".to_string(),
        durable_queue: "postgres".to_string(),
    };
    let secret = RuntimeSecretContract {
        secret_refs: vec!["vault://training/api".to_string()],
        injection_mode: "env".to_string(),
        redaction_required: true,
    };

    assert_eq!(contract.namespace, "training");
    assert_eq!(identity.tenant_id, "tenant-a");
    assert_eq!(queue.max_concurrency, 4);
    assert_eq!(scaling.worker_count, 3);
    assert!(ha.enabled);
    assert!(secret.redaction_required);
    assert_eq!(ArtifactStoreBackend::ObjectStorage, ArtifactStoreBackend::ObjectStorage);
    assert_eq!(RegistryPersistenceBackend::Database, RegistryPersistenceBackend::Database);
}
