//! Top-level `dev` command handlers.

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::plugin::{list_plugins, FUTURE_PRODUCT_NAMESPACES};

pub(crate) fn try_handle(
    normalized_path: &[String],
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "dev" => Ok(Some(json!({
            "status": "ok",
            "entry_surface": "dev-cli",
            "recommended_command": "bijux dev cli status",
        }))),
        [a, b] if a == "dev" && b == "atlas" => Ok(Some(json!({
            "status": "ok",
            "mount": "atlas",
            "entry_surface": "dev-cli",
        }))),
        [a, b] if a == "dev" && b == "di" => Ok(Some(json!({
            "status": "ok",
            "container": "built-in",
            "entry_surface": "dev-cli",
        }))),
        [a, b] if a == "dev" && b == "list-products" => Ok(Some(json!({
            "status": "ok",
            "products": FUTURE_PRODUCT_NAMESPACES,
        }))),
        [a, b] if a == "dev" && b == "list-plugins" => Ok(Some(json!({
            "status": "ok",
            "plugins": list_plugins(plugin_registry_path)?,
        }))),
        _ => Ok(None),
    }
}
