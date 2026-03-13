//! Canonical report envelope and text style helpers for dev-cli control-plane output.

use serde_json::{json, Value};

/// Canonical text report style identifier.
pub const DEV_CLI_TEXT_REPORT_STYLE: &str = "dev-cli-v1";

/// Wraps a payload in the canonical machine-readable report envelope.
#[must_use]
pub fn machine_report_envelope(command: &str, payload: Value) -> Value {
    json!({
        "namespace": "bijux-dev-cli",
        "command": command,
        "style": DEV_CLI_TEXT_REPORT_STYLE,
        "payload": payload,
    })
}

/// Renders the canonical text report heading style.
#[must_use]
pub fn text_report_heading(command: &str, generated_at: &str) -> String {
    format!("{command}\nstyle: {DEV_CLI_TEXT_REPORT_STYLE}\ngenerated_at: {generated_at}")
}
