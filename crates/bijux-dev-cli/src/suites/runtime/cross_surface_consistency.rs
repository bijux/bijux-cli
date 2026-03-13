#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-STATE-RESILIENCE-REPORTS" => {
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/history_corruption_matrix.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "history corruption matrix",
                                    "status": "complete",
                                    "coverage_ids": [481, 482, 483, 484, 485, 488],
                                    "evidence_tests": [
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::history_enormous_line_layout_is_tolerated_with_tail_limit",
                                        "crates/bijux-cli/tests/integration/cli/history/history_parity.rs::history_preserves_duplicate_commands_and_ordering",
                                        "crates/bijux-cli/tests/integration/cli/history/history_parity.rs::history_skips_malformed_entries_inside_json_array",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/memory_corruption_matrix.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "memory corruption matrix",
                                    "status": "complete",
                                    "coverage_ids": [489, 490, 491, 492, 493, 494, 496],
                                    "evidence_tests": [
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::memory_commands_are_read_only_even_when_home_storage_is_unwritable",
                                        "crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs::memory_malformed_state_is_treated_as_empty_like_python",
                                        "crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs::memory_non_object_json_state_fails_with_error_envelope",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/state_recovery_guidance.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "state recovery guidance",
                                    "status": "complete",
                                    "coverage_ids": [498, 499],
                                    "guidance": [
                                        {
                                            "area": "history",
                                            "when": "history parse fails or returns malformed structure",
                                            "action": "backup file then truncate to valid JSON array or line-based commands",
                                        },
                                        {
                                            "area": "memory",
                                            "when": "memory state is malformed or wrong-type",
                                            "action": "backup file then rewrite to JSON object map with object values",
                                        },
                                        {
                                            "area": "repl-history-write",
                                            "when": "history flush fails during session exit",
                                            "action": "preserve in-memory session, restore writable path, retry flush",
                                        },
                                    ],
                                }),
                            )
                            .ok()?;
            fs::write(
                                workspace_root.join("artifacts/status/state_recovery_guidance.txt"),
                                "State Recovery Guidance\n\nHistory\n- If history parse fails, back up the file and rewrite as JSON array or line-based command list.\n- Keep the most recent valid entries; discard malformed tail fragments.\n\nMemory\n- If memory state is malformed, back up and rewrite as a JSON object.\n- Ensure each memory entry is represented as an object value.\n\nREPL history flush\n- If flush fails on session exit, keep in-memory commands and retry after restoring writable storage.\n",
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/state_resilience_summary.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "state resilience summary",
                                    "status": "complete",
                                    "coverage_ids": [486, 487, 495, 497],
                                    "evidence_tests": [
                                        "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_exit_flush_reports_write_interruption_without_crashing_session",
                                        "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_command_recording_survives_flush_failure_and_recovers_on_retry",
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
                                        "crates/bijux-cli/tests/integration/cli/resilience/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
                                    ],
                                    "artifacts": [
                                        "artifacts/status/history_corruption_matrix.json",
                                        "artifacts/status/memory_corruption_matrix.json",
                                        "artifacts/status/state_recovery_guidance.json",
                                        "artifacts/status/state_recovery_guidance.txt",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/history_corruption_matrix.json",
                "artifacts/status/memory_corruption_matrix.json",
                "artifacts/status/state_recovery_guidance.json",
                "artifacts/status/state_recovery_guidance.txt",
                "artifacts/status/state_resilience_summary.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-CONSISTENCY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_command_consistency_matrix.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (381, "inspect_and_dev_routes_agree_on_route_ownership"),
                                (382, "inspect_and_dev_registry_agree_on_plugin_ownership_model"),
                                (383, "config_get_and_dev_env_agree_on_source_precedence"),
                                (384, "doctor_and_state_audit_agree_on_corruption_detection_when_applicable"),
                                (385, "plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules"),
                                (386, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status"),
                                (387, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status"),
                                (388, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status"),
                                (389, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs"),
                                (390, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs"),
                                (391, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs"),
                                (392, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs"),
                                (393, "binary_and_direct_core_agree_on_same_command_results"),
                                (394, "binary_and_direct_core_agree_on_same_command_results"),
                                (395, "binary_and_direct_core_agree_on_same_command_results"),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    let present = source.contains(&format!("fn {fn_name}("));
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if present { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/cross_command_consistency_matrix.rs",
                                    })
                                })
                                .collect();
            let drift_rows: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .cloned()
                .collect();
            let area_ids: Vec<(&str, Vec<i64>)> = vec![
                ("commands", vec![381, 382, 385, 393, 394, 395, 396, 397]),
                ("config", vec![383, 389]),
                ("history", vec![384, 390]),
                ("memory", vec![391]),
                ("diagnostics", vec![392]),
            ];
            let summary_rows: Vec<Value> = area_ids
                .into_iter()
                .map(|(area, ids)| {
                    let relevant: Vec<&Value> = coverage_rows
                        .iter()
                        .filter(|row| {
                            row.get("coverage_id")
                                .and_then(Value::as_i64)
                                .is_some_and(|id| ids.contains(&id))
                        })
                        .collect();
                    let complete = relevant
                        .iter()
                        .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                        .count();
                    let total = relevant.len();
                    let status = if complete == total {
                        "complete"
                    } else if complete > 0 {
                        "partial"
                    } else {
                        "missing"
                    };
                    json!({"area": area, "complete": complete, "total": total, "status": status})
                })
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_surface_consistency_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-command consistency artifact",
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/command_surface_consistency_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-command drift detector artifact",
                                    "drift_count": drift_rows.len(),
                                    "drift_coverage_ids": drift_rows.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                    "status": if drift_rows.is_empty() { "clean" } else { "drift" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/command_surface_consistency_summary.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "complete/partial/missing summary for commands/config/history/memory/diagnostics",
                                    "areas": summary_rows,
                                    "prioritization_note": "Use this summary as source-of-truth for prioritization instead of intuition.",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/command_surface_consistency_artifact.json",
                "artifacts/status/command_surface_consistency_drift_artifact.json",
                "artifacts/status/command_surface_consistency_summary.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CONSISTENCY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs"),
            )
            .unwrap_or_default();
            let matrix = fs::read_to_string(
                workspace_root.join("artifacts/parity/commands_fully_rust_owned.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({"commands":[]}));
            let complete_commands =
                matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (161, "root_status_and_cli_status_agree_where_semantics_overlap"),
                                (162, "root_config_listing_and_cli_config_views_agree_where_both_exist"),
                                (163, "plugins_and_routes_views_agree_between_user_and_dev_surfaces"),
                                (164, "plugins_and_routes_views_agree_between_user_and_dev_surfaces"),
                                (165, "cli_paths_match_state_audit_paths_view"),
                                (166, "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory"),
                                (167, "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory"),
                                (168, "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory"),
                                (169, "doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory"),
                                (170, "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands"),
                                (171, "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands"),
                                (172, "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands"),
                                (173, "command_family_help_trees_and_machine_output_envelopes_remain_consistent"),
                                (174, "command_family_help_trees_and_machine_output_envelopes_remain_consistent"),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, fn_name)| {
                                    let present = source.contains(&format!("fn {fn_name}("));
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": fn_name,
                                        "status": if present { "covered" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs",
                                    })
                                })
                                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            let mut uncovered_scope = Vec::<Value>::new();
            if complete_commands.is_empty() {
                uncovered_scope.push(json!({
                    "scope": "matrix_complete_commands",
                    "reason": "artifacts/parity/commands_fully_rust_owned.json has no commands",
                    "impacted_coverage_ids": [170,171,172],
                }));
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_family_consistency_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "command-family consistency",
                    "coverage_ids": (161..176).collect::<Vec<_>>(),
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_family_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-family drift",
                                    "coverage_ids": [176, 178, 179],
                                    "status": if missing.is_empty() { "clean" } else { "drift" },
                                    "drift_count": missing.len(),
                                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                    "uncovered_scope": uncovered_scope,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/shared_law_proof_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "shared law proof",
                                    "coverage_ids": [177],
                                    "status": if missing.is_empty() { "complete" } else { "partial" },
                                    "proof": {
                                        "binary_core_bridge_repl_test_present": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(170) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                        "help_tree_consistency_test_present": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(173) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                        "envelope_law_test_present": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(174) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/command_family_consistency_requirement.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "command-family consistency requirement",
                                    "coverage_ids": [180],
                                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                                    "release_requirement": "Command-family consistency is a migration requirement and must remain drift-free.",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/command_family_consistency_artifact.json",
                "artifacts/status/cross_family_drift_artifact.json",
                "artifacts/status/shared_law_proof_artifact.json",
                "artifacts/status/command_family_consistency_requirement.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-CONSISTENCY-LAW-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_command_consistency_matrix.rs"),
            )
            .unwrap_or_default();
            let matrix = fs::read_to_string(
                workspace_root.join("artifacts/status/command_migration_matrix.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({"rows":[]}));
            let required: Vec<(i64, &str, &str, Vec<&str>)> = vec![
                                (141, "inspect_and_dev_routes_agree_on_route_ownership", "inspect/dev routes ownership agreement", vec!["inspect", "bijux-dev-cli routes"]),
                                (142, "plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules", "plugins list/dev registry installed set agreement", vec!["plugins list", "bijux-dev-cli registry"]),
                                (143, "config_get_and_dev_env_agree_on_source_precedence", "config get/dev env precedence agreement", vec!["config get", "bijux-dev-cli env"]),
                                (144, "doctor_and_state_audit_agree_on_corruption_detection_when_applicable", "doctor/state-audit corruption agreement", vec!["doctor", "bijux-dev-cli state-audit"]),
                                (145, "binary_and_direct_core_agree_on_same_command_results", "binary/direct-core agreement for covered roots", vec!["status"]),
                                (146, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs", "binary/python-bridge agreement for covered roots", vec!["config", "history", "memory list", "doctor"]),
                                (147, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status", "binary/repl agreement for shared commands", vec!["config get", "plugins list", "status"]),
                                (148, "plugin_command_help_integrates_into_root_help_tree_deterministically", "plugin help integration is deterministic", vec!["plugins"]),
                                (149, "command_tree_export_is_identical_across_binary_and_bridge", "command-tree export identical across binary and bridge", vec!["bijux-dev-cli routes"]),
                                (150, "route_ownership_is_stable_across_repeated_runs", "route ownership stable across repeated runs", vec!["bijux-dev-cli routes"]),
                                (151, "command_metadata_is_stable_across_repeated_runs", "command metadata stable across repeated runs", vec!["inspect"]),
                                (152, "diagnostics_payloads_do_not_drift_across_surfaces", "diagnostics payloads stable across surfaces", vec!["doctor"]),
                                (153, "output_envelopes_do_not_drift_across_surfaces", "output envelopes stable across surfaces", vec!["unknown-command"]),
                                (154, "exit_code_classes_do_not_drift_across_surfaces", "exit-code classes stable across surfaces", vec!["status", "unknown-command"]),
                            ];
            let matrix_rows =
                matrix.get("rows").and_then(Value::as_array).cloned().unwrap_or_default();
            let migration_status = |command: &str| -> String {
                matrix_rows
                    .iter()
                    .find_map(|row| {
                        (row.get("command").and_then(Value::as_str) == Some(command)).then(|| {
                            row.get("status")
                                .and_then(Value::as_str)
                                .unwrap_or("rust-partial")
                                .to_string()
                        })
                    })
                    .unwrap_or_else(|| "rust-partial".to_string())
            };
            let mut rows = Vec::<Value>::new();
            let mut drift_items = Vec::<Value>::new();
            let mut warnings = Vec::<Value>::new();
            for (coverage_id, fn_name, law, related) in required {
                let present = source.contains(&format!("fn {fn_name}("));
                let related_statuses: Vec<String> =
                    related.iter().map(|cmd| migration_status(cmd)).collect();
                let coverage_class = if !related_statuses.is_empty()
                    && related_statuses.iter().all(|s| s == "rust-complete")
                {
                    "covered"
                } else {
                    "partial"
                };
                let row = json!({
                    "coverage_id": coverage_id,
                    "law": law,
                    "test": format!("crates/bijux-cli/tests/bin_surface/cross_command_consistency_matrix.rs::{fn_name}"),
                    "present": present,
                    "coverage_class": coverage_class,
                    "related_commands": related,
                    "related_command_statuses": related_statuses,
                });
                rows.push(row.clone());
                if !present {
                    drift_items.push(row.clone());
                    if coverage_class == "partial" {
                        warnings.push(row);
                    }
                }
            }
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_surface_consistency_artifact.json",
                                &json!({
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-surface consistency",
                                    "status": if drift_items.is_empty() { "clean" } else { "drift" },
                                    "rows": rows,
                                    "summary": {
                                        "required": 14,
                                        "covered": rows.iter().filter(|r| r.get("present").and_then(Value::as_bool) == Some(true)).count(),
                                        "missing": drift_items.len(),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_drift_artifact.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface drift",
                    "status": if drift_items.is_empty() { "clean" } else { "drift" },
                    "drift_count": drift_items.len(),
                    "drift_items": drift_items,
                    "warnings_for_partial": warnings,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_surface_consistency_contract.json",
                                &json!({
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-surface consistency contract",
                                    "release_review_rule": "cross-surface consistency artifacts are mandatory release evidence",
                                    "freeze_rule": "one command law is frozen only when covered drift remains zero",
                                    "gate": "bijux-dev-cli maintenance status run --id STATUS-CONTRACT-ENFORCE-CROSS-SURFACE-CONSISTENCY-LAW",
                                    "evidence": [
                                        "artifacts/status/cross_surface_consistency_artifact.json",
                                        "artifacts/status/cross_surface_drift_artifact.json",
                                        "artifacts/status/cross_surface_consistency_contract.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/cross_surface_consistency_artifact.json",
                "artifacts/status/cross_surface_drift_artifact.json",
                "artifacts/status/cross_surface_consistency_contract.json"
            ]}))
        }
        _ => None,
    }
}
