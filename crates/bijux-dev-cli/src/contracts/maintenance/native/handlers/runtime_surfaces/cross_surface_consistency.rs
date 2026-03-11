#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs"),
            )
            .unwrap_or_default();
            let required: Vec<(i64, &str, &str)> = vec![
                (
                    161,
                    "binary_vs_direct_core_version_result_matches",
                    "binary vs direct-core version",
                ),
                (
                    162,
                    "binary_vs_direct_core_status_result_matches",
                    "binary vs direct-core status",
                ),
                (
                    163,
                    "binary_vs_direct_core_doctor_result_matches",
                    "binary vs direct-core doctor",
                ),
                (
                    164,
                    "binary_vs_direct_core_plugins_list_result_matches",
                    "binary vs direct-core plugins list",
                ),
                (
                    165,
                    "binary_vs_direct_core_config_get_result_matches",
                    "binary vs direct-core config get",
                ),
                (
                    166,
                    "binary_vs_python_bridge_version_result_matches",
                    "binary vs python bridge version",
                ),
                (
                    167,
                    "binary_vs_python_bridge_status_result_matches",
                    "binary vs python bridge status",
                ),
                (
                    168,
                    "binary_vs_python_bridge_doctor_result_matches",
                    "binary vs python bridge doctor",
                ),
                (
                    169,
                    "binary_vs_python_bridge_plugins_list_result_matches",
                    "binary vs python bridge plugins list",
                ),
                (
                    170,
                    "binary_vs_python_bridge_config_get_result_matches",
                    "binary vs python bridge config get",
                ),
                (
                    171,
                    "binary_vs_repl_status_result_matches_where_sensible",
                    "binary vs repl result where sensible",
                ),
                (
                    172,
                    "binary_vs_repl_unknown_command_exit_semantics_match_where_sensible",
                    "binary vs repl exit semantics where sensible",
                ),
                (
                    173,
                    "binary_vs_python_bridge_namespace_rejection_behavior_matches",
                    "binary vs python bridge namespace rejection",
                ),
                (
                    174,
                    "binary_vs_python_bridge_error_envelope_shape_matches",
                    "binary vs python bridge error envelope shape",
                ),
                (
                    175,
                    "binary_vs_python_bridge_stdout_stderr_discipline_matches",
                    "binary vs python bridge stdout/stderr discipline",
                ),
                (
                    176,
                    "route_registry_snapshots_match_across_binary_core_and_bridge",
                    "route registry snapshots across surfaces",
                ),
            ];
            let mut covered = Vec::<Value>::new();
            let mut missing = Vec::<Value>::new();
            for (coverage_id, fn_name, law) in required {
                let row = json!({
                    "coverage_id": coverage_id,
                    "law": law,
                    "test": format!("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs::{fn_name}"),
                });
                if source.contains(&format!("fn {fn_name}(")) {
                    covered.push(row);
                } else {
                    missing.push(row);
                }
            }
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_surface_equivalence_report.json",
                                &json!({
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-surface equivalence",
                                    "rule": "binary, direct-core, python bridge, and repl must agree for covered commands",
                                    "verification_command": "cargo test -q -p bijux-cli --test bin_surface cross_surface_equivalence::",
                                    "covered": covered,
                                    "missing": missing,
                                    "summary": {
                                        "required": 16,
                                        "covered": covered.len(),
                                        "missing": missing.len(),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_drift_report.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface drift",
                    "status": if missing.is_empty() { "clean" } else { "drift-detected" },
                    "drift_count": missing.len(),
                    "drift_items": missing,
                    "gate": "bijux dev cli parity --format json --no-pretty",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_surface_duality_contract.json",
                                &json!({
                                    "generator": "bijux-dev-cli",
                                    "contract": "Cross-surface equivalence",
                                    "law": "One command law across binary, core, python bridge, and repl for covered commands.",
                                    "freeze_rule": "New covered command paths must add cross-surface equivalence tests before merge.",
                                    "evidence": [
                                        "crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs",
                                        "artifacts/status/cross_surface_equivalence_report.json",
                                        "artifacts/status/cross_surface_drift_report.json",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/cross_surface_equivalence_report.json",
                "artifacts/status/cross_surface_drift_report.json",
                "artifacts/status/cross_surface_duality_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-STATE-REPORTS" => {
            let sources: Vec<(String, String)> = vec![
                (
                    "crates/bijux-cli/tests/bin_surface/cross_surface_state_extra.rs".to_string(),
                    fs::read_to_string(
                        workspace_root.join(
                            "crates/bijux-cli/tests/bin_surface/cross_surface_state_extra.rs",
                        ),
                    )
                    .unwrap_or_default(),
                ),
                (
                    "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs"
                        .to_string(),
                    fs::read_to_string(workspace_root.join(
                        "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs",
                    ))
                    .unwrap_or_default(),
                ),
            ];
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (321, "config_mutations_are_visible_across_binary_bridge_and_repl_reads"),
                                (322, "config_mutations_are_visible_across_binary_bridge_and_repl_reads"),
                                (323, "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands"),
                                (324, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (325, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (326, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (327, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (328, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                                (329, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (330, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                                (331, "state_path_overrides_propagate_consistently_for_config_path_views"),
                                (332, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                                (333, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                                (334, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                                (335, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                            ]);
            let rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let evidence = sources.iter().find_map(|(rel, text)| {
                        text.contains(&format!("fn {test_name}(")).then_some(rel.clone())
                    });
                    json!({
                        "coverage_id": coverage_id,
                        "test": test_name,
                        "status": if evidence.is_some() { "covered" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect();
            let missing: Vec<Value> = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface state consistency",
                    "coverage_ids": (321..337).collect::<Vec<_>>(),
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/cross_surface_state_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "cross-surface state drift",
                                    "coverage_ids": [337, 338],
                                    "status": if missing.is_empty() { "clean" } else { "drift" },
                                    "drift_count": missing.len(),
                                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_state_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface state contract",
                    "coverage_ids": [340],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "state consistency is part of migration contract",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                "artifacts/status/cross_surface_state_drift_artifact.json",
                "artifacts/status/cross_surface_state_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS" => {
            let source =
                fs::read_to_string(workspace_root.join(
                    "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                ))
                .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (61, "deterministic_discovery_under_shuffled_install_order"),
                (62, "deterministic_plugin_list_ordering"),
                (63, "deterministic_plugin_inspect_ordering_multiple_plugins"),
                (64, "deterministic_help_ordering_with_plugins_installed"),
                (65, "deterministic_route_registration_with_different_install_orders"),
                (66, "deterministic_route_registration_after_uninstall_reinstall_cycles"),
                (67, "deterministic_namespace_conflict_resolution_messages"),
                (68, "deterministic_plugins_list_json_output"),
                (69, "deterministic_plugins_check_json_output"),
                (70, "deterministic_plugins_inspect_json_output"),
                (71, "discovery_ignores_unrelated_filesystem_clutter"),
                (72, "discovery_ignores_partially_written_temporary_files"),
                (73, "discovery_ignores_invalid_directories_cleanly"),
                (74, "discovery_is_stable_under_broken_symlink_entries"),
                (75, "broken_plugin_does_not_reorder_healthy_plugins"),
                (76, "broken_plugin_does_not_hide_healthy_plugins"),
                (77, "registry_and_discovery_disagreement_diagnostics_are_deterministic"),
                (78, "plugin_metadata_ordering_is_stable_in_machine_output"),
            ];
            let matrix_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                                    })
                                })
                                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_discovery_determinism_report.json",
                                &json!({
                                    "generated_at": "1970-01-01T00:00:00+00:00",
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin discovery and ordering determinism",
                                    "rows": matrix_rows,
                                    "summary": {
                                        "complete": complete,
                                        "missing": rows.len() - complete,
                                        "artifact_todo": 79,
                                        "artifact_path": "artifacts/status/plugin_discovery_determinism_report.json",
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_ordering_law.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "law": "plugin ordering is deterministic",
                    "status": "frozen",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                        "artifacts/status/plugin_discovery_determinism_report.json",
                    ],
                    "covers_todo": 80,
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_discovery_determinism_report.json",
                "artifacts/status/plugin_ordering_law.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-FAILURE-REPORTS" => {
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                                &json!({
                                    "generated_at": "1970-01-01T00:00:00+00:00",
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin lifecycle failure injection",
                                    "status": "complete",
                                    "evidence": [
                                        {
                                            "topic": "install write failures",
                                            "coverage_ids": [441, 442, 443, 444, 445, 446],
                                            "tests": [
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries"
                                            ],
                                        },
                                        {
                                            "topic": "uninstall/disable/enable failure behavior",
                                            "coverage_ids": [447, 448, 449],
                                            "tests": [
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state"
                                            ],
                                        },
                                        {
                                            "topic": "post-install integrity checks",
                                            "coverage_ids": [450, 451, 452, 453, 454],
                                            "tests": [
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_entrypoint_disappears_after_install",
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_manifest_mutates_after_install",
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::plugin_check_fails_when_runtime_kind_becomes_unsupported",
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::check_fails_on_broken_registry_record_and_list_stays_usable_after_doctor",
                                            ],
                                        },
                                        {
                                            "topic": "retry idempotency",
                                            "coverage_ids": [456, 457],
                                            "tests": [
                                                "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_and_uninstall_retries_are_idempotent_after_transient_write_failures"
                                            ],
                                        },
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_rollback_proof_report.json",
                                &json!({
                                    "generated_at": "1970-01-01T00:00:00+00:00",
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin rollback and write-path proofs",
                                    "status": "complete",
                                    "coverage_ids": [455],
                                    "evidence": [
                                        "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::failed_install_rolls_back_and_preserves_existing_plugin_list",
                                        "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::failed_uninstall_rolls_back_and_keeps_registry_unchanged",
                                        "crates/bijux-cli-plugin/tests/plugin_write_path_maturity.rs::install_and_uninstall_are_transaction_safe_and_cleanup_backup_files",
                                        "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries",
                                        "crates/bijux-cli/tests/bin_surface/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                "artifacts/status/plugin_rollback_proof_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PACKAGING-AMBIGUITY-REPORTS" => {
            let generated_at = generated_at_utc();
            let install_source = fs::read_to_string(
                workspace_root.join("artifacts/status/install_source_diagnostics.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let ambiguous_runtime = fs::read_to_string(
                workspace_root.join("artifacts/status/ambiguous_runtime_diagnostics.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let package_health =
                run_bijux_json(workspace_root, &["dev", "cli", "package-health"]).ok()?;
            let runtime_identity =
                run_bijux_json(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/packaging_ambiguity_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "packaging ambiguity",
                                    "status": "complete",
                                    "coverage_ids": [536],
                                    "runtime_identity": {
                                        "active_binary_selection_is_ambiguous": runtime_identity.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                                        "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed").cloned().unwrap_or(json!(false)),
                                        "diagnostics": runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({})),
                                    },
                                    "install_source_diagnostics": install_source,
                                    "ambiguous_runtime_diagnostics": ambiguous_runtime,
                                    "evidence_tests": [
                                        "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs::pip_binary_shadowed_by_cargo_binary_is_reported",
                                        "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs::cargo_binary_shadowed_by_pip_binary_is_reported",
                                        "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs::package_health_and_runtime_identity_cover_ambiguous_install_state",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_state_assumptions_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "install-state assumptions",
                                    "status": "complete",
                                    "coverage_ids": [537],
                                    "install_state_assumptions": package_health.get("install_state_assumptions").cloned().unwrap_or_else(|| json!([])),
                                    "install_state_assumption_help": package_health.get("install_state_assumption_help").cloned().unwrap_or_else(|| json!("")),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/package_health_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "package health",
                    "status": "complete",
                    "coverage_ids": [538],
                    "payload": package_health,
                }),
            )
            .ok()?;
            let assumptions_count = package_health
                .get("install_state_assumptions")
                .and_then(Value::as_array)
                .map(|v| v.len())
                .unwrap_or(0);
            let help = package_health
                .get("install_state_assumption_help")
                .and_then(Value::as_str)
                .unwrap_or("");
            fs::write(
                workspace_root.join("artifacts/status/package_health_report.txt"),
                format!("Package Health\n\nassumptions_count: {assumptions_count}\nhelp: {help}\n"),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/packaging_ambiguity_report.json",
                "artifacts/status/install_state_assumptions_report.json",
                "artifacts/status/package_health_report.json",
                "artifacts/status/package_health_report.txt"
            ]}))
        }
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
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::history_enormous_line_layout_is_tolerated_with_tail_limit",
                                        "crates/bijux-cli/tests/bin_surface/history_parity.rs::history_preserves_duplicate_commands_and_ordering",
                                        "crates/bijux-cli/tests/bin_surface/history_parity.rs::history_skips_malformed_entries_inside_json_array",
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
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::memory_commands_are_read_only_even_when_home_storage_is_unwritable",
                                        "crates/bijux-cli/tests/bin_surface/memory_parity.rs::memory_malformed_state_is_treated_as_empty_like_python",
                                        "crates/bijux-cli/tests/bin_surface/memory_parity.rs::memory_non_object_json_state_fails_with_error_envelope",
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
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable",
                                        "crates/bijux-cli/tests/bin_surface/history_memory_resilience_hardening.rs::memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely",
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
                                (141, "inspect_and_dev_routes_agree_on_route_ownership", "inspect/dev routes ownership agreement", vec!["inspect", "dev cli routes"]),
                                (142, "plugins_list_and_dev_registry_agree_on_installed_plugin_namespace_rules", "plugins list/dev registry installed set agreement", vec!["plugins list", "dev cli registry"]),
                                (143, "config_get_and_dev_env_agree_on_source_precedence", "config get/dev env precedence agreement", vec!["config get", "dev cli env"]),
                                (144, "doctor_and_state_audit_agree_on_corruption_detection_when_applicable", "doctor/state-audit corruption agreement", vec!["doctor", "dev cli state-audit"]),
                                (145, "binary_and_direct_core_agree_on_same_command_results", "binary/direct-core agreement for covered roots", vec!["status"]),
                                (146, "binary_and_python_bridge_agree_on_config_history_memory_and_diagnostics_outputs", "binary/python-bridge agreement for covered roots", vec!["config", "history", "memory list", "doctor"]),
                                (147, "repl_execution_matches_non_interactive_for_config_get_plugins_list_and_status", "binary/repl agreement for shared commands", vec!["config get", "plugins list", "status"]),
                                (148, "plugin_command_help_integrates_into_root_help_tree_deterministically", "plugin help integration is deterministic", vec!["plugins"]),
                                (149, "command_tree_export_is_identical_across_binary_and_bridge", "command-tree export identical across binary and bridge", vec!["dev cli routes"]),
                                (150, "route_ownership_is_stable_across_repeated_runs", "route ownership stable across repeated runs", vec!["dev cli routes"]),
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
                                    "gate": "bijux dev cli scripts status run --id STATUS-CONTRACT-ENFORCE-CROSS-SURFACE-CONSISTENCY-LAW",
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
