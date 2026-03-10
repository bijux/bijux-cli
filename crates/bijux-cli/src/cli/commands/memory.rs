//! Backward-compatible shim for memory command handlers.

#[allow(dead_code)]
pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &crate::cli::context::ResolvedStatePaths,
) -> anyhow::Result<Option<serde_json::Value>> {
    crate::features::memory::command::try_handle(normalized_path, argv, paths)
}
