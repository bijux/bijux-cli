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

use bijux_dag_runtime::{
    compute_platform_maturity, detect_extension_compatibility_issues,
    extension_discovery_inventory, extension_failure_isolated, extension_point_status_report,
    internal_hook_ready_for_promotion, negotiate_plugin_version, register_extension,
    validate_extension_descriptor, validate_plugin_conformance, CapabilityRange,
    ExtensionDescriptor, ExtensionRegistration, InternalHookPromotionChecklist, PluginBoundaryKind,
    PluginIsolationPolicy, PluginMetadata, PluginTrustPolicy,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn plugin_version_negotiation_and_conformance_are_typed() {
    let metadata = PluginMetadata {
        name: "official-local-adapter".to_string(),
        version: "1.2.0".to_string(),
        boundary: PluginBoundaryKind::TaskAdapter,
        capabilities: vec!["local-process".to_string(), "container-task".to_string()],
        policy_requirements: vec!["deterministic".to_string()],
        compatibility: CapabilityRange {
            min_contract_version: "v0.1".to_string(),
            max_contract_version: "v0.3".to_string(),
        },
    };
    assert!(negotiate_plugin_version(&metadata.compatibility, "v0.2"));

    let result = validate_plugin_conformance(
        &metadata,
        &PluginTrustPolicy {
            require_signature: true,
            allowlisted_publishers: vec!["bijux".to_string()],
            allowed_environments: vec!["local".to_string(), "ci".to_string()],
        },
        &PluginIsolationPolicy {
            deny_undeclared_effects: true,
            require_deterministic_mode: true,
            enforce_resource_caps: true,
        },
    );
    assert!(result.passed);
}

#[test]
fn extension_discovery_and_scorecard_are_stable() {
    let discovery = extension_discovery_inventory(&[
        ExtensionRegistration {
            plugin_name: "observer".to_string(),
            plugin_version: "0.1.0".to_string(),
            boundary: PluginBoundaryKind::ObservabilitySink,
            registered_unix_ms: 1,
        },
        ExtensionRegistration {
            plugin_name: "artifact-fs".to_string(),
            plugin_version: "0.1.0".to_string(),
            boundary: PluginBoundaryKind::ArtifactStore,
            registered_unix_ms: 1,
        },
    ]);
    assert_eq!(discovery.len(), 2);
    assert_eq!(discovery[0].plugin_name, "artifact-fs");
    assert_eq!(compute_platform_maturity(&[80, 70, 90, 85, 60, 75, 65]), 75);
}

#[test]
fn extension_registration_conflict_is_rejected() {
    let descriptor = ExtensionDescriptor {
        plugin_name: "local-adapter".to_string(),
        plugin_version: "1.0.0".to_string(),
        boundary: PluginBoundaryKind::TaskAdapter,
        contract_version: "v0.1".to_string(),
        capabilities: vec!["execute".to_string()],
        trust_model: "signed".to_string(),
    };
    let mut registry = BTreeMap::new();
    register_extension(&mut registry, descriptor.clone()).unwrap();
    let err = register_extension(&mut registry, descriptor).unwrap_err();
    assert!(err.contains("conflict"));
}

#[test]
fn unknown_extension_versions_and_missing_capabilities_are_reported() {
    let descriptors = vec![ExtensionDescriptor {
        plugin_name: "broken".to_string(),
        plugin_version: "1.0.0".to_string(),
        boundary: PluginBoundaryKind::ExecutorBackend,
        contract_version: "v9.9".to_string(),
        capabilities: vec!["launch".to_string()],
        trust_model: "signed".to_string(),
    }];
    let supported = BTreeSet::from(["v0.1".to_string()]);
    let required = BTreeSet::from(["observe".to_string()]);
    let issues = detect_extension_compatibility_issues(&descriptors, &supported, &required);
    assert!(issues.iter().any(|i| i.reason.contains("unsupported contract version")));
    assert!(issues.iter().any(|i| i.reason.contains("missing required capability")));
}

#[test]
fn extension_descriptor_requires_contract_shape() {
    let err = validate_extension_descriptor(&ExtensionDescriptor {
        plugin_name: "x".to_string(),
        plugin_version: "1.0".to_string(),
        boundary: PluginBoundaryKind::TaskAdapter,
        contract_version: "0.1".to_string(),
        capabilities: vec!["run".to_string()],
        trust_model: "none".to_string(),
    })
    .unwrap_err();
    assert!(err.contains("v-prefixed"));
}

#[test]
fn extension_point_report_and_internal_hook_promotion_checks_are_explicit() {
    let report = extension_point_status_report();
    assert!(!report.is_empty());
    let checklist = InternalHookPromotionChecklist {
        hook_name: "validation_hook".to_string(),
        has_contract_doc: true,
        has_versioning_policy: true,
        has_negative_tests: true,
        has_failure_isolation: true,
    };
    assert!(internal_hook_ready_for_promotion(&checklist));
}

#[test]
fn extension_failure_isolation_prevents_engine_crash_model() {
    assert!(extension_failure_isolated("extension_only", false));
    assert!(!extension_failure_isolated("engine", true));
}
