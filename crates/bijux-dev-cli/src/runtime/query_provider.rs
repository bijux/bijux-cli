//! Runtime-query provider backed by `bijux-cli` runtime services.

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_cli::api::config::validate_config_file;
use bijux_cli::api::diagnostics::{
    registry_inventory, route_inventory, state_diagnostics_query,
    state_paths::{
        env_map, resolve_state_paths, state_diagnostics, state_path_status_value,
        ResolvedStatePaths,
    },
};
use bijux_cli::api::install::{
    canonical_crate_name, cargo_install_strategy, install_health_report, pip_install_strategy,
    query::runtime_identity_query,
};
use bijux_cli::api::parser::ParsedGlobalFlags;
use bijux_cli::api::plugins::{list_plugins, load_time_diagnostics};
use bijux_cli::api::routing::registry::RouteRegistry;
use bijux_cli::api::version::runtime_semver;
use bijux_cli::contracts::{contracts_schema_query, KNOWN_BIJUX_TOOLS};
use serde_json::{json, Value};

use crate::cli::dispatch::{
    ContractsSchemaInput, DoctorReportInput, RouteInventoryQuery, RuntimeQueryProvider,
    StateAuditInput,
};
use crate::reports::{
    control_plane::{ProductContractRow, ProductSurfaceRow},
    env as dev_env, registry as dev_registry, runtime_identity as dev_runtime_identity,
    state_audit as dev_state_audit,
};

/// Runtime context used to satisfy dev-cli runtime query interfaces.
#[derive(Debug, Clone)]
pub struct RuntimeQueryContext {
    registry: RouteRegistry,
    paths: ResolvedStatePaths,
    plugin_registry_path: PathBuf,
}

impl RuntimeQueryContext {
    /// Build runtime query context from resolved CLI global flags.
    pub fn from_flags(flags: &ParsedGlobalFlags) -> Result<Self> {
        let mut registry = RouteRegistry::default();
        let _ = registry.register_plugin_namespace("community");
        let paths = resolve_state_paths(flags)?;
        let plugin_registry_path = paths.plugin_registry_file.clone();

        Ok(Self { registry, paths, plugin_registry_path })
    }

    /// View this context as a dispatch runtime query provider.
    #[must_use]
    pub fn provider(&self) -> RuntimeQueryAdapter<'_> {
        RuntimeQueryAdapter {
            registry: &self.registry,
            paths: &self.paths,
            plugin_registry_path: &self.plugin_registry_path,
        }
    }
}

/// Runtime query adapter that maps `bijux-cli` query APIs into dev-cli contracts.
pub struct RuntimeQueryAdapter<'a> {
    registry: &'a RouteRegistry,
    paths: &'a ResolvedStatePaths,
    plugin_registry_path: &'a Path,
}

fn state_area_issues(report: &Value, area: &str) -> Vec<Value> {
    report
        .get("issues")
        .and_then(Value::as_array)
        .map(|issues| {
            issues
                .iter()
                .filter(|issue| issue.get("area").and_then(Value::as_str) == Some(area))
                .cloned()
                .collect()
        })
        .unwrap_or_default()
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
        match list_plugins(self.plugin_registry_path) {
            Ok(plugins) => {
                let mut rows = Vec::new();
                for plugin in plugins {
                    match serde_json::to_value(plugin) {
                        Ok(value) => rows.push(value),
                        Err(error) => rows.push(json!({
                            "_integrity_error": true,
                            "source": "plugin-serialization",
                            "message": error.to_string(),
                        })),
                    }
                }
                rows
            }
            Err(error) => vec![json!({
                "_integrity_error": true,
                "source": "plugin-registry",
                "message": error.to_string(),
            })],
        }
    }

    fn product_contracts(&self) -> Vec<ProductContractRow> {
        KNOWN_BIJUX_TOOLS
            .iter()
            .map(|tool| ProductContractRow {
                namespace: tool.namespace.to_string(),
                repository: tool.runtime_binary(),
                runtime: ProductSurfaceRow {
                    command_surface: format!("bijux {}", tool.namespace),
                    binary: tool.runtime_binary(),
                    package: tool.runtime_binary(),
                },
                control: ProductSurfaceRow {
                    command_surface: format!("bijux dev {}", tool.namespace),
                    binary: tool.control_binary(),
                    package: tool.control_binary(),
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
            runtime_semver(),
        );
        let state_report = state_diagnostics(self.paths);
        let history_issues = state_area_issues(&state_report, "history");
        let memory_issues = state_area_issues(&state_report, "memory");
        let mut plugin_issues = Vec::<Value>::new();
        let plugin_diagnostics =
            match load_time_diagnostics(self.plugin_registry_path, runtime_semver()) {
                Ok(diagnostics) => diagnostics,
                Err(error) => {
                    plugin_issues.push(json!({
                        "category": "plugins",
                        "severity": "error",
                        "message": format!("failed to load plugin diagnostics: {error}"),
                        "path": self.plugin_registry_path,
                    }));
                    Vec::new()
                }
            };

        let config_issues =
            validate_config_file(&self.paths.config_file).err().map_or_else(Vec::new, |message| {
                vec![json!({
                    "category": "config",
                    "message": message,
                    "path": self.paths.config_file,
                })]
            });

        let path_issues =
            if install_report.has_path_shadowing || install_report.has_duplicate_installs {
                vec![
                    json!({
                        "category": "paths",
                        "has_path_shadowing": install_report.has_path_shadowing,
                    }),
                    json!({
                        "category": "paths",
                        "has_duplicate_installs": install_report.has_duplicate_installs,
                    }),
                ]
            } else {
                Vec::new()
            };

        plugin_issues.extend(plugin_diagnostics.into_iter().map(|diagnostic| {
            json!({
                "category": "plugins",
                "namespace": diagnostic.namespace,
                "severity": diagnostic.severity,
                "message": diagnostic.message,
            })
        }));

        DoctorReportInput {
            config_issues,
            path_issues,
            plugin_issues,
            history_issues,
            memory_issues,
        }
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
            runtime_semver(),
        );

        let python_bridge_supported = !matches!(
            env::var("BIJUX_PYTHON_BRIDGE_SUPPORTED"),
            Ok(value) if matches!(value.as_str(), "0" | "false" | "FALSE")
        );
        let cargo_canonical = cargo_install_strategy();
        let pip_canonical = pip_install_strategy();
        let stale_wrapper_maintenance = install_query.stale_wrapper_maintenance();

        dev_runtime_identity::RuntimeIdentityInput {
            install_report: dev_runtime_identity::InstallHealthReport {
                active_binary: install_query.active_binary,
                path_binaries: install_query.path_binaries,
                has_path_shadowing: install_query.has_path_shadowing,
                has_duplicate_installs: install_query.has_duplicate_installs,
                stale_wrapper_maintenance,
                has_mismatched_wheel_binary_versions: install_query
                    .has_mismatched_wheel_binary_versions,
                legacy_installer_conflicts: install_query.legacy_installer_conflicts,
                active_binary_missing: install_query.active_binary_missing,
                broken_symlink_active_binary: install_query.broken_symlink_active_binary,
            },
            python_bridge_supported,
            cargo_canonical_package: cargo_canonical.package_name,
            pip_canonical_package: pip_canonical.package_name,
            canonical_crate_name: canonical_crate_name().to_string(),
        }
    }
}
