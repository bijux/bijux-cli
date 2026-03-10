//! Backward-compatible shim for history command handlers.

#[allow(dead_code)]
pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    paths: &crate::cli::context::ResolvedStatePaths,
) -> anyhow::Result<Option<serde_json::Value>> {
    crate::features::history::command::try_handle(normalized_path, argv, paths)
}
