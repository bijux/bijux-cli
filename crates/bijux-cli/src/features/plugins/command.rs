//! Plugin command handlers.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use crate::argv::command_positionals;
use crate::cli::context::{
    command_has_flag, command_option_value, scaffold_plugin_layout, ResolvedStatePaths,
};
use crate::plugin::{
    compatibility_warnings, disable_plugin, enable_plugin, inspect_plugin,
    install_plugin as install_plugin_manifest, is_reserved_namespace, list_plugins,
    load_time_diagnostics, plugin_doctor, uninstall_plugin, validate_manifest,
    InstallPluginRequest, PluginTrustLevel, CORE_NAMESPACES, FUTURE_PRODUCT_NAMESPACES,
    RESERVED_NAMESPACES,
};

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "plugins" => {
            let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
            Ok(Some(json!({
                "status": "ok",
                "count": plugins.len(),
                "plugins": plugins,
                "directory": paths.plugins_dir,
            })))
        }
        [a, b] if a == "plugins" && b == "info" => {
            let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            Ok(Some(json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => Ok(Some(
            json!({"plugins": list_plugins(plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir}),
        )),
        [a, b, c] if a == "cli" && b == "plugins" && c == "info" => {
            let plugins = list_plugins(plugin_registry_path).unwrap_or_default();
            let warnings = compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION"))
                .unwrap_or_default();
            Ok(Some(json!({
                "status": "ok",
                "plugins": plugins,
                "compatibility_warnings": warnings,
                "registry_file": plugin_registry_path,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => Ok(Some(json!({
            "plugins": list_plugins(plugin_registry_path).unwrap_or_default(),
            "status": "loaded",
            "compatibility_warnings": compatibility_warnings(plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
        }))),
        [a, b, c] if a == "cli" && b == "plugins" && c == "check" => {
            let plugin =
                command_positionals(argv, &["cli", "plugins", "check"])
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin name required"))?;
            let record = inspect_plugin(plugin_registry_path, &plugin)?;
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
            Ok(Some(
                json!({"plugin": plugin, "status": "healthy", "state": format!("{:?}", record.state)}),
            ))
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
            Ok(Some(json!({
                "status": "scaffolded",
                "kind": kind,
                "namespace": namespace,
                "path": target,
                "manifest": manifest,
            })))
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
                plugin_registry_path,
                InstallPluginRequest { manifest_text, source, trust_level },
                env!("CARGO_PKG_VERSION"),
            )?;
            Ok(Some(json!({
                "status": "installed",
                "plugin": installed,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "uninstall" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "uninstall"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            uninstall_plugin(plugin_registry_path, &namespace)?;
            Ok(Some(json!({
                "status": "uninstalled",
                "namespace": namespace,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "enable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "enable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = enable_plugin(plugin_registry_path, &namespace)?;
            Ok(Some(json!({
                "status": "enabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "disable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "disable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("plugin namespace is required"))?;
            let record = disable_plugin(plugin_registry_path, &namespace)?;
            Ok(Some(json!({
                "status": "disabled",
                "namespace": namespace,
                "state": format!("{:?}", record.state),
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "doctor" => {
            let repaired = crate::plugin::self_repair_registry(plugin_registry_path).is_ok();
            let report = plugin_doctor(plugin_registry_path)?;
            Ok(Some(json!({
                "status": "ok",
                "doctor": {
                    "installed": report.installed,
                    "broken": report.broken,
                    "incompatible": report.incompatible,
                },
                "self_repair_attempted": true,
                "self_repair_success": repaired,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "reserved-names" => Ok(Some(json!({
            "reserved_namespaces": RESERVED_NAMESPACES,
            "core_namespaces": CORE_NAMESPACES,
            "future_product_namespaces": FUTURE_PRODUCT_NAMESPACES,
        }))),
        [a, b, c] if a == "cli" && b == "plugins" && c == "where" => Ok(Some(json!({
            "plugins_dir": paths.plugins_dir,
            "registry_file": plugin_registry_path,
        }))),
        [a, b, c] if a == "cli" && b == "plugins" && c == "explain" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "explain"]).first().cloned();
            let diagnostics =
                load_time_diagnostics(plugin_registry_path, env!("CARGO_PKG_VERSION"))
                    .unwrap_or_default();
            let report = plugin_doctor(plugin_registry_path).ok();
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
            Ok(Some(json!({
                "plugin": plugin,
                "diagnostics": filtered,
                "summary": summary,
            })))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "schema" => Ok(Some(json!({
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
        }))),
        _ => Ok(None),
    }
}
