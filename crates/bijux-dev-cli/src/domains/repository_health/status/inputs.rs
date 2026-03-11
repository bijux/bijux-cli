//! Status command artifact input loading.

use std::path::Path;

use serde_json::{Map, Value};

use crate::infrastructure::artifacts::{read_json_if_exists, read_text_if_exists};

const REPORT_JSON_ITEMS: &[(&str, &str)] = &[
    ("root_commands", "artifacts/status/status_root_commands.json"),
    (
        "root_command_remaining_inventory",
        "artifacts/status/root_command_remaining_inventory.json",
    ),
    (
        "root_command_impact_ranking",
        "artifacts/status/root_command_impact_ranking.json",
    ),
    (
        "root_command_completion_report",
        "artifacts/status/root_command_completion_report.json",
    ),
    (
        "root_command_closure_set",
        "artifacts/status/root_command_closure_set.json",
    ),
    ("cli_subcommands", "artifacts/status/status_cli_subcommands.json"),
    (
        "dev_cli_subcommands",
        "artifacts/status/status_dev_cli_subcommands.json",
    ),
    (
        "cli_command_remaining_inventory",
        "artifacts/status/cli_command_remaining_inventory.json",
    ),
    (
        "cli_command_value_ranking",
        "artifacts/status/cli_command_value_ranking.json",
    ),
    (
        "cli_command_completion_report",
        "artifacts/status/cli_command_completion_report.json",
    ),
    (
        "cli_command_closure_set",
        "artifacts/status/cli_command_closure_set.json",
    ),
    (
        "dev_cli_command_remaining_inventory",
        "artifacts/status/dev_cli_command_remaining_inventory.json",
    ),
    (
        "dev_cli_command_value_ranking",
        "artifacts/status/dev_cli_command_value_ranking.json",
    ),
    (
        "dev_cli_command_completion_report",
        "artifacts/status/dev_cli_command_completion_report.json",
    ),
    (
        "dev_cli_command_closure_set",
        "artifacts/status/dev_cli_command_closure_set.json",
    ),
    (
        "cli_dev_command_closure_report",
        "artifacts/status/cli_dev_command_closure_report.json",
    ),
    ("plugin_commands", "artifacts/status/status_plugin_commands.json"),
    (
        "repl_parity_coverage",
        "artifacts/status/status_repl_parity_coverage.json",
    ),
    (
        "python_bridge_parity_coverage",
        "artifacts/status/status_python_bridge_parity_coverage.json",
    ),
    (
        "install_packaging_parity_coverage",
        "artifacts/status/status_install_packaging_parity_coverage.json",
    ),
    (
        "state_behavior_coverage",
        "artifacts/status/status_state_behavior_coverage.json",
    ),
    ("state_paths_report", "artifacts/status/status_state_paths_report.json"),
    (
        "state_corruption_health_report",
        "artifacts/status/status_state_corruption_health_report.json",
    ),
    ("state_migration_status", "artifacts/status/state_migration_status.json"),
    (
        "unified_state_behavior_report",
        "artifacts/status/unified_state_behavior_report.json",
    ),
    (
        "unified_state_corruption_report",
        "artifacts/status/unified_state_corruption_report.json",
    ),
    (
        "unified_state_rollback_report",
        "artifacts/status/unified_state_rollback_report.json",
    ),
    (
        "unified_state_path_resolution_report",
        "artifacts/status/unified_state_path_resolution_report.json",
    ),
    (
        "unified_state_doctor_snapshots",
        "artifacts/status/unified_state_doctor_snapshots.json",
    ),
    (
        "unified_state_audit_payload",
        "artifacts/status/unified_state_audit_payload.json",
    ),
    ("snapshot_coverage", "artifacts/status/status_snapshot_coverage.json"),
    ("stream_coverage", "artifacts/status/status_stream_coverage.json"),
    (
        "exit_code_coverage",
        "artifacts/status/status_exit_code_coverage.json",
    ),
    (
        "failure_path_coverage",
        "artifacts/status/status_failure_path_coverage.json",
    ),
    (
        "compatibility_aliases",
        "artifacts/status/status_compatibility_aliases.json",
    ),
    (
        "known_parity_gaps",
        "artifacts/status/status_known_parity_gaps.json",
    ),
    (
        "intentional_differences",
        "artifacts/status/status_intentional_differences.json",
    ),
    (
        "unowned_maintenance",
        "artifacts/status/status_unowned_maintenance.json",
    ),
    (
        "maintainer_maintenance_outside_dev_cli",
        "artifacts/status/maintainer_maintenance_outside_dev_cli.json",
    ),
    (
        "maintainer_control_plane_commands",
        "artifacts/status/maintainer_control_plane_commands.json",
    ),
    (
        "maintainer_control_plane_report",
        "artifacts/status/maintainer_control_plane_report.json",
    ),
    (
        "repl_only_behaviors",
        "artifacts/status/repl_only_behaviors.json",
    ),
    (
        "plugin_lifecycle_ownership_report",
        "artifacts/status/plugin_lifecycle_ownership_report.json",
    ),
    (
        "plugin_scaffold_efficiency_report",
        "artifacts/status/plugin_scaffold_efficiency_report.json",
    ),
    (
        "plugin_scaffold_lifecycle_proof_report",
        "artifacts/status/plugin_scaffold_lifecycle_proof_report.json",
    ),
    (
        "plugin_namespace_abuse_proof_report",
        "artifacts/status/plugin_namespace_abuse_proof_report.json",
    ),
    (
        "plugin_doctor_clarity_report",
        "artifacts/status/plugin_doctor_clarity_report.json",
    ),
    (
        "plugin_explain_clarity_report",
        "artifacts/status/plugin_explain_clarity_report.json",
    ),
    (
        "plugin_where_ownership_report",
        "artifacts/status/plugin_where_ownership_report.json",
    ),
    (
        "plugin_command_set_status",
        "artifacts/status/plugin_command_set_status.json",
    ),
    (
        "plugin_migration_report",
        "artifacts/status/plugin_migration_report.json",
    ),
    (
        "config_closure_report",
        "artifacts/status/config_closure_report.json",
    ),
    (
        "plugins_closure_report",
        "artifacts/status/plugins_closure_report.json",
    ),
    (
        "history_closure_report",
        "artifacts/status/history_closure_report.json",
    ),
    (
        "memory_closure_report",
        "artifacts/status/memory_closure_report.json",
    ),
    (
        "diagnostics_closure_report",
        "artifacts/status/diagnostics_closure_report.json",
    ),
    (
        "repl_shared_law_closure_report",
        "artifacts/status/repl_shared_law_closure_report.json",
    ),
    (
        "command_family_closure_report",
        "artifacts/status/command_family_closure_report.json",
    ),
    (
        "command_family_partial_area_acceptance",
        "artifacts/status/command_family_partial_area_acceptance.json",
    ),
    (
        "cross_surface_consistency_artifact",
        "artifacts/status/cross_surface_consistency_artifact.json",
    ),
    (
        "cross_surface_drift_artifact",
        "artifacts/status/cross_surface_drift_artifact.json",
    ),
    (
        "cross_surface_consistency_contract",
        "artifacts/status/cross_surface_consistency_contract.json",
    ),
    (
        "cross_crate_duplication_report",
        "artifacts/status/cross_crate_duplication_report.json",
    ),
    (
        "public_api_inventory_report",
        "artifacts/status/public_api_inventory_report.json",
    ),
    (
        "crate_complexity_report",
        "artifacts/status/crate_complexity_report.json",
    ),
    (
        "candidate_merge_later_report",
        "artifacts/status/candidate_merge_later_report.json",
    ),
    (
        "candidate_keep_separate_report",
        "artifacts/status/candidate_keep_separate_report.json",
    ),
    (
        "simplification_deletion_artifact",
        "artifacts/status/simplification_deletion_artifact.json",
    ),
    (
        "command_surface_consistency_summary",
        "artifacts/status/command_surface_consistency_summary.json",
    ),
];

const REPORT_TEXT_ITEMS: &[(&str, &str)] = &[
    (
        "root_command_completion_report_text",
        "artifacts/status/root_command_completion_report.txt",
    ),
    (
        "cli_dev_command_closure_report_text",
        "artifacts/status/cli_dev_command_closure_report.txt",
    ),
    (
        "maintainer_control_plane_text_report",
        "artifacts/status/maintainer_control_plane_text_report.txt",
    ),
    (
        "command_family_closure_report_text",
        "artifacts/status/command_family_closure_report.txt",
    ),
    (
        "simplification_deletion_artifact_text",
        "artifacts/status/simplification_deletion_artifact.txt",
    ),
];

const COMMAND_MIGRATION_JSON_ITEMS: &[(&str, &str)] = &[
    (
        "rust_partial",
        "artifacts/status/command_migration_rust_partial.json",
    ),
    (
        "python_only",
        "artifacts/status/command_migration_python_only.json",
    ),
    (
        "intentional_differences",
        "artifacts/status/command_migration_intentional_differences.json",
    ),
    (
        "documented_python_not_proven",
        "artifacts/status/documented_python_commands_not_proven_in_rust.json",
    ),
    (
        "public_python_paths_still_reachable",
        "artifacts/status/public_python_paths_still_reachable.json",
    ),
    (
        "legacy_alias_paths_still_accepted",
        "artifacts/status/legacy_alias_paths_still_accepted.json",
    ),
    (
        "compatibility_shims_still_active",
        "artifacts/status/compatibility_shims_still_active.json",
    ),
    (
        "bridge_duplicate_law_report",
        "artifacts/status/bridge_duplicate_law_report.json",
    ),
    (
        "bridge_wrapper_only_closure_report",
        "artifacts/status/bridge_wrapper_only_closure_report.json",
    ),
];

const COMMAND_MIGRATION_TEXT_ITEMS: &[(&str, &str)] = &[
    ("text", "artifacts/status/command_migration_matrix.txt"),
    (
        "bridge_wrapper_only_closure_report_text",
        "artifacts/status/bridge_wrapper_only_closure_report.txt",
    ),
];

/// Loaded status artifact inputs.
pub struct StatusInputs {
    pub state: Value,
    pub parity: Value,
    pub status_report: Value,
    pub reports: Map<String, Value>,
    pub command_migration: Map<String, Value>,
    pub priority_plan: Value,
    pub priority_plan_text: String,
    pub simplification_priorities: Value,
    pub simplification_priorities_text: String,
}

fn load_json_entries(workspace_root: &Path, entries: &[(&str, &str)]) -> Map<String, Value> {
    entries
        .iter()
        .map(|(key, rel_path)| {
            (
                (*key).to_string(),
                read_json_if_exists(&workspace_root.join(rel_path)),
            )
        })
        .collect()
}

fn load_text_entries(workspace_root: &Path, entries: &[(&str, &str)]) -> Map<String, Value> {
    entries
        .iter()
        .map(|(key, rel_path)| {
            (
                (*key).to_string(),
                Value::String(read_text_if_exists(&workspace_root.join(rel_path))),
            )
        })
        .collect()
}

fn normalize_migration_matrix(matrix: &mut Value) {
    if let Some(commands) = matrix.get_mut("commands").and_then(Value::as_array_mut) {
        for row in commands {
            let is_partial = row.get("status").and_then(Value::as_str) == Some("rust-partial");
            let has_blocker =
                row.get("blocker").and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty());
            if is_partial && !has_blocker {
                row["blocker"] = Value::String("parity coverage incomplete".to_string());
            }
        }
    }
}

/// Load all status report inputs from workspace artifacts.
#[must_use]
pub fn load_status_inputs(workspace_root: &Path) -> StatusInputs {
    let mut reports = load_json_entries(workspace_root, REPORT_JSON_ITEMS);
    reports.extend(load_text_entries(workspace_root, REPORT_TEXT_ITEMS));

    let mut command_migration = load_json_entries(workspace_root, COMMAND_MIGRATION_JSON_ITEMS);
    command_migration.extend(load_text_entries(workspace_root, COMMAND_MIGRATION_TEXT_ITEMS));

    let mut matrix = read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json"));
    normalize_migration_matrix(&mut matrix);
    command_migration.insert("matrix".to_string(), matrix);

    StatusInputs {
        state: read_json_if_exists(&workspace_root.join("artifacts/status/current_rust_state.json")),
        parity: read_json_if_exists(
            &workspace_root.join("artifacts/parity/rust_python_parity_report.json"),
        ),
        status_report: read_json_if_exists(&workspace_root.join("artifacts/status/status.json")),
        reports,
        command_migration,
        priority_plan: read_json_if_exists(&workspace_root.join("artifacts/status/priority_plan.json")),
        priority_plan_text: read_text_if_exists(&workspace_root.join("artifacts/status/priority_plan.txt")),
        simplification_priorities: read_json_if_exists(
            &workspace_root.join("artifacts/status/simplification_priorities.json"),
        ),
        simplification_priorities_text: read_text_if_exists(
            &workspace_root.join("artifacts/status/simplification_priorities.txt"),
        ),
    }
}
