//! Built-in route execution and registry resolution.

use anyhow::Result;
use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::resolve_state_paths;
use crate::features::plugins::list_plugins;
use crate::interface::cli::handlers::{
    cli as cli_handlers, config as config_handlers, history as history_handlers,
    memory as memory_handlers, plugins as plugins_handlers, root as root_handlers,
};
use crate::interface::cli::parser::ParsedGlobalFlags;
use crate::routing::registry::{RouteError, RouteRegistry, RouteTarget};

fn populate_plugin_namespaces(
    registry: &mut RouteRegistry,
    plugin_registry_path: &std::path::Path,
) {
    let _ = registry.register_plugin_namespace("community");
    if let Ok(installed_plugins) = list_plugins(plugin_registry_path) {
        for plugin in installed_plugins {
            let namespace = plugin.manifest.namespace.0;
            let _ = registry.register_plugin_namespace(&namespace);
        }
    }
}

fn should_preload_plugin_namespaces(normalized_path: &[String]) -> bool {
    matches!(normalized_path, [a] if a == "plugins")
        || matches!(normalized_path, [a, b] if a == "cli" && b == "inspect")
}

pub(super) fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    let paths = resolve_state_paths(global_flags)?;
    let plugin_registry_path = paths.plugin_registry_file.clone();

    let mut registry = RouteRegistry::default();
    let mut plugin_namespaces_loaded = false;
    if should_preload_plugin_namespaces(normalized_path) {
        populate_plugin_namespaces(&mut registry, &plugin_registry_path);
        plugin_namespaces_loaded = true;
    }

    let target = match registry.resolve(normalized_path) {
        Ok(target) => target,
        Err(RouteError::Unknown(_)) if !plugin_namespaces_loaded => {
            populate_plugin_namespaces(&mut registry, &plugin_registry_path);
            registry.resolve(normalized_path)?
        }
        Err(error) => return Err(error.into()),
    };
    if let RouteTarget::Plugin(namespace) = target {
        anyhow::bail!(
            "plugin route execution is not implemented: namespace={namespace}, route={}",
            normalized_path.join(" ")
        );
    }

    if let Some(payload) =
        config_handlers::execute_config_command(normalized_path, argv, &paths.config_file)?
    {
        return Ok(payload);
    }
    if let Some(payload) = history_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) = memory_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) =
        plugins_handlers::try_handle(normalized_path, argv, &paths, &plugin_registry_path)?
    {
        return Ok(payload);
    }
    if let Some(payload) =
        cli_handlers::try_handle(normalized_path, &paths, &registry, &plugin_registry_path)
    {
        return Ok(payload);
    }
    if let Some(payload) = root_handlers::try_handle(normalized_path, argv) {
        return Ok(payload);
    }

    Ok(json!({"status": "error", "message": "unknown route"}))
}
