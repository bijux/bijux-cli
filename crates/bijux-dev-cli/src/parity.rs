//! Maintainer parity report assembly.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

/// Builds the maintainer parity report envelope.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    let parity_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/rust_python_parity_report.json"),
    );
    let bridge_parity = read_json_if_exists(
        &workspace_root.join("artifacts/parity/binary_vs_python_bridge_parity_report.json"),
    );
    let command_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_matrix.json"));
    let plugin_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/plugin_parity_matrix.json"));
    let repl_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/parity/repl_parity_matrix.json"));
    let python_bridge_matrix = read_json_if_exists(
        &workspace_root.join("artifacts/parity/python_bridge_parity_matrix.json"),
    );
    let state_behavior_matrix = read_json_if_exists(
        &workspace_root.join("artifacts/parity/state_behavior_parity_matrix.json"),
    );
    let repl_cli_output_diff =
        read_json_if_exists(&workspace_root.join("artifacts/parity/repl_cli_output_diff.json"));
    let parity_diffs =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_diffs.json"));
    let text_summary =
        fs::read_to_string(workspace_root.join("artifacts/parity/command_parity_summary.txt"))
            .unwrap_or_default();
    let commands_fully_rust_owned = read_json_if_exists(
        &workspace_root.join("artifacts/parity/commands_fully_rust_owned.json"),
    );
    let commands_using_compatibility_shims = read_json_if_exists(
        &workspace_root.join("artifacts/parity/commands_using_compatibility_shims.json"),
    );
    let commands_python_only =
        read_json_if_exists(&workspace_root.join("artifacts/parity/commands_python_only.json"));
    let coverage =
        read_json_if_exists(&workspace_root.join("artifacts/parity/parity_coverage_matrix.json"));
    let precedence_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_precedence_report.json"),
    );
    let flag_norm_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_flag_normalization_report.json"),
    );
    let stream_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_stream_report.json"));
    let exit_code_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_exit_code_report.json"));
    let help_diff_report =
        read_json_if_exists(&workspace_root.join("artifacts/parity/command_help_diff_report.json"));
    let machine_output_report = read_json_if_exists(
        &workspace_root.join("artifacts/parity/command_machine_output_diff_report.json"),
    );
    let parity_dashboard =
        read_json_if_exists(&workspace_root.join("artifacts/parity/parity_dashboard.json"));
    let parity_dashboard_text =
        fs::read_to_string(workspace_root.join("artifacts/parity/parity_dashboard.txt"))
            .unwrap_or_default();
    let config_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/config_parity_report.json"));
    let history_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/history_parity_report.json"));
    let memory_parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/memory_parity_report.json"));

    json!({
        "migration_dashboard_default": "bijux dev cli parity",
        "evidence_ids": [
            "EVIDENCE-1002-PARITY-COVERAGE",
            "EVIDENCE-1103-PLUGIN-LIFECYCLE",
            "EVIDENCE-1107-DIAGNOSTICS-CONSISTENCY",
            "EVIDENCE-1108-REPL-PARITY",
            "EVIDENCE-1109-PYTHON-BRIDGE-EQUIVALENCE"
        ],
        "rust_python": parity_report,
        "binary_bridge": bridge_parity,
        "command_matrix": command_matrix,
        "plugin_matrix": plugin_matrix,
        "repl_matrix": repl_matrix,
        "python_bridge_matrix": python_bridge_matrix,
        "state_behavior_matrix": state_behavior_matrix,
        "diffs": parity_diffs,
        "text_summary": text_summary,
        "plugin_lifecycle": command_matrix.get("plugin_lifecycle").cloned().unwrap_or_else(|| json!({})),
        "commands_fully_rust_owned": commands_fully_rust_owned,
        "commands_using_compatibility_shims": commands_using_compatibility_shims,
        "commands_python_only": commands_python_only,
        "coverage": coverage,
        "precedence_report": precedence_report,
        "flag_normalization_report": flag_norm_report,
        "stream_report": stream_report,
        "exit_code_report": exit_code_report,
        "help_diff_report": help_diff_report,
        "machine_output_diff_report": machine_output_report,
        "parity_dashboard": parity_dashboard,
        "parity_dashboard_text": parity_dashboard_text,
        "repl_cli_output_diff": repl_cli_output_diff,
        "state_parity": {
            "config": config_parity,
            "history": history_parity,
            "memory": memory_parity,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::build_report;

    #[test]
    fn parity_report_shape_is_stable() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&workspace_root);
        assert!(report.get("command_matrix").is_some());
        assert!(report.get("state_parity").is_some());
    }
}
