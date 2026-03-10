//! Maintainer parity report assembly.

use serde_json::{json, Value};

use crate::{DevCliCommand, ReportContext};

/// Builds the maintainer parity report envelope.
#[must_use]
pub fn build_report(context: &ReportContext) -> Value {
    json!({
        "command": DevCliCommand::Parity.as_str(),
        "generated_at": context.generated_at,
        "data_source": context.data_source,
        "owner": "bijux-dev-cli",
    })
}
