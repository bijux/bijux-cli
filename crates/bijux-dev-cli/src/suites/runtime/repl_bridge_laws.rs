#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-KERNEL-INVARIANTS-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/src/kernel_pipeline_tests.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (1, "kernel_pipeline_uses_one_canonical_entrypoint"),
                (2, "fast_path_commands_keep_valid_envelope_metadata_when_emitted"),
                (3, "cancellation_paths_never_skip_exit_code_mapping"),
                (4, "cancellation_paths_never_emit_partial_success_envelopes"),
                (5, "plugin_lifecycle_hooks_run_in_stable_order_around_execution"),
                (6, "repl_lifecycle_hooks_do_not_mutate_non_repl_command_semantics"),
                (7, "sync_and_async_handlers_produce_equivalent_normalized_results"),
                (8, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (9, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (10, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (11, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (12, "internal_failure_is_normalized_before_crossing_cli_surface"),
                (13, "trace_mode_adds_diagnostics_without_changing_payload_shape"),
                (14, "quiet_mode_suppresses_streams_but_preserves_result_category"),
                (15, "kernel_resolution_is_deterministic_under_reordered_inputs"),
                (16, "kernel_resolution_is_deterministic_under_reordered_inputs"),
                (17, "repeated_run_kernel_invariants_harness_for_representative_commands"),
            ]);
            let rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let covered = source.contains(&format!("fn {test_name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": test_name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli/src/kernel_pipeline_tests.rs",
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
                "artifacts/status/kernel_invariants_report.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "kernel pipeline invariants",
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_ids": (1..19).collect::<Vec<_>>(),
                    "rows": rows,
                    "missing": missing,
                    "summary": {
                        "covered": required.len() - missing.len(),
                        "missing": missing.len(),
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/kernel_invariants_diff.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "kernel invariants drift",
                    "status": if missing.is_empty() { "clean" } else { "drift-detected" },
                    "coverage_ids": [19],
                    "drift_items": missing
                        .iter()
                        .map(|row| json!({
                            "coverage_id": row.get("coverage_id").cloned().unwrap_or(Value::Null),
                            "kind": "missing-kernel-invariant-test",
                            "test_name": row.get("test_name").cloned().unwrap_or(Value::Null),
                        }))
                        .collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/kernel_invariants_report.json",
                "artifacts/status/kernel_invariants_diff.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-HELP-TREE-LAW-REPORTS" => {
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let sources: BTreeMap<String, String> = collect_files(&tests_root)
                .into_iter()
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                .map(|path| {
                    (rel(&path, workspace_root), fs::read_to_string(path).unwrap_or_default())
                })
                .collect();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (341, "root_help_lists_commands_in_stable_order"),
                (342, "cli_help_lists_subcommands_in_stable_order"),
                (343, "maintainer_help_lists_subcommands_in_stable_order"),
                (344, "plugin_installed_help_keeps_builtin_order_stable"),
                (345, "no_color_root_help_and_grouped_help_are_stable"),
                (346, "no_color_root_help_and_grouped_help_are_stable"),
                (347, "unknown_command_suggestions_are_deterministic_and_namespace_scoped"),
                (348, "unknown_command_suggestions_are_deterministic_and_namespace_scoped"),
                (349, "hidden_aliases_do_not_appear_as_canonical_help_entries"),
                (350, "inspect_metadata_agrees_with_help_names_and_command_tree_export"),
                (351, "inspect_metadata_agrees_with_help_names_and_command_tree_export"),
                (352, "binary_and_bridge_help_trees_are_identical_for_covered_commands"),
                (353, "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful"),
                (354, "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful"),
                (355, "command_tree_is_stable_across_repeated_plugin_discovery_runs"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let evidence = sources
                        .iter()
                        .find(|(_, src)| src.contains(&format!("fn {test_name}(")))
                        .map(|(path, _)| path.clone());
                    json!({
                        "coverage_id": coverage_id,
                        "test": test_name,
                        "status": if evidence.is_some() { "covered" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/help_law_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "help law",
                    "coverage_ids": (341..357).collect::<Vec<_>>(),
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/command_tree_help_consistency_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "command-tree help consistency",
                                    "coverage_ids": [350, 351, 352, 355, 357],
                                    "status": if missing.is_empty() { "complete" } else { "partial" },
                                    "proof": {
                                        "inspect_help_agreement": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(350) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                        "routes_help_agreement": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(351) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                        "bridge_help_parity": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(352) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                        "repeated_discovery_stability": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(355) && row.get("status").and_then(Value::as_str) == Some("covered")),
                                    },
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/help_drift_artifact.json",
                                &json!({
                                    "generated_at": generated_at_utc(),
                                    "generator": "bijux-dev-cli",
                                    "scope": "help drift",
                                    "coverage_ids": [358, 359],
                                    "status": if missing.is_empty() { "clean" } else { "drift" },
                                    "drift_count": missing.len(),
                                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/help_tree_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "help tree contract",
                    "coverage_ids": [360],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "help tree is a law surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/help_law_artifact.json",
                "artifacts/status/command_tree_help_consistency_artifact.json",
                "artifacts/status/help_drift_artifact.json",
                "artifacts/status/help_tree_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-LAW-REPORTS" => {
            let search_roots = [workspace_root.join("crates"), workspace_root.join("maintenance")];
            let bucket_patterns = [
                ("runtime", vec!["runtime-identity", "runtime_unity", "execution_outcome"]),
                ("state", vec!["state-audit", "state-doctor", "history", "memory"]),
                (
                    "plugin",
                    vec![
                        "plugins doctor",
                        "plugin-health",
                        "load_time_diagnostics",
                        "plugin_doctor",
                    ],
                ),
                ("package", vec!["package-health", "install_health_report", "packaging"]),
                ("parity", vec!["parity", "binary_vs_python_bridge"]),
                ("route", vec!["route-audit", "routes_report", "registry_report"]),
                ("health", vec!["doctor", "diagnostics"]),
            ];
            let mut taxonomy_rows = Vec::<Value>::new();
            for (bucket, patterns) in bucket_patterns {
                let mut hits = Vec::<String>::new();
                for root in &search_roots {
                    for file in collect_files(&root) {
                        let rel = rel(&file, workspace_root);
                        let ext = Path::new(&rel)
                            .extension()
                            .and_then(|v| v.to_str())
                            .unwrap_or_default();
                        if !ext.eq_ignore_ascii_case("rs") && !ext.eq_ignore_ascii_case("py") {
                            continue;
                        }
                        let content = fs::read_to_string(&file).unwrap_or_default();
                        for (idx, line) in content.lines().enumerate() {
                            if patterns.iter().any(|p| line.contains(p)) {
                                hits.push(format!("{rel}:{}:{line}", idx + 1));
                            }
                        }
                    }
                }
                hits.sort();
                taxonomy_rows.push(json!({
                    "type": bucket,
                    "evidence_count": hits.len(),
                    "examples": hits.into_iter().take(20).collect::<Vec<_>>(),
                }));
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_taxonomy.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "taxonomy": taxonomy_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_usefulness_review.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "severity_model": ["error", "warning", "info"],
                    "actionable_next_step_model": {
                        "required_fields": ["area", "severity", "message"],
                        "optional_fields": ["path", "action", "next_step"],
                    },
                    "removed_low_value_diagnostics": [
                        "legacy dev routes hidden alias diagnostics",
                        "legacy dev registry hidden alias diagnostics",
                        "duplicate route special-case counters not tied to canonical paths",
                    ],
                    "consistency_targets": {
                        "json_shape": ["status", "diagnostics"],
                        "text_output": ["header line", "plain action lines"],
                        "exit_code_expectations": {
                            "usage_error": 2,
                            "runtime_error": 1,
                            "success": 0,
                        },
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_taxonomy.json",
                "artifacts/status/diagnostics_usefulness_review.json"
            ]}))
        }
        _ => None,
    }
}
