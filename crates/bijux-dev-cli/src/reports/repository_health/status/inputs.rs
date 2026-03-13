//! Status command artifact input loading.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Map, Value};

use crate::infra::artifacts::{read_json_if_exists, read_text_if_exists};

const REPORT_JSON_ITEMS: &[(&str, &str)] = &[
    ("root_commands", "artifacts/status/status_root_commands.json"),
    ("root_command_remaining_inventory", "artifacts/status/root_command_remaining_inventory.json"),
    ("root_command_impact_ranking", "artifacts/status/root_command_impact_ranking.json"),
    ("root_command_completion_report", "artifacts/status/root_command_completion_report.json"),
    ("root_command_closure_set", "artifacts/status/root_command_closure_set.json"),
    ("cli_subcommands", "artifacts/status/status_cli_subcommands.json"),
    ("maintainer_subcommands", "artifacts/status/status_maintainer_subcommands.json"),
    ("cli_command_remaining_inventory", "artifacts/status/cli_command_remaining_inventory.json"),
    ("cli_command_value_ranking", "artifacts/status/cli_command_value_ranking.json"),
    ("cli_command_completion_report", "artifacts/status/cli_command_completion_report.json"),
    ("cli_command_closure_set", "artifacts/status/cli_command_closure_set.json"),
    (
        "maintainer_command_remaining_inventory",
        "artifacts/status/maintainer_command_remaining_inventory.json",
    ),
    ("maintainer_command_value_ranking", "artifacts/status/maintainer_command_value_ranking.json"),
    (
        "maintainer_command_completion_report",
        "artifacts/status/maintainer_command_completion_report.json",
    ),
    ("maintainer_command_closure_set", "artifacts/status/maintainer_command_closure_set.json"),
    (
        "cli_maintainer_command_closure_report",
        "artifacts/status/cli_maintainer_command_closure_report.json",
    ),
    ("plugin_commands", "artifacts/status/status_plugin_commands.json"),
    ("repl_parity_coverage", "artifacts/status/status_repl_parity_coverage.json"),
    ("python_bridge_parity_coverage", "artifacts/status/status_python_bridge_parity_coverage.json"),
    (
        "install_packaging_parity_coverage",
        "artifacts/status/status_install_packaging_parity_coverage.json",
    ),
    ("state_behavior_coverage", "artifacts/status/status_state_behavior_coverage.json"),
    ("state_paths_report", "artifacts/status/status_state_paths_report.json"),
    (
        "state_corruption_health_report",
        "artifacts/status/status_state_corruption_health_report.json",
    ),
    ("state_migration_status", "artifacts/status/state_migration_status.json"),
    ("unified_state_behavior_report", "artifacts/status/unified_state_behavior_report.json"),
    ("unified_state_corruption_report", "artifacts/status/unified_state_corruption_report.json"),
    ("unified_state_rollback_report", "artifacts/status/unified_state_rollback_report.json"),
    (
        "unified_state_path_resolution_report",
        "artifacts/status/unified_state_path_resolution_report.json",
    ),
    ("unified_state_doctor_snapshots", "artifacts/status/unified_state_doctor_snapshots.json"),
    ("unified_state_audit_payload", "artifacts/status/unified_state_audit_payload.json"),
    ("snapshot_coverage", "artifacts/status/status_snapshot_coverage.json"),
    ("stream_coverage", "artifacts/status/status_stream_coverage.json"),
    ("exit_code_coverage", "artifacts/status/status_exit_code_coverage.json"),
    ("failure_path_coverage", "artifacts/status/status_failure_path_coverage.json"),
    ("compatibility_aliases", "artifacts/status/status_compatibility_aliases.json"),
    ("known_parity_gaps", "artifacts/status/status_known_parity_gaps.json"),
    ("intentional_differences", "artifacts/status/status_intentional_differences.json"),
    ("unowned_maintenance", "artifacts/status/status_unowned_maintenance.json"),
    (
        "maintainer_maintenance_outside_control_plane",
        "artifacts/status/maintainer_maintenance_outside_control_plane.json",
    ),
    (
        "maintainer_control_plane_commands",
        "artifacts/status/maintainer_control_plane_commands.json",
    ),
    ("maintainer_control_plane_report", "artifacts/status/maintainer_control_plane_report.json"),
    ("repl_only_behaviors", "artifacts/status/repl_only_behaviors.json"),
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
    ("plugin_doctor_clarity_report", "artifacts/status/plugin_doctor_clarity_report.json"),
    ("plugin_explain_clarity_report", "artifacts/status/plugin_explain_clarity_report.json"),
    ("plugin_where_ownership_report", "artifacts/status/plugin_where_ownership_report.json"),
    ("plugin_command_set_status", "artifacts/status/plugin_command_set_status.json"),
    ("plugin_migration_report", "artifacts/status/plugin_migration_report.json"),
    ("config_closure_report", "artifacts/status/config_closure_report.json"),
    ("plugins_closure_report", "artifacts/status/plugins_closure_report.json"),
    ("history_closure_report", "artifacts/status/history_closure_report.json"),
    ("memory_closure_report", "artifacts/status/memory_closure_report.json"),
    ("diagnostics_closure_report", "artifacts/status/diagnostics_closure_report.json"),
    ("repl_shared_law_closure_report", "artifacts/status/repl_shared_law_closure_report.json"),
    ("command_family_closure_report", "artifacts/status/command_family_closure_report.json"),
    (
        "command_family_partial_area_acceptance",
        "artifacts/status/command_family_partial_area_acceptance.json",
    ),
    (
        "cross_surface_consistency_artifact",
        "artifacts/status/cross_surface_consistency_artifact.json",
    ),
    ("cross_surface_drift_artifact", "artifacts/status/cross_surface_drift_artifact.json"),
    (
        "cross_surface_consistency_contract",
        "artifacts/status/cross_surface_consistency_contract.json",
    ),
    ("cross_crate_duplication_report", "artifacts/status/cross_crate_duplication_report.json"),
    ("public_api_inventory_report", "artifacts/status/public_api_inventory_report.json"),
    ("crate_complexity_report", "artifacts/status/crate_complexity_report.json"),
    ("candidate_merge_later_report", "artifacts/status/candidate_merge_later_report.json"),
    ("candidate_keep_separate_report", "artifacts/status/candidate_keep_separate_report.json"),
    ("simplification_deletion_artifact", "artifacts/status/simplification_deletion_artifact.json"),
    (
        "command_surface_consistency_summary",
        "artifacts/status/command_surface_consistency_summary.json",
    ),
];

const REPORT_TEXT_ITEMS: &[(&str, &str)] = &[
    ("root_command_completion_report_text", "artifacts/status/root_command_completion_report.txt"),
    (
        "cli_maintainer_command_closure_report_text",
        "artifacts/status/cli_maintainer_command_closure_report.txt",
    ),
    (
        "maintainer_control_plane_text_report",
        "artifacts/status/maintainer_control_plane_text_report.txt",
    ),
    ("command_family_closure_report_text", "artifacts/status/command_family_closure_report.txt"),
    (
        "simplification_deletion_artifact_text",
        "artifacts/status/simplification_deletion_artifact.txt",
    ),
];

const COMMAND_MIGRATION_JSON_ITEMS: &[(&str, &str)] = &[
    ("rust_partial", "artifacts/status/command_migration_rust_partial.json"),
    ("python_only", "artifacts/status/command_migration_python_only.json"),
    ("intentional_differences", "artifacts/status/command_migration_intentional_differences.json"),
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
    ("compatibility_shims_still_active", "artifacts/status/compatibility_shims_still_active.json"),
    ("bridge_duplicate_law_report", "artifacts/status/bridge_duplicate_law_report.json"),
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
            ((*key).to_string(), read_json_if_exists(&workspace_root.join(rel_path)))
        })
        .collect()
}

fn load_text_entries(workspace_root: &Path, entries: &[(&str, &str)]) -> Map<String, Value> {
    entries
        .iter()
        .map(|(key, rel_path)| {
            ((*key).to_string(), Value::String(read_text_if_exists(&workspace_root.join(rel_path))))
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

fn is_empty_object(value: &Value) -> bool {
    matches!(value, Value::Object(map) if map.is_empty())
}

fn map_matrix_status_to_status(matrix_status: &str) -> &'static str {
    match matrix_status {
        "complete" | "rust-complete" => "complete",
        "missing" | "python-only" => "missing",
        "shim" => "shim",
        _ => "partial",
    }
}

fn map_status_to_migration(status: &str) -> &'static str {
    match status {
        "complete" | "rust-complete" => "rust-complete",
        "missing" | "python-only" => "python-only",
        "intentionally-different" | "different-by-decision" => "intentionally-different",
        _ => "rust-partial",
    }
}

fn parity_matrix_rows(workspace_root: &Path) -> Vec<Value> {
    read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_matrix.json"))
        .get("commands")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn synthesize_status_report(parity_rows: &[Value]) -> Value {
    let mut commands = Vec::new();
    for row in parity_rows {
        let command =
            row.get("command").and_then(Value::as_str).unwrap_or_default().trim().to_string();
        if command.is_empty() {
            continue;
        }
        let matrix_status = row.get("status").and_then(Value::as_str).unwrap_or("partial");
        commands.push(json!({
            "command": command,
            "status": map_matrix_status_to_status(matrix_status),
            "matrix_status": matrix_status,
            "group": row.get("group").cloned().unwrap_or_else(|| json!("root")),
            "owner": row.get("owner").cloned().unwrap_or_else(|| json!("rust-foundation")),
            "reason": row.get("reason").cloned().unwrap_or_else(|| json!("")),
            "blocker": row.get("blocker").cloned().unwrap_or_else(|| json!(if matrix_status == "complete" { "" } else { "parity coverage incomplete" })),
            "confidence": row.get("confidence").cloned().unwrap_or_else(|| json!(0.35))
        }));
    }

    let complete = commands.iter().filter(|row| row["status"] == "complete").count();
    let missing = commands.iter().filter(|row| row["status"] == "missing").count();
    let partial = commands.iter().filter(|row| row["status"] == "partial").count();
    let shim = commands.iter().filter(|row| row["status"] == "shim").count();

    json!({
        "generated_at": "1970-01-01T00:00:00+00:00",
        "generator": "bijux-dev-cli",
        "commands": commands,
        "summary": {
            "total": complete + missing + partial + shim,
            "complete": complete,
            "missing": missing,
            "partial": partial,
            "shim": shim,
        }
    })
}

fn ensure_status_report(status_report: Value, parity_rows: &[Value]) -> Value {
    let mut report = if is_empty_object(&status_report) {
        synthesize_status_report(parity_rows)
    } else {
        status_report
    };

    let commands = report.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    let complete = commands.iter().filter(|row| row["status"] == "complete").count();
    let missing = commands.iter().filter(|row| row["status"] == "missing").count();
    let partial = commands.iter().filter(|row| row["status"] == "partial").count();
    let shim = commands.iter().filter(|row| row["status"] == "shim").count();

    if !report.is_object() {
        report = json!({});
    }
    if let Some(obj) = report.as_object_mut() {
        obj.entry("commands".to_string()).or_insert_with(|| Value::Array(commands.clone()));
        obj.entry("summary".to_string()).or_insert_with(|| {
            json!({
                "total": complete + missing + partial + shim,
                "complete": complete,
                "missing": missing,
                "partial": partial,
                "shim": shim,
            })
        });
    }

    report
}

fn matrix_row_from_status_row(row: &Value) -> Option<Value> {
    let command = row.get("command").and_then(Value::as_str).unwrap_or_default().trim().to_string();
    if command.is_empty() {
        return None;
    }

    let matrix_status = row
        .get("matrix_status")
        .and_then(Value::as_str)
        .or_else(|| row.get("status").and_then(Value::as_str))
        .unwrap_or("partial");
    let migration_status = map_status_to_migration(matrix_status);

    let reason = row
        .get("reason")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if migration_status == "intentionally-different" {
                "intentional compatibility difference".to_string()
            } else {
                String::new()
            }
        });
    let blocker = row
        .get("blocker")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            if migration_status == "rust-partial" {
                "parity coverage incomplete".to_string()
            } else {
                String::new()
            }
        });

    let evidence_links = row
        .get("evidence_links")
        .and_then(Value::as_array)
        .cloned()
        .filter(|rows| !rows.is_empty())
        .unwrap_or_else(|| vec![json!("artifacts/status/status.json")]);
    let parity_coverage = if migration_status == "rust-complete" {
        json!({"stdout": true, "stderr": true, "exit_code": true})
    } else {
        json!({"stdout": false, "stderr": false, "exit_code": false})
    };

    Some(json!({
        "command": command,
        "status": migration_status,
        "group": row.get("group").cloned().unwrap_or_else(|| json!("root")),
        "owner": row.get("owner").cloned().unwrap_or_else(|| json!("rust-foundation")),
        "confidence": row.get("confidence").cloned().unwrap_or_else(|| json!(0.35)),
        "reason": reason,
        "blocker": blocker,
        "evidence_links": evidence_links,
        "shim_alias_dependency": {"aliases": [], "shims": []},
        "parity_coverage": parity_coverage
    }))
}

fn synthesize_migration_matrix(status_report: &Value, parity_rows: &[Value]) -> Value {
    let mut by_command: BTreeMap<String, Value> = BTreeMap::new();

    for row in parity_rows {
        if let Some(mapped) = matrix_row_from_status_row(row) {
            if let Some(command) = mapped.get("command").and_then(Value::as_str) {
                by_command.insert(command.to_string(), mapped);
            }
        }
    }
    for row in status_report.get("commands").and_then(Value::as_array).into_iter().flatten() {
        if let Some(mapped) = matrix_row_from_status_row(row) {
            if let Some(command) = mapped.get("command").and_then(Value::as_str) {
                by_command.insert(command.to_string(), mapped);
            }
        }
    }

    let commands: Vec<Value> = by_command.into_values().collect();
    let rust_complete = commands.iter().filter(|row| row["status"] == "rust-complete").count();
    let rust_partial = commands.iter().filter(|row| row["status"] == "rust-partial").count();
    let python_only = commands.iter().filter(|row| row["status"] == "python-only").count();
    let intentional =
        commands.iter().filter(|row| row["status"] == "intentionally-different").count();

    json!({
        "commands": commands,
        "summary": {
            "total": rust_complete + rust_partial + python_only + intentional,
            "rust-complete": rust_complete,
            "rust-partial": rust_partial,
            "python-only": python_only,
            "intentionally-different": intentional
        }
    })
}

fn ensure_migration_matrix(
    mut matrix: Value,
    status_report: &Value,
    parity_rows: &[Value],
) -> Value {
    normalize_migration_matrix(&mut matrix);
    let missing_rows = matrix.get("commands").and_then(Value::as_array).is_none_or(Vec::is_empty);
    if is_empty_object(&matrix) || missing_rows {
        matrix = synthesize_migration_matrix(status_report, parity_rows);
    }

    matrix
}

fn ensure_migration_sidecars(command_migration: &mut Map<String, Value>, matrix: &Value) {
    if command_migration.get("bridge_duplicate_law_report").is_some_and(is_empty_object) {
        command_migration.insert(
            "bridge_duplicate_law_report".to_string(),
            json!({
                "status": "unavailable",
                "source": "artifacts/status/bridge_duplicate_law_report.json"
            }),
        );
    }

    let rows = matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    let rust_partial: Vec<Value> =
        rows.iter().filter(|row| row["status"] == "rust-partial").cloned().collect();
    let python_only: Vec<Value> =
        rows.iter().filter(|row| row["status"] == "python-only").cloned().collect();
    let intentional: Vec<Value> =
        rows.iter().filter(|row| row["status"] == "intentionally-different").cloned().collect();

    if command_migration.get("rust_partial").is_some_and(is_empty_object) {
        command_migration.insert(
            "rust_partial".to_string(),
            json!({"commands": rust_partial, "summary": {"count": rust_partial.len()}}),
        );
    }
    if command_migration.get("python_only").is_some_and(is_empty_object) {
        command_migration.insert(
            "python_only".to_string(),
            json!({"commands": python_only, "summary": {"count": python_only.len()}}),
        );
    }
    if command_migration.get("intentional_differences").is_some_and(is_empty_object) {
        command_migration.insert(
            "intentional_differences".to_string(),
            json!({"commands": intentional, "summary": {"count": intentional.len()}}),
        );
    }
}

/// Load all status report inputs from workspace artifacts.
#[must_use]
pub fn load_status_inputs(workspace_root: &Path) -> StatusInputs {
    let mut reports = load_json_entries(workspace_root, REPORT_JSON_ITEMS);
    reports.extend(load_text_entries(workspace_root, REPORT_TEXT_ITEMS));

    let mut command_migration = load_json_entries(workspace_root, COMMAND_MIGRATION_JSON_ITEMS);
    command_migration.extend(load_text_entries(workspace_root, COMMAND_MIGRATION_TEXT_ITEMS));

    let parity_rows = parity_matrix_rows(workspace_root);
    let status_report = ensure_status_report(
        read_json_if_exists(&workspace_root.join("artifacts/status/status.json")),
        &parity_rows,
    );
    let matrix = ensure_migration_matrix(
        read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json")),
        &status_report,
        &parity_rows,
    );
    command_migration.insert("matrix".to_string(), matrix);
    if let Some(matrix_payload) = command_migration.get("matrix").cloned() {
        ensure_migration_sidecars(&mut command_migration, &matrix_payload);
    }

    StatusInputs {
        state: read_json_if_exists(
            &workspace_root.join("artifacts/status/current_rust_state.json"),
        ),
        parity: read_json_if_exists(
            &workspace_root.join("artifacts/parity/rust_python_parity_report.json"),
        ),
        status_report,
        reports,
        command_migration,
        priority_plan: read_json_if_exists(
            &workspace_root.join("artifacts/status/priority_plan.json"),
        ),
        priority_plan_text: read_text_if_exists(
            &workspace_root.join("artifacts/status/priority_plan.txt"),
        ),
        simplification_priorities: read_json_if_exists(
            &workspace_root.join("artifacts/status/simplification_priorities.json"),
        ),
        simplification_priorities_text: read_text_if_exists(
            &workspace_root.join("artifacts/status/simplification_priorities.txt"),
        ),
    }
}
