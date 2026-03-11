//! Built-in route execution and registry resolution.

use anyhow::Result;
use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::resolve_state_paths;
use crate::interface::cli::handlers::{
    cli as cli_handlers, config as config_handlers, history as history_handlers,
    memory as memory_handlers, plugins as plugins_handlers, root as root_handlers,
};
use crate::interface::cli::parser::ParsedGlobalFlags;
use crate::routing::registry::{RouteRegistry, RouteTarget};

pub(super) fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    let mut registry = RouteRegistry::default();
    let _ = registry.register_plugin_namespace("community");

    let target = registry.resolve(normalized_path)?;
    if matches!(target, RouteTarget::Plugin(_)) {
        return Ok(json!({
            "status": "ok",
            "route": normalized_path.join(" "),
            "owner": "plugin"
        }));
    }

    let paths = resolve_state_paths(global_flags)?;
    let plugin_registry_path = paths.plugin_registry_file.clone();

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
