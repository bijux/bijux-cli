#![forbid(unsafe_code)]
//! Path resolution and filesystem bootstrap utilities.

use std::path::{Path, PathBuf};
use std::{fs, io};

use super::metadata::CANONICAL_EXECUTABLE;

fn is_executable_like(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return fs::metadata(path)
            .map(|meta| meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false);
    }

    #[cfg(windows)]
    {
        let Some(ext) = path.extension().and_then(|value| value.to_str()) else {
            return false;
        };
        let ext = format!(".{}", ext.to_ascii_uppercase());
        return executable_extensions()
            .iter()
            .any(|allowed| allowed == &ext);
    }

    #[cfg(not(any(unix, windows)))]
    {
        true
    }
}

fn is_wrapper_script(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if is_executable_like(path) {
        return true;
    }

    #[cfg(windows)]
    {
        return path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ps1"));
    }

    #[cfg(not(windows))]
    {
        false
    }
}

fn path_entries(path_value: &str) -> impl Iterator<Item = PathBuf> + '_ {
    std::env::split_paths(path_value)
}

fn canonical_executable_name() -> String {
    let extension = std::env::consts::EXE_EXTENSION;
    if extension.is_empty() {
        CANONICAL_EXECUTABLE.to_string()
    } else {
        format!("{CANONICAL_EXECUTABLE}.{extension}")
    }
}

#[cfg(windows)]
fn executable_extensions() -> Vec<String> {
    if let Some(raw) = std::env::var_os("PATHEXT") {
        let parsed: Vec<String> = raw
            .to_string_lossy()
            .split(';')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| {
                let normalized = if part.starts_with('.') {
                    part.to_string()
                } else {
                    format!(".{part}")
                };
                normalized.to_ascii_uppercase()
            })
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![
        ".COM".to_string(),
        ".EXE".to_string(),
        ".BAT".to_string(),
        ".CMD".to_string(),
    ]
}

#[cfg(windows)]
fn executable_candidates(entry: PathBuf) -> Vec<PathBuf> {
    executable_extensions()
        .into_iter()
        .map(|ext| entry.join(format!("{CANONICAL_EXECUTABLE}{ext}")))
        .collect()
}

#[cfg(not(windows))]
fn executable_candidates(entry: PathBuf) -> Vec<PathBuf> {
    vec![entry.join(canonical_executable_name())]
}

/// Collect discovered `bijux` binaries in PATH order.
#[must_use]
pub fn discover_path_binaries(path_value: &str) -> Vec<String> {
    path_entries(path_value)
        .flat_map(executable_candidates)
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
    const WRAPPER_CANDIDATES: &[&str] = &[
        "bijux.sh",
        "bijux.cmd",
        "bijux.bat",
        "bijux.ps1",
        "bijux-cli.sh",
    ];
    let canonical = canonical_executable_name();
    path_entries(path_value)
        .flat_map(|entry| WRAPPER_CANDIDATES.iter().map(move |name| entry.join(name)))
        .filter(|wrapper| is_wrapper_script(wrapper))
        .filter(|wrapper| !wrapper.with_file_name(&canonical).exists())
        .map(|wrapper| wrapper.display().to_string())
        .collect()
}

/// Detect known legacy wrappers that could shadow the canonical binary.
#[must_use]
pub fn legacy_installer_conflicts(path_value: &str) -> Vec<String> {
    const LEGACY_CANDIDATES: &[&str] = &["bijux.py", "bijux-legacy", "bijux_old", "bijux-cli.sh"];
    path_entries(path_value)
        .flat_map(|entry| LEGACY_CANDIDATES.iter().map(move |name| entry.join(name)))
        .filter(|candidate| is_executable_like(candidate))
        .map(|candidate| candidate.display().to_string())
        .collect()
}

/// Initialize first-run filesystem state and return whether setup ran this invocation.
#[allow(dead_code)]
pub fn initialize_first_run_state(state_root: &Path) -> io::Result<bool> {
    fs::create_dir_all(state_root)?;
    let marker = state_root.join(".first-run-ready");
    if marker.exists() {
        return Ok(false);
    }
    fs::write(marker, b"ready")?;
    Ok(true)
}
