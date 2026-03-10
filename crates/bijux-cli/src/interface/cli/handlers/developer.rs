//! Top-level `dev` command handlers.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::features::developer::operations::{
    atlas_mount_report, dependency_injection_report, entry_surface_report, plugin_inventory_report,
    product_namespaces_report,
};

pub(crate) fn try_handle(
    normalized_path: &[String],
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    match normalized_path {
        [a] if a == "dev" => Ok(Some(entry_surface_report())),
        [a, b] if a == "dev" && b == "atlas" => Ok(Some(atlas_mount_report())),
        [a, b] if a == "dev" && b == "di" => Ok(Some(dependency_injection_report())),
        [a, b] if a == "dev" && b == "list-products" => Ok(Some(product_namespaces_report())),
        [a, b] if a == "dev" && b == "list-plugins" => {
            Ok(Some(plugin_inventory_report(plugin_registry_path)?))
        }
        _ => Ok(None),
    }
}
