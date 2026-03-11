//! Shared helpers for reading and traversing maintainer artifact inputs.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

/// Read JSON payload from disk and return `{}` when the file is missing or malformed.
#[must_use]
pub fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}))
}

/// Read text payload from disk and return empty string when unavailable.
#[must_use]
pub fn read_text_if_exists(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// Recursively collect all files under a base directory in deterministic order.
#[must_use]
pub fn collect_files_recursive(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

/// Render a path relative to workspace root with normalized separators.
#[must_use]
pub fn relative_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
