use bijux_dag_runtime::{
    compute_platform_maturity, extension_discovery_inventory, negotiate_plugin_version,
    validate_plugin_conformance, CapabilityRange, ExtensionRegistration, PluginBoundaryKind,
    PluginIsolationPolicy, PluginMetadata, PluginTrustPolicy,
};

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
