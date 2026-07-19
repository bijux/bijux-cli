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
    check_scheduler_admission, compose_tenant_run_id, enforce_tenant_plugin_allowlist,
    resolve_tenant_overlay, scope_lineage_query, tenant_provisioning_bootstrap,
    validate_tenant_isolation, TenantConfigOverlay, TenantId, TenantLineageScope,
    TenantPluginAllowlist, TenantPolicyBundleRef, TenantProvisioningSpec,
    TenantQueueIsolationPolicy, TenantRegistryPartition, TenantSchedulerAdmission,
};
use std::collections::BTreeMap;

#[test]
fn tenant_overlay_and_run_indexing_are_deterministic() {
    let tenant = TenantId::parse("tenant_alpha").expect("valid tenant");
    let overlay = TenantConfigOverlay {
        tenant_id: tenant.clone(),
        values: BTreeMap::from([("JOBS".to_string(), "8".to_string())]),
        overrides: BTreeMap::from([("LOG_LEVEL".to_string(), "info".to_string())]),
    };
    let resolved =
        resolve_tenant_overlay(&BTreeMap::from([("JOBS".to_string(), "4".to_string())]), &overlay);
    assert_eq!(resolved.get("JOBS"), Some(&"8".to_string()));
    assert_eq!(compose_tenant_run_id(&tenant, "run_001"), "tenant_alpha::run_001");
}

#[test]
fn tenant_plugin_and_lineage_scopes_enforce_boundaries() {
    let tenant = TenantId::parse("tenant_alpha").expect("valid tenant");
    let allowlist = TenantPluginAllowlist {
        tenant_id: tenant.clone(),
        allowed_plugins: vec!["official-local-adapter".to_string()],
    };
    assert!(enforce_tenant_plugin_allowlist("official-local-adapter", &allowlist));
    assert!(!enforce_tenant_plugin_allowlist("unknown-third-party", &allowlist));

    let scoped = scope_lineage_query(
        &["a1".to_string(), "a2".to_string()],
        &TenantLineageScope { tenant_id: tenant, allowed_artifact_ids: vec!["a2".to_string()] },
    );
    assert_eq!(scoped, vec!["a2".to_string()]);
}

#[test]
fn tenant_admission_bootstrap_and_isolation_conformance_hold() {
    let tenant = TenantId::parse("tenant_alpha").expect("valid tenant");
    let spec = TenantProvisioningSpec {
        tenant_id: tenant.clone(),
        namespace: "tenant-alpha".to_string(),
        registry_partition: TenantRegistryPartition {
            tenant_id: tenant.clone(),
            storage_partition: "registry/tenant-alpha".to_string(),
            index_prefix: "tenant-alpha".to_string(),
        },
        default_queue_isolation: TenantQueueIsolationPolicy {
            tenant_id: tenant.clone(),
            queue_names: vec!["tenant-alpha-default".to_string()],
            hard_isolation: true,
        },
        default_policy_bundle: TenantPolicyBundleRef {
            tenant_id: tenant.clone(),
            policy_bundle_id: "tenant-alpha-policy".to_string(),
            policy_bundle_version: "2026.03".to_string(),
        },
    };
    let steps = tenant_provisioning_bootstrap(&spec);
    assert_eq!(steps.len(), 4);

    let admitted = check_scheduler_admission(
        10,
        3,
        &TenantSchedulerAdmission {
            tenant_id: tenant.clone(),
            max_enqueued_runs: 20,
            max_dispatches_per_tick: 5,
        },
    );
    assert!(admitted);

    let conformance =
        validate_tenant_isolation(&tenant, &tenant, &tenant, &tenant, &tenant, &tenant);
    assert!(conformance.violations.is_empty());
}
