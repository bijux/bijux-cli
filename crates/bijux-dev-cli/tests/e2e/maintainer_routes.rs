#![forbid(unsafe_code)]
//! Maintainer route dispatch e2e contracts.

use std::collections::BTreeMap;

use bijux_dev_cli::cli::dispatch::{
    try_handle, ContractsSchemaInput, DoctorReportInput, RouteInventoryQuery, RuntimeQueryProvider,
    StateAuditInput,
};
use bijux_dev_cli::reports::control_plane::ProductContractRow;
use bijux_dev_cli::reports::env;
use bijux_dev_cli::reports::registry;
use bijux_dev_cli::reports::runtime_identity;
use serde_json::{json, Value};

struct StubRuntime;

impl RuntimeQueryProvider for StubRuntime {
    fn route_inventory(&self) -> RouteInventoryQuery {
        RouteInventoryQuery { routes: vec![vec!["registry".into()]], aliases: vec![] }
    }

    fn registry_inventory(&self) -> Vec<registry::NamespaceInventoryRow> {
        vec![registry::NamespaceInventoryRow {
            name: "cli".into(),
            reserved: true,
            owner: "bijux-cli".into(),
        }]
    }

    fn plugin_list(&self) -> Vec<Value> {
        Vec::new()
    }

    fn product_contracts(&self) -> Vec<ProductContractRow> {
        Vec::new()
    }

    fn env_map(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn active_paths(&self) -> env::ActivePaths {
        env::ActivePaths {
            config_file: "/tmp/config".into(),
            history_file: "/tmp/history".into(),
            plugins_dir: "/tmp/plugins".into(),
        }
    }

    fn doctor_report_input(&self) -> DoctorReportInput {
        DoctorReportInput {
            config_issues: vec![],
            path_issues: vec![],
            plugin_issues: vec![],
            history_issues: vec![],
            memory_issues: vec![],
        }
    }

    fn state_audit_input(&self) -> StateAuditInput {
        StateAuditInput {
            path_status: bijux_dev_cli::reports::state_audit::StatePathStatusInput {
                config: json!({}),
                history: json!({}),
                plugins_registry: json!({}),
                memory: json!({}),
            },
            corruption_health: json!({}),
        }
    }

    fn state_doctor_report(&self) -> Value {
        json!({})
    }

    fn contracts_schema_input(&self) -> ContractsSchemaInput {
        ContractsSchemaInput { schema_ids: vec![], schema_version: "v1".into() }
    }

    fn runtime_identity_input(&self) -> runtime_identity::RuntimeIdentityInput {
        runtime_identity::RuntimeIdentityInput {
            install_report: runtime_identity::InstallHealthReport {
                active_binary: None,
                path_binaries: vec![],
                has_path_shadowing: false,
                has_duplicate_installs: false,
                stale_wrapper_maintenance: vec![],
                has_mismatched_wheel_binary_versions: false,
                legacy_installer_conflicts: vec![],
                active_binary_missing: false,
                broken_symlink_active_binary: false,
            },
            python_bridge_supported: true,
            cargo_canonical_package: "bijux-cli".into(),
            pip_canonical_package: "bijux-cli".into(),
            canonical_crate_name: "bijux-cli".into(),
        }
    }
}

#[test]
fn known_route_returns_payload_and_unknown_route_returns_none() {
    let runtime = StubRuntime;
    let payload = try_handle(&["registry".into()], &[], &runtime)
        .expect("dispatch should succeed")
        .expect("known route should return payload");

    assert!(payload.get("registry").is_some());

    let none_payload = try_handle(&["unknown".into()], &[], &runtime)
        .expect("dispatch should succeed for unknown paths");
    assert!(none_payload.is_none());
}
