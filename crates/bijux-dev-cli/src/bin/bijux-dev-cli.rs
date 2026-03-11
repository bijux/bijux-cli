#![forbid(unsafe_code)]
//! Standalone dev-cli executable delegated from `bijux dev cli ...`.

use std::collections::BTreeMap;
use std::env;
use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use bijux_cli::features::config::storage::validate_config_file;
use bijux_cli::features::diagnostics::state_diagnostics_query;
use bijux_cli::features::diagnostics::state_paths::{
    env_map, resolve_state_paths, state_diagnostics, state_path_status_value, ResolvedStatePaths,
};
use bijux_cli::features::install::{
    canonical_crate_name, cargo_install_strategy, install_health_report, pip_install_strategy,
    query::runtime_identity_query, PackageChannel,
};
use bijux_cli::features::plugins::{list_plugins, load_time_diagnostics};
use bijux_cli::interface::cli::parser::{parse_intent, ParsedGlobalFlags};
use bijux_cli::routing::catalog::is_known_route as is_known_catalog_route;
use bijux_cli::routing::inventory::{registry_inventory, route_inventory};
use bijux_cli::routing::query::contracts_schema_query;
use bijux_cli::routing::registry::RouteRegistry;
use bijux_cli::routing::{ColorMode, LogLevel, OutputFormat, PrettyMode, KNOWN_BIJUX_TOOLS};
use bijux_cli::shared::output::{render_value, EmitterConfig};
use bijux_dev_cli::cli::dispatch::{
    self as dev_dispatch, ContractsSchemaInput, DoctorReportInput, RouteInventoryQuery,
    RuntimeQueryProvider, StateAuditInput,
};
use bijux_dev_cli::reports::{
    control_plane::{ProductContractRow, ProductSurfaceRow},
    env as dev_env, registry as dev_registry, runtime_identity as dev_runtime_identity,
    state_audit as dev_state_audit,
};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppRunResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

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

        let config_issues = validate_config_file(&self.paths.config_file)
            .err()
            .map_or_else(Vec::new, |err| {
                vec![json!({"category":"config", "message": err})]
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

fn synthetic_root_argv(argv: &[String]) -> Vec<String> {
    let mut root = vec!["bijux".to_string()];
    if argv.get(1).map(String::as_str) == Some("dev")
        && argv.get(2).map(String::as_str) == Some("cli")
    {
        root.extend_from_slice(&argv[1..]);
    } else {
        root.push("dev".to_string());
        root.push("cli".to_string());
        root.extend_from_slice(&argv[1..]);
    }
    root
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var("NO_COLOR").ok().as_deref() == Some("1"),
    }
}

fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    let mut registry = RouteRegistry::default();
    let _ = registry.register_plugin_namespace("community");

    let paths = resolve_state_paths(global_flags)?;
    let runtime = RuntimeQueryAdapter {
        registry: &registry,
        paths: &paths,
        plugin_registry_path: &paths.plugin_registry_file,
    };

    if let Some(payload) = dev_dispatch::try_handle(normalized_path, argv, &runtime)? {
        return Ok(payload);
    }

    Ok(json!({"status": "error", "message": "unknown route"}))
}

fn maintenance_route_exit_code(normalized_path: &[String], payload: &Value) -> Option<i32> {
    let is_maintenance_runner = matches!(
        normalized_path,
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && (d == "generate" || d == "generate-all")
    ) || matches!(
        normalized_path,
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && (e == "run" || e == "run-all")
    );

    if !is_maintenance_runner {
        return None;
    }

    if payload
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status == "failed" || status == "error")
    {
        let exit_code =
            payload.get("exit_code").and_then(Value::as_i64).filter(|code| *code > 0).unwrap_or(1);
        return Some(exit_code as i32);
    }

    if payload.get("failed").and_then(Value::as_u64).is_some_and(|count| count > 0) {
        return Some(1);
    }

    if payload.get("results").and_then(Value::as_array).is_some_and(|rows| {
        rows.iter().any(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .is_some_and(|status| status == "failed" || status == "error")
        })
    }) {
        return Some(1);
    }

    Some(0)
}

fn with_trailing_newline(content: String) -> String {
    if content.ends_with('\n') {
        content
    } else {
        format!("{content}\n")
    }
}

fn run_app(argv: &[String]) -> Result<AppRunResult> {
    let synthetic_argv = synthetic_root_argv(argv);
    let intent = parse_intent(&synthetic_argv)?;

    if intent.normalized_path.is_empty() {
        let rendered = render_value(
            &json!({
                "status": "error",
                "code": 2,
                "message": "unknown route",
                "command": "dev cli",
            }),
            emitter_config(&ParsedGlobalFlags {
                output_format: None,
                pretty_mode: None,
                color_mode: None,
                log_level: None,
                quiet: false,
                config_path: None,
            }),
        )?;
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: with_trailing_newline(rendered),
        });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let payload =
        match route_response(&intent.normalized_path, &synthetic_argv, &intent.global_flags) {
            Ok(value) => value,
            Err(error) => {
                let message = error.to_string();
                let code = if message.contains("Missing argument")
                    || message.contains("Invalid argument")
                    || message.contains("Key cannot be empty")
                    || message.contains("Invalid key")
                    || message.contains("Unknown config section")
                    || message.contains("Config key not found")
                    || message.contains("Missing parameter")
                    || message.contains("Unsupported format")
                    || message.contains("Failed to load config")
                {
                    2
                } else if message.contains("Non-ASCII") || message.contains("Control characters") {
                    3
                } else {
                    1
                };
                let rendered_error = render_value(
                    &json!({
                        "status": "error",
                        "code": code,
                        "message": message,
                        "command": intent.normalized_path.join(" "),
                    }),
                    emitter_config(&intent.global_flags),
                )?;
                return Ok(AppRunResult {
                    exit_code: code,
                    stdout: String::new(),
                    stderr: with_trailing_newline(rendered_error),
                });
            }
        };

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = with_trailing_newline(rendered);

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    let route_exit_code = maintenance_route_exit_code(&intent.normalized_path, &payload).unwrap_or(0);

    if intent.global_flags.quiet {
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult { exit_code: route_exit_code, stdout: content, stderr: String::new() })
}

fn main() -> ExitCode {
    let argv: Vec<String> = env::args().collect();
    match run_app(&argv) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            ExitCode::from(result.exit_code.clamp(0, 255) as u8)
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
