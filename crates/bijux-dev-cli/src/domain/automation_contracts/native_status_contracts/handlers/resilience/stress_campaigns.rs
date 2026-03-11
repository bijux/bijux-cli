#[allow(clippy::wildcard_imports)]
use crate::domain::automation_contracts::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-PRODUCT-MOUNT-READINESS-REPORTS" => {
            let registry = fs::read_to_string(
                workspace_root.join("docs/constitution/official_product_namespace_registry.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let contract = fs::read_to_string(
                workspace_root.join("docs/constitution/product_mount_metadata_contract.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let namespaces = registry
                .get("entries")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|entry| {
                    entry.get("namespace").and_then(Value::as_str).map(ToString::to_string)
                })
                .collect::<Vec<_>>();
            let placeholder_entries =
                registry.get("placeholder_entries").cloned().unwrap_or_else(|| json!([]));
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/official_product_mount_registry.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "registry": registry,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/product_mount_readiness_report.json",
                                &json!({
                                    "generated_at": "1970-01-01T00:00:00+00:00",
                                    "generator": "bijux-dev-cli",
                                    "official_namespaces": namespaces,
                                    "placeholder_entries": placeholder_entries,
                                    "metadata_contract": contract,
                                    "freeze_rule": "future-ready via metadata and tests; no speculative runtime expansion",
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/product_mount_support_report.json",
                                &json!({
                                    "generated_at": "1970-01-01T00:00:00+00:00",
                                    "generator": "bijux-dev-cli",
                                    "supports_today": [
                                        "reserved namespace rejection for official mounts",
                                        "route-tree visibility for reserved official namespaces",
                                        "stable metadata contract for runtime and control binaries",
                                        "plugin lifecycle guardrails remain independent from product runtime binaries",
                                    ],
                                    "evidence": [
                                        "crates/bijux-cli-plugin/tests/plugin_namespace_regression.rs",
                                        "crates/bijux-cli-plugin/tests/official_namespace_registry.rs",
                                        "crates/bijux-cli/tests/routing/route_law_consistency.rs",
                                        "docs/constitution/official_product_namespace_registry.json",
                                        "docs/constitution/product_mount_metadata_contract.json",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/product_mount_gap_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "not_committed": [
                        "dynamic product runtime loading",
                        "external ABI stability guarantee for product plugins",
                        "network-distributed namespace registry",
                    ],
                    "why_missing": "kept intentionally out to avoid speculative core complexity",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/official_product_mount_registry.json",
                "artifacts/status/product_mount_readiness_report.json",
                "artifacts/status/product_mount_support_report.json",
                "artifacts/status/product_mount_gap_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CONFIG-FUZZ-HARDENING-REPORTS" => {
            let targets =
                workspace_root.join("crates/bijux-cli/tests/bin_surface/config_fuzz_targets.rs");
            let regression = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/config_fuzz_regressions.rs");
            let min_dir = workspace_root.join("crates/bijux-cli/tests/fuzz/config_minimized_cases");
            let targets_text = fs::read_to_string(&targets).unwrap_or_default();
            let regression_text = fs::read_to_string(&regression).unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (41, "fuzz_dotenv_style_config_parsing_is_stable"),
                (42, "fuzz_malformed_config_lines_fail_consistently"),
                (43, "fuzz_duplicate_key_handling_keeps_last_value"),
                (44, "fuzz_weird_whitespace_handling_is_stable"),
                (45, "fuzz_quote_parsing_and_escape_parsing_are_stable"),
                (46, "fuzz_quote_parsing_and_escape_parsing_are_stable"),
                (47, "fuzz_null_byte_and_control_characters_are_handled_deterministically"),
                (48, "fuzz_mixed_valid_invalid_content_never_silently_succeeds"),
                (49, "fuzz_config_export_serialization_roundtrips_for_random_inputs"),
                (50, "fuzz_config_load_import_parsing_is_deterministic"),
                (51, "fuzz_roundtrip_parse_serialize_parse_is_semantically_stable"),
                (52, "fuzz_key_normalization_and_value_validation_are_stable"),
                (53, "fuzz_key_normalization_and_value_validation_are_stable"),
                (57, "minimized_config_cases_replay_with_stable_exit_behavior"),
                (58, "fuzz_roundtrip_parse_serialize_parse_is_semantically_stable"),
                (59, "fuzz_no_silent_key_loss_invariant_holds_under_repeated_exports"),
            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(coverage_id, test_name)| {
                                    let source = if *test_name == "minimized_config_cases_replay_with_stable_exit_behavior" {
                                        &regression_text
                                    } else {
                                        &targets_text
                                    };
                                    json!({
                                        "coverage_id": coverage_id,
                                        "test": test_name,
                                        "status": if source.contains(&format!("fn {test_name}(")) { "covered" } else { "missing" },
                                        "evidence": if *test_name == "minimized_config_cases_replay_with_stable_exit_behavior" {
                                            "crates/bijux-cli/tests/bin_surface/config_fuzz_regressions.rs"
                                        } else {
                                            "crates/bijux-cli/tests/bin_surface/config_fuzz_targets.rs"
                                        },
                                    })
                                })
                                .collect();
            let minimized_cases: Vec<String> = collect_files(&min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("env"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let replay_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "config_fuzz_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let targets_ok = Command::new("cargo")
                .args(["test", "-p", "bijux-cli", "--test", "integration", "config_fuzz_targets::"])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let missing: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_parser_crash_triage_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config parser fuzz triage",
                    "coverage_ids": [54],
                    "status": if targets_ok && replay_ok { "clean" } else { "needs-triage" },
                    "regression_replay_ok": replay_ok,
                    "target_suite_ok": targets_ok,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_serializer_crash_triage_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config serializer fuzz triage",
                    "coverage_ids": [55],
                    "status": if targets_ok { "clean" } else { "needs-triage" },
                    "target_suite_ok": targets_ok,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_fuzz_regression_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config fuzz regression",
                    "coverage_ids": [56, 57],
                    "status": if replay_ok { "clean" } else { "drift" },
                    "minimized_case_count": minimized_cases.len(),
                    "regression_replay_ok": replay_ok,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/config_fuzz_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "config fuzz hardening",
                                    "coverage_ids": (41..61).collect::<Vec<_>>(),
                                    "status": if missing.is_empty() && replay_ok && targets_ok && !minimized_cases.is_empty() { "frozen" } else { "partial" },
                                    "coverage_rows": coverage_rows,
                                    "missing_coverage_ids": missing,
                                    "minimized_cases": minimized_cases,
                                    "policy": "config fuzzing is required before release claims",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_parser_crash_triage_artifact.json",
                "artifacts/status/config_serializer_crash_triage_artifact.json",
                "artifacts/status/config_fuzz_regression_artifact.json",
                "artifacts/status/config_fuzz_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-ADVERSARIAL-FS-PROCESS-REPORTS" => {
            let campaign_test = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/adversarial_fs_process_campaigns.rs");
            let min_cases_dir = workspace_root
                .join("crates/bijux-cli/tests/fuzz/adversarial_fs_process_minimized_cases");
            let campaign_text = fs::read_to_string(&campaign_test).unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (181, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
                                (182, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
                                (183, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
                                (184, "missing_parent_and_type_flip_path_cases_are_handled_without_corruption"),
                                (185, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (186, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (187, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (188, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (189, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (190, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (191, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (192, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (193, "broken_symlink_and_permission_denied_paths_surface_stable_failures"),
                                (194, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
                                (195, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
                                (196, "rename_race_and_temp_leftovers_keep_commands_non_panicking"),
                                (197, "child_process_failure_paths_surface_normalized_failures_when_plugins_are_broken"),
                                (198, "interrupted_process_behavior_is_normalized_for_interactive_entrypoint"),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(id, test_name)| {
                                    json!({
                                        "coverage_id": id,
                                        "test": test_name,
                                        "status": if campaign_text.contains(&format!("fn {test_name}(")) { "covered" } else { "missing" },
                                        "evidence": "crates/bijux-cli/tests/bin_surface/adversarial_fs_process_campaigns.rs",
                                    })
                                })
                                .collect();
            let campaign_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "adversarial_fs_process_campaigns::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let regression_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "adversarial_fs_process_campaign_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let minimized_cases: Vec<String> = collect_files(&min_cases_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let missing: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/adversarial_fs_process_matrix.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "adversarial filesystem/process matrix",
                                    "coverage_ids": (181..199).collect::<Vec<_>>(),
                                    "status": if campaign_ok && missing.is_empty() { "complete" } else { "partial" },
                                    "coverage_rows": coverage_rows,
                                    "campaign_suite": {
                                        "ok": campaign_ok,
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/adversarial_fs_process_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "adversarial filesystem/process evidence artifact",
                    "coverage_ids": [199],
                    "status": if campaign_ok && regression_ok { "complete" } else { "partial" },
                    "minimized_case_count": minimized_cases.len(),
                    "minimized_cases": minimized_cases,
                    "regression_suite": {
                        "ok": regression_ok,
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/adversarial_fs_process_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "adversarial filesystem/process hardening contract",
                                    "coverage_ids": (181..201).collect::<Vec<_>>(),
                                    "status": if campaign_ok && regression_ok && !minimized_cases.is_empty() && missing.is_empty() { "frozen" } else { "partial" },
                                    "missing_coverage_ids": missing,
                                    "policy": "adversarial fs/process behavior is first-class hardening and permanently gated",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/adversarial_fs_process_matrix.json",
                "artifacts/status/adversarial_fs_process_artifact.json",
                "artifacts/status/adversarial_fs_process_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-STATE-CORRUPTION-HARNESS-REPORTS" => {
            let harness_test = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/randomized_state_corruption_harness.rs");
            let regression_test = workspace_root.join(
                "crates/bijux-cli/tests/bin_surface/randomized_state_corruption_regressions.rs",
            );
            let min_dir =
                workspace_root.join("crates/bijux-cli/tests/fuzz/state_corruption_minimized_cases");
            let harness_text = fs::read_to_string(&harness_test).unwrap_or_default();
            let regression_text = fs::read_to_string(&regression_test).unwrap_or_default();
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                                (101, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (102, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (103, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (104, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (105, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (106, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (107, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (108, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (109, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (110, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (111, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (112, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (113, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (114, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (115, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (116, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (117, ("harness", "randomized_state_corruption_harness_exercises_all_mutators_across_supported_domains")),
                                (119, ("regression", "minimized_corrupted_state_reproducers_replay_without_crashing")),
                            ]);
            let coverage_rows: Vec<Value> = required
                                .iter()
                                .map(|(id, (src, test_name))| {
                                    let text = if *src == "regression" {
                                        &regression_text
                                    } else {
                                        &harness_text
                                    };
                                    json!({
                                        "coverage_id": id,
                                        "test": test_name,
                                        "status": if text.contains(&format!("fn {test_name}(")) { "covered" } else { "missing" },
                                        "evidence": if *src == "regression" {
                                            "crates/bijux-cli/tests/bin_surface/randomized_state_corruption_regressions.rs"
                                        } else {
                                            "crates/bijux-cli/tests/bin_surface/randomized_state_corruption_harness.rs"
                                        },
                                    })
                                })
                                .collect();
            let campaign_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "randomized_state_corruption_harness::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let replay_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "randomized_state_corruption_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let minimized_cases: Vec<String> = collect_files(&min_dir)
                .into_iter()
                .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                .map(|p| rel(&p, workspace_root))
                .collect();
            let missing: Vec<i64> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_corruption_campaign_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "randomized corruption campaign",
                    "coverage_ids": (101..119).collect::<Vec<_>>(),
                    "status": if campaign_ok { "clean" } else { "needs-triage" },
                    "campaign_suite_ok": campaign_ok,
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/state_corruption_reproducer_retention_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "minimized corrupted-state reproducer retention",
                                    "coverage_ids": [119],
                                    "status": if replay_ok && !minimized_cases.is_empty() { "clean" } else { "needs-triage" },
                                    "replay_suite_ok": replay_ok,
                                    "minimized_case_count": minimized_cases.len(),
                                    "minimized_cases": minimized_cases,
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/state_corruption_harness_contract.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "randomized state corruption harness",
                                    "coverage_ids": (101..121).collect::<Vec<_>>(),
                                    "status": if missing.is_empty() && campaign_ok && replay_ok && !minimized_cases.is_empty() { "frozen" } else { "partial" },
                                    "missing_coverage_ids": missing,
                                    "campaign_suite": {"ok": campaign_ok},
                                    "replay_suite": {"ok": replay_ok},
                                    "policy": "randomized state corruption harness is shared test utility and release hardening evidence",
                                }),
                            )
                            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/state_corruption_campaign_artifact.json",
                "artifacts/status/state_corruption_reproducer_retention_artifact.json",
                "artifacts/status/state_corruption_harness_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-COMMAND-SURFACE-INVENTORY" => {
            let generated_at = generated_at_utc();
            let matrix: Value = fs::read_to_string(
                workspace_root.join("artifacts/status/command_migration_matrix.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let matrix_rows =
                matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let documented = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/routing/fixtures/python_documented_commands.txt"),
            )
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToString::to_string)
            .collect::<BTreeSet<_>>();
            let mut matrix_by_command = BTreeMap::<String, Value>::new();
            for row in matrix_rows.iter().filter(|row| row.is_object()) {
                if let Some(command) = row.get("command").and_then(Value::as_str) {
                    matrix_by_command.insert(command.trim().to_string(), row.clone());
                }
            }
            let documented_not_proven = documented
                                .iter()
                                .map(|command| {
                                    if let Some(row) = matrix_by_command.get(command) {
                                        json!({
                                            "command": command,
                                            "status": row.get("status").and_then(Value::as_str).unwrap_or("python-only"),
                                            "surface": row.get("surface").and_then(Value::as_str).unwrap_or("root"),
                                            "blocker": row.get("blocker").and_then(Value::as_str).unwrap_or("missing rust route or implementation"),
                                        })
                                    } else {
                                        json!({
                                            "command": command,
                                            "status": "python-only",
                                            "surface": "root",
                                            "blocker": "missing rust route or implementation",
                                        })
                                    }
                                })
                                .filter(|row| row.get("status").and_then(Value::as_str) != Some("rust-complete"))
                                .collect::<Vec<_>>();
            let python_only_rows = matrix_rows
                .iter()
                .filter_map(|row| row.as_object())
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("python-only"))
                .map(|row| {
                    json!({
                        "command": row.get("command").and_then(Value::as_str).unwrap_or(""),
                        "surface": row.get("surface").and_then(Value::as_str).unwrap_or("root"),
                        "blocker": row.get("blocker").and_then(Value::as_str).unwrap_or(""),
                    })
                })
                .collect::<Vec<_>>();
            let alias_inventory: Value = fs::read_to_string(
                workspace_root.join("artifacts/status/compatibility_alias_inventory.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let shim_inventory: Value = fs::read_to_string(
                workspace_root.join("artifacts/status/compatibility_shim_inventory.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let active_aliases = alias_inventory
                                .get("aliases")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default()
                                .into_iter()
                                .filter_map(|item| item.as_object().cloned())
                                .map(|entry| {
                                    json!({
                                        "alias": entry.get("alias").and_then(Value::as_str).unwrap_or(""),
                                        "canonical": entry.get("canonical").and_then(Value::as_str).unwrap_or(""),
                                        "justification": entry.get("justification").and_then(Value::as_str).unwrap_or("compatibility path"),
                                    })
                                })
                                .collect::<Vec<_>>();
            let active_shims = shim_inventory
                                .get("shims")
                                .and_then(Value::as_array)
                                .cloned()
                                .unwrap_or_default()
                                .into_iter()
                                .filter_map(|item| item.as_object().cloned())
                                .map(|entry| {
                                    json!({
                                        "path": entry.get("path").and_then(Value::as_str).unwrap_or(""),
                                        "kind": entry.get("kind").and_then(Value::as_str).unwrap_or("compatibility-shim"),
                                        "justification": entry.get("justification").and_then(Value::as_str).unwrap_or("compatibility path"),
                                    })
                                })
                                .collect::<Vec<_>>();
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/documented_python_commands_not_proven_in_rust.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "source": "crates/bijux-cli/tests/routing/fixtures/python_documented_commands.txt",
                                    "commands": documented_not_proven,
                                    "count": documented_not_proven.len(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/public_python_paths_still_reachable.json",
                &json!({
                    "generated_at": generated_at,
                    "source": "artifacts/status/command_migration_matrix.json",
                    "commands": python_only_rows,
                    "count": python_only_rows.len(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/legacy_alias_paths_still_accepted.json",
                &json!({
                    "generated_at": generated_at,
                    "source": "artifacts/status/compatibility_alias_inventory.json",
                    "aliases": active_aliases,
                    "count": active_aliases.len(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/compatibility_shims_still_active.json",
                &json!({
                    "generated_at": generated_at,
                    "source": "artifacts/status/compatibility_shim_inventory.json",
                    "shims": active_shims,
                    "count": active_shims.len(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/documented_python_commands_not_proven_in_rust.json",
                "artifacts/status/public_python_paths_still_reachable.json",
                "artifacts/status/legacy_alias_paths_still_accepted.json",
                "artifacts/status/compatibility_shims_still_active.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-COMMAND-FAMILY-CLOSURE-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |path: &str| -> Value {
                fs::read_to_string(workspace_root.join(path))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let to_closure = |status: &str| -> &str {
                if status == "frozen" {
                    "complete"
                } else if status == "partial" || status == "missing" {
                    "partial"
                } else {
                    "evolving"
                }
            };
            let config_read = read("artifacts/status/config_read_domain_contract.json");
            let config_mutation = read("artifacts/status/config_mutation_domain_contract.json");
            let config_source = read("artifacts/status/config_source_precedence_contract.json");
            let plugin_status = read("artifacts/status/plugin_command_set_status.json");
            let history_read = read("artifacts/status/history_read_domain_contract.json");
            let memory_read = read("artifacts/status/memory_read_domain_contract.json");
            let diagnostics = read("artifacts/status/diagnostics_operator_truth_contract.json");
            let repl_parity = read("artifacts/status/status_repl_parity_coverage.json");
            let repl_only = read("artifacts/status/repl_only_behaviors.json");
            let config_statuses = [
                to_closure(config_read.get("status").and_then(Value::as_str).unwrap_or("")),
                to_closure(config_mutation.get("status").and_then(Value::as_str).unwrap_or("")),
                to_closure(config_source.get("status").and_then(Value::as_str).unwrap_or("")),
            ];
            let config_closure = if config_statuses.iter().all(|item| *item == "complete") {
                "complete"
            } else if config_statuses.iter().any(|item| *item == "partial") {
                "partial"
            } else {
                "evolving"
            };
            let plugin_partial = plugin_status
                .get("plugin_commands")
                .and_then(Value::as_object)
                .and_then(|m| m.get("partial"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut plugin_closure = if plugin_partial.is_empty() { "complete" } else { "partial" };
            if plugin_status.get("classification").and_then(Value::as_str) == Some("evolving")
                && plugin_closure == "complete"
            {
                plugin_closure = "evolving";
            }
            let history_closure =
                to_closure(history_read.get("status").and_then(Value::as_str).unwrap_or(""));
            let memory_closure =
                to_closure(memory_read.get("status").and_then(Value::as_str).unwrap_or(""));
            let diagnostics_closure =
                to_closure(diagnostics.get("status").and_then(Value::as_str).unwrap_or(""));
            let repl_partial_count = repl_parity
                .get("summary")
                .and_then(Value::as_object)
                .and_then(|s| s.get("statuses"))
                .and_then(Value::as_object)
                .map(|statuses| {
                    statuses.get("partial").and_then(Value::as_i64).unwrap_or(0)
                        + statuses.get("shim").and_then(Value::as_i64).unwrap_or(0)
                })
                .unwrap_or(0);
            let repl_only_count =
                repl_only.get("repl_only_behaviors").and_then(Value::as_array).map_or(0, Vec::len);
            let repl_closure = if repl_partial_count > 0 {
                "partial"
            } else if repl_only_count > 0 {
                "evolving"
            } else {
                "complete"
            };
            let reports = BTreeMap::from([
                (
                    "config",
                    json!({"area":"config","status":config_closure,"evidence":["artifacts/status/config_read_domain_contract.json","artifacts/status/config_mutation_domain_contract.json","artifacts/status/config_source_precedence_contract.json"]}),
                ),
                (
                    "plugins",
                    json!({"area":"plugins","status":plugin_closure,"evidence":["artifacts/status/plugin_command_set_status.json","artifacts/status/plugin_migration_report.json"]}),
                ),
                (
                    "history",
                    json!({"area":"history","status":history_closure,"evidence":["artifacts/status/history_read_domain_contract.json"]}),
                ),
                (
                    "memory",
                    json!({"area":"memory","status":memory_closure,"evidence":["artifacts/status/memory_read_domain_contract.json"]}),
                ),
                (
                    "diagnostics",
                    json!({"area":"diagnostics","status":diagnostics_closure,"evidence":["artifacts/status/diagnostics_operator_truth_contract.json"]}),
                ),
                (
                    "repl_shared_law",
                    json!({"area":"repl_shared_law","status":repl_closure,"evidence":["artifacts/status/status_repl_parity_coverage.json","artifacts/status/repl_only_behaviors.json"]}),
                ),
            ]);
            for (key, payload) in &reports {
                let mut with_meta = payload.clone();
                with_meta["generated_at"] = json!(generated_at);
                with_meta["generator"] = json!("bijux-dev-cli");
                write_status_artifact_json(
                    workspace_root,
                    &format!("artifacts/status/{key}_closure_report.json"),
                    &with_meta,
                )
                .ok()?;
            }
            let mut summary = BTreeMap::from([("complete", 0), ("partial", 0), ("evolving", 0)]);
            for payload in reports.values() {
                if let Some(status) = payload.get("status").and_then(Value::as_str) {
                    if let Some(slot) = summary.get_mut(status) {
                        *slot += 1;
                    }
                }
            }
            let combined = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "scope": "command family closure",
                "reports": reports,
                "summary": summary,
                "status": if summary["partial"] == 0 { "green" } else { "attention-required" },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_family_closure_report.json",
                &combined,
            )
            .ok()?;
            let accepted_areas = reports
                .iter()
                .filter_map(|(name, payload)| {
                    (payload.get("status").and_then(Value::as_str) != Some("complete"))
                        .then_some(*name)
                })
                .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_family_partial_area_acceptance.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "partial area acceptance",
                    "required_when_partial_exists": true,
                    "accepted_areas": accepted_areas,
                    "status": if accepted_areas.is_empty() { "not-required" } else { "accepted" },
                }),
            )
            .ok()?;
            let mut lines = vec![
                "Command Family Closure Report".to_string(),
                format!(
                    "status: {}",
                    combined.get("status").and_then(Value::as_str).unwrap_or("attention-required")
                ),
                format!("complete: {}", summary["complete"]),
                format!("partial: {}", summary["partial"]),
                format!("evolving: {}", summary["evolving"]),
                String::new(),
                "areas:".to_string(),
            ];
            for (name, payload) in &reports {
                lines.push(format!(
                    "- {name}: {}",
                    payload.get("status").and_then(Value::as_str).unwrap_or("evolving")
                ));
            }
            lines.push(String::new());
            lines.push("review step: explicitly accept every non-complete area in artifacts/status/command_family_partial_area_acceptance.json".to_string());
            fs::write(
                workspace_root.join("artifacts/status/command_family_closure_report.txt"),
                format!("{}\n", lines.join("\n")),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/config_closure_report.json",
                "artifacts/status/plugins_closure_report.json",
                "artifacts/status/history_closure_report.json",
                "artifacts/status/memory_closure_report.json",
                "artifacts/status/diagnostics_closure_report.json",
                "artifacts/status/repl_shared_law_closure_report.json",
                "artifacts/status/command_family_closure_report.json",
                "artifacts/status/command_family_closure_report.txt",
                "artifacts/status/command_family_partial_area_acceptance.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-COMMAND-MIGRATION-MATRIX" => {
            let generated_at = generated_at_utc();
            let read = |path: &str| -> Value {
                fs::read_to_string(workspace_root.join(path))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let normalize = |status: &str| -> &str {
                match status {
                    "complete" => "rust-complete",
                    "partial" => "rust-partial",
                    "missing" => "python-only",
                    "different-by-decision" => "intentionally-different",
                    _ => "rust-partial",
                }
            };
            let command_surface = |command: &str| -> &str {
                let parts = command.split_whitespace().collect::<Vec<_>>();
                if parts.is_empty() {
                    return "unknown";
                }
                if parts[0] == "plugins" || (parts[0] == "cli" && parts.get(1) == Some(&"plugins"))
                {
                    return "plugin";
                }
                if parts[0] == "dev" && parts.get(1) == Some(&"cli") {
                    return "dev-cli";
                }
                if parts[0] == "cli" {
                    return "cli";
                }
                if parts.iter().any(|p| *p == "repl") {
                    return "repl";
                }
                "root"
            };
            let parity = read("artifacts/parity/command_parity_matrix.json");
            let repl = read("artifacts/parity/repl_parity_matrix.json");
            let bridge = read("artifacts/parity/python_bridge_parity_matrix.json");
            let source_rows =
                parity.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let repl_rows = repl.get("rows").and_then(Value::as_array).cloned().unwrap_or_default();
            let bridge_rows =
                bridge.get("rows").and_then(Value::as_array).cloned().unwrap_or_default();

            let mut rows = Vec::<Value>::new();
            for item in source_rows {
                let Some(command) = item.get("command").and_then(Value::as_str) else {
                    continue;
                };
                let status =
                    normalize(item.get("status").and_then(Value::as_str).unwrap_or("partial"));
                let links =
                    item.get("evidence_links").and_then(Value::as_array).cloned().unwrap_or_else(
                        || vec![json!("artifacts/parity/command_parity_matrix.json")],
                    );
                rows.push(json!({
                                    "command": command.trim(),
                                    "surface": command_surface(command.trim()),
                                    "status": status,
                                    "owner": item.get("owner").and_then(Value::as_str).unwrap_or(if status == "rust-partial" { "rust-foundation" } else { "" }),
                                    "blocker": item.get("blocker").and_then(Value::as_str).unwrap_or(if status == "python-only" { "missing rust route or implementation" } else if status == "rust-partial" { "parity coverage incomplete" } else { "" }),
                                    "reason": item.get("reason").and_then(Value::as_str).unwrap_or(if status == "intentionally-different" { "documented behavior divergence" } else { "" }),
                                    "evidence_links": links,
                                    "evidence": links,
                                }));
            }
            for item in repl_rows {
                let Some(command) = item.get("command").and_then(Value::as_str) else {
                    continue;
                };
                let status =
                    normalize(item.get("status").and_then(Value::as_str).unwrap_or("partial"));
                let mut links = item
                    .get("evidence_links")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                if links.is_empty() {
                    links.push(json!("artifacts/parity/command_parity_matrix.json"));
                }
                links.push(json!("artifacts/parity/repl_parity_matrix.json"));
                rows.push(json!({
                                    "command": command.trim(),
                                    "surface": "repl",
                                    "status": status,
                                    "owner": item.get("owner").and_then(Value::as_str).unwrap_or(if status == "rust-partial" { "rust-foundation" } else { "" }),
                                    "blocker": item.get("blocker").and_then(Value::as_str).unwrap_or(if status == "python-only" { "missing rust route or implementation" } else if status == "rust-partial" { "parity coverage incomplete" } else { "" }),
                                    "reason": item.get("reason").and_then(Value::as_str).unwrap_or(if status == "intentionally-different" { "documented behavior divergence" } else { "" }),
                                    "evidence_links": links,
                                    "evidence": links,
                                }));
            }
            for item in bridge_rows {
                let Some(command) = item.get("command").and_then(Value::as_str) else {
                    continue;
                };
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .map(normalize)
                    .unwrap_or_else(|| {
                        if item.get("stdout_match").and_then(Value::as_bool).unwrap_or(false)
                            && item.get("stderr_match").and_then(Value::as_bool).unwrap_or(false)
                            && item.get("exit_match").and_then(Value::as_bool).unwrap_or(false)
                        {
                            "rust-complete"
                        } else {
                            "rust-partial"
                        }
                    });
                rows.push(json!({
                                    "command": command.trim(),
                                    "surface": "python-bridge",
                                    "status": status,
                                    "owner": item.get("owner").and_then(Value::as_str).unwrap_or(if status == "rust-partial" { "rust-foundation" } else { "" }),
                                    "blocker": item.get("blocker").and_then(Value::as_str).unwrap_or(if status == "python-only" { "missing rust route or implementation" } else if status == "rust-partial" { "parity coverage incomplete" } else { "" }),
                                    "reason": item.get("reason").and_then(Value::as_str).unwrap_or(if status == "intentionally-different" { "documented behavior divergence" } else { "" }),
                                    "evidence_links": ["artifacts/parity/python_bridge_parity_matrix.json"],
                                    "evidence": ["artifacts/parity/python_bridge_parity_matrix.json"],
                                }));
            }
            rows.sort_by(|a, b| {
                let asurf = a.get("surface").and_then(Value::as_str).unwrap_or("");
                let bsurf = b.get("surface").and_then(Value::as_str).unwrap_or("");
                let acmd = a.get("command").and_then(Value::as_str).unwrap_or("");
                let bcmd = b.get("command").and_then(Value::as_str).unwrap_or("");
                (asurf, acmd).cmp(&(bsurf, bcmd))
            });
            let rust_partial = rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) == Some("rust-partial"))
                .cloned()
                .collect::<Vec<_>>();
            let python_only = rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) == Some("python-only"))
                .cloned()
                .collect::<Vec<_>>();
            let intentional = rows
                .iter()
                .filter(|r| {
                    r.get("status").and_then(Value::as_str) == Some("intentionally-different")
                })
                .cloned()
                .collect::<Vec<_>>();
            let surfaces = json!({
                "root": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("root")).cloned().collect::<Vec<_>>(),
                "cli": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("cli")).cloned().collect::<Vec<_>>(),
                "dev_cli": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("dev-cli")).cloned().collect::<Vec<_>>(),
                "plugin": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("plugin")).cloned().collect::<Vec<_>>(),
                "repl": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("repl")).cloned().collect::<Vec<_>>(),
                "python_bridge": rows.iter().filter(|r| r.get("surface").and_then(Value::as_str) == Some("python-bridge")).cloned().collect::<Vec<_>>(),
            });
            let summary = json!({
                "total": rows.len(),
                "rust-complete": rows.iter().filter(|r| r.get("status").and_then(Value::as_str) == Some("rust-complete")).count(),
                "rust-partial": rust_partial.len(),
                "python-only": python_only.len(),
                "intentionally-different": intentional.len(),
            });
            write_status_artifact_json(workspace_root, "artifacts/status/command_migration_matrix.json", &json!({
                                "generated_at": generated_at,
                                "generator": "bijux-dev-cli",
                                "status_model": ["rust-complete","rust-partial","python-only","intentionally-different"],
                                "summary": summary,
                                "commands": rows,
                                "surfaces": surfaces,
                            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_migration_rust_partial.json",
                &json!({
                    "generated_at": generated_at,
                    "commands": rust_partial,
                    "count": rust_partial.len(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_migration_python_only.json",
                &json!({
                    "generated_at": generated_at,
                    "commands": python_only,
                    "count": python_only.len(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_migration_intentional_differences.json",
                &json!({
                    "generated_at": generated_at,
                    "commands": intentional,
                    "count": intentional.len(),
                }),
            )
            .ok()?;
            let text = format!(
                                "Command Migration Matrix\ntotal: {}\nrust-complete: {}\nrust-partial: {}\npython-only: {}\nintentionally-different: {}\n",
                                summary["total"], summary["rust-complete"], summary["rust-partial"], summary["python-only"], summary["intentionally-different"]
                            );
            fs::write(workspace_root.join("artifacts/status/command_migration_matrix.txt"), text)
                .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_migration_repl_paths.json",
                &json!({
                    "generated_at": generated_at,
                    "source": "artifacts/parity/repl_parity_matrix.json",
                    "commands": surfaces.get("repl").cloned().unwrap_or_else(|| json!([])),
                    "count": surfaces.get("repl").and_then(Value::as_array).map_or(0, Vec::len),
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/command_migration_python_bridge_entrypoints.json", &json!({
                                "generated_at": generated_at,
                                "source": "artifacts/parity/python_bridge_parity_matrix.json",
                                "commands": surfaces.get("python_bridge").cloned().unwrap_or_else(|| json!([])),
                                "count": surfaces.get("python_bridge").and_then(Value::as_array).map_or(0, Vec::len),
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/command_migration_matrix.json",
                "artifacts/status/command_migration_rust_partial.json",
                "artifacts/status/command_migration_python_only.json",
                "artifacts/status/command_migration_intentional_differences.json",
                "artifacts/status/command_migration_matrix.txt",
                "artifacts/status/command_migration_repl_paths.json",
                "artifacts/status/command_migration_python_bridge_entrypoints.json"
            ]}))
        }
        _ => None,
    }
}
