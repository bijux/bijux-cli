#![forbid(unsafe_code)]
//! State diagnostics query model and resolver.

use std::fs;
use std::path::{Path, PathBuf};

/// File-system status for a single state path.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct StatePathStatus {
    /// Absolute or resolved path.
    pub path: PathBuf,
    /// Whether the path exists.
    pub exists: bool,
    /// Whether the path is a regular file.
    pub is_file: bool,
    /// Whether the path is a directory.
    pub is_dir: bool,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Whether the path can be read by the current process.
    pub readable: bool,
    /// Whether the path can be written by the current process.
    pub writable: bool,
}

/// Structured state diagnostics queried from runtime state paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDiagnosticsQuery {
    /// Config file status.
    pub config: StatePathStatus,
    /// History file status.
    pub history: StatePathStatus,
    /// Plugin registry status.
    pub plugins_registry: StatePathStatus,
    /// Memory file status.
    pub memory: StatePathStatus,
}

fn state_path_status(path: &Path) -> StatePathStatus {
    let metadata = fs::metadata(path);
    StatePathStatus {
        path: path.to_path_buf(),
        exists: metadata.is_ok(),
        is_file: metadata.as_ref().is_ok_and(std::fs::Metadata::is_file),
        is_dir: metadata.as_ref().is_ok_and(std::fs::Metadata::is_dir),
        size_bytes: metadata.as_ref().map_or(0_u64, |entry| entry.len()),
        readable: fs::read(path).is_ok(),
        writable: if path.exists() {
            fs::OpenOptions::new().append(true).open(path).is_ok()
        } else {
            path.parent().is_some_and(|parent| fs::create_dir_all(parent).is_ok())
        },
    }
}

/// Query state diagnostics from resolved runtime paths.
#[must_use]
pub fn state_diagnostics_query(
    config: &Path,
    history: &Path,
    plugins_registry: &Path,
    memory: &Path,
) -> StateDiagnosticsQuery {
    StateDiagnosticsQuery {
        config: state_path_status(config),
        history: state_path_status(history),
        plugins_registry: state_path_status(plugins_registry),
        memory: state_path_status(memory),
    }
}
