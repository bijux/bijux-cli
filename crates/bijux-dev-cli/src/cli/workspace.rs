//! Workspace path resolution for dev-cli report assembly.

use std::path::{Path, PathBuf};

/// Resolve workspace root based on this crate location.
#[must_use]
pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(".").to_path_buf())
}
