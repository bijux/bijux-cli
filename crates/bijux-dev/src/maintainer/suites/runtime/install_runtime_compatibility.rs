#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

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
                "generated_at": generated_at_utc(),
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
                .join("crates/bijux-cli/tests/integration/cli/resilience/hostile_state_determinism.rs");
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
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "scope": "deterministic hostile-state behavior",
                                "rows": rows.iter().map(|(id,name)| json!({
                                    "coverage_id": id,
                                    "test_name": name,
                                    "status": if text.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                    "evidence": "crates/bijux-cli/tests/integration/cli/resilience/hostile_state_determinism.rs"
                                })).collect::<Vec<_>>(),
                            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/failure_class_stability_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "harness_file": "artifacts/status/repeated_run_corruption_harness.json",
                    "coverage_ids": [157]
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/deterministic_failure_quality_bar.json", &json!({
                                "generated_at": generated_at_utc(),
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
            let test_file = workspace_root
                .join("crates/bijux-cli/tests/integration/cli/root/precedence_matrix.rs");
            let text = fs::read_to_string(&test_file).unwrap_or_default();
            let env_payload =
                run_bijux_json(workspace_root, &["env"]).unwrap_or_else(|_| json!({}));
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
                                    "evidence":"crates/bijux-cli/tests/integration/cli/root/precedence_matrix.rs"
                                })
                            })
                            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_regression_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
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
                    "generated_at": generated_at_utc(),
                    "source_precedence": source_precedence,
                    "shared_contract": "flags > env > config > defaults"
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
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
        _ => None,
    }
}
