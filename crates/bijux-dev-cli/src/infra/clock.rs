//! Clock helpers for maintainer report generation.

use std::process::Command;

/// Return UTC timestamp formatted as RFC3339 seconds precision.
#[must_use]
pub fn generated_at_utc() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z\n".to_string())
        .trim()
        .to_string()
}
