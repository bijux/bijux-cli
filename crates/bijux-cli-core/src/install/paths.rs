#![forbid(unsafe_code)]
//! Path resolution and filesystem bootstrap utilities.

use std::path::{Path, PathBuf};
use std::{fs, io};

use super::metadata::CANONICAL_EXECUTABLE;

fn is_executable_like(path: &Path) -> bool {
    path.is_file()
}

fn path_entries(path_value: &str) -> impl Iterator<Item = PathBuf> + '_ {
    std::env::split_paths(path_value)
}

/// Collect discovered `bijux` binaries in PATH order.
#[must_use]
pub fn discover_path_binaries(path_value: &str) -> Vec<String> {
    path_entries(path_value)
        .map(|entry| entry.join(CANONICAL_EXECUTABLE))
        .filter(|candidate| is_executable_like(candidate))
        .map(|candidate| candidate.display().to_string())
        .collect()
}

/// Resolve active binary from override or PATH discovery.
#[must_use]
pub fn resolve_active_binary(path_value: &str, bin_override: Option<&str>) -> Option<String> {
    if let Some(override_path) = bin_override.filter(|value| !value.trim().is_empty()) {
        return Some(override_path.to_string());
    }
    discover_path_binaries(path_value).into_iter().next()
}

/// Detect stale wrapper scripts in PATH.
#[must_use]
pub fn detect_stale_wrapper_scripts(path_value: &str) -> Vec<String> {
    path_entries(path_value)
        .map(|entry| entry.join(format!("{CANONICAL_EXECUTABLE}.sh")))
        .filter(|wrapper| is_executable_like(wrapper))
        .filter(|wrapper| !wrapper.with_file_name(CANONICAL_EXECUTABLE).exists())
        .map(|wrapper| wrapper.display().to_string())
        .collect()
}

/// Detect known legacy wrappers that could shadow the canonical binary.
#[must_use]
pub fn legacy_installer_conflicts(path_value: &str) -> Vec<String> {
    const LEGACY_CANDIDATES: &[&str] = &["bijux.py", "bijux-legacy", "bijux_old", "bijux-cli.sh"];
    path_entries(path_value)
        .flat_map(|entry| LEGACY_CANDIDATES.iter().map(move |name| entry.join(name)))
        .filter(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
        .collect()
}

/// Initialize first-run filesystem state and return whether setup ran this invocation.
pub fn initialize_first_run_state(state_root: &Path) -> io::Result<bool> {
    fs::create_dir_all(state_root)?;
    let marker = state_root.join(".first-run-ready");
    if marker.exists() {
        return Ok(false);
    }
    fs::write(marker, b"ready")?;
    Ok(true)
}
