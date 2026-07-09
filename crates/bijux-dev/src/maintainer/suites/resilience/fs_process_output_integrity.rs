#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DETERMINISTIC-OUTPUT-REPORTS" => {
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let sources: BTreeMap<String, String> = collect_files(&tests_root)
                .into_iter()
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                .map(|path| {
                    (rel(&path, workspace_root), fs::read_to_string(path).unwrap_or_default())
                })
                .collect();
            let rows: Vec<(i64, &str)> = vec![
                (121, "status_json_is_byte_stable_across_runs"),
                (122, "plugins_list_json_is_byte_stable_across_runs"),
                (123, "config_get_json_is_byte_stable_across_runs"),
                (124, "inspect_json_is_byte_stable_across_runs"),
                (125, "help_text_is_stable_across_runs"),
                (126, "json_envelope_field_order_is_stable"),
                (127, "yaml_envelope_field_order_is_stable"),
                (128, "plugin_list_machine_output_order_is_stable"),
                (129, "diagnostic_ordering_is_stable_in_machine_output"),
                (130, "state_doctor_ordering_is_stable_in_machine_output"),
                (131, "repeated_runs_do_not_introduce_timestamp_noise_when_disallowed"),
                (132, "repeated_runs_do_not_introduce_path_order_noise"),
                (133, "repeated_runs_do_not_introduce_plugin_discovery_order_noise"),
                (134, "repeated_runs_do_not_introduce_environment_order_noise"),
                (135, "text_output_stability_holds_under_no_color_mode"),
                (136, "stderr_payloads_are_stable_for_identical_failures"),
                (137, "exit_codes_are_stable_for_identical_failures"),
            ];
            let report_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    let evidence = sources
                        .iter()
                        .find(|(_, src)| src.contains(&format!("fn {name}(")))
                        .map(|(path, _)| path.clone());
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if evidence.is_some() { "complete" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect();
            let complete = report_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/deterministic_output_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "deterministic output tests",
                    "rows": report_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "coverage_window_end": 138,
                        "artifact_path": "artifacts/status/deterministic_output_report.json",
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/determinism_dashboard.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "dashboard": "command-by-command determinism",
                    "commands": [
                        "status --format json --no-pretty",
                        "cli plugins list --format json --no-pretty",
                        "cli config get alpha --format json --no-pretty",
                        "inspect --format json --no-pretty",
                        "help cli plugins",
                        "bijux-dev-cli state-doctor --format json --no-pretty",
                    ],
                    "evidence": [
                        "crates/bijux-cli/tests",
                        "artifacts/status/deterministic_output_report.json",
                    ],
                    "coverage_ids": [139],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/determinism_expectations.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "expectation": "byte stability is required where explicitly claimed",
                    "status": "frozen",
                    "evidence": [
                        "artifacts/status/deterministic_output_report.json",
                        "artifacts/status/determinism_dashboard.json",
                    ],
                    "coverage_ids": [140],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/deterministic_output_report.json",
                "artifacts/status/determinism_dashboard.json",
                "artifacts/status/determinism_expectations.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-OUTPUT-BRIDGE-FUZZ-REPORTS" => {
            let output_targets = workspace_root
                .join("crates/bijux-cli-output/tests/output_envelope_fuzz_targets.rs");
            let output_regression = workspace_root
                .join("crates/bijux-cli-output/tests/output_envelope_fuzz_regressions.rs");
            let bridge_targets = workspace_root
                .join("crates/bijux-cli-python/tests/bridge_conversion_stability.rs");
            let bridge_regression = workspace_root
                .join("crates/bijux-cli-python/tests/bridge_conversion_fuzz_regressions.rs");
            let output_min_dir =
                workspace_root.join("crates/bijux-cli-output/tests/fuzz/output_minimized_cases");
            let bridge_min_dir = workspace_root
                .join("crates/bijux-cli-python/tests/fuzz/bridge_conversion_minimized_cases");
            let texts = BTreeMap::from([
                (output_targets.clone(), fs::read_to_string(&output_targets).unwrap_or_default()),
                (
                    output_regression.clone(),
                    fs::read_to_string(&output_regression).unwrap_or_default(),
                ),
                (bridge_targets.clone(), fs::read_to_string(&bridge_targets).unwrap_or_default()),
                (
                    bridge_regression.clone(),
                    fs::read_to_string(&bridge_regression).unwrap_or_default(),
                ),
            ]);
            let required: BTreeMap<i64, (PathBuf, &str)> = BTreeMap::from([
                                (81, (output_targets.clone(), "fuzz_success_envelope_serialization_is_stable")),
                                (82, (output_targets.clone(), "fuzz_error_envelope_serialization_is_stable")),
                                (83, (output_targets.clone(), "fuzz_json_yaml_text_emitters_render_without_corruption")),
                                (84, (output_targets.clone(), "fuzz_json_yaml_text_emitters_render_without_corruption")),
                                (85, (output_targets.clone(), "fuzz_json_yaml_text_emitters_render_without_corruption")),
                                (86, (output_targets.clone(), "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering")),
                                (87, (output_targets.clone(), "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering")),
                                (88, (output_targets.clone(), "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering")),
                                (89, (output_targets.clone(), "fuzz_nested_diagnostics_multiline_unicode_empty_and_large_payload_rendering")),
                                (90, (output_targets.clone(), "fuzz_malformed_envelope_deserialization_is_rejected")),
                                (91, (bridge_targets.clone(), "fuzz_bridge_conversion_of_success_envelopes_is_stable")),
                                (92, (bridge_targets.clone(), "fuzz_bridge_conversion_of_error_envelopes_is_stable")),
                                (93, (output_targets.clone(), "fuzz_route_inspection_json_rendering_is_deterministic")),
                                (96, (output_regression.clone(), "minimized_output_cases_replay_with_stable_parse_behavior")),
                                (97, (bridge_regression.clone(), "minimized_bridge_conversion_cases_replay_deterministically")),
                                (98, (output_regression.clone(), "minimized_output_cases_replay_with_stable_parse_behavior")),
                                (99, (output_targets.clone(), "fuzz_output_field_order_invariant_for_machine_rendering")),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, (path, test_name))| {
                                    let text = texts.get(path).cloned().unwrap_or_default();
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": test_name,
                                        "status": if text.contains(&format!("fn {test_name}(")) { "covered" } else { "missing" },
                                        "evidence": rel(path, workspace_root),
                                    })
                                })
                                .collect();
            let output_cases: Vec<String> = collect_files(&output_min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let bridge_cases: Vec<String> = collect_files(&bridge_min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let run = |args: &[&str]| -> bool {
                Command::new("cargo")
                    .args(args)
                    .current_dir(workspace_root)
                    .status()
                    .ok()
                    .is_some_and(|s| s.success())
            };
            let output_targets_ok =
                run(&["test", "-p", "bijux-cli", "--test", "output_envelope_fuzz_targets"]);
            let output_reg_ok =
                run(&["test", "-p", "bijux-cli", "--test", "output_envelope_fuzz_regressions"]);
            let bridge_targets_ok = run(&[
                "test",
                "-p",
                "bijux-cli-python",
                "--test",
                "bridge_conversion_stability",
            ]);
            let bridge_reg_ok = run(&[
                "test",
                "-p",
                "bijux-cli-python",
                "--test",
                "bridge_conversion_fuzz_regressions",
            ]);
            let missing_ids: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/output_crash_triage_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "output crash triage",
                                    "coverage_ids": [94],
                                    "status": if output_targets_ok && output_reg_ok { "clean" } else { "needs-triage" },
                                    "target_suite_ok": output_targets_ok,
                                    "regression_suite_ok": output_reg_ok,
                                    "minimized_case_count": output_cases.len(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/bridge_conversion_crash_triage_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "bridge conversion crash triage",
                                    "coverage_ids": [95],
                                    "status": if bridge_targets_ok && bridge_reg_ok { "clean" } else { "needs-triage" },
                                    "target_suite_ok": bridge_targets_ok,
                                    "regression_suite_ok": bridge_reg_ok,
                                    "minimized_case_count": bridge_cases.len(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/output_fuzz_regression_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "output fuzz regressions",
                    "coverage_ids": [96, 98],
                    "status": if output_reg_ok { "clean" } else { "drift" },
                    "minimized_cases": output_cases,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_conversion_fuzz_regression_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "bridge conversion fuzz regressions",
                    "coverage_ids": [97],
                    "status": if bridge_reg_ok { "clean" } else { "drift" },
                    "minimized_cases": bridge_cases,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/output_envelope_fuzz_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "output and envelope fuzz hardening",
                                    "coverage_ids": (81..101).collect::<Vec<_>>(),
                                    "status": if missing_ids.is_empty() && output_targets_ok && output_reg_ok && bridge_targets_ok && bridge_reg_ok && !output_cases.is_empty() && !bridge_cases.is_empty() { "frozen" } else { "partial" },
                                    "coverage_rows": coverage_rows,
                                    "missing_coverage_ids": missing_ids,
                                    "output_minimized_case_count": output_cases.len(),
                                    "bridge_minimized_case_count": bridge_cases.len(),
                                    "policy": "envelope/output fuzzing is contract hardening and remains permanently gated",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/output_crash_triage_artifact.json",
                "artifacts/status/bridge_conversion_crash_triage_artifact.json",
                "artifacts/status/output_fuzz_regression_artifact.json",
                "artifacts/status/bridge_conversion_fuzz_regression_artifact.json",
                "artifacts/status/output_envelope_fuzz_contract.json"
            ]}))
        }
        _ => None,
    }
}
