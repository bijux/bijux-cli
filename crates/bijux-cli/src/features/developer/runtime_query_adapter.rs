#![forbid(unsafe_code)]
//! Runtime query adapter for `dev cli` command handlers.

use std::collections::BTreeMap;
use std::env;
use std::path::Path;

use anyhow::Result;
use bijux_dev_cli::{
    cli::dispatch::{
        self as dev_dispatch, ContractsSchemaInput, DoctorReportInput, RouteInventoryQuery,
        RuntimeQueryProvider, StateAuditInput,
    },
    reports::{
        control_plane::{ProductContractRow, ProductSurfaceRow},
        env as dev_env, registry as dev_registry, runtime_identity as dev_runtime_identity,
        state_audit as dev_state_audit,
    },
};
use serde_json::{json, Value};

use crate::features::config::storage::{ConfigRepository, FileConfigRepository};
use crate::features::diagnostics::state_diagnostics_query;
use crate::features::diagnostics::state_paths::{
    env_map, state_diagnostics, state_path_status_value, ResolvedStatePaths,
};
use crate::features::install::{
    canonical_crate_name, cargo_install_strategy, install_health_report, pip_install_strategy,
    query::runtime_identity_query, PackageChannel,
};
use crate::features::plugins::{list_plugins, load_time_diagnostics};
use crate::routing::inventory::{registry_inventory, route_inventory};
use crate::routing::query::contracts_schema_query;
use crate::routing::registry::RouteRegistry;
use crate::routing::KNOWN_BIJUX_TOOLS;

struct RuntimeQueryAdapter<'a> {
    registry: &'a RouteRegistry,
    paths: &'a ResolvedStatePaths,
    plugin_registry_path: &'a Path,
}

impl RuntimeQueryProvider for RuntimeQueryAdapter<'_> {
    fn route_inventory(&self) -> RouteInventoryQuery {
        let inventory = route_inventory(self.registry);
        RouteInventoryQuery { routes: inventory.routes, aliases: inventory.aliases }
    }

    fn registry_inventory(&self) -> Vec<dev_registry::NamespaceInventoryRow> {
        registry_inventory(self.registry)
            .namespaces
            .into_iter()
            .map(|row| dev_registry::NamespaceInventoryRow {
                name: row.name.0,
                reserved: row.reserved,
                owner: row.owner,
            })
            .collect()
    }

    fn plugin_list(&self) -> Vec<Value> {
        list_plugins(self.plugin_registry_path)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|plugin| serde_json::to_value(plugin).ok())
            .collect()
    }

    fn product_contracts(&self) -> Vec<ProductContractRow> {
        KNOWN_BIJUX_TOOLS
            .iter()
            .map(|tool| ProductContractRow {
                namespace: tool.namespace.to_string(),
                repository: tool.repository.to_string(),
                runtime: ProductSurfaceRow {
                    command_surface: format!("bijux {}", tool.namespace),
                    binary: tool.runtime_binary.to_string(),
                    package: tool.runtime_package.to_string(),
                },
                control: ProductSurfaceRow {
                    command_surface: format!("bijux dev {}", tool.namespace),
                    binary: tool.control_binary.to_string(),
                    package: tool.control_package.to_string(),
                },
            })
            .collect()
    }

    fn env_map(&self) -> BTreeMap<String, String> {
        env_map().into_iter().collect()
    }

    fn active_paths(&self) -> dev_env::ActivePaths {
        dev_env::ActivePaths {
            config_file: self.paths.config_file.clone(),
            history_file: self.paths.history_file.clone(),
            plugins_dir: self.paths.plugins_dir.clone(),
        }
    }

    fn doctor_report_input(&self) -> DoctorReportInput {
        let install_report = install_health_report(
            &env::var("PATH").unwrap_or_default(),
            env::var("BIJUX_BIN").ok().as_deref(),
            env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
            env!("CARGO_PKG_VERSION"),
        );
        let plugin_diagnostics =
            load_time_diagnostics(self.plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
        let repository = FileConfigRepository;
        let config_issues =
            repository.load(&self.paths.config_file).err().map_or_else(Vec::new, |err| {
                vec![json!({"category":"config", "message": err.to_string()})]
            });
        let path_issues = if install_report.has_path_shadowing
            || install_report.has_duplicate_installs
        {
            vec![
                json!({"category":"paths", "has_path_shadowing": install_report.has_path_shadowing}),
                json!({"category":"paths", "has_duplicate_installs": install_report.has_duplicate_installs}),
            ]
        } else {
            Vec::new()
        };
        let plugin_issues: Vec<Value> = plugin_diagnostics
            .into_iter()
            .map(|diag| {
                json!({
                    "category": "plugins",
                    "namespace": diag.namespace,
                    "severity": diag.severity,
                    "message": diag.message,
                })
            })
            .collect();

        DoctorReportInput { config_issues, path_issues, plugin_issues }
    }

    fn state_audit_input(&self) -> StateAuditInput {
        let corruption = state_diagnostics(self.paths);
        let state_query = state_diagnostics_query(
            &self.paths.config_file,
            &self.paths.history_file,
            self.plugin_registry_path,
            &self.paths.memory_file,
        );
        StateAuditInput {
            path_status: dev_state_audit::StatePathStatusInput {
                config: state_path_status_value(&state_query.config),
                history: state_path_status_value(&state_query.history),
                plugins_registry: state_path_status_value(&state_query.plugins_registry),
                memory: state_path_status_value(&state_query.memory),
            },
            corruption_health: corruption,
        }
    }

    fn state_doctor_report(&self) -> Value {
        state_diagnostics(self.paths)
    }

    fn contracts_schema_input(&self) -> ContractsSchemaInput {
        let query = contracts_schema_query();
        ContractsSchemaInput { schema_ids: query.schema_ids, schema_version: query.schema_version }
    }

    fn runtime_identity_input(&self) -> dev_runtime_identity::RuntimeIdentityInput {
        let install_query = runtime_identity_query(
            &env::var("PATH").unwrap_or_default(),
            env::var("BIJUX_BIN").ok().as_deref(),
            env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
            env!("CARGO_PKG_VERSION"),
        );

        let python_bridge_supported = !matches!(
            env::var("BIJUX_PYTHON_BRIDGE_SUPPORTED"),
            Ok(value) if matches!(value.as_str(), "0" | "false" | "FALSE")
        );
        let cargo_canonical = cargo_install_strategy(PackageChannel::Canonical);
        let cargo_compat = cargo_install_strategy(PackageChannel::Compatibility);
        let pip_canonical = pip_install_strategy(PackageChannel::Canonical);
        let pip_compat = pip_install_strategy(PackageChannel::Compatibility);

        dev_runtime_identity::RuntimeIdentityInput {
            install_report: dev_runtime_identity::InstallHealthReport {
                active_binary: install_query.active_binary,
                path_binaries: install_query.path_binaries,
                has_path_shadowing: install_query.has_path_shadowing,
                has_duplicate_installs: install_query.has_duplicate_installs,
                stale_wrapper_maintenance: install_query.stale_wrapper_scripts,
                has_mismatched_wheel_binary_versions: install_query
                    .has_mismatched_wheel_binary_versions,
                legacy_installer_conflicts: install_query.legacy_installer_conflicts,
                active_binary_missing: install_query.active_binary_missing,
                broken_symlink_active_binary: install_query.broken_symlink_active_binary,
            },
            python_bridge_supported,
            cargo_canonical_package: cargo_canonical.package_name,
            cargo_compat_package: cargo_compat.package_name,
            pip_canonical_package: pip_canonical.package_name,
            pip_compat_package: pip_compat.package_name,
            canonical_crate_name: canonical_crate_name().to_string(),
        }
    }
}

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    registry: &RouteRegistry,
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    let runtime = RuntimeQueryAdapter { registry, paths, plugin_registry_path };
    dev_dispatch::try_handle(normalized_path, argv, &runtime)
}
