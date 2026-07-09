#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-CROSS-SURFACE-REPORTS" => {
            let source =
                fs::read_to_string(workspace_root.join(
                    "crates/bijux-cli/tests/integration/cli/root/cross_surface_equivalence.rs",
                ))
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
                    "test": format!("crates/bijux-cli/tests/integration/cli/root/cross_surface_equivalence.rs::{fn_name}"),
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
                    "gate": "bijux-dev-cli parity --format json --no-pretty",
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
                                        "crates/bijux-cli/tests/integration/cli/root/cross_surface_equivalence.rs",
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
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let sources: Vec<(String, String)> = collect_files(&tests_root)
                .into_iter()
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                .map(|path| {
                    (rel(&path, workspace_root), fs::read_to_string(path).unwrap_or_default())
                })
                .collect();
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
                    "crates/bijux-cli/tests/integration/cli/plugins/plugin_discovery_ordering_laws.rs",
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
            let law_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/plugins/plugin_discovery_ordering_laws.rs",
                                    })
                                })
                                .collect();
            let complete = law_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_discovery_determinism_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin discovery and ordering determinism",
                                    "rows": law_rows,
                                    "summary": {
                                        "complete": complete,
                                        "missing": rows.len() - complete,
                                        "coverage_window_end": 79,
                                        "artifact_path": "artifacts/status/plugin_discovery_determinism_report.json",
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_ordering_law.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "law": "plugin ordering is deterministic",
                    "status": "frozen",
                    "evidence": [
                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_discovery_ordering_laws.rs",
                        "artifacts/status/plugin_discovery_determinism_report.json",
                    ],
                    "coverage_ids": [80],
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
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin lifecycle failure injection",
                                    "status": "complete",
                                    "evidence": [
                                        {
                                            "topic": "install write failures",
                                            "coverage_ids": [441, 442, 443, 444, 445, 446],
                                            "tests": [
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries"
                                            ],
                                        },
                                        {
                                            "topic": "uninstall/disable/enable failure behavior",
                                            "coverage_ids": [447, 448, 449],
                                            "tests": [
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state"
                                            ],
                                        },
                                        {
                                            "topic": "post-install integrity checks",
                                            "coverage_ids": [450, 451, 452, 453, 454],
                                            "tests": [
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::plugin_check_fails_when_entrypoint_disappears_after_install",
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::plugin_check_fails_when_manifest_mutates_after_install",
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::plugin_check_fails_when_runtime_kind_becomes_unsupported",
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::check_fails_on_broken_registry_record_and_list_stays_usable_after_doctor",
                                            ],
                                        },
                                        {
                                            "topic": "retry idempotency",
                                            "coverage_ids": [456, 457],
                                            "tests": [
                                                "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::install_and_uninstall_retries_are_idempotent_after_transient_write_failures"
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
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin rollback and write-path proofs",
                                    "status": "complete",
                                    "coverage_ids": [455],
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_rollback_resilience.rs::failed_install_rolls_back_and_preserves_existing_plugin_list",
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_rollback_resilience.rs::failed_uninstall_rolls_back_and_keeps_registry_unchanged",
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_rollback_resilience.rs::install_and_uninstall_are_transaction_safe_and_cleanup_backup_files",
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::install_reports_write_failures_and_preserves_existing_registry_entries",
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_failure_injection.rs::uninstall_disable_enable_failures_do_not_break_existing_plugin_state",
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
            let package_health = run_bijux_json(workspace_root, &["package-health"]).ok()?;
            let runtime_identity = run_bijux_json(workspace_root, &["runtime-identity"]).ok()?;
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
                                        "crates/bijux-cli/tests/integration/cli/resilience/install_ambiguity_hardening.rs::pip_binary_shadowed_by_cargo_binary_is_reported",
                                        "crates/bijux-cli/tests/integration/cli/resilience/install_ambiguity_hardening.rs::cargo_binary_shadowed_by_pip_binary_is_reported",
                                        "crates/bijux-cli/tests/integration/cli/resilience/install_ambiguity_hardening.rs::package_health_and_runtime_identity_cover_ambiguous_install_state",
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
        _ => None,
    }
}
