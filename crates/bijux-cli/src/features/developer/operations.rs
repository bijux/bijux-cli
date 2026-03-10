#![forbid(unsafe_code)]
//! Developer-facing feature operations.

use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};

use crate::features::plugins::{list_plugins, FUTURE_PRODUCT_NAMESPACES};

pub(crate) fn entry_surface_report() -> Value {
    json!({
        "status": "ok",
        "entry_surface": "dev-cli",
        "recommended_command": "bijux dev cli status",
    })
}

pub(crate) fn atlas_mount_report() -> Value {
    json!({
        "status": "ok",
        "mount": "atlas",
        "entry_surface": "dev-cli",
    })
}

pub(crate) fn dependency_injection_report() -> Value {
    json!({
        "status": "ok",
        "container": "built-in",
        "entry_surface": "dev-cli",
    })
}

pub(crate) fn product_namespaces_report() -> Value {
    json!({
        "status": "ok",
        "products": FUTURE_PRODUCT_NAMESPACES,
    })
}

pub(crate) fn plugin_inventory_report(plugin_registry_path: &Path) -> Result<Value> {
    Ok(json!({
        "status": "ok",
        "plugins": list_plugins(plugin_registry_path)?,
    }))
}
