#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-PRODUCT-MOUNT-READINESS-REPORTS" => {
            let registry = fs::read_to_string(
                workspace_root.join("contracts/official_product_namespace_registry.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let contract = fs::read_to_string(
                workspace_root.join("contracts/product_mount_metadata_contract.json"),
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
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "registry": registry,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/product_mount_readiness_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "official_namespaces": namespaces,
                                    "placeholder_entries": placeholder_entries,
                                    "metadata_contract": contract,
                                    "freeze_rule": "release-boundary enforced via metadata and tests; no speculative runtime expansion",
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/product_mount_support_report.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "supports_today": [
                                        "reserved namespace rejection for official mounts",
                                        "route-tree visibility for reserved official namespaces",
                                        "stable metadata contract for runtime and control binaries",
                                        "plugin lifecycle guardrails remain independent from product runtime binaries",
                                    ],
                                    "evidence": [
                                        "crates/bijux-cli/tests/integration/cli/plugins/plugin_namespace_law.rs",
                                        "crates/bijux-cli/tests/routing/registry/registry_namespace_policy.rs",
                                        "crates/bijux-cli/tests/routing/route_law_consistency.rs",
                                        "contracts/official_product_namespace_registry.json",
                                        "contracts/product_mount_metadata_contract.json",
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/product_mount_gap_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
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
            let targets = workspace_root
                .join("crates/bijux-cli/tests/integration/cli/config/config_parser_stability.rs");
            let regression = workspace_root
                .join("crates/bijux-cli/tests/integration/cli/config/config_case_replays.rs");
            let min_dir = workspace_root.join("crates/bijux-cli/tests/fuzz/config_minimized_cases");
            let targets_text = fs::read_to_string(&targets).unwrap_or_default();
            let regression_text = fs::read_to_string(&regression).unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (41, "fuzz_dotenv_style_config_parsing_is_stable"),
                (42, "fuzz_malformed_config_lines_fail_consistently"),
                (43, "fuzz_duplicate_key_handling_rejects_ambiguous_state"),
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
                                            "crates/bijux-cli/tests/integration/cli/config/config_case_replays.rs"
                                        } else {
                                            "crates/bijux-cli/tests/integration/cli/config/config_parser_stability.rs"
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
                    "config_case_replays::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let targets_ok = Command::new("cargo")
                .args(["test", "-p", "bijux-cli", "--test", "integration", "config_parser_stability::"])
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
                .join("crates/bijux-cli/tests/integration/cli/resilience/adversarial_fs_process_campaigns.rs");
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
                                        "evidence": "crates/bijux-cli/tests/integration/cli/resilience/adversarial_fs_process_campaigns.rs",
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
                .join("crates/bijux-cli/tests/integration/cli/resilience/randomized_state_corruption_harness.rs");
            let regression_test = workspace_root.join(
                "crates/bijux-cli/tests/integration/cli/resilience/randomized_state_corruption_regressions.rs",
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
                                            "crates/bijux-cli/tests/integration/cli/resilience/randomized_state_corruption_regressions.rs"
                                        } else {
                                            "crates/bijux-cli/tests/integration/cli/resilience/randomized_state_corruption_harness.rs"
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
        _ => None,
    }
}
