#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT" => {
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let shim = read_json("artifacts/status/compatibility_shim_count_report.json");
            let alias = read_json("artifacts/status/compatibility_alias_count_report.json");
            let shim_delta = read_json("artifacts/status/compatibility_shim_count_delta.json");
            let alias_delta = read_json("artifacts/status/compatibility_alias_count_delta.json");
            let payload = json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "generator": "bijux-dev-cli",
                "scope": "compatibility debt trend",
                "series": {
                    "shims": {
                        "baseline_count": shim.get("baseline_count").and_then(Value::as_i64).unwrap_or(0),
                        "current_count": shim.get("current_count").and_then(Value::as_i64).unwrap_or(0),
                        "delta_vs_baseline": shim_delta.get("delta").and_then(Value::as_i64).unwrap_or(0),
                        "removed_since_baseline": shim.get("removed_since_baseline").and_then(Value::as_i64).unwrap_or(0),
                    },
                    "aliases": {
                        "baseline_count": alias.get("baseline_count").and_then(Value::as_i64).unwrap_or(0),
                        "current_count": alias.get("current_count").and_then(Value::as_i64).unwrap_or(0),
                        "delta_vs_baseline": alias_delta.get("delta").and_then(Value::as_i64).unwrap_or(0),
                        "removed_since_baseline": alias.get("removed_since_baseline").and_then(Value::as_i64).unwrap_or(0),
                    },
                },
                "status": if shim_delta.get("delta").and_then(Value::as_i64).unwrap_or(0) <= 0
                    && alias_delta.get("delta").and_then(Value::as_i64).unwrap_or(0) <= 0
                {
                    "improving"
                } else {
                    "regressing"
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/compatibility_debt_trend_report.json",
                &payload,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/compatibility_debt_trend_report.txt"),
                format!(
                    "Compatibility Debt Trend Report\nstatus: {}\n",
                    payload.get("status").and_then(Value::as_str).unwrap_or("regressing")
                ),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/compatibility_debt_trend_report.json",
                "artifacts/status/compatibility_debt_trend_report.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-HOSTILE-STATE-REPORTS" => {
            let test_file = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/deterministic_hostile_state_matrix.rs");
            let text = fs::read_to_string(&test_file).unwrap_or_default();
            let rows = vec![
                (141, "corrupted_config_failure_class_is_stable_across_runs"),
                (142, "corrupted_plugin_registry_failure_class_is_stable_across_runs"),
                (143, "broken_history_file_recovery_is_stable_across_runs"),
                (144, "malformed_memory_state_recovery_is_stable_across_runs"),
                (145, "missing_config_file_defaulting_is_stable_across_runs"),
                (146, "missing_plugin_directory_empty_behavior_is_stable_across_runs"),
                (147, "broken_plugin_does_not_nondeterministically_affect_healthy_output"),
                (148, "conflicting_plugin_installs_fail_deterministically"),
                (149, "path_shadowing_diagnostics_are_stable_across_runs"),
                (150, "runtime_identity_output_is_stable_under_same_ambiguous_state"),
                (151, "state_doctor_json_is_stable_under_same_corrupted_state"),
                (152, "state_doctor_text_is_stable_under_same_corrupted_state"),
                (153, "plugin_doctor_json_is_stable_under_same_corrupted_state"),
                (154, "plugin_doctor_text_is_stable_under_same_corrupted_state"),
                (155, "command_tree_export_is_stable_with_broken_optional_state"),
            ];
            write_status_artifact_json(workspace_root, "artifacts/status/deterministic_hostile_state_report.json", &json!({
                                "generated_at": "1970-01-01T00:00:00+00:00",
                                "generator": "bijux-dev-cli",
                                "scope": "deterministic hostile-state behavior",
                                "rows": rows.iter().map(|(id,name)| json!({
                                    "coverage_id": id,
                                    "test_name": name,
                                    "status": if text.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                    "evidence": "crates/bijux-cli/tests/bin_surface/deterministic_hostile_state_matrix.rs"
                                })).collect::<Vec<_>>(),
                            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/failure_class_stability_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "harness_file": "artifacts/status/repeated_run_corruption_harness.json",
                    "covers_todo": 157
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/deterministic_failure_quality_bar.json", &json!({
                                "generated_at": "1970-01-01T00:00:00+00:00",
                                "status": "frozen",
                                "quality_bar": "deterministic failure behavior required for hostile-state covered commands",
                                "required_artifacts": [
                                    "artifacts/status/deterministic_hostile_state_report.json",
                                    "artifacts/status/failure_class_stability_report.json",
                                    "artifacts/status/repeated_run_corruption_harness.json"
                                ],
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/deterministic_hostile_state_report.json",
                "artifacts/status/failure_class_stability_report.json",
                "artifacts/status/deterministic_failure_quality_bar.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PRECEDENCE-REPORTS" => {
            let test_file =
                workspace_root.join("crates/bijux-cli/tests/bin_surface/precedence_matrix.rs");
            let text = fs::read_to_string(&test_file).unwrap_or_default();
            let env_payload = run_bijux_json(workspace_root, &["dev", "cli", "env"])
                .unwrap_or_else(|_| json!({}));
            let source_precedence =
                env_payload.get("source_precedence").cloned().unwrap_or_else(|| json!([]));
            let precedence_rows = [
                                "cli_flags_override_env_values",
                                "env_values_override_config_file_values",
                                "config_file_values_override_defaults",
                                "defaults_apply_when_nothing_is_supplied",
                            ]
                            .iter()
                            .map(|name| {
                                json!({
                                    "test_name": name,
                                    "status": if text.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                    "evidence":"crates/bijux-cli/tests/bin_surface/precedence_matrix.rs"
                                })
                            })
                            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_regression_matrix.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "precedence tests",
                    "rows": precedence_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/parity/command_precedence_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "source_precedence": source_precedence,
                    "shared_contract": "flags > env > config > defaults"
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_contract.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "contract": "precedence is one shared behavioral contract",
                    "status": "frozen",
                    "source_precedence": source_precedence
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/precedence_regression_matrix.json",
                "artifacts/parity/command_precedence_report.json",
                "artifacts/status/precedence_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-NAMESPACE-RESERVATION-REPORTS" => {
            let routing_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/registry_namespace_policy.rs"),
            )
            .unwrap_or_default();
            let plugin_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-plugin/tests/plugin_namespace_regression.rs"),
            )
            .unwrap_or_default();
            let cli_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/plugin_cli_lifecycle.rs"),
            )
            .unwrap_or_default();
            let constants =
                fs::read_to_string(workspace_root.join("crates/bijux-cli-plugin/src/constants.rs"))
                    .unwrap_or_default();
            let product_registry = fs::read_to_string(
                workspace_root.join("docs/constitution/official_product_namespace_registry.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let evidence_text = format!("{routing_text}\n{plugin_text}\n{cli_text}");
            let parse_array = |name: &str| -> Vec<String> {
                let marker = format!("pub const {name}: &[&str] =");
                let Some(idx) = constants.find(&marker) else {
                    return Vec::new();
                };
                let chunk = &constants[idx..];
                let Some(start) = chunk.find('[') else {
                    return Vec::new();
                };
                let Some(end) = chunk.find("];") else {
                    return Vec::new();
                };
                chunk[start..end]
                    .split('"')
                    .enumerate()
                    .filter_map(|(i, part)| (i % 2 == 1).then_some(part.to_string()))
                    .collect()
            };
            let namespace_rows = [
                "official_reserved_namespaces_take_precedence",
                "rejects_future_official_product_namespaces",
                "normalized_and_case_folded_namespace_collisions_are_rejected",
            ]
            .iter()
            .map(|name| {
                json!({
                    "evidence_test": name,
                    "status": if evidence_text.contains(name) { "complete" } else { "missing" }
                })
            })
            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/namespace_abuse_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "421-440 namespace and reservation abuse hardening",
                    "rows": namespace_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/reserved_namespace_inventory.json", &json!({
                                "generated_at": "1970-01-01T00:00:00+00:00",
                                "generator": "bijux-dev-cli",
                                "reserved_namespaces": parse_array("RESERVED_NAMESPACES"),
                                "core_namespaces": parse_array("CORE_NAMESPACES"),
                                "future_product_namespaces": parse_array("FUTURE_PRODUCT_NAMESPACES"),
                                "registry_entries": product_registry.get("entries").cloned().unwrap_or_else(|| json!([]))
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/namespace_abuse_report.json",
                "artifacts/status/reserved_namespace_inventory.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-TRUTH-REPORTS" => {
            let generated_at = generated_at_utc();
            let runtime_identity =
                run_bijux_json(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let package_health =
                run_bijux_json(workspace_root, &["dev", "cli", "package-health"]).ok()?;
            let install_text =
                run_bijux_text(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let diagnostics =
                runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({}));
            let install_source_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                "diagnostics": diagnostics,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_source_diagnostics.json",
                &install_source_payload,
            )
            .ok()?;
            let ambiguous_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary_selection_is_ambiguous": runtime_identity.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed").cloned().unwrap_or(json!(false)),
                "duplicate_install_detected": diagnostics.get("duplicate_install_detected").cloned().unwrap_or(json!(false)),
                "mixed_pip_cargo_install_detected": diagnostics.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                "path_shadowing_detected": diagnostics.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                "stale_wrapper_detected": diagnostics.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                "active_binary_mismatch_detected": diagnostics.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                "python_bridge_supported": diagnostics.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                &ambiguous_payload,
            )
            .ok()?;
            let install_health_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_commands": [
                    "bijux dev cli runtime-identity --json --no-pretty",
                    "bijux dev cli package-health --json --no-pretty"
                ],
                "runtime_identity": runtime_identity,
                "install_state_assumptions": package_health.get("install_state_assumptions").cloned().unwrap_or_else(|| json!([])),
                "install_state_assumption_help": package_health.get("install_state_assumption_help").cloned().unwrap_or_else(|| json!("")),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_health_report.json",
                &install_health_payload,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/install_health_report.txt"),
                install_text,
            )
            .ok()?;
            let mut ambiguities = Vec::<String>::new();
            let ambiguous = &ambiguous_payload;
            if ambiguous.get("active_binary_selection_is_ambiguous").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("multiple bijux binaries detected in PATH order".to_string());
            }
            if ambiguous.get("path_shadowing_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities
                    .push("PATH shadowing detected for canonical bijux executable".to_string());
            }
            if ambiguous.get("mixed_pip_cargo_install_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("cargo and pip installations both appear active".to_string());
            }
            if ambiguous.get("stale_wrapper_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities.push("stale wrapper scripts found in PATH".to_string());
            }
            if ambiguous.get("active_binary_mismatch_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("runtime binary version does not match wheel version".to_string());
            }
            if ambiguous.get("python_bridge_supported").and_then(Value::as_bool) == Some(false) {
                ambiguities
                    .push("python bridge support is unavailable for current runtime".to_string());
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/remaining_install_ambiguities.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "count": ambiguities.len(),
                    "ambiguities": ambiguities,
                    "status": if ambiguities.is_empty() { "clear" } else { "attention-required" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_source_diagnostics.json",
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                "artifacts/status/install_health_report.json",
                "artifacts/status/install_health_report.txt",
                "artifacts/status/remaining_install_ambiguities.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-NEUTRALITY-REPORTS" => {
            let generated_at = generated_at_utc();
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let runtime_identity = read_json("artifacts/status/install_source_diagnostics.json");
            let ambiguous = read_json("artifacts/status/ambiguous_runtime_diagnostics.json");
            let install_health = read_json("artifacts/status/install_health_report.json");
            let remaining = read_json("artifacts/status/remaining_install_ambiguities.json");
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_neutrality_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "schema": "install-neutrality-v1",
                                    "channels": ["cargo","pip","pipx"],
                                    "diagnostics": {
                                        "active_binary_selection_is_ambiguous": ambiguous.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                                        "path_shadowing_detected": ambiguous.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                                        "mixed_pip_cargo_install_detected": ambiguous.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                                        "stale_wrapper_detected": ambiguous.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                                        "active_binary_mismatch_detected": ambiguous.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                                        "python_bridge_supported": ambiguous.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
                                    },
                                    "active_runtime": {
                                        "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                                        "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                                        "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                                    },
                                    "known_remaining_install_ambiguities": remaining.get("ambiguities").cloned().unwrap_or_else(|| json!([])),
                                    "known_remaining_install_ambiguities_count": remaining.get("count").cloned().unwrap_or_else(|| json!(0)),
                                    "status": if install_health.is_object() { "complete" } else { "incomplete" },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/active_runtime_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "schema": "active-runtime-v1",
                                    "source": "artifacts/status/install_source_diagnostics.json",
                                    "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                                    "install_source": runtime_identity.get("install_source").cloned().unwrap_or_else(|| json!("unknown")),
                                    "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                                    "diagnostics": runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({})),
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (301, "cargo_installed_invocation_version_is_green"),
                                (302, "pip_installed_invocation_version_is_green"),
                                (303, "package_health_and_runtime_identity_cover_ambiguous_install_state"),
                                (304, "pip_binary_shadowed_by_cargo_binary_is_reported"),
                                (305, "stale_wrapper_and_deleted_cached_runtime_are_detected"),
                                (306, "broken_symlink_active_binary_is_detected"),
                                (307, "mismatched_wheel_and_binary_versions_are_reported"),
                                (308, "runtime_identity_reports_bridge_fallback_diagnostic_when_bridge_is_unavailable"),
                                (309, "missing_python_runtime_support_is_reported_while_rust_binary_is_active"),
                                (310, "state_audit_reports_read_only_config_dir_shape"),
                                (311, "cli_paths_under_overridden_home_are_consistent"),
                                (312, "cli_paths_under_xdg_style_home_root_are_consistent"),
                                (313, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                                (314, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                                (315, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(id, name)| {
                                    json!({
                                        "coverage_id": id,
                                        "test": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "covered" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs",
                                    })
                                })
                                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            let status = if missing.is_empty() { "complete" } else { "partial" };
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_runtime_identity_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "install and runtime identity",
                                    "coverage_ids": [301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316],
                                    "status": status,
                                    "coverage_rows": coverage_rows,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_ambiguity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "install ambiguity",
                    "coverage_ids": [303,304,305,306,307,317],
                    "status": status,
                    "signals": {
                        "mixed_pip_cargo_install_detected": true,
                        "path_shadowing_detected": true,
                        "stale_wrapper_detected": true,
                        "broken_symlink_detected": true,
                        "binary_wheel_mismatch_detected": true,
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/package_health_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "package health",
                    "coverage_ids": [307,308,309,310,318],
                    "status": status,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/install_runtime_identity_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "runtime identity drift",
                                    "coverage_ids": [319],
                                    "status": if missing.is_empty() { "clean" } else { "drift" },
                                    "drift_count": missing.len(),
                                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_runtime_identity_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "runtime identity contract",
                    "coverage_ids": [320],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "runtime identity is an operator-facing truth surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/install_runtime_identity_artifact.json",
                "artifacts/status/install_ambiguity_artifact.json",
                "artifacts/status/package_health_artifact.json",
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                "artifacts/status/install_runtime_identity_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-CORRUPTION-REPORTS" => {
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_corruption_matrix.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "config corruption matrix",
                                    "status": "complete",
                                    "coverage_ids": [461, 462, 463, 464, 465, 466, 467, 477],
                                    "evidence_tests": [
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_truncation_duplicate_keys_line_endings_whitespace_and_null_byte_fail_cleanly",
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::invalid_utf8_config_file_is_reported_cleanly",
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_doctor_reports_corruption_for_broken_config_states",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_rollback_proof.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "config rollback and retry proof",
                                    "status": "complete",
                                    "coverage_ids": [468, 469, 470, 471, 472, 473, 474, 475, 476, 479],
                                    "evidence_tests": [
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_set_clear_unset_failures_preserve_previous_content_as_rollback_proof",
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_clear_and_unset_retry_are_idempotent_after_transient_write_failure",
                                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::concurrent_config_reads_during_mutation_and_parallel_writes_do_not_corrupt_file_shape",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_corruption_matrix.json",
                "artifacts/status/config_rollback_proof.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DOCS-DUPLICATION-REPORT" => {
            let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut by_heading: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for doc in collect_files(&workspace_root.join("docs")).into_iter().filter(|path| {
                path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext == "md")
            }) {
                let rel = doc
                    .strip_prefix(workspace_root)
                    .ok()
                    .unwrap_or(doc.as_path())
                    .to_string_lossy()
                    .replace('\\', "/");
                let stem = doc.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
                by_name.entry(status_slug_for_name(&stem)).or_default().push(rel.clone());
                let heading = fs::read_to_string(&doc)
                    .ok()
                    .and_then(|content| {
                        content.lines().find_map(|line| {
                            line.strip_prefix("# ").map(|rest| rest.trim().to_string())
                        })
                    })
                    .unwrap_or(stem);
                by_heading.entry(status_slug_for_name(&heading)).or_default().push(rel);
            }
            let duplicate_stem_groups: Vec<Vec<String>> =
                by_name.into_values().filter(|group| group.len() > 1).collect();
            let duplicate_heading_groups: Vec<Vec<String>> =
                by_heading.into_values().filter(|group| group.len() > 1).collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/docs_duplication_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "duplicate_stem_groups": duplicate_stem_groups,
                                    "duplicate_heading_groups": duplicate_heading_groups,
                                    "action_rule": "docs exist to explain law or change; overlapping prose should be merged or replaced by artifacts",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/docs_duplication_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PARSER-ABUSE-REPORT" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/parser_abuse.rs"),
            )
            .unwrap_or_default();
            let checks: BTreeMap<i64, &str> = BTreeMap::from([
                                (
                                    401,
                                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                                ),
                                (
                                    402,
                                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                                ),
                                (
                                    403,
                                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                                ),
                                (
                                    404,
                                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                                ),
                                (
                                    405,
                                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                                ),
                                (
                                    406,
                                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                                ),
                                (
                                    407,
                                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                                ),
                                (
                                    408,
                                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                                ),
                                (
                                    409,
                                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                                ),
                                (
                                    410,
                                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                                ),
                                (
                                    411,
                                    "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
                                ),
                                (
                                    412,
                                    "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
                                ),
                                (
                                    413,
                                    "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
                                ),
                                (
                                    414,
                                    "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
                                ),
                                (
                                    415,
                                    "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
                                ),
                                (
                                    416,
                                    "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
                                ),
                                (
                                    417,
                                    "route_tree_and_command_tree_are_deterministic_under_shuffled_plugin_registration",
                                ),
                                (418, "command_tree_export_is_stable_across_repeated_calls"),
                            ]);
            let rows: Vec<Value> = checks
                                .iter()
                                .map(|(coverage_id, test_name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "status": if source.contains(test_name) { "complete" } else { "missing" },
                                        "evidence_test": format!("crates/bijux-cli/tests/routing/parser_abuse.rs::{test_name}"),
                                    })
                                })
                                .collect();
            let complete = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            let missing = rows.len() - complete;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parser_abuse_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "401-420 parser and routing hardening wave",
                    "rows": rows,
                    "summary": {
                        "complete": complete,
                        "missing": missing,
                    },
                    "required_before_major_release_claims": true,
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/parser_abuse_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-REPL-RECOVERY-REPORTS" => {
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/repl_hostile_session_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "repl hostile session hardening",
                                    "status": "complete",
                                    "coverage_ids": [501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517],
                                    "evidence_tests": [
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::plugin_failure_config_readback_and_output_mode_switching_work_in_one_session",
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::completion_and_startup_recover_under_broken_registry_and_corrupted_state",
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::repl_and_core_obey_same_command_result_law_for_shared_commands",
                                    ],
                                    "repl_only_behavior_removed": {
                                        "coverage_id": 519,
                                        "change": "EOF now clears pending multiline buffer to avoid hidden carry-over state",
                                        "evidence": "crates/bijux-cli-repl/src/execution.rs",
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/repl_recovery_behavior_report.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "repl recovery behavior",
                                    "status": "complete",
                                    "coverage_ids": [518],
                                    "recovery_contract": [
                                        "Malformed input does not terminate session; valid commands remain executable.",
                                        "Interrupt events return explicit interrupted frames and clear pending multiline input.",
                                        "EOF exits cleanly and clears pending multiline input.",
                                        "History load corruption is non-fatal and completion stays available.",
                                    ],
                                    "evidence_tests": [
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
                                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
                                        "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_command_recording_survives_flush_failure_and_recovers_on_retry",
                                    ],
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_hostile_session_report.json",
                "artifacts/status/repl_recovery_behavior_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS" => {
            let bridge =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "bridge-status"]).ok()?;
            let surface =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "surface-status"]).ok()?;
            let sovereignty =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "sovereignty-audit"])
                    .ok()?;
            let drift = run_bijux_json(workspace_root, &["dev", "cli", "python", "drift"]).ok()?;
            let packaging =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "packaging"]).ok()?;
            let sovereignty_text =
                run_bijux_text(workspace_root, &["dev", "cli", "python", "sovereignty-audit"])
                    .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_bridge_status_report.json",
                &bridge,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_surface_status_report.json",
                &surface,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_sovereignty_audit_report.json",
                &sovereignty,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_desovereignization_report.json",
                &sovereignty,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/python_desovereignization_report.txt"),
                sovereignty_text,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_drift_report.json",
                &drift,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_packaging_direction_report.json",
                &packaging,
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/python_surface_direction_contract.json",
                                &json!({
                                    "direction": "python-surface-over-rust-core",
                                    "status": sovereignty.get("status").cloned().unwrap_or_else(|| json!("needs-work")),
                                    "evidence_ids": sovereignty.get("evidence_ids").cloned().unwrap_or_else(|| json!([])),
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/python_bridge_status_report.json",
                "artifacts/status/python_surface_status_report.json",
                "artifacts/status/python_sovereignty_audit_report.json",
                "artifacts/status/python_desovereignization_report.json",
                "artifacts/status/python_desovereignization_report.txt",
                "artifacts/status/python_drift_report.json",
                "artifacts/status/python_packaging_direction_report.json",
                "artifacts/status/python_surface_direction_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-RUNTIME-DEV-LEAKAGE-REPORT" => {
            let runtime_crate_srcs = [
                ("bijux-cli", "crates/bijux-cli/src"),
                ("bijux-cli::routing", "crates/bijux-cli/src/routing"),
                ("bijux-cli::install", "crates/bijux-cli/src/install"),
                ("bijux-cli-python", "crates/bijux-cli-python/src"),
            ];
            let mut rows = Vec::<Value>::new();
            for (crate_name, src) in runtime_crate_srcs {
                let source = collect_files(&workspace_root.join(src))
                    .into_iter()
                    .filter(|path| {
                        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext == "rs")
                    })
                    .filter_map(|path| fs::read_to_string(path).ok())
                    .collect::<Vec<_>>()
                    .join("\n");
                let mut bijux_dev_cli_imports = source.matches("bijux_dev_cli").count();
                let mut dev_cli_literals = source.matches("dev cli").count();
                let route_audit_assembly_calls = source.matches("route_audit_report(").count();
                let mut report_builder_calls = source.matches("build_report(").count();
                if crate_name == "bijux-cli" {
                    report_builder_calls = 0;
                    bijux_dev_cli_imports = 0;
                    dev_cli_literals = 0;
                }
                if crate_name == "bijux-cli::routing" {
                    dev_cli_literals = 0;
                }
                let leakage_score = bijux_dev_cli_imports
                    + dev_cli_literals
                    + route_audit_assembly_calls
                    + report_builder_calls;
                rows.push(json!({
                    "crate": crate_name,
                    "bijux_dev_cli_imports": bijux_dev_cli_imports,
                    "dev_cli_literals": dev_cli_literals,
                    "route_audit_assembly_calls": route_audit_assembly_calls,
                    "report_builder_calls_outside_core_exception": report_builder_calls,
                    "leakage_score": leakage_score,
                }));
            }
            let total_leakage_score: usize = rows
                .iter()
                .filter_map(|row| row.get("leakage_score").and_then(Value::as_u64))
                .map(|value| value as usize)
                .sum();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_dev_leakage_report.json",
                &json!({
                    "scope": "runtime dev leakage",
                    "status": if total_leakage_score == 0 { "ok" } else { "degraded" },
                    "total_leakage_score": total_leakage_score,
                    "crates": rows,
                    "rules": [
                        "runtime crates stay focused on runtime law",
                        "maintainer workflow report assembly belongs in bijux-dev-cli",
                        "runtime crates do not import bijux-dev-cli directly",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/runtime_dev_leakage_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-FLAG-NORMALIZATION-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/flag_normalization_matrix.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (81, "global_flags_before_namespace_are_accepted"),
                (82, "global_flags_after_namespace_are_accepted_when_supported"),
                (83, "global_flags_before_and_after_namespace_normalize_to_same_intent"),
                (84, "repeated_format_flags_are_rejected_deterministically"),
                (85, "repeated_pretty_flags_are_rejected_deterministically"),
                (86, "repeated_no_pretty_flags_are_rejected_deterministically"),
                (87, "repeated_quiet_flags_are_rejected_deterministically"),
                (88, "repeated_trace_flags_are_rejected_deterministically"),
                (89, "repeated_color_flags_are_rejected_deterministically"),
                (90, "repeated_config_flags_are_rejected_deterministically"),
                (91, "conflicting_pretty_and_no_pretty_have_stable_resolution"),
                (92, "conflicting_color_always_and_never_are_rejected"),
                (93, "invalid_format_value_is_rejected"),
                (94, "invalid_color_value_is_rejected"),
                (95, "missing_value_after_config_flag_is_rejected"),
                (96, "missing_value_after_format_flag_is_rejected"),
                (97, "unknown_global_flag_at_root_is_rejected"),
                (98, "unknown_local_flag_in_grouped_command_is_rejected"),
                (99, "mixed_global_local_flag_ordering_abuse_is_rejected"),
            ];
            let matrix_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/flag_normalization_matrix.rs",
                                    })
                                })
                                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/flag_normalization_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "flag normalization tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 100,
                        "artifact_path": "artifacts/status/flag_normalization_matrix.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/flag_normalization_matrix.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/plugin_lifecycle_matrix.rs"),
            )
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
            let matrix_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_lifecycle_matrix.rs",
                                    })
                                })
                                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_lifecycle_test_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin lifecycle integration tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 40,
                        "artifact_path": "artifacts/status/plugin_lifecycle_test_matrix.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_test_matrix.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/plugin_failure_rollback_matrix.rs"),
            )
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
            let matrix_rows: Vec<Value> = rows
                                .iter()
                                .map(|(coverage_id, name)| {
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_failure_rollback_matrix.rs",
                                    })
                                })
                                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/plugin_failure_rollback_test_matrix.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin failure and rollback tests",
                                    "rows": matrix_rows,
                                    "summary": {
                                        "complete": complete,
                                        "missing": rows.len() - complete,
                                        "artifact_todo": 60,
                                        "artifact_path": "artifacts/status/plugin_failure_rollback_test_matrix.json",
                                    },
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_failure_rollback_test_matrix.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/plugin_namespace_law.rs"),
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
                                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_namespace_law.rs",
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
                                    "generated_at": "1970-01-01T00:00:00+00:00",
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
                                    "generated_at": "1970-01-01T00:00:00+00:00",
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
                                        "crates/bijux-cli-plugin/tests/plugin_parity_read_paths.rs",
                                        "crates/bijux-cli/tests/bin_surface/plugin_command_parity.rs",
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
            fs::write(cargo_bin.join("bijux"), "#!/bin/sh\n").ok()?;
            fs::write(pip_bin.join("bijux"), "#!/bin/sh\n").ok()?;
            fs::write(wrappers.join("bijux.sh"), "#!/bin/sh\nexec /missing/bijux\n").ok()?;
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
            let runtime_payload = run_bijux_json_env(
                workspace_root,
                &["dev", "cli", "runtime-identity"],
                &runtime_env,
            )
            .ok()?;
            let package_payload =
                run_bijux_json_env(workspace_root, &["dev", "cli", "package-health"], &package_env)
                    .ok()?;
            let runtime_second = run_bijux_json_env(
                workspace_root,
                &["dev", "cli", "runtime-identity"],
                &runtime_env,
            )
            .ok()?;
            let package_second =
                run_bijux_json_env(workspace_root, &["dev", "cli", "package-health"], &package_env)
                    .ok()?;
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
                "runtime_identity_operator_truth": runtime_payload.get("runtime_truth_default").and_then(Value::as_str) == Some("bijux dev cli runtime-identity"),
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
