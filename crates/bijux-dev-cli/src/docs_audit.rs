//! Maintainer documentation audit report assembly.

use serde_json::{json, Value};

use crate::{DevCliCommand, ReportContext};

/// Builds the maintainer documentation audit report envelope.
#[must_use]
pub fn build_report(context: &ReportContext) -> Value {
    json!({
        "command": DevCliCommand::DocsAudit.as_str(),
        "generated_at": context.generated_at,
        "data_source": context.data_source,
        "owner": "bijux-dev-cli",
    })
}
