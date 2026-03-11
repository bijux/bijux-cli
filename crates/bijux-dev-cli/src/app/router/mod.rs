//! Dev-cli command routing.

mod config_python_repo;
mod evidence;
mod maintenance;
mod release;
mod root;
mod rustdoc;

use anyhow::Result;
use serde_json::Value;

use crate::app::runtime_query::RuntimeQueryProvider;

/// Return true when the normalized path belongs to `dev cli` dispatch ownership.
#[must_use]
pub fn owns_path(normalized_path: &[String]) -> bool {
    match normalized_path {
        [a, b, _] if a == "dev" && b == "cli" => true,
        [a, b, c, _]
            if a == "dev"
                && b == "cli"
                && matches!(
                    c.as_str(),
                    "maintenance" | "rustdoc" | "release" | "evidence" | "config" | "python" | "repo"
                ) =>
        {
            true
        }
        [a, b, c, d, _] if a == "dev" && b == "cli" && c == "maintenance" && d == "status" => {
            true
        }
        _ => false,
    }
}

/// Dispatch `dev cli` command paths and return report payloads.
pub fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    runtime: &dyn RuntimeQueryProvider,
) -> Result<Option<Value>> {
    if let Some(payload) = root::try_handle(normalized_path, argv, runtime)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = maintenance::try_handle(normalized_path, argv)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = rustdoc::try_handle(normalized_path)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = release::try_handle(normalized_path)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = evidence::try_handle(normalized_path, argv)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = config_python_repo::try_handle(normalized_path)? {
        return Ok(Some(payload));
    }
    Ok(None)
}
