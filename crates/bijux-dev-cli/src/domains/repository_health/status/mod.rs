//! Maintainer status report assembly.

use std::path::Path;

use serde_json::{json, Value};

mod inputs;

/// Builds the maintainer status report envelope.
#[must_use]
pub fn build_report(workspace_root: &Path, inventory: Value) -> Value {
    let inputs = inputs::load_status_inputs(workspace_root);

    json!({
        "maintainer_dashboard_default": "bijux dev cli status",
        "control_plane_crate": "bijux-dev-cli",
        "status_report": inputs.status_report,
        "reports": Value::Object(inputs.reports),
        "command_migration": Value::Object(inputs.command_migration),
        "priority_plan_priorities": inputs.priority_plan,
        "priority_plan_summary_text": inputs.priority_plan_text,
        "next_simplification_priorities": inputs.simplification_priorities,
        "next_simplification_summary_text": inputs.simplification_priorities_text,
        "current_rust_state": inputs.state,
        "parity": inputs.parity,
        "inventory": inventory,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::build_report;

    #[test]
    fn status_report_shape_is_stable() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&workspace_root, json!({}));
        assert!(report.get("status_report").is_some());
        assert!(report.get("reports").is_some());
        assert!(report.get("command_migration").is_some());
    }
}
