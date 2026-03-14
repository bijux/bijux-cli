//! Built-in route execution and registry resolution.

use anyhow::Result;
use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::resolve_state_paths;
use crate::features::plugins::list_plugins;
use crate::features::plugins::runtime::{execute_plugin_route, PluginRouteOutput};
use crate::interface::cli::dispatch::AppRunResult;
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
    if let Ok(installed_plugins) = list_plugins(plugin_registry_path) {
        for plugin in installed_plugins {
            let namespace = plugin.manifest.namespace.0;
            let _ = registry
                .register_plugin_namespace_with_aliases(&namespace, &plugin.manifest.aliases);
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
) -> Result<RouteResponse> {
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
        let route_root = normalized_path.first().map(String::as_str).unwrap_or(namespace.as_str());
        return Ok(
            match execute_plugin_route(&plugin_registry_path, &namespace, route_root, argv)? {
                PluginRouteOutput::Structured(payload) => RouteResponse::Payload(payload),
                PluginRouteOutput::Process(result) => RouteResponse::Process(AppRunResult {
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                }),
            },
        );
    }

    if let Some(payload) =
        config_handlers::execute_config_command(normalized_path, argv, &paths.config_file)?
    {
        return Ok(RouteResponse::Payload(payload));
    }
    if let Some(payload) = history_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(RouteResponse::Payload(payload));
    }
    if let Some(payload) = memory_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(RouteResponse::Payload(payload));
    }
    if let Some(payload) =
        plugins_handlers::try_handle(normalized_path, argv, &paths, &plugin_registry_path)?
    {
        return Ok(RouteResponse::Payload(payload));
    }
    if let Some(payload) =
        cli_handlers::try_handle(normalized_path, argv, &paths, &registry, &plugin_registry_path)
    {
        return Ok(RouteResponse::Payload(payload));
    }
    if let Some(payload) =
        root_handlers::try_handle(normalized_path, argv, &paths, &plugin_registry_path)
    {
        return Ok(RouteResponse::Payload(payload));
    }

    Ok(RouteResponse::Payload(json!({"status": "error", "message": "unknown route"})))
}

pub(super) enum RouteResponse {
    Payload(Value),
    Process(AppRunResult),
}
