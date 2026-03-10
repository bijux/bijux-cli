//! Maintainer status report assembly.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

/// Builds the maintainer status report envelope.
#[must_use]
pub fn build_report(workspace_root: &Path, inventory: Value) -> Value {
    let state =
        read_json_if_exists(&workspace_root.join("artifacts/status/current_rust_state.json"));
    let parity = read_json_if_exists(
        &workspace_root.join("artifacts/parity/rust_python_parity_report.json"),
    );
    let status_report = read_json_if_exists(&workspace_root.join("artifacts/status/status.json"));
    let root_commands =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_root_commands.json"));
    let root_command_remaining_inventory = read_json_if_exists(
        &workspace_root.join("artifacts/status/root_command_remaining_inventory.json"),
    );
    let root_command_impact_ranking = read_json_if_exists(
        &workspace_root.join("artifacts/status/root_command_impact_ranking.json"),
    );
    let root_command_completion_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/root_command_completion_report.json"),
    );
    let root_command_closure_set =
        read_json_if_exists(&workspace_root.join("artifacts/status/root_command_closure_set.json"));
    let root_command_completion_report_text = fs::read_to_string(
        workspace_root.join("artifacts/status/root_command_completion_report.txt"),
    )
    .unwrap_or_default();
    let cli_subcommands =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_cli_subcommands.json"));
    let dev_cli_subcommands = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_dev_cli_subcommands.json"),
    );
    let cli_command_remaining_inventory = read_json_if_exists(
        &workspace_root.join("artifacts/status/cli_command_remaining_inventory.json"),
    );
    let cli_command_value_ranking = read_json_if_exists(
        &workspace_root.join("artifacts/status/cli_command_value_ranking.json"),
    );
    let cli_command_completion_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/cli_command_completion_report.json"),
    );
    let cli_command_closure_set =
        read_json_if_exists(&workspace_root.join("artifacts/status/cli_command_closure_set.json"));
    let dev_cli_command_remaining_inventory = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_command_remaining_inventory.json"),
    );
    let dev_cli_command_value_ranking = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_command_value_ranking.json"),
    );
    let dev_cli_command_completion_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_command_completion_report.json"),
    );
    let dev_cli_command_closure_set = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_command_closure_set.json"),
    );
    let cli_dev_command_closure_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/cli_dev_command_closure_report.json"),
    );
    let cli_dev_command_closure_report_text = fs::read_to_string(
        workspace_root.join("artifacts/status/cli_dev_command_closure_report.txt"),
    )
    .unwrap_or_default();
    let plugin_commands =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_plugin_commands.json"));
    let repl_parity = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_repl_parity_coverage.json"),
    );
    let python_bridge_parity = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_python_bridge_parity_coverage.json"),
    );
    let install_packaging = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_install_packaging_parity_coverage.json"),
    );
    let state_behavior = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_state_behavior_coverage.json"),
    );
    let state_paths = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_state_paths_report.json"),
    );
    let state_corruption = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_state_corruption_health_report.json"),
    );
    let state_migration_status =
        read_json_if_exists(&workspace_root.join("artifacts/status/state_migration_status.json"));
    let unified_state_behavior = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_behavior_report.json"),
    );
    let unified_state_corruption = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_corruption_report.json"),
    );
    let unified_state_rollback = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_rollback_report.json"),
    );
    let unified_state_path_resolution = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_path_resolution_report.json"),
    );
    let unified_state_doctor_snapshots = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_doctor_snapshots.json"),
    );
    let unified_state_audit_payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/unified_state_audit_payload.json"),
    );
    let snapshot_coverage =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_snapshot_coverage.json"));
    let stream_coverage =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_stream_coverage.json"));
    let exit_code_coverage = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_exit_code_coverage.json"),
    );
    let failure_path_coverage = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_failure_path_coverage.json"),
    );
    let compatibility_aliases = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_compatibility_aliases.json"),
    );
    let known_parity_gaps =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_known_parity_gaps.json"));
    let intentional_differences = read_json_if_exists(
        &workspace_root.join("artifacts/status/status_intentional_differences.json"),
    );
    let unowned_scripts =
        read_json_if_exists(&workspace_root.join("artifacts/status/status_unowned_scripts.json"));
    let maintainer_scripts = read_json_if_exists(
        &workspace_root.join("artifacts/status/maintainer_scripts_outside_dev_cli.json"),
    );
    let maintainer_control_plane_commands = read_json_if_exists(
        &workspace_root.join("artifacts/status/maintainer_control_plane_commands.json"),
    );
    let maintainer_control_plane_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/maintainer_control_plane_report.json"),
    );
    let maintainer_control_plane_text = fs::read_to_string(
        workspace_root.join("artifacts/status/maintainer_control_plane_text_report.txt"),
    )
    .unwrap_or_default();
    let repl_only_behaviors =
        read_json_if_exists(&workspace_root.join("artifacts/status/repl_only_behaviors.json"));
    let next_phase = read_json_if_exists(&workspace_root.join("artifacts/status/next_phase.json"));
    let next_phase_text =
        fs::read_to_string(workspace_root.join("artifacts/status/next_phase.txt"))
            .unwrap_or_default();
    let next_phase_minimalism =
        read_json_if_exists(&workspace_root.join("artifacts/status/next_phase_minimalism.json"));
    let next_phase_minimalism_text =
        fs::read_to_string(workspace_root.join("artifacts/status/next_phase_minimalism.txt"))
            .unwrap_or_default();
    let migration_matrix =
        read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json"));
    let migration_rust_partial = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_migration_rust_partial.json"),
    );
    let migration_python_only = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_migration_python_only.json"),
    );
    let migration_intentional = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_migration_intentional_differences.json"),
    );
    let migration_text =
        fs::read_to_string(workspace_root.join("artifacts/status/command_migration_matrix.txt"))
            .unwrap_or_default();
    let documented_not_proven = read_json_if_exists(
        &workspace_root.join("artifacts/status/documented_python_commands_not_proven_in_rust.json"),
    );
    let public_python_paths = read_json_if_exists(
        &workspace_root.join("artifacts/status/public_python_paths_still_reachable.json"),
    );
    let legacy_alias_paths = read_json_if_exists(
        &workspace_root.join("artifacts/status/legacy_alias_paths_still_accepted.json"),
    );
    let active_compat_shims = read_json_if_exists(
        &workspace_root.join("artifacts/status/compatibility_shims_still_active.json"),
    );
    let bridge_duplicate_law = read_json_if_exists(
        &workspace_root.join("artifacts/status/bridge_duplicate_law_report.json"),
    );
    let bridge_wrapper_only_closure = read_json_if_exists(
        &workspace_root.join("artifacts/status/bridge_wrapper_only_closure_report.json"),
    );
    let bridge_wrapper_only_closure_text = fs::read_to_string(
        workspace_root.join("artifacts/status/bridge_wrapper_only_closure_report.txt"),
    )
    .unwrap_or_default();
    let plugin_lifecycle_ownership = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_lifecycle_ownership_report.json"),
    );
    let plugin_scaffold_efficiency = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_scaffold_efficiency_report.json"),
    );
    let plugin_scaffold_lifecycle_proof = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_scaffold_lifecycle_proof_report.json"),
    );
    let plugin_namespace_abuse_proof = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_namespace_abuse_proof_report.json"),
    );
    let plugin_doctor_clarity = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_doctor_clarity_report.json"),
    );
    let plugin_explain_clarity = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_explain_clarity_report.json"),
    );
    let plugin_where_ownership = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_where_ownership_report.json"),
    );
    let plugin_command_set_status = read_json_if_exists(
        &workspace_root.join("artifacts/status/plugin_command_set_status.json"),
    );
    let plugin_migration_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/plugin_migration_report.json"));
    let config_closure_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/config_closure_report.json"));
    let plugins_closure_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/plugins_closure_report.json"));
    let history_closure_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/history_closure_report.json"));
    let memory_closure_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/memory_closure_report.json"));
    let diagnostics_closure_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/diagnostics_closure_report.json"),
    );
    let repl_shared_law_closure_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/repl_shared_law_closure_report.json"),
    );
    let command_family_closure_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_family_closure_report.json"),
    );
    let command_family_partial_area_acceptance = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_family_partial_area_acceptance.json"),
    );
    let command_family_closure_report_text = fs::read_to_string(
        workspace_root.join("artifacts/status/command_family_closure_report.txt"),
    )
    .unwrap_or_default();
    let cross_surface_consistency_artifact = read_json_if_exists(
        &workspace_root.join("artifacts/status/cross_surface_consistency_artifact.json"),
    );
    let cross_surface_drift_artifact = read_json_if_exists(
        &workspace_root.join("artifacts/status/cross_surface_drift_artifact.json"),
    );
    let cross_surface_consistency_contract = read_json_if_exists(
        &workspace_root.join("artifacts/status/cross_surface_consistency_contract.json"),
    );
    let cross_crate_duplication_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/cross_crate_duplication_report.json"),
    );
    let public_api_inventory_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/public_api_inventory_report.json"),
    );
    let crate_complexity_report =
        read_json_if_exists(&workspace_root.join("artifacts/status/crate_complexity_report.json"));
    let candidate_merge_later_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/candidate_merge_later_report.json"),
    );
    let candidate_keep_separate_report = read_json_if_exists(
        &workspace_root.join("artifacts/status/candidate_keep_separate_report.json"),
    );
    let simplification_deletion_artifact = read_json_if_exists(
        &workspace_root.join("artifacts/status/simplification_deletion_artifact.json"),
    );
    let simplification_deletion_artifact_text = fs::read_to_string(
        workspace_root.join("artifacts/status/simplification_deletion_artifact.txt"),
    )
    .unwrap_or_default();
    let command_surface_consistency_summary = read_json_if_exists(
        &workspace_root.join("artifacts/status/command_surface_consistency_summary.json"),
    );

    json!({
        "status_report": status_report,
        "reports": {
            "root_commands": root_commands,
            "root_command_remaining_inventory": root_command_remaining_inventory,
            "root_command_impact_ranking": root_command_impact_ranking,
            "root_command_completion_report": root_command_completion_report,
            "root_command_closure_set": root_command_closure_set,
            "root_command_completion_report_text": root_command_completion_report_text,
            "cli_subcommands": cli_subcommands,
            "dev_cli_subcommands": dev_cli_subcommands,
            "cli_command_remaining_inventory": cli_command_remaining_inventory,
            "cli_command_value_ranking": cli_command_value_ranking,
            "cli_command_completion_report": cli_command_completion_report,
            "cli_command_closure_set": cli_command_closure_set,
            "dev_cli_command_remaining_inventory": dev_cli_command_remaining_inventory,
            "dev_cli_command_value_ranking": dev_cli_command_value_ranking,
            "dev_cli_command_completion_report": dev_cli_command_completion_report,
            "dev_cli_command_closure_set": dev_cli_command_closure_set,
            "cli_dev_command_closure_report": cli_dev_command_closure_report,
            "cli_dev_command_closure_report_text": cli_dev_command_closure_report_text,
            "plugin_commands": plugin_commands,
            "repl_parity_coverage": repl_parity,
            "python_bridge_parity_coverage": python_bridge_parity,
            "install_packaging_parity_coverage": install_packaging,
            "state_behavior_coverage": state_behavior,
            "state_paths_report": state_paths,
            "state_corruption_health_report": state_corruption,
            "state_migration_status": state_migration_status,
            "unified_state_behavior_report": unified_state_behavior,
            "unified_state_corruption_report": unified_state_corruption,
            "unified_state_rollback_report": unified_state_rollback,
            "unified_state_path_resolution_report": unified_state_path_resolution,
            "unified_state_doctor_snapshots": unified_state_doctor_snapshots,
            "unified_state_audit_payload": unified_state_audit_payload,
            "snapshot_coverage": snapshot_coverage,
            "stream_coverage": stream_coverage,
            "exit_code_coverage": exit_code_coverage,
            "failure_path_coverage": failure_path_coverage,
            "compatibility_aliases": compatibility_aliases,
            "known_parity_gaps": known_parity_gaps,
            "intentional_differences": intentional_differences,
            "unowned_scripts": unowned_scripts,
            "maintainer_scripts_outside_dev_cli": maintainer_scripts,
            "maintainer_control_plane_commands": maintainer_control_plane_commands,
            "maintainer_control_plane_report": maintainer_control_plane_report,
            "maintainer_control_plane_text_report": maintainer_control_plane_text,
            "repl_only_behaviors": repl_only_behaviors,
            "plugin_lifecycle_ownership_report": plugin_lifecycle_ownership,
            "plugin_scaffold_efficiency_report": plugin_scaffold_efficiency,
            "plugin_scaffold_lifecycle_proof_report": plugin_scaffold_lifecycle_proof,
            "plugin_namespace_abuse_proof_report": plugin_namespace_abuse_proof,
            "plugin_doctor_clarity_report": plugin_doctor_clarity,
            "plugin_explain_clarity_report": plugin_explain_clarity,
            "plugin_where_ownership_report": plugin_where_ownership,
            "plugin_command_set_status": plugin_command_set_status,
            "plugin_migration_report": plugin_migration_report,
            "config_closure_report": config_closure_report,
            "plugins_closure_report": plugins_closure_report,
            "history_closure_report": history_closure_report,
            "memory_closure_report": memory_closure_report,
            "diagnostics_closure_report": diagnostics_closure_report,
            "repl_shared_law_closure_report": repl_shared_law_closure_report,
            "command_family_closure_report": command_family_closure_report,
            "command_family_closure_report_text": command_family_closure_report_text,
            "command_family_partial_area_acceptance": command_family_partial_area_acceptance,
            "cross_surface_consistency_artifact": cross_surface_consistency_artifact,
            "cross_surface_drift_artifact": cross_surface_drift_artifact,
            "cross_surface_consistency_contract": cross_surface_consistency_contract,
            "cross_crate_duplication_report": cross_crate_duplication_report,
            "public_api_inventory_report": public_api_inventory_report,
            "crate_complexity_report": crate_complexity_report,
            "candidate_merge_later_report": candidate_merge_later_report,
            "candidate_keep_separate_report": candidate_keep_separate_report,
            "simplification_deletion_artifact": simplification_deletion_artifact,
            "simplification_deletion_artifact_text": simplification_deletion_artifact_text,
            "command_surface_consistency_summary": command_surface_consistency_summary,
        },
        "command_migration": {
            "matrix": migration_matrix,
            "rust_partial": migration_rust_partial,
            "python_only": migration_python_only,
            "intentional_differences": migration_intentional,
            "text": migration_text,
            "documented_python_not_proven": documented_not_proven,
            "public_python_paths_still_reachable": public_python_paths,
            "legacy_alias_paths_still_accepted": legacy_alias_paths,
            "compatibility_shims_still_active": active_compat_shims,
            "bridge_duplicate_law_report": bridge_duplicate_law,
            "bridge_wrapper_only_closure_report": bridge_wrapper_only_closure,
            "bridge_wrapper_only_closure_report_text": bridge_wrapper_only_closure_text,
        },
        "next_phase_priorities": next_phase,
        "next_phase_summary_text": next_phase_text,
        "next_simplification_priorities": next_phase_minimalism,
        "next_simplification_summary_text": next_phase_minimalism_text,
        "current_rust_state": state,
        "parity": parity,
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
