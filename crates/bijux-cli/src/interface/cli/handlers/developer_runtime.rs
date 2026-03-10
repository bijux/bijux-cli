//! Runtime query adapter entrypoint for `dev cli` command handlers.

use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use crate::features::developer::runtime_query_adapter;
use crate::features::diagnostics::state_paths::ResolvedStatePaths;
use crate::routing::registry::RouteRegistry;

pub(crate) fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    registry: &RouteRegistry,
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Result<Option<Value>> {
    runtime_query_adapter::try_handle(normalized_path, argv, registry, paths, plugin_registry_path)
}
