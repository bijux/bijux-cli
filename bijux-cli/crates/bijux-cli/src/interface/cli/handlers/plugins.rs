//! Plugin command handlers.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::features::plugins::operations as plugin_operations;
use crate::features::plugins::PluginTrustLevel;
use crate::shared::argv::{command_has_flag, command_option_value, command_positionals};

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "plugins" => {
            Ok(Some(plugin_operations::plugins_overview(plugin_registry_path, &paths.plugins_dir)))
        }
        [a, b] if a == "plugins" && b == "info" => {
            Ok(Some(plugin_operations::plugins_info(plugin_registry_path, &paths.plugins_dir)))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            Ok(Some(plugin_operations::plugins_list(plugin_registry_path, &paths.plugins_dir)))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "info" => {
            Ok(Some(plugin_operations::plugins_info(plugin_registry_path, &paths.plugins_dir)))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "inspect"]).first().cloned();
            Ok(Some(plugin_operations::plugins_inspect(plugin_registry_path, plugin.as_deref())?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "check" => {
            let plugin =
                command_positionals(argv, &["cli", "plugins", "check"])
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin name required"))?;
            Ok(Some(plugin_operations::check_plugin_health(plugin_registry_path, &plugin)?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "scaffold" => {
            let positional = command_positionals(argv, &["cli", "plugins", "scaffold"]);
            let kind = positional
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin kind required"))?;
            let namespace = positional
                .get(1)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin namespace required"))?;
            let force = command_has_flag(argv, "--force");
            let target = command_option_value(argv, &["cli", "plugins", "scaffold"], "--path")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(&namespace)
                });

            Ok(Some(plugin_operations::scaffold_plugin(&kind, &namespace, force, &target)?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "install" => {
            let manifest_arg = command_positionals(argv, &["cli", "plugins", "install"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: manifest path required"))?;
            let manifest_path = PathBuf::from(&manifest_arg);
            let default_source = manifest_path
                .canonicalize()
                .unwrap_or_else(|_| manifest_path.clone())
                .to_string_lossy()
                .into_owned();
            let source = command_option_value(argv, &["cli", "plugins", "install"], "--source")
                .unwrap_or(default_source);
            let trust = command_option_value(argv, &["cli", "plugins", "install"], "--trust");

            Ok(Some(plugin_operations::install_plugin_from_manifest(
                plugin_registry_path,
                &manifest_path,
                Some(&source),
                parse_trust_level(trust.as_deref())?,
            )?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "uninstall" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "uninstall"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin namespace required"))?;
            Ok(Some(plugin_operations::uninstall_plugin_namespace(
                plugin_registry_path,
                &namespace,
            )?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "enable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "enable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin namespace required"))?;
            Ok(Some(plugin_operations::enable_plugin_namespace(plugin_registry_path, &namespace)?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "disable" => {
            let namespace = command_positionals(argv, &["cli", "plugins", "disable"])
                .first()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Missing argument: plugin namespace required"))?;
            Ok(Some(plugin_operations::disable_plugin_namespace(plugin_registry_path, &namespace)?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "doctor" => {
            Ok(Some(plugin_operations::plugin_doctor_report(plugin_registry_path)?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "reserved-names" => {
            Ok(Some(plugin_operations::reserved_namespaces_report()))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "where" => Ok(Some(
            plugin_operations::plugin_locations_report(&paths.plugins_dir, plugin_registry_path),
        )),
        [a, b, c] if a == "cli" && b == "plugins" && c == "explain" => {
            let plugin = command_positionals(argv, &["cli", "plugins", "explain"]).first().cloned();
            Ok(Some(plugin_operations::explain_plugin_report(
                plugin_registry_path,
                plugin.as_deref(),
            )?))
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "schema" => {
            Ok(Some(plugin_operations::plugin_schema_report()))
        }
        _ => Ok(None),
    }
}

fn parse_trust_level(raw: Option<&str>) -> Result<PluginTrustLevel> {
    match raw.unwrap_or("community") {
        "core" => Ok(PluginTrustLevel::Core),
        "verified" => Ok(PluginTrustLevel::Verified),
        "community" => Ok(PluginTrustLevel::Community),
        "unknown" => Ok(PluginTrustLevel::Unknown),
        invalid => Err(anyhow::anyhow!(
            "Invalid argument: trust level must be one of: core, verified, community, unknown; got {invalid}"
        )),
    }
}
