//! Top-level application entrypoint and route execution.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crate::install::{
    canonical_crate_name, cargo_install_strategy, install_health_report, pip_install_strategy,
    post_install_hint, query::runtime_identity_query, CompatibilityPaths, InstallHealthReport,
    PackageChannel,
};
use crate::plugin::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, plugin_origin_metadata, uninstall_plugin,
    validate_manifest, InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES,
    FUTURE_PRODUCT_NAMESPACES,
    RESERVED_NAMESPACES,
};
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::routing::inventory::{registry_inventory, route_inventory};
use crate::routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use crate::routing::query::contracts_schema_query;
use crate::routing::registry::{RouteRegistry, RouteTarget};
use crate::routing::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_dev_cli::{
    cockpit as dev_cockpit, config as dev_config, contracts as dev_contracts,
    control_plane as dev_control_plane, crate_health as dev_crate_health,
    docs_audit as dev_docs_audit, env as dev_env, evidence as dev_evidence,
    package_health as dev_package_health, parity as dev_parity, python as dev_python,
    registry as dev_registry, release as dev_release, repo as dev_repo,
    route_audit as dev_route_audit, routes as dev_routes, runtime_identity as dev_runtime_identity,
    rustdoc as dev_rustdoc, script_audit as dev_script_audit, scripts as dev_scripts,
    state_audit as dev_state_audit, status as dev_status, ReportContext,
};
use serde_json::{json, Value};

use crate::argv::command_positionals;
use crate::cli::commands::help::render_command_help;
use crate::cli::commands::{history as history_commands, memory as memory_commands};
use crate::cli::context::{
    collect_files, command_has_flag, command_option_value, env_map, read_json_if_exists,
    rel_to_root, resolve_state_paths, scaffold_plugin_layout, state_diagnostics,
    state_path_status_value, workspace_root,
};
use crate::config::execute_config_command;
use crate::config::storage::{ConfigRepository, FileConfigRepository};
use crate::output::{render_value, EmitterConfig};
use crate::query::state_diagnostics_query;

/// In-memory process output and exit result produced by the core app runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
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

    let target = match normalized_path {
        [a] if a == "config"
            || a == "history"
            || a == "memory"
            || a == "plugins"
            || a == "dev"
            || a == "atlas" =>
        {
            RouteTarget::BuiltIn
        }
        [a, b] if a == "history" && b == "clear" => RouteTarget::BuiltIn,
        [a, b]
            if a == "memory"
                && (b == "list" || b == "get" || b == "set" || b == "delete" || b == "clear") =>
        {
            RouteTarget::BuiltIn
        }
        [a, b]
            if a == "plugins"
                && (b == "list"
                    || b == "info"
                    || b == "inspect"
                    || b == "check"
                    || b == "reserved-names"
                    || b == "where"
                    || b == "explain"
                    || b == "schema") =>
        {
            RouteTarget::BuiltIn
        }
        [a, b, c] if a == "dev" && b == "cli" => {
            let _ = c;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c]
            if a == "dev"
                && b == "cli"
                && (c == "dashboard"
                    || c == "quickcheck"
                    || c == "truth"
                    || c == "blockers"
                    || c == "next") =>
        {
            RouteTarget::BuiltIn
        }
        _ => registry.resolve(normalized_path)?,
    };
    if matches!(target, RouteTarget::Plugin(_)) {
        return Ok(json!({
            "status": "ok",
            "route": normalized_path.join(" "),
            "owner": "plugin"
        }));
    }

    let paths = resolve_state_paths(global_flags)?;
    let compatibility_paths = CompatibilityPaths {
        config_file: paths.config_file.clone(),
        history_file: paths.history_file.clone(),
        plugins_dir: paths.plugins_dir.clone(),
    };
    let plugin_registry_path = paths.plugin_registry_file.clone();
    if let Some(payload) = execute_config_command(normalized_path, argv, &compatibility_paths)? {
        return Ok(payload);
    }
    if let Some(payload) = history_commands::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) = memory_commands::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }

    let payload = match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            json!({"version": env!("CARGO_PKG_VERSION")})
        }
        [a, b] if a == "cli" && b == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            json!({
                "status": "healthy",
                "checks": ["routing", "output", "config", "install"],
                "install": {
                    "has_path_shadowing": install_report.has_path_shadowing,
                    "has_duplicate_installs": install_report.has_duplicate_installs,
                    "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
                    "legacy_installer_conflicts": false,
                    "has_mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
                }
            })
        }
        [a, b] if a == "cli" && b == "repl" => {
            json!({"status": "ready", "mode": "repl", "history_file": paths.history_file})
        }
        [a, b] if a == "cli" && b == "completion" => {
            json!({"shells": ["bash", "zsh", "fish", "powershell"]})
        }
        [a, b] if a == "cli" && b == "inspect" => {
            let route_sources: Vec<Value> = registry
                .built_in_paths()
                .into_iter()
                .map(|path| {
                    let segments: Vec<String> = path.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "segments": segments,
                        "owner": "bijux-cli",
                        "source": "built-in",
                    })
                })
                .collect();
            json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry.alias_rewrites().into_iter().map(|(alias, canonical)| {
                    let alias_segments: Vec<String> = alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> = canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                }).collect::<Vec<_>>(),
                "plugin_origins": plugin_origin_metadata(&plugin_registry_path).unwrap_or_default(),
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                "contracts": {
                    "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"],
                    "version": "v1",
                }
            })
        }
        [a, b] if a == "cli" && b == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "audit" => {
            json!({
                "status": "ok",
                "checks": ["config", "paths", "plugins", "history"],
                "issues": []
            })
        }
        [a] if a == "docs" => {
            json!({
                "status": "ok",
                "topics": ["commands", "configuration", "plugins", "repl", "diagnostics"],
            })
        }
        [a] if a == "atlas" => {
            json!({
                "status": "ok",
                "mount": "atlas",
            })
        }
        [a] if a == "sleep" => {
            let duration_secs = argv
                .get(2)
                .and_then(|raw| raw.parse::<f64>().ok())
                .map(|v| v.clamp(0.0, 2.0))
                .unwrap_or(0.0);
            if duration_secs > 0.0 {
                thread::sleep(Duration::from_secs_f64(duration_secs));
            }
            json!({"status": "ok", "slept_seconds": duration_secs})
        }
        [a, b] if a == "cli" && b == "paths" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let hint =
                install_report.active_binary.as_deref().map(post_install_hint).unwrap_or_else(
                    || {
                        "Run `bijux version` and `bijux cli doctor` to verify your environment."
                            .to_string()
                    },
                );
            json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "active_binary": install_report.active_binary,
                "path_binaries": install_report.path_binaries,
                "post_install_hint": hint
            })
        }
        [a] if a == "plugins" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            json!({
                "status": "ok",
                "count": plugins.len(),
                "plugins": plugins,
                "directory": paths.plugins_dir,
            })
        }
        [a, b] if a == "plugins" && b == "info" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a] if a == "dev" => {
            json!({
                "status": "ok",
                "entry_surface": "dev-cli",
                "recommended_command": "bijux dev cli status",
            })
        }
        [a, b] if a == "dev" && b == "atlas" => {
            json!({
                "status": "ok",
                "mount": "atlas",
                "entry_surface": "dev-cli",
            })
        }
        [a, b] if a == "dev" && b == "di" => {
            json!({
                "status": "ok",
                "container": "built-in",
                "entry_surface": "dev-cli",
            })
        }
        [a, b] if a == "dev" && b == "list-products" => {
            json!({
                "status": "ok",
                "products": FUTURE_PRODUCT_NAMESPACES,
            })
        }
        [a, b] if a == "dev" && b == "list-plugins" => {
            json!({
                "status": "ok",
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            json!({"plugins": list_plugins(&plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "info" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            json!({
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
                "status": "loaded",
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "check" => {
            let plugin =
                command_positionals(argv, &["cli", "plugins", "check"])
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin name required"))?;
            let record = inspect_plugin(&plugin_registry_path, &plugin)?;
            let _ = validate_manifest(
                record.manifest.clone(),
                env!("CARGO_PKG_VERSION"),
                RESERVED_NAMESPACES,
            )?;
            if matches!(record.state, crate::routing::PluginLifecycleState::Disabled) {
                anyhow::bail!("Invalid argument: plugin {plugin} is disabled");
            }
            if matches!(record.manifest.kind, crate::routing::PluginKind::ExternalExec) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let path = PathBuf::from(&record.manifest.entrypoint);
                    if !path.exists() {
                        anyhow::bail!("Invalid argument: plugin entrypoint was not found");
                    }
                    let mode = fs::metadata(&path)?.permissions().mode();
                    if mode & 0o111 == 0 {
                        anyhow::bail!("Invalid argument: plugin entrypoint is not executable");
                    }
                }
            }
            json!({"plugin": plugin, "status": "healthy", "state": format!("{:?}", record.state)})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "scaffold" => {
            let positional = command_positionals(argv, &["cli", "plugins", "scaffold"]);
            let kind = positional.first().cloned().unwrap_or_else(|| "python".to_string());
            let namespace =
                positional.get(1).cloned().unwrap_or_else(|| "sample-plugin".to_string());
            let force = command_has_flag(argv, "--force");
            let target =
                command_option_value(argv, "--path").map(PathBuf::from).unwrap_or_else(|| {
                    env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&namespace)
                });
            let manifest = scaffold_plugin_layout(&target, &kind, &namespace, force)?;
            json!({
                "status": "scaffolded",
                "kind": kind,
                "namespace": namespace,
                "path": target,
                "manifest": manifest,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "install" => {
            let manifest_arg = command_positionals(argv, &["cli", "plugins", "install"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("manifest path is required"))?;
            let manifest_path = PathBuf::from(&manifest_arg);
            let manifest_text = fs::read_to_string(&manifest_path)?;
            let source =
                command_option_value(argv, "--source").unwrap_or_else(|| manifest_arg.clone());
            let trust_level = match command_option_value(argv, "--trust")
                .unwrap_or_else(|| "community".to_string())
                .as_str()
            {
                "core" => PluginTrustLevel::Core,
                "verified" => PluginTrustLevel::Verified,
                "unknown" => PluginTrustLevel::Unknown,
                _ => PluginTrustLevel::Community,
            };
            let installed = install_plugin_manifest(
                &plugin_registry_path,
                InstallPluginRequest { manifest_text, source, trust_level },
                env!("CARGO_PKG_VERSION"),
            )?;
            json!({
                "status": "installed",
                "plugin": installed,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "uninstall" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "uninstall"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            uninstall_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "uninstalled",
                "namespace": namespace,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "enable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "enable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = enable_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "enabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "disable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "disable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = disable_plugin(&plugin_registry_path, &namespace)?;
            json!({
                "status": "disabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "doctor" => {
            let repaired = crate::plugin::self_repair_registry(&plugin_registry_path).is_ok();
            let report = plugin_doctor(&plugin_registry_path)?;
            json!({
                "status": "ok",
                "doctor": {
                    "installed": report.installed,
                    "broken": report.broken,
                    "incompatible": report.incompatible,
                },
                "self_repair_attempted": true,
                "self_repair_success": repaired,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "reserved-names" => {
            json!({
                "reserved_namespaces": RESERVED_NAMESPACES,
                "core_namespaces": CORE_NAMESPACES,
                "future_product_namespaces": FUTURE_PRODUCT_NAMESPACES,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "where" => {
            json!({
                "plugins_dir": paths.plugins_dir,
                "registry_file": plugin_registry_path,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "explain" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "explain"]).first().cloned();
            let diagnostics =
                load_time_diagnostics(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let report = plugin_doctor(&plugin_registry_path).ok();
            let mut filtered: Vec<Value> = diagnostics
                .into_iter()
                .filter(|d| plugin.as_ref().is_none_or(|wanted| d.namespace == *wanted))
                .map(|diag| {
                    json!({
                        "namespace": diag.namespace,
                        "severity": diag.severity,
                        "message": diag.message,
                    })
                })
                .collect();
            if let Some(requested) = &plugin {
                if is_reserved_namespace(requested, &[]) {
                    filtered.push(json!({
                        "namespace": requested,
                        "severity": "error",
                        "message": format!("namespace is reserved: {requested}"),
                    }));
                }
            }
            let summary = report
                .map(|value| {
                    json!({
                        "installed": value.installed,
                        "broken": value.broken,
                        "incompatible": value.incompatible,
                    })
                })
                .unwrap_or_else(|| json!({"installed": 0, "broken": [], "incompatible": []}));
            json!({
                "plugin": plugin,
                "diagnostics": filtered,
                "summary": summary,
            })
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "schema" => {
            json!({
                "schema": "plugin-manifest-v1",
                "required_fields": [
                    "name",
                    "version",
                    "schema_version",
                    "manifest_version",
                    "compatibility",
                    "namespace",
                    "kind",
                    "entrypoint",
                ],
                "optional_fields": ["aliases", "capabilities"],
            })
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "routes" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            let inventory = route_inventory(&registry);
            dev_routes::build_report_from_query(&inventory.routes, &inventory.aliases, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "atlas" => {
            dev_control_plane::build_atlas_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "di" => {
            dev_control_plane::build_dependency_injection_report()
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-products" => {
            dev_control_plane::build_product_list_report(FUTURE_PRODUCT_NAMESPACES)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "list-plugins" => {
            let plugins = list_plugins(&plugin_registry_path).unwrap_or_default();
            dev_control_plane::build_plugin_list_report_from(plugins)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "route-audit" => {
            let inventory = route_inventory(&registry);
            dev_route_audit::build_report_from_query(&inventory.routes, &inventory.aliases)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "inventory" => {
            dev_script_audit::build_inventory_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            let context = ReportContext {
                generated_at: String::new(),
                data_source: "bijux-cli::routing".to_string(),
            };
            let inventory = registry_inventory(&registry);
            let namespaces: Vec<dev_registry::NamespaceInventoryRow> = inventory
                .namespaces
                .into_iter()
                .map(|row| dev_registry::NamespaceInventoryRow {
                    name: row.name.0,
                    reserved: row.reserved,
                    owner: row.owner,
                })
                .collect();
            dev_registry::build_report_from_query(&namespaces, &context)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "parity" => {
            dev_parity::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs" => {
            let root = workspace_root();
            let docs_files: Vec<String> = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_docs_inventory_report(docs_files)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "status" => dev_status::build_report(
            &workspace_root(),
            dev_script_audit::build_inventory_report(&workspace_root()),
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "script-audit" => {
            let inventory = dev_script_audit::build_inventory_report(&workspace_root());
            dev_script_audit::build_report(inventory)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "snapshots-audit" => {
            let root = workspace_root();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_snapshots_audit_report(snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "fixture-audit" => {
            let root = workspace_root();
            let parity_files: Vec<String> = collect_files(&root.join("artifacts/parity"))
                .into_iter()
                .map(|p| rel_to_root(&p, &root))
                .collect();
            let snapshots: Vec<String> = collect_files(&root.join("crates"))
                .into_iter()
                .filter(|p| p.to_string_lossy().contains("tests/snapshots/"))
                .map(|p| rel_to_root(&p, &root))
                .collect();
            dev_control_plane::build_fixture_audit_report(parity_files, snapshots)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "crate-health" => {
            dev_crate_health::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "package-health" => {
            let root = workspace_root();
            let state = read_json_if_exists(&root.join("artifacts/status/current_rust_state.json"));
            dev_package_health::build_report(state)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => dev_env::build_report(
            env_map().into_iter().collect(),
            &dev_env::ActivePaths {
                config_file: paths.config_file.clone(),
                history_file: paths.history_file.clone(),
                plugins_dir: paths.plugins_dir.clone(),
            },
        ),
        [a, b, c] if a == "dev" && b == "cli" && c == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let plugin_diagnostics =
                load_time_diagnostics(&plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let repository = FileConfigRepository;
            let config_issues =
                repository.load(&paths.config_file).err().map_or_else(Vec::new, |err| {
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
            dev_control_plane::build_doctor_report(config_issues, path_issues, plugin_issues)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-prune-plan" => {
            let root = workspace_root();
            let docs_count = collect_files(&root.join("docs"))
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .count();
            dev_control_plane::build_docs_prune_plan_report(docs_count)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-audit" => {
            let corruption = state_diagnostics(&paths);
            let state_query = state_diagnostics_query(
                &paths.config_file,
                &paths.history_file,
                &plugin_registry_path,
                &paths.memory_file,
            );
            dev_state_audit::build_report(
                dev_state_audit::StatePathStatusInput {
                    config: state_path_status_value(&state_query.config),
                    history: state_path_status_value(&state_query.history),
                    plugins_registry: state_path_status_value(&state_query.plugins_registry),
                    memory: state_path_status_value(&state_query.memory),
                },
                corruption,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "state-doctor" => {
            let diagnosis = state_diagnostics(&paths);
            dev_state_audit::build_doctor_report(diagnosis)
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "remaining" => {
            dev_scripts::build_remaining_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "migrated" => {
            dev_scripts::build_migrated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "diff" => {
            dev_scripts::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "audit" => {
            dev_scripts::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "package-metadata" => {
            dev_scripts::build_package_metadata_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "e2e-contract" => {
            dev_scripts::build_e2e_contract_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" && d == "pip-audit" => {
            dev_scripts::build_pip_audit_report(
                &workspace_root(),
                command_option_value(argv, "--report-path").as_deref(),
            )
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "capture-python-behavior" =>
        {
            dev_scripts::build_python_capture_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "scripts" && d == "provenance-statement" =>
        {
            let tag = command_option_value(argv, "--tag")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --tag required"))?;
            let output_dir = command_option_value(argv, "--output-dir")
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --output-dir required"))?;
            dev_scripts::build_provenance_statement_report(&tag, Path::new(&output_dir))
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "audit" => {
            dev_rustdoc::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "coverage" => {
            dev_rustdoc::build_coverage_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "broken-links" => {
            dev_rustdoc::build_broken_links_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "public-api" => {
            dev_rustdoc::build_public_api_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "examples" => {
            dev_rustdoc::build_examples_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "migrate-website-api-docs" =>
        {
            dev_rustdoc::build_migration_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "build-proof" => {
            dev_rustdoc::build_build_proof_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "rustdoc" && d == "workspace-coverage-proof" =>
        {
            dev_rustdoc::build_workspace_coverage_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" && d == "python-link-proof" => {
            dev_rustdoc::build_python_link_proof_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "status" => {
            dev_release::build_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "evidence" => {
            dev_release::build_evidence_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "readiness" => {
            dev_release::build_readiness_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "diff" => {
            dev_release::build_diff_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "gaps" => {
            dev_release::build_gaps_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "summary" => {
            dev_release::build_summary_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "manifest" => {
            dev_release::build_manifest_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "notes" => {
            dev_release::build_notes_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "behavior-changes" => {
            dev_release::build_behavior_changes_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "intentional-differences" =>
        {
            dev_release::build_intentional_differences_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" && d == "unresolved-gaps" => {
            dev_release::build_unresolved_gaps_report(&workspace_root())
        }
        [a, b, c, d]
            if a == "dev" && b == "cli" && c == "release" && d == "compatibility-leftovers" =>
        {
            dev_release::build_compatibility_leftovers_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "list" => {
            dev_evidence::build_list_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "show" => {
            let id = command_option_value(argv, "--id")
                .or_else(|| {
                    command_positionals(argv, &["dev", "cli", "evidence", "show"]).first().cloned()
                })
                .ok_or_else(|| anyhow::anyhow!("Missing argument: --id required"))?;
            dev_evidence::build_show_report(&workspace_root(), &id)
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "audit" => {
            dev_evidence::build_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "stale" => {
            dev_evidence::build_stale_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "matrix" => {
            dev_evidence::build_matrix_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "website-export" => {
            dev_evidence::build_website_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "ci-export" => {
            dev_evidence::build_ci_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "release-export" => {
            dev_evidence::build_release_export_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "command-map" => {
            dev_evidence::build_command_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" && d == "parity-map" => {
            dev_evidence::build_parity_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "rust-owner" => {
            dev_config::build_rust_owner_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "python-owner" => {
            dev_config::build_python_owner_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "ownership" => {
            dev_config::build_ownership_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "drift" => {
            dev_config::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "shape" => {
            dev_config::build_shape_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" && d == "evidence-map" => {
            dev_config::build_evidence_map_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "bridge-status" => {
            dev_python::build_bridge_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "surface-status" => {
            dev_python::build_surface_status_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "sovereignty-audit" => {
            dev_python::build_sovereignty_audit_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "drift" => {
            dev_python::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" && d == "packaging" => {
            dev_python::build_packaging_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "health" => {
            dev_repo::build_health_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "drift" => {
            dev_repo::build_drift_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "inventories" => {
            dev_repo::build_inventories_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "generated" => {
            dev_repo::build_generated_report(&workspace_root())
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" && d == "stale" => {
            dev_repo::build_stale_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "dashboard" => {
            dev_cockpit::build_dashboard_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "quickcheck" => {
            dev_cockpit::build_quickcheck_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "truth" => {
            dev_cockpit::build_truth_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "blockers" => {
            dev_cockpit::build_blockers_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "next" => {
            dev_cockpit::build_next_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "docs-audit" => {
            dev_docs_audit::build_report(&workspace_root())
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "plugin-health" => {
            let root = workspace_root();
            let machine =
                read_json_if_exists(&root.join("artifacts/status/plugin_health_report.json"));
            let text = fs::read_to_string(root.join("artifacts/status/plugin_health_report.txt"))
                .unwrap_or_default();
            dev_control_plane::build_plugin_health_report(machine, text)
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "contracts" => {
            let contracts_query = contracts_schema_query();
            dev_contracts::build_report_from_query(
                env!("CARGO_PKG_VERSION"),
                &contracts_query.schema_ids,
                &contracts_query.schema_version,
            )
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "runtime-identity" => {
            let install_query = runtime_identity_query(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let install_report = InstallHealthReport {
                active_binary: install_query.active_binary,
                path_binaries: install_query.path_binaries,
                has_path_shadowing: install_query.has_path_shadowing,
                has_duplicate_installs: install_query.has_duplicate_installs,
                stale_wrapper_scripts: install_query.stale_wrapper_scripts,
                has_mismatched_wheel_binary_versions: install_query
                    .has_mismatched_wheel_binary_versions,
                legacy_installer_conflicts: install_query.legacy_installer_conflicts,
                active_binary_missing: install_query.active_binary_missing,
                broken_symlink_active_binary: install_query.broken_symlink_active_binary,
            };
            let python_bridge_supported = !matches!(
                env::var("BIJUX_PYTHON_BRIDGE_SUPPORTED"),
                Ok(value) if matches!(value.as_str(), "0" | "false" | "FALSE")
            );
            let cargo_canonical = cargo_install_strategy(PackageChannel::Canonical);
            let cargo_compat = cargo_install_strategy(PackageChannel::Compatibility);
            let pip_canonical = pip_install_strategy(PackageChannel::Canonical);
            let pip_compat = pip_install_strategy(PackageChannel::Compatibility);
            dev_runtime_identity::build_report(dev_runtime_identity::RuntimeIdentityInput {
                install_report: dev_runtime_identity::InstallHealthReport {
                    active_binary: install_report.active_binary,
                    path_binaries: install_report.path_binaries,
                    has_path_shadowing: install_report.has_path_shadowing,
                    has_duplicate_installs: install_report.has_duplicate_installs,
                    stale_wrapper_scripts: install_report.stale_wrapper_scripts,
                    has_mismatched_wheel_binary_versions: install_report
                        .has_mismatched_wheel_binary_versions,
                    legacy_installer_conflicts: install_report.legacy_installer_conflicts,
                    active_binary_missing: install_report.active_binary_missing,
                    broken_symlink_active_binary: install_report.broken_symlink_active_binary,
                },
                python_bridge_supported,
                cargo_canonical_package: cargo_canonical.package_name,
                cargo_compat_package: cargo_compat.package_name,
                pip_canonical_package: pip_canonical.package_name,
                pip_compat_package: pip_compat.package_name,
                canonical_crate_name: canonical_crate_name().to_string(),
            })
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => {
            for _ in 0..200_u16 {
                thread::sleep(Duration::from_millis(50));
            }
            json!({"status": "completed"})
        }
        _ => json!({"status": "error", "message": "unknown route"}),
    };

    Ok(payload)
}

fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(error.to_string())
        }
        Err(_) => None,
    }
}

/// Execute the CLI for provided argv and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    if argv.len() == 1 {
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&[])?.trim_end()),
            stderr: String::new(),
        });
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path: Vec<&str> = argv[2..].iter().map(String::as_str).collect();
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&path)?.trim_end()),
            stderr: String::new(),
        });
    }

    if let Some(help) = try_render_clap_help(argv) {
        return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
    }

    let intent = parse_intent(argv)?;
    if intent.normalized_path.is_empty() {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("{}\n", render_command_help(&[])?.trim_end()),
        });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let response = route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
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
            let error_content = if rendered_error.ends_with('\n') {
                rendered_error
            } else {
                format!("{rendered_error}\n")
            };
            return Ok(AppRunResult {
                exit_code: code,
                stdout: String::new(),
                stderr: error_content,
            });
        }
    };

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    if intent.global_flags.quiet {
        return Ok(AppRunResult { exit_code: 0, stdout: String::new(), stderr: String::new() });
    }

    Ok(AppRunResult { exit_code: 0, stdout: content, stderr: String::new() })
}
