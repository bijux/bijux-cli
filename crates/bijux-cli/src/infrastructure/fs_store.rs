#![forbid(unsafe_code)]
//! Filesystem storage adapter primitives.

use std::fs;
use std::path::Path;

/// Read UTF-8 text from a file path.
pub fn read_text(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path)
}

/// Write UTF-8 text to a file path.
pub fn write_text(path: &Path, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

/// Check whether a path exists.
#[must_use]
pub fn exists(path: &Path) -> bool {
    path.exists()
}
