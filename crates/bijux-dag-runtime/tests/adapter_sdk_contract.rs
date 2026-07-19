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

use bijux_dag_runtime::{ExecutorBackend, PluginManifest};

#[test]
fn plugin_manifest_has_stable_shape() {
    let manifest = PluginManifest {
        plugin_name: "example-adapter".to_string(),
        plugin_version: "1.0.0".to_string(),
        plugin_type: "adapter".to_string(),
        contract_version: "v0.1".to_string(),
    };
    let payload = serde_json::to_value(&manifest).expect("manifest should serialize");
    assert_eq!(payload.get("plugin_name").and_then(|v| v.as_str()), Some("example-adapter"));
}

#[test]
fn backend_enum_includes_external_service_variant() {
    let backend = ExecutorBackend::ExternalService;
    let payload = serde_json::to_string(&backend).expect("backend should serialize");
    assert!(payload.contains("ExternalService"));
}
