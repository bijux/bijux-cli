//! Backward-compatible shim for developer command handlers.

#[allow(dead_code)]
pub(crate) fn try_handle(
    normalized_path: &[String],
    plugin_registry_path: &std::path::Path,
) -> anyhow::Result<Option<serde_json::Value>> {
    crate::features::developer::command::try_handle(normalized_path, plugin_registry_path)
}
