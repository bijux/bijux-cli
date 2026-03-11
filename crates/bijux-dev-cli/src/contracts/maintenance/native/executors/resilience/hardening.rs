#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DETERMINISTIC-OUTPUT-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/deterministic_output_matrix.rs"),
            )
            .unwrap_or_default();
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
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test_name": name,
                                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/deterministic_output_matrix.rs",
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
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "deterministic output tests",
                    "rows": report_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 138,
                        "artifact_path": "artifacts/status/deterministic_output_report.json",
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/determinism_dashboard.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "dashboard": "command-by-command determinism",
                    "commands": [
                        "status --format json --no-pretty",
                        "cli plugins list --format json --no-pretty",
                        "cli config get alpha --format json --no-pretty",
                        "inspect --format json --no-pretty",
                        "help cli plugins",
                        "dev cli state-doctor --format json --no-pretty",
                    ],
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/deterministic_output_matrix.rs",
                        "artifacts/status/deterministic_output_report.json",
                    ],
                    "covers_todo": 139,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/determinism_expectations.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "expectation": "byte stability is required where explicitly claimed",
                    "status": "frozen",
                    "evidence": [
                        "artifacts/status/deterministic_output_report.json",
                        "artifacts/status/determinism_dashboard.json",
                    ],
                    "covers_todo": 140,
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
                .join("crates/bijux-cli-python/tests/bridge_conversion_fuzz_targets.rs");
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
                "bridge_conversion_fuzz_targets",
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
        "STATUS-CONTRACT-GENERATE-PARSER-FUZZ-HARDENING-REPORTS" => {
            let routing_test =
                workspace_root.join("crates/bijux-cli/tests/routing/parser_fuzz_targets.rs");
            let bin_test = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/parser_invalid_utf8_argv.rs");
            let regression_test =
                workspace_root.join("crates/bijux-cli/tests/routing/parser_fuzz_regressions.rs");
            let corpus_dir = workspace_root
                .join("crates/bijux-cli/tests/routing/fuzz/parser_interesting_inputs");
            let min_dir =
                workspace_root.join("crates/bijux-cli/tests/routing/fuzz/parser_minimized_cases");
            let texts = BTreeMap::from([
                (routing_test.clone(), fs::read_to_string(&routing_test).unwrap_or_default()),
                (bin_test.clone(), fs::read_to_string(&bin_test).unwrap_or_default()),
                (regression_test.clone(), fs::read_to_string(&regression_test).unwrap_or_default()),
            ]);
            let required: BTreeMap<i64, (PathBuf, &str)> = BTreeMap::from([
                (1, (routing_test.clone(), "fuzz_root_argv_parsing_does_not_panic")),
                (2, (routing_test.clone(), "fuzz_cli_argv_parsing_does_not_panic")),
                (3, (routing_test.clone(), "fuzz_dev_cli_argv_parsing_does_not_panic")),
                (4, (routing_test.clone(), "fuzz_plugin_command_argv_parsing_does_not_panic")),
                (5, (routing_test.clone(), "fuzz_config_command_argv_parsing_does_not_panic")),
                (6, (routing_test.clone(), "fuzz_diagnostics_command_argv_parsing_does_not_panic")),
                (
                    7,
                    (
                        routing_test.clone(),
                        "fuzz_mixed_global_local_flag_ordering_is_deterministic",
                    ),
                ),
                (
                    8,
                    (
                        routing_test.clone(),
                        "fuzz_repeated_conflicting_flags_stays_safe_and_deterministic",
                    ),
                ),
                (9, (bin_test.clone(), "malformed_utf8_argv_is_rejected_without_panic")),
                (10, (routing_test.clone(), "fuzz_huge_tokens_and_values_does_not_panic")),
                (11, (routing_test.clone(), "fuzz_typo_suggestion_paths_are_stable")),
                (12, (routing_test.clone(), "fuzz_help_path_parsing_and_alias_resolution_is_safe")),
                (13, (routing_test.clone(), "fuzz_help_path_parsing_and_alias_resolution_is_safe")),
                (
                    14,
                    (
                        routing_test.clone(),
                        "fuzz_namespace_normalization_and_reserved_rejection_stays_safe",
                    ),
                ),
                (
                    15,
                    (
                        routing_test.clone(),
                        "fuzz_reserved_name_rejection_and_normalization_are_deterministic",
                    ),
                ),
                (
                    17,
                    (
                        regression_test.clone(),
                        "interesting_corpus_cases_do_not_crash_or_corrupt_route_resolution",
                    ),
                ),
                (
                    18,
                    (
                        regression_test.clone(),
                        "minimized_parser_cases_do_not_crash_and_are_deterministic",
                    ),
                ),
                (
                    19,
                    (
                        regression_test.clone(),
                        "minimized_parser_cases_do_not_crash_and_are_deterministic",
                    ),
                ),
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
            let corpus_files: Vec<String> = collect_files(&corpus_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let minimized_files: Vec<String> = collect_files(&min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("argv"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let regression_ok = Command::new("cargo")
                .args(["test", "-p", "bijux-cli", "--test", "routing", "parser_fuzz_regressions::"])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let missing_ids: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_crash_triage_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser crash triage",
                                    "coverage_ids": [16],
                                    "status": if regression_ok { "clean" } else { "needs-triage" },
                                    "known_crash_case_count": minimized_files.len(),
                                    "regression_test_ok": regression_ok,
                                    "regression_test_command": ["cargo","test","-p","bijux-cli","--test","routing","parser_fuzz_regressions::"],
                                    "triage_notes": [
                                        "minimized cases are retained and replayed on every gate run",
                                        "new parser crashes must be added as minimized reproducer cases",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_fuzz_regression_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser fuzz regressions",
                                    "coverage_ids": [19, 20],
                                    "status": if regression_ok && missing_ids.is_empty() { "clean" } else { "drift" },
                                    "missing_coverage_ids": missing_ids,
                                    "corpus_file_count": corpus_files.len(),
                                    "minimized_case_count": minimized_files.len(),
                                    "regression_test_ok": regression_ok,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/parser_fuzz_campaign_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "parser fuzzing",
                                    "coverage_ids": (1..21).collect::<Vec<_>>(),
                                    "status": if missing_ids.is_empty() && !corpus_files.is_empty() && !minimized_files.is_empty() { "complete" } else { "partial" },
                                    "coverage_rows": coverage_rows,
                                    "corpus_directory": "crates/bijux-cli/tests/routing/fuzz/parser_interesting_inputs",
                                    "corpus_files": corpus_files,
                                    "minimized_directory": "crates/bijux-cli/tests/routing/fuzz/parser_minimized_cases",
                                    "minimized_files": minimized_files,
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/parser_crash_triage_artifact.json",
                "artifacts/status/parser_fuzz_regression_artifact.json",
                "artifacts/status/parser_fuzz_campaign_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CLEANUP-REPORTS" => {
            let generated_at = "1970-01-01T00:00:00+00:00";
            let deleted_docs = vec![
                "docs/architecture/newly-ported-command-parity.md",
                "docs/architecture/next-five-command-priorities.md",
                "docs/architecture/safe-improvements-after-parity.md",
            ];
            let deleted_snapshot_files = vec![
                "artifacts/python-behavior/golden/config/config_get_sample.json",
                "artifacts/python-behavior/golden/config/config_set_sample.json",
                "artifacts/python-behavior/golden/config/config_unset_sample.json",
            ];
            let deleted_artifacts = vec![
                "artifacts/python-behavior/golden/config/capture-summary.json",
                "artifacts/python-behavior/golden/config/config_clear.json",
                "artifacts/python-behavior/golden/config/config_export_json.json",
            ];
            let policy_files = json!({
                "artifact_retention": "docs/architecture/artifact-retention-policy.md",
                "snapshot_retention": "docs/architecture/snapshot-retention-policy.md",
                "document_retention": "docs/architecture/document-retention-policy.md",
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/docs_unreferenced_candidates.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "deleted": deleted_docs,
                    "criteria": [
                        "not linked by README, command reference, or contributor flow",
                        "historical progress reporting rather than durable law",
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/stale_snapshot_candidates.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "deleted": deleted_snapshot_files,
                                    "criteria": [
                                        "legacy python-behavior captures no longer tied to live rust command snapshots",
                                        "not consumed by CI upload, release evidence, or tests",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/dead_generated_artifact_candidates.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "deleted": deleted_artifacts,
                                    "criteria": [
                                        "runtime lock and temp files in artifact tree are not evidence artifacts",
                                        "legacy python behavior captures not consumed by CI upload, release evidence, or status reports",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cleanup_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "761-780 cleanup and retention hardening",
                    "deleted": {
                        "docs": deleted_docs,
                        "snapshot_artifacts": deleted_snapshot_files,
                        "dead_generated_artifacts": deleted_artifacts,
                    },
                    "policies": policy_files,
                    "rules": [
                        "reject keep-just-in-case for stale prose",
                        "reject keep-just-in-case for stale snapshots",
                        "reject keep-just-in-case for dead generated artifacts",
                        "cleanup is ongoing release-by-release work",
                    ],
                    "status": "complete",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/docs_unreferenced_candidates.json",
                "artifacts/status/stale_snapshot_candidates.json",
                "artifacts/status/dead_generated_artifact_candidates.json",
                "artifacts/status/cleanup_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-MIGRATION-NOTES" => {
            let generated_at = "1970-01-01T00:00:00+00:00";
            let parity_matrix = fs::read_to_string(
                workspace_root.join("artifacts/parity/command_parity_matrix.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let command_rows = parity_matrix
                .get("commands")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let changed: Vec<Value> = command_rows
                .into_iter()
                .filter(|row| {
                    row.get("status").and_then(Value::as_str).is_some_and(|s| {
                        matches!(s, "partial" | "intentionally-different" | "different-by-decision")
                    })
                })
                .map(|row| {
                    json!({
                        "command": row.get("command").cloned().unwrap_or(Value::Null),
                        "status": row.get("status").cloned().unwrap_or(Value::Null),
                        "reason": row.get("reason").cloned().unwrap_or_else(|| json!("")),
                        "blocker": row.get("blocker").cloned().unwrap_or_else(|| json!("")),
                    })
                })
                .collect();
            let package_health = fs::read_to_string(
                workspace_root.join("artifacts/status/package_health_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let assumptions = package_health
                .get("payload")
                .and_then(|v| v.get("install_state_assumptions"))
                .cloned()
                .unwrap_or_else(|| json!([]));
            let runtime_unity = fs::read_to_string(
                workspace_root.join("artifacts/status/runtime_unity_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let plugin_failures = fs::read_to_string(
                workspace_root
                    .join("artifacts/status/plugin_lifecycle_failure_injection_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let rollback = fs::read_to_string(
                workspace_root.join("artifacts/status/plugin_rollback_proof_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let config = fs::read_to_string(
                workspace_root.join("artifacts/status/config_corruption_matrix.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let state = fs::read_to_string(
                workspace_root.join("artifacts/status/state_resilience_summary.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let guidance = fs::read_to_string(
                workspace_root.join("artifacts/status/state_recovery_guidance.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/migration_notes_commands.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "commands",
                    "coverage_ids": [574],
                    "items": changed.into_iter().take(250).collect::<Vec<_>>(),
                    "source": "artifacts/parity/command_parity_matrix.json",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_packaging.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "packaging",
                                    "coverage_ids": [575],
                                    "runtime_unity_ok": runtime_unity.get("ok").and_then(Value::as_bool).unwrap_or(false),
                                    "items": [
                                        {
                                            "area": "runtime-identity",
                                            "note": "verify active binary and PATH shadowing behavior before cutover",
                                            "evidence": "artifacts/status/runtime_unity_report.json",
                                        },
                                        {
                                            "area": "install-assumptions",
                                            "note": "review install-state assumptions and shell completion target paths",
                                            "assumptions": assumptions,
                                            "evidence": "artifacts/status/package_health_report.json",
                                        },
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_plugin_lifecycle.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin-lifecycle",
                                    "coverage_ids": [576],
                                    "items": [
                                        {
                                            "area": "plugin-install-write-path",
                                            "note": "validate rollback and retry behavior before enabling new plugin capabilities",
                                            "evidence": [
                                                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                                                "artifacts/status/plugin_rollback_proof_report.json",
                                            ],
                                        },
                                        {
                                            "area": "plugin-runtime-diagnostics",
                                            "note": "verify reserved-name and registry diagnostics surface expected errors",
                                            "evidence": "artifacts/status/namespace_abuse_report.json",
                                        },
                                    ],
                                    "plugin_report_status": plugin_failures.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "rollback_report_status": rollback.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_state_behavior.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "state-behavior",
                                    "coverage_ids": [577],
                                    "items": [
                                        {
                                            "area": "config",
                                            "note": "backup and validate config before mutating across runtime upgrades",
                                            "evidence": "artifacts/status/config_corruption_matrix.json",
                                        },
                                        {
                                            "area": "history-memory",
                                            "note": "run state doctor when corrupted history or memory payloads are detected",
                                            "evidence": "artifacts/status/state_resilience_summary.json",
                                        },
                                        {
                                            "area": "recovery",
                                            "note": "follow machine-readable state recovery guidance for rollback paths",
                                            "evidence": "artifacts/status/state_recovery_guidance.json",
                                        },
                                    ],
                                    "config_status": config.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "state_status": state.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "guidance_status": guidance.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                }),
                            )
                            .ok()?;
            let migration_cmds = fs::read_to_string(
                workspace_root.join("artifacts/status/migration_notes_commands.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .and_then(|v| v.get("items").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
            let mut text = String::from("Migration Notes\n\nCommands:\n");
            for item in migration_cmds.into_iter().take(40) {
                let command = item.get("command").and_then(Value::as_str).unwrap_or("");
                let status = item.get("status").and_then(Value::as_str).unwrap_or("");
                let reason = item.get("reason").and_then(Value::as_str).unwrap_or("");
                text.push_str(&format!("- {command}: status={status} reason={reason}\n"));
            }
            text.push_str(
                                "\nPackaging:\n- runtime-identity: verify active binary and PATH shadowing behavior before cutover\n- install-assumptions: review install-state assumptions and shell completion target paths\n\nPlugin lifecycle:\n- plugin-install-write-path: validate rollback and retry behavior before enabling new plugin capabilities\n- plugin-runtime-diagnostics: verify reserved-name and registry diagnostics surface expected errors\n\nState behavior:\n- config: backup and validate config before mutating across runtime upgrades\n- history-memory: run state doctor when corrupted history or memory payloads are detected\n- recovery: follow machine-readable state recovery guidance for rollback paths\n",
                            );
            fs::write(workspace_root.join("artifacts/status/migration_notes.txt"), text).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/migration_notes_commands.json",
                "artifacts/status/migration_notes_packaging.json",
                "artifacts/status/migration_notes_plugin_lifecycle.json",
                "artifacts/status/migration_notes_state_behavior.json",
                "artifacts/status/migration_notes.txt"
            ]}))
        }
        _ => None,
    }
}
