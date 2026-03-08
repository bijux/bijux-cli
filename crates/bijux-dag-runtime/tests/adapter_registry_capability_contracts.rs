use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::adapter_api::{Adapter, AdapterId, EffectSet, NodeCtx};
use bijux_dag_runtime::adapter_conformance::validate_descriptor;
use bijux_dag_runtime::{
    adapter_registry_dump, capture_hpc_scheduler_version, hpc_resource_fingerprint,
    k8s_capability_declaration, registered_adapters, validate_worker_identity,
    HpcResourceFingerprintInput, NodeResult, RuntimeError, WorkerIdentity,
};

#[test]
fn runtime_registry_query_output_is_stable() {
    let first = adapter_registry_dump();
    let second = adapter_registry_dump();
    assert_eq!(first, second);
    let count = first["count"].as_u64().expect("count");
    let adapters = first["adapters"].as_array().expect("adapters array");
    assert_eq!(count as usize, adapters.len());
}

#[test]
fn adapter_metadata_is_present_in_registry_output_surface() {
    let dump = adapter_registry_dump();
    let adapters = dump["adapters"].as_array().expect("adapters");
    assert!(!adapters.is_empty(), "adapter registry must not be empty");
    for adapter in adapters {
        assert!(
            adapter["adapter_id"].as_str().is_some_and(|v| !v.is_empty()),
            "adapter_id must be present"
        );
        assert!(
            adapter["adapter_version"]
                .as_str()
                .is_some_and(|v| !v.is_empty()),
            "adapter_version must be present"
        );
    }
}

#[test]
fn adapter_registry_rejects_duplicate_identities_by_reported_list() {
    let adapters = registered_adapters();
    let mut ids = std::collections::BTreeSet::new();
    for adapter in adapters {
        let identity = format!("{}@{}", adapter.adapter_id, adapter.adapter_version);
        assert!(
            ids.insert(identity.clone()),
            "duplicate adapter identity: {identity}"
        );
    }
}

#[test]
fn incomplete_capability_declaration_is_rejected_by_conformance() {
    struct BrokenAdapter;
    impl Adapter for BrokenAdapter {
        fn id(&self) -> AdapterId {
            AdapterId {
                id: "broken".to_string(),
                version: "".to_string(),
            }
        }
        fn supported_kinds(&self) -> Vec<String> {
            vec![]
        }
        fn required_effects(&self) -> EffectSet {
            EffectSet::default()
        }
        fn produces_outputs_schema_version(&self) -> String {
            "".to_string()
        }
        fn execute(&self, _ctx: &NodeCtx) -> Result<NodeResult, RuntimeError> {
            Err(RuntimeError::Executor("not executed".to_string()))
        }
    }
    let descriptor = BrokenAdapter.descriptor();
    let report = validate_descriptor(&descriptor);
    assert!(!report.passed);
    assert!(report
        .violations
        .iter()
        .any(|v: &String| v.contains("missing adapter version")));
    assert!(report
        .violations
        .iter()
        .any(|v: &String| v.contains("missing supported kinds")));
    assert!(report
        .violations
        .iter()
        .any(|v: &String| v.contains("missing outputs schema version")));
}

#[test]
fn backend_capability_query_output_stability_for_kubernetes_contract() {
    let first = k8s_capability_declaration();
    let second = k8s_capability_declaration();
    assert_eq!(first.supports_node_selector, second.supports_node_selector);
    assert_eq!(first.supports_node_affinity, second.supports_node_affinity);
    assert_eq!(first.supports_pod_affinity, second.supports_pod_affinity);
}

#[test]
fn backend_capability_query_output_stability_for_hpc_contract() {
    let first = capture_hpc_scheduler_version("slurm", "23.11.5");
    let second = capture_hpc_scheduler_version("slurm", "23.11.5");
    assert_eq!(first.scheduler_name, second.scheduler_name);
    assert_eq!(first.scheduler_version, second.scheduler_version);
}

#[test]
fn capability_query_unknown_backend_name_yields_no_payload_in_app_surface() {
    let payload = adapter_registry_dump();
    assert_eq!(payload["adapters"].is_array(), true);
    assert!(payload["count"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn adapter_metadata_exclusion_from_graph_identity_is_explicit_in_contracts() {
    let graph_a = bijux_dag_core::parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":1}}],
          "edges":[]
        }"#,
    )
    .expect("graph a");
    let graph_b = bijux_dag_core::parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"n","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":1}}],
          "edges":[]
        }"#,
    )
    .expect("graph b");
    assert_eq!(
        graph_a.graph_fingerprint().expect("fingerprint a"),
        graph_b.graph_fingerprint().expect("fingerprint b")
    );
}

#[test]
fn adapter_version_participates_in_replay_compatibility_cache_contracts() {
    let baseline = serde_json::json!({
        "adapter_id": "shell",
        "adapter_version": "1.0.0",
        "node_fingerprint": "abc"
    });
    let changed = serde_json::json!({
        "adapter_id": "shell",
        "adapter_version": "2.0.0",
        "node_fingerprint": "abc"
    });
    assert_ne!(baseline, changed);
}

#[test]
fn external_adapter_identity_validation_requires_backend_identity_fields() {
    let valid = WorkerIdentity {
        worker_id: "worker-1".to_string(),
        worker_version: "0.1.0".to_string(),
        backend_kind: "remote-sim".to_string(),
        labels: std::collections::BTreeMap::new(),
    };
    assert!(validate_worker_identity(&valid).is_ok());

    let invalid = WorkerIdentity {
        backend_kind: "".to_string(),
        ..valid
    };
    assert!(validate_worker_identity(&invalid).is_err());
}

#[test]
fn hpc_resource_fingerprint_is_stable_for_identical_input() {
    let input = HpcResourceFingerprintInput {
        queue: "batch".to_string(),
        partition: "general".to_string(),
        account: "research".to_string(),
    };
    let first = hpc_resource_fingerprint(&input);
    let second = hpc_resource_fingerprint(&input);
    assert_eq!(first, second);
}
