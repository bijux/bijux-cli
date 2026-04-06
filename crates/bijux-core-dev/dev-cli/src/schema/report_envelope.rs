//! Canonical report envelope and text style helpers for maintainer control-plane output.

use serde_json::{json, Value};

/// Canonical text report style identifier.
pub const MAINTAINER_TEXT_REPORT_STYLE: &str = "maintainer-v1";

/// Wraps a payload in the canonical machine-readable report envelope.
#[must_use]
pub fn machine_report_envelope(command: &str, payload: Value) -> Value {
    json!({
        "namespace": "bijux-dev-cli",
        "command": command,
        "style": MAINTAINER_TEXT_REPORT_STYLE,
        "payload": payload,
    })
}

/// Renders the canonical text report heading style.
#[must_use]
pub fn text_report_heading(command: &str, generated_at: &str) -> String {
    format!("{command}\nstyle: {MAINTAINER_TEXT_REPORT_STYLE}\ngenerated_at: {generated_at}")
}
