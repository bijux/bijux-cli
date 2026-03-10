//! Backward-compatible shim for developer runtime adapter handlers.

#[allow(dead_code)]
pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    registry: &crate::routing::registry::RouteRegistry,
    paths: &crate::cli::context::ResolvedStatePaths,
    plugin_registry_path: &std::path::Path,
) -> anyhow::Result<Option<serde_json::Value>> {
    crate::features::developer::runtime_adapter::try_handle(
        normalized_path,
        argv,
        registry,
        paths,
        plugin_registry_path,
    )
}
