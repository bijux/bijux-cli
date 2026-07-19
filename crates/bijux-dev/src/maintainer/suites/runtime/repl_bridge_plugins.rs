#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-TEST-COVERAGE" => {
            let source = fs::read_to_string(workspace_root.join(
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_lifecycle_coverage.rs",
            ))
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (21, "python_scaffold_install_list_inspect_uninstall_end_to_end"),
                (22, "rust_scaffold_install_list_inspect_uninstall_end_to_end"),
                (23, "installed_plugin_help_entrypoint_is_deterministic"),
                (24, "installed_plugin_disable_rejects_plugin_check"),
                (25, "disabled_plugin_enable_restores_plugin_check"),
                (26, "duplicate_install_without_force_is_deterministic_rejection"),
                (27, "duplicate_install_force_flag_behavior_is_deterministic_when_unsupported"),
                (28, "uninstall_missing_plugin_returns_stable_failure"),
                (29, "inspect_broken_registry_returns_stable_diagnostics"),
                (30, "plugin_check_after_entrypoint_deletion_reports_stable_failure"),
                (31, "plugin_help_flows_through_root_help_tree"),
                (32, "plugin_command_output_uses_core_envelope_rules"),
                (33, "plugin_command_stderr_stdout_discipline_is_stable"),
                (34, "plugin_command_exit_codes_map_through_core_rules"),
                (35, "two_plugins_keep_stable_ordering_in_list"),
                (36, "uninstalling_one_plugin_does_not_affect_other"),
                (37, "registry_survives_restart_after_successful_install"),
                (38, "registry_survives_restart_after_successful_uninstall"),
                (39, "plugin_check_reports_healthy_and_unhealthy_in_same_registry"),
            ];
            let coverage_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/plugins/plugin_lifecycle_coverage.rs",
                                    })
                                })
                                .collect();
            let complete = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_lifecycle_test_coverage.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin lifecycle integration tests",
                    "rows": coverage_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "coverage_window_end": 40,
                        "artifact_path": "artifacts/status/plugin_lifecycle_test_coverage.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_test_coverage.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-ROLLBACK-TEST-COVERAGE" => {
            let source = fs::read_to_string(workspace_root.join(
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_rollback_resilience.rs",
            ))
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (41, "simulated_disk_write_failure_during_install"),
                (42, "simulated_partial_copy_failure_during_install"),
                (43, "simulated_registry_write_failure_during_install"),
                (44, "simulated_manifest_parse_failure_during_install"),
                (45, "simulated_compatibility_range_failure_during_install"),
                (46, "simulated_missing_entrypoint_failure_during_install"),
                (47, "simulated_permission_denied_failure_during_install"),
                (48, "simulated_partial_uninstall_failure"),
                (49, "simulated_registry_write_failure_during_uninstall"),
                (50, "simulated_enable_failure_when_plugin_files_missing"),
                (51, "simulated_disable_failure_when_registry_is_corrupted"),
                (52, "rollback_proof_install_failure_preserves_existing_plugins"),
                (53, "rollback_proof_uninstall_failure_preserves_existing_plugins"),
                (54, "retry_install_after_partial_failure_is_idempotent"),
                (55, "retry_uninstall_after_partial_failure_is_idempotent"),
                (56, "failed_install_does_not_leave_claimed_namespace"),
                (57, "failed_uninstall_does_not_orphan_registry_state_silently"),
                (58, "plugin_doctor_reports_rollback_relevant_damage_clearly"),
                (59, "machine_readable_rollback_diagnostics_are_stable"),
            ];
            let coverage_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/plugins/plugin_rollback_resilience.rs",
                                    })
                                })
                                .collect();
            let complete = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_rollback_test_coverage.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin rollback resilience tests",
                    "rows": coverage_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "coverage_window_end": 60,
                        "artifact_path": "artifacts/status/plugin_rollback_test_coverage.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_rollback_test_coverage.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/integration/cli/plugins/plugin_namespace_law.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (1, "rejects_plugin_namespace_cli"),
                (2, "rejects_plugin_namespace_dev"),
                (3, "rejects_plugin_namespace_help"),
                (4, "rejects_plugin_namespace_version"),
                (5, "rejects_plugin_namespace_doctor"),
                (6, "rejects_plugin_namespace_plugins"),
                (7, "rejects_plugin_namespace_repl"),
                (8, "rejects_official_product_namespace_dag"),
                (9, "rejects_official_product_namespace_atlas"),
                (10, "rejects_normalized_collision_my_plugin_vs_my_plugin_hyphen"),
                (11, "rejects_case_insensitive_normalized_collision"),
                (12, "rejects_namespace_with_leading_digit"),
                (13, "rejects_namespace_with_whitespace"),
                (14, "rejects_namespace_with_shell_hostile_punctuation"),
                (15, "rejects_empty_namespace"),
                (16, "rejects_namespace_differing_only_by_hidden_alias_collision"),
                (17, "rejection_messages_explain_the_reason_clearly"),
                (18, "json_error_envelopes_for_namespace_rejection_are_stable"),
                (19, "text_errors_for_namespace_rejection_are_stable"),
            ];
            let matrix_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/integration/cli/plugins/plugin_namespace_law.rs",
                                    })
                                })
                                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/reserved_namespace_test_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin namespace law tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/reserved_namespace_test_matrix.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT" => {
            let source =
                fs::read_to_string(workspace_root.join("crates/bijux-cli-python/src/bindings.rs"))
                    .unwrap_or_default();
            let checks: Vec<(&str, Vec<&str>)> = vec![
                (
                    "routing",
                    vec![
                        "parse_intent",
                        "RouteRegistry",
                        "root_command(",
                        "normalize_command_path",
                    ],
                ),
                (
                    "exit_mapping",
                    vec!["map_error_category_to_exit", "USAGE_EXIT_CODE", "INTERNAL_EXIT_CODE"],
                ),
                ("output_shaping", vec!["render_value(", "EmitterConfig", "render_command_help("]),
                (
                    "namespace_validation",
                    vec![
                        "is_reserved_namespace(",
                        "register_plugin_namespace(",
                        "validate_manifest(",
                    ],
                ),
            ];
            let details: Vec<Value> = checks
                .iter()
                .map(|(area, tokens)| {
                    let hits: Vec<&str> =
                        tokens.iter().copied().filter(|token| source.contains(token)).collect();
                    json!({
                        "area": area,
                        "duplicate_rules": hits,
                        "count": hits.len(),
                    })
                })
                .collect();
            let duplicate_rule_count: usize = details
                .iter()
                .filter_map(|item| item.get("count").and_then(Value::as_u64))
                .map(|value| value as usize)
                .sum();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/bridge_duplicate_law_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "source": "crates/bijux-cli-python/src/bindings.rs",
                                    "checks": details,
                                    "summary": {
                                        "duplicate_rule_count": duplicate_rule_count,
                                        "status": if duplicate_rule_count == 0 { "clean" } else { "duplicates-found" },
                                    },
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/bridge_duplicate_law_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-STATE-REPORT" => {
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_state_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "plugin_commands": {
                                        "complete": [
                                            "plugins list",
                                            "plugins inspect",
                                            "plugins check",
                                            "plugins reserved-names",
                                            "plugins where",
                                            "plugins explain",
                                            "plugins schema",
                                        ],
                                        "partial": [
                                            "plugins scaffold",
                                            "plugins install",
                                            "plugins uninstall",
                                            "plugins enable",
                                            "plugins disable",
                                        ],
                                        "python_only": [],
                                    },
                                    "beyond_python": [
                                        "reserved namespace diagnostics surface",
                                        "plugin registry origin metadata",
                                        "transaction rollback assertions for install/uninstall failures",
                                        "explicit plugin schema command",
                                    ],
                                    "overlap_parity_tests": [
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_command_parity.rs",
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_command_parity.rs",
                                    ],
                                    "remaining_gaps": [
                                        "scaffold command parity against Python templates",
                                        "full CLI lifecycle command parity for install/uninstall/enable/disable",
                                        "end-to-end CLI plugin diagnostics parity for all failure classes",
                                    ],
                                    "frozen_law": "plugin v1 contract is frozen before expanding command cleverness",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_state_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS" => {
            let pid = std::process::id();
            let temp_root =
                workspace_root.join(format!("artifacts/status/.runtime-diagnostics-tmp-{pid}"));
            let cargo_bin = temp_root.join(".cargo/bin");
            let pip_bin = temp_root.join("site-packages/bin");
            let wrappers = temp_root.join("wrappers");
            fs::create_dir_all(&cargo_bin).ok()?;
            fs::create_dir_all(&pip_bin).ok()?;
            fs::create_dir_all(&wrappers).ok()?;
            fs::write(cargo_bin.join("bijux"), "placeholder").ok()?;
            fs::write(pip_bin.join("bijux"), "placeholder").ok()?;
            fs::write(wrappers.join("bijux-wrapper"), "/missing/bijux").ok()?;
            let path = std::env::var("PATH").unwrap_or_default();
            let path_mixed = format!("{}:{}:{}", cargo_bin.display(), pip_bin.display(), path);
            let runtime_env = vec![
                ("PATH", path_mixed.clone()),
                ("BIJUX_BIN", temp_root.join("missing-bijux").display().to_string()),
                ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
                ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string()),
            ];
            let package_env =
                vec![("PATH", path_mixed), ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string())];
            let runtime_payload =
                run_bijux_json_env(workspace_root, &["runtime-identity"], &runtime_env).ok()?;
            let package_payload =
                run_bijux_json_env(workspace_root, &["package-health"], &package_env).ok()?;
            let runtime_second =
                run_bijux_json_env(workspace_root, &["runtime-identity"], &runtime_env).ok()?;
            let package_second =
                run_bijux_json_env(workspace_root, &["package-health"], &package_env).ok()?;
            let _ = fs::remove_dir_all(&temp_root);

            let runtime_checks = json!({
                "has_entrypoints": runtime_payload.get("entrypoints").map(Value::is_object).unwrap_or(false),
                "detects_mixed_install": runtime_payload.get("diagnostics").and_then(|d| d.get("mixed_pip_cargo_install_detected")).and_then(Value::as_bool) == Some(true),
                "detects_path_shadowing": runtime_payload.get("diagnostics").and_then(|d| d.get("path_shadowing_detected")).and_then(Value::as_bool) == Some(true),
                "detects_stale_wrapper_or_missing_binary": runtime_payload.get("diagnostics").and_then(|d| d.get("active_binary_missing")).and_then(Value::as_bool) == Some(true),
                "detects_wheel_binary_mismatch": runtime_payload.get("diagnostics").and_then(|d| d.get("mismatched_wheel_binary_versions")).and_then(Value::as_bool) == Some(true),
                "runtime_output_deterministic": runtime_payload == runtime_second,
            });
            let package_checks = json!({
                "has_install_assumptions": package_payload.get("install_state_assumptions").map(Value::is_array).unwrap_or(false),
                "has_runtime_identity_rules": package_payload.get("runtime_identity_rules").map(Value::is_object).unwrap_or(false),
                "package_output_deterministic": package_payload == package_second,
            });
            let ambiguity_checks = json!({
                "runtime_identity_operator_truth": runtime_payload.get("runtime_truth_default").and_then(Value::as_str) == Some("bijux-dev-cli runtime-identity"),
                "package_health_reports_assumptions": package_payload.get("install_state_assumptions").and_then(Value::as_array).map(|v| !v.is_empty()).unwrap_or(false),
                "python_runtime_relevance_present": package_payload.get("runtime_identity_rules").map(Value::is_object).unwrap_or(false),
            });
            let mut drift_checks = Vec::<String>::new();
            for (name, checks) in [
                ("runtime", &runtime_checks),
                ("package", &package_checks),
                ("ambiguity", &ambiguity_checks),
            ] {
                if let Some(obj) = checks.as_object() {
                    for (key, value) in obj {
                        if value.as_bool() != Some(true) {
                            drift_checks.push(format!("{name}.{key}"));
                        }
                    }
                }
            }
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                                &json!({
                                    "scope": "runtime identity diagnostics",
                                    "generator": "bijux-dev-cli",
                                    "checks": runtime_checks,
                                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("runtime.")) { "complete" } else { "partial" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/package_health_diagnostics_artifact.json",
                                &json!({
                                    "scope": "package health diagnostics",
                                    "generator": "bijux-dev-cli",
                                    "checks": package_checks,
                                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("package.")) { "complete" } else { "partial" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                                &json!({
                                    "scope": "install ambiguity diagnostics",
                                    "generator": "bijux-dev-cli",
                                    "checks": ambiguity_checks,
                                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("ambiguity.")) { "complete" } else { "partial" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json",
                &json!({
                    "scope": "runtime/package diagnostics drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                "artifacts/status/package_health_diagnostics_artifact.json",
                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json"
            ]}))
        }
        _ => None,
    }
}
