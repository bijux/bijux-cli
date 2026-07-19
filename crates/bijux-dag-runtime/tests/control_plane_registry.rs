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
    register_dag_version, resolve_environment_values, select_dag_version, CompatibilityDecision,
    DagRegistry, DagVersionRecord, DagVersionSelectionPolicy, DagVersionStatus,
    EnvironmentConfiguration, EnvironmentMode, RunControlOperation, TypedControlPlaneRequest,
    TypedControlPlaneResponse,
};
use std::collections::BTreeMap;

#[test]
fn registers_versions_and_rejects_duplicates() {
    let mut registry = DagRegistry::default();
    let first = DagVersionRecord {
        version_id: "2026.03.01".to_string(),
        compatibility_line: "v0.1".to_string(),
        status: DagVersionStatus::Validated,
        created_unix_ms: 1,
    };
    register_dag_version(
        &mut registry,
        "artifact-release",
        "platform",
        vec!["release".to_string()],
        first,
    )
    .expect("first registration should succeed");

    let duplicate = DagVersionRecord {
        version_id: "2026.03.01".to_string(),
        compatibility_line: "v0.1".to_string(),
        status: DagVersionStatus::Active,
        created_unix_ms: 2,
    };
    let error = register_dag_version(
        &mut registry,
        "artifact-release",
        "platform",
        vec!["release".to_string()],
        duplicate,
    )
    .expect_err("duplicate registration should fail");
    assert!(error.contains("already contains version"));
}

#[test]
fn selects_latest_pinned_and_compatible_versions() {
    let mut registry = DagRegistry::default();
    for record in [
        DagVersionRecord {
            version_id: "2026.03.01".to_string(),
            compatibility_line: "v0.1".to_string(),
            status: DagVersionStatus::Validated,
            created_unix_ms: 1,
        },
        DagVersionRecord {
            version_id: "2026.03.02".to_string(),
            compatibility_line: "v0.1".to_string(),
            status: DagVersionStatus::Active,
            created_unix_ms: 2,
        },
        DagVersionRecord {
            version_id: "2026.04.01".to_string(),
            compatibility_line: "v0.2".to_string(),
            status: DagVersionStatus::Draft,
            created_unix_ms: 3,
        },
    ] {
        register_dag_version(
            &mut registry,
            "artifact-release",
            "platform",
            vec!["release".to_string()],
            record,
        )
        .expect("registration should succeed");
    }

    let latest =
        select_dag_version(&registry, "artifact-release", &DagVersionSelectionPolicy::RunLatest);
    assert_eq!(
        latest,
        CompatibilityDecision::Selected {
            version_id: "2026.03.02".to_string(),
            reason: "selected latest validated or active version".to_string()
        }
    );

    let pinned = select_dag_version(
        &registry,
        "artifact-release",
        &DagVersionSelectionPolicy::RunPinned { version_id: "2026.03.01".to_string() },
    );
    assert_eq!(
        pinned,
        CompatibilityDecision::Selected {
            version_id: "2026.03.01".to_string(),
            reason: "selected pinned version".to_string()
        }
    );

    let compatible = select_dag_version(
        &registry,
        "artifact-release",
        &DagVersionSelectionPolicy::RunCompatible { compatibility_line: "v0.1".to_string() },
    );
    assert_eq!(
        compatible,
        CompatibilityDecision::Selected {
            version_id: "2026.03.02".to_string(),
            reason: "selected highest compatible version in 'v0.1'".to_string()
        }
    );
}

#[test]
fn resolves_environment_values_with_parent_and_overrides() {
    let parent = EnvironmentConfiguration {
        mode: EnvironmentMode::Ci,
        parent: None,
        values: BTreeMap::from([("CACHE_DIR".to_string(), "artifacts/cache".to_string())]),
        overrides: BTreeMap::from([("JOBS".to_string(), "4".to_string())]),
    };
    let child = EnvironmentConfiguration {
        mode: EnvironmentMode::Staging,
        parent: Some("ci".to_string()),
        values: BTreeMap::from([("LOG_LEVEL".to_string(), "info".to_string())]),
        overrides: BTreeMap::from([("JOBS".to_string(), "8".to_string())]),
    };
    let resolved = resolve_environment_values(&child, Some(&parent));
    assert_eq!(resolved.get("CACHE_DIR"), Some(&"artifacts/cache".to_string()));
    assert_eq!(resolved.get("LOG_LEVEL"), Some(&"info".to_string()));
    assert_eq!(resolved.get("JOBS"), Some(&"8".to_string()));
}

#[test]
fn control_plane_operations_use_typed_request_response_contracts() {
    let request = TypedControlPlaneRequest {
        operation: RunControlOperation::Submit,
        dag_name: "artifact-release".to_string(),
        run_id: None,
        payload: serde_json::json!({"version_policy": "run-compatible"}),
    };
    assert_eq!(request.dag_name, "artifact-release");
    let response = TypedControlPlaneResponse {
        accepted: true,
        message: "submission accepted".to_string(),
        details: serde_json::json!({"run_id": "run-20260306"}),
    };
    assert!(response.accepted);
}
