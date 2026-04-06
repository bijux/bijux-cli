//! Filesystem adapter helpers for maintainer workflows.

use std::fs;
use std::path::Path;

/// Read text file contents.
pub fn read_text(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path)
}

/// Write text file contents.
pub fn write_text(path: &Path, contents: &str) -> std::io::Result<()> {
    fs::write(path, contents)
}
