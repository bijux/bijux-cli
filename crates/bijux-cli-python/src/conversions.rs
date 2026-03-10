#![forbid(unsafe_code)]
//! Conversion and error-classification helpers for Python bridge APIs.

use std::fmt::Display;
use bijux_cli_routing::ExitCode;

/// Coarse error categories surfaced to Python callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeErrorKind {
    /// Usage/parse style failures.
    Usage,
    /// Validation failures.
    Validation,
    /// Internal/runtime failures.
    Internal,
}

/// Classify failure shape based on exit policy and stderr content.
#[must_use]
pub fn classify_failure(exit_code: i32, stderr: &str) -> BridgeErrorKind {
    if exit_code == ExitCode::Usage as i32 {
        return BridgeErrorKind::Usage;
    }
    let lower = stderr.to_ascii_lowercase();
    if lower.contains("unknown route")
        || lower.contains("unknown namespace")
        || lower.contains("usage")
    {
        return BridgeErrorKind::Usage;
    }
    if lower.contains("validation") || lower.contains("invalid") {
        return BridgeErrorKind::Validation;
    }
    BridgeErrorKind::Internal
}

/// Classify core app construction/runtime errors.
#[must_use]
pub fn classify_core_error(error: &impl Display) -> BridgeErrorKind {
    let msg = error.to_string().to_ascii_lowercase();
    if msg.contains("unknown") || msg.contains("invalid") || msg.contains("usage") {
        return BridgeErrorKind::Usage;
    }
    if msg.contains("validation") {
        return BridgeErrorKind::Validation;
    }
    BridgeErrorKind::Internal
}

/// Return stable exception tag for Python exception mapping.
#[must_use]
pub fn python_exception_tag(kind: BridgeErrorKind) -> &'static str {
    match kind {
        BridgeErrorKind::Usage => "UsageError",
        BridgeErrorKind::Validation => "ValidationError",
        BridgeErrorKind::Internal => "InternalError",
    }
}
