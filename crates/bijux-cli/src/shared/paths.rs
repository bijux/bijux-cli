#![forbid(unsafe_code)]
//! Shared path normalization helpers.

use std::path::{Path, PathBuf};

/// Normalize separators for stable cross-platform reporting.
#[must_use]
pub fn normalize_for_report(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Join path segments from report-friendly fragments.
#[must_use]
pub fn join_report_path(base: &Path, segment: &str) -> PathBuf {
    base.join(segment)
}
