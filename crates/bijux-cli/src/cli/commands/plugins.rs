//! Backward-compatible shim for plugin command handlers.

#[allow(dead_code)]
pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &crate::cli::context::ResolvedStatePaths,
    plugin_registry_path: &std::path::Path,
) -> anyhow::Result<Option<serde_json::Value>> {
    crate::features::plugins::command::try_handle(normalized_path, argv, paths, plugin_registry_path)
}
