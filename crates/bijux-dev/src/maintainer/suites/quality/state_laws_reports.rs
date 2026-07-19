#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MEMORY-SURFACE-REPORTS" => {
            let coverage_source =
                fs::read_to_string(workspace_root.join(
                    "crates/bijux-cli/tests/integration/cli/memory/memory_command_coverage.rs",
                ))
                .unwrap_or_default();
            let parity_source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (342, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (343, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (344, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (345, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (346, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (347, "memory_root_and_list_missing_empty_valid_text_json_yaml"),
                (348, "memory_malformed_wrong_type_missing_required_and_extra_fields"),
                (349, "memory_malformed_wrong_type_missing_required_and_extra_fields"),
                (350, "memory_malformed_wrong_type_missing_required_and_extra_fields"),
                (351, "memory_malformed_wrong_type_missing_required_and_extra_fields"),
                (352, "memory_quiet_no_color_and_deterministic_repeated_runs"),
                (353, "memory_quiet_no_color_and_deterministic_repeated_runs"),
                (354, "memory_quiet_no_color_and_deterministic_repeated_runs"),
                (355, "memory_unwritable_storage_conditions_for_read_and_write_paths"),
                (356, "memory_config_path_override_does_not_change_home_memory_resolution"),
                (357, "memory_quiet_no_color_and_deterministic_repeated_runs"),
                (358, "memory_malformed_wrong_type_missing_required_and_extra_fields"),
                (359, "memory_root_parity_with_python_summary_command"),
            ]);
            let coverage_rows = required.iter().map(|(id, name)| {
                                        let in_coverage = coverage_source.contains(&format!("fn {name}("));
                                        let in_parity = parity_source.contains(&format!("fn {name}("));
                                        json!({
                                            "coverage_id": id,
                                            "test": name,
                                            "status": if in_coverage || in_parity { "complete" } else { "missing" },
                                            "evidence": if in_coverage { "crates/bijux-cli/tests/integration/cli/memory/memory_command_coverage.rs" } else { "crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs" },
                                        })
                                    }).collect::<Vec<_>>();
            let generated_at = generated_at_utc();
            write_status_artifact_json(workspace_root, "artifacts/status/memory_command_coverage_report.json", &json!({
                                        "generated_at": generated_at,
                                        "generator":"bijux-dev-cli",
                                        "scope":"memory command coverage",
                                        "commands": coverage_rows,
                                        "summary":{
                                            "total":coverage_rows.len(),
                                            "complete":coverage_rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("complete")).count(),
                                            "partial":0,"shim":0,
                                            "missing":coverage_rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("missing")).count(),
                                        }
                                    })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_command_coverage_artifact.json",
                &json!({
                    "generated_at": generated_at,
                    "generator":"bijux-dev-cli",
                    "scope":"memory command coverage artifact",
                    "coverage_rows":coverage_rows,
                    "commands":coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/memory_corruption_matrix_artifact.json", &json!({
                                        "generated_at": generated_at,
                                        "generator":"bijux-dev-cli",
                                        "scope":"memory corruption matrix",
                                        "cases":[
                                            {"name":"malformed memory state and wrong-type fields","status":"complete","evidence":"memory_malformed_wrong_type_missing_required_and_extra_fields"},
                                            {"name":"unwritable storage write path","status":"complete","evidence":"memory_unwritable_storage_conditions_for_read_and_write_paths"},
                                        ],
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/memory_python_parity_artifact.json", &json!({
                                        "generated_at": generated_at,
                                        "generator":"bijux-dev-cli",
                                        "scope":"memory parity versus overlapping python behavior",
                                        "status": if parity_source.contains("fn memory_root_parity_with_python_summary_command(") { "complete" } else { "partial" },
                                        "evidence":[
                                            "crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs",
                                            "crates/bijux-cli/tests/integration/cli/memory/memory_command_coverage.rs",
                                        ],
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/memory_read_domain_contract.json", &json!({
                                        "generated_at": generated_at,
                                        "generator":"bijux-dev-cli",
                                        "domain":"memory-read-behavior",
                                        "status":"frozen",
                                        "rule":"Memory read behavior is accepted only when determinism and corruption handling remain green.",
                                        "evidence":[
                                            "crates/bijux-cli/tests/integration/cli/memory/memory_command_coverage.rs",
                                            "artifacts/status/memory_command_coverage_artifact.json",
                                            "artifacts/status/memory_corruption_matrix_artifact.json",
                                            "artifacts/status/memory_python_parity_artifact.json",
                                        ],
                                    })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/memory_command_coverage_report.json",
                "artifacts/status/memory_command_coverage_artifact.json",
                "artifacts/status/memory_corruption_matrix_artifact.json",
                "artifacts/status/memory_python_parity_artifact.json",
                "artifacts/status/memory_read_domain_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-STATE-LAW-REPORTS" => {
            let generated_at = generated_at_utc();
            let rg_lines = |pattern: &str| -> Vec<String> {
                Command::new("rg")
                    .args(["-n", pattern, "crates", "-S"])
                    .current_dir(workspace_root)
                    .output()
                    .ok()
                    .and_then(|output| String::from_utf8(output.stdout).ok())
                    .unwrap_or_default()
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .map(ToString::to_string)
                    .collect()
            };
            let inventory = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "state_files": [
                    {"id":"config_file","classification":"core","path_source":"discover_compatibility_paths","reader":"FileConfigRepository::load","writer":"FileConfigRepository::save"},
                    {"id":"history_file","classification":"core","path_source":"discover_compatibility_paths","reader":"read_history_entries","writer":"repl::flush_history"},
                    {"id":"plugin_registry_file","classification":"core","path_source":"registry_path_from_plugins_dir","reader":"plugin::load_registry","writer":"plugin::save_registry"},
                    {"id":"memory_file","classification":"optional","path_source":"resolve_state_paths","reader":"read_memory_map","writer":"write_memory_map"},
                    {"id":"compatibility_config_file","classification":"optional","path_source":"default_compatibility_paths","reader":"load_compatibility_config","writer":"write_compatibility_config"}
                ],
            });
            let readers = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "matches": rg_lines("read_to_string|load_registry|load_history|read_history_entries|read_memory_map"),
            });
            let writers = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "matches": rg_lines("atomic_write_text|save_registry|flush_history|write_compatibility_config|FileConfigRepository::save"),
            });
            let mutations = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "matches": rg_lines("set_pair|unset_key|clear_all|install_plugin|uninstall_plugin|enable_plugin|disable_plugin"),
            });
            let write_guarantees = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "guarantees": [
                    {"name":"core config writes are atomic","evidence":"crates/bijux-cli/src/features/config/storage.rs uses atomic_write_text"},
                    {"name":"compatibility config writes are atomic","evidence":"crates/bijux-cli/src/features/install/compatibility.rs uses atomic_write_text"},
                    {"name":"plugin registry writes use temp+rename","evidence":"crates/bijux-cli/src/features/plugins/registry.rs::save_registry"},
                    {"name":"repl history writes are atomic","evidence":"crates/bijux-cli/src/interface/repl/history.rs::flush_history uses atomic_write_text"},
                    {"name":"core history and memory writes are atomic","evidence":"crates/bijux-cli/src/infrastructure/state_store.rs::write_json_document uses atomic_write_text"},
                ],
            });
            let recovery_guarantees = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "guarantees": [
                    {"name":"plugin registry rollback on mutation failure","evidence":"crates/bijux-cli/src/features/plugins/registry.rs::update_registry"},
                    {"name":"state doctor surfaces degraded state with issues","evidence":"crates/bijux-cli/src/features/diagnostics/state_paths.rs::state_diagnostics"},
                    {"name":"history corruption is tolerated with fallback parser","evidence":"crates/bijux-cli/src/infrastructure/state_store.rs::parse_history_entries"},
                ],
            });
            let complexity = json!({
                "generated_at": generated_at,
                "generator":"bijux-dev-cli",
                "canonical_services":[
                    "crates/bijux-cli/src/features/diagnostics/state_paths.rs::resolve_state_paths",
                    "crates/bijux-cli/src/features/install/io.rs::atomic_write_text",
                ],
                "hotspots":[
                    "crates/bijux-cli/src/infrastructure/state_store.rs",
                    "crates/bijux-cli/src/features/plugins/registry.rs",
                    "crates/bijux-cli/src/interface/repl/history.rs",
                ],
                "summary":{
                    "inventory_count": inventory.get("state_files").and_then(Value::as_array).map_or(0, Vec::len),
                    "reader_matches": readers.get("matches").and_then(Value::as_array).map_or(0, Vec::len),
                    "writer_matches": writers.get("matches").and_then(Value::as_array).map_or(0, Vec::len),
                    "mutation_matches": mutations.get("matches").and_then(Value::as_array).map_or(0, Vec::len),
                }
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_file_inventory.json",
                &inventory,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_file_readers.json",
                &readers,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_file_writers.json",
                &writers,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_file_mutation_paths.json",
                &mutations,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_write_guarantees.json",
                &write_guarantees,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_recovery_guarantees.json",
                &recovery_guarantees,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_complexity_report.json",
                &complexity,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/state_file_inventory.json",
                "artifacts/status/state_file_readers.json",
                "artifacts/status/state_file_writers.json",
                "artifacts/status/state_file_mutation_paths.json",
                "artifacts/status/state_write_guarantees.json",
                "artifacts/status/state_recovery_guarantees.json",
                "artifacts/status/state_complexity_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-STREAM-DISCIPLINE-REPORTS" => {
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let sources: BTreeMap<String, String> = collect_files(&tests_root)
                .into_iter()
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
                .map(|path| {
                    (rel(&path, workspace_root), fs::read_to_string(path).unwrap_or_default())
                })
                .collect();
            let cases: Vec<(i64, &str, Vec<&str>, i32, bool, bool)> = vec![
                (
                    41,
                    "success_machine_json_stderr_empty",
                    vec!["status", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    42,
                    "success_text_no_stderr_noise",
                    vec!["status", "--format", "text"],
                    0,
                    true,
                    true,
                ),
                (43, "usage_error_stderr_only", vec!["config", "get"], 2, false, false),
                (
                    44,
                    "validation_error_stderr_only",
                    vec!["--format", "not-a-format", "status"],
                    1,
                    false,
                    false,
                ),
                (45, "plugin_error_stderr_only", vec!["plugins", "uninstall"], 1, false, false),
                (46, "internal_like_error_stderr_only", vec!["plugins", "enable"], 1, false, false),
                (
                    47,
                    "quiet_mode_suppresses_stdout",
                    vec!["--quiet", "status", "--format", "json", "--no-pretty"],
                    0,
                    false,
                    true,
                ),
                (
                    48,
                    "quiet_mode_suppresses_nonessential_stderr",
                    vec!["--quiet", "status", "--format", "json", "--no-pretty"],
                    0,
                    false,
                    true,
                ),
                (
                    49,
                    "trace_mode_stream_contract",
                    vec!["--log-level", "trace", "status", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    50,
                    "pretty_json_stream_contract",
                    vec!["status", "--format", "json", "--pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    51,
                    "compact_json_stream_contract",
                    vec!["status", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    52,
                    "yaml_stream_contract",
                    vec!["status", "--format", "yaml", "--pretty"],
                    0,
                    true,
                    true,
                ),
                (53, "help_no_unrelated_stderr", vec!["help", "status"], 0, true, true),
                (54, "version_no_unrelated_stderr", vec!["version"], 0, true, true),
                (
                    55,
                    "plugin_commands_follow_stream_law",
                    vec!["plugins", "list", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    56,
                    "state_doctor_follows_stream_law",
                    vec!["state-doctor", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
                (
                    57,
                    "binary_bridge_stream_routing_consistency",
                    vec!["status", "--format", "json", "--no-pretty"],
                    0,
                    true,
                    true,
                ),
            ];
            let mut rows = Vec::<Value>::new();
            let mut drift_items = Vec::<Value>::new();
            for (
                coverage_id,
                name,
                args,
                expect_code,
                expect_stdout_nonempty,
                expect_stderr_empty,
            ) in cases
            {
                let output = Command::new("cargo")
                    .args(["run", "-q", "-p", "bijux-cli", "--"])
                    .args(&args)
                    .current_dir(workspace_root)
                    .output()
                    .ok();
                let (observed_exit_code, observed_stdout_nonempty, observed_stderr_empty) =
                    if let Some(output) = output {
                        (
                            output.status.code().unwrap_or(1),
                            !output.stdout.is_empty(),
                            output.stderr.is_empty(),
                        )
                    } else {
                        (1, false, false)
                    };
                let covered = observed_exit_code == expect_code
                    && observed_stdout_nonempty == expect_stdout_nonempty
                    && observed_stderr_empty == expect_stderr_empty;
                let row = json!({
                    "coverage_id": coverage_id,
                    "name": name,
                    "command": args.join(" "),
                    "expected_exit_code": expect_code,
                    "observed_exit_code": observed_exit_code,
                    "expected_stdout_nonempty": expect_stdout_nonempty,
                    "observed_stdout_nonempty": observed_stdout_nonempty,
                    "expected_stderr_empty": expect_stderr_empty,
                    "observed_stderr_empty": observed_stderr_empty,
                    "status": if covered { "covered" } else { "drift" },
                });
                if !covered {
                    drift_items.push(row.clone());
                }
                rows.push(row);
            }
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (41, "successful_machine_readable_commands_keep_stderr_empty"),
                (42, "text_success_commands_do_not_leak_diagnostics_to_stderr_in_normal_mode"),
                (43, "usage_validation_plugin_and_internal_failures_route_to_stderr_only"),
                (44, "usage_validation_plugin_and_internal_failures_route_to_stderr_only"),
                (45, "usage_validation_plugin_and_internal_failures_route_to_stderr_only"),
                (46, "usage_validation_plugin_and_internal_failures_route_to_stderr_only"),
                (47, "quiet_mode_suppresses_success_stdout_and_nonessential_stderr_noise"),
                (48, "quiet_mode_suppresses_success_stdout_and_nonessential_stderr_noise"),
                (49, "trace_mode_preserves_stream_contract_without_corrupting_output_envelope"),
                (50, "pretty_compact_json_and_yaml_all_respect_stream_discipline"),
                (51, "pretty_compact_json_and_yaml_all_respect_stream_discipline"),
                (52, "pretty_compact_json_and_yaml_all_respect_stream_discipline"),
                (53, "help_and_version_fast_paths_do_not_leak_unrelated_diagnostics_to_stderr"),
                (54, "help_and_version_fast_paths_do_not_leak_unrelated_diagnostics_to_stderr"),
                (55, "plugin_and_state_doctor_commands_obey_builtin_stream_law"),
                (56, "plugin_and_state_doctor_commands_obey_builtin_stream_law"),
                (57, "binary_and_bridge_agree_on_stream_routing_for_success_and_failure"),
            ]);
            let coverage_rows = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let evidence = sources
                        .iter()
                        .find(|(_, src)| src.contains(&format!("fn {test_name}(")))
                        .map(|(path, _)| path.clone());
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": test_name,
                        "status": if evidence.is_some() { "covered" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect::<Vec<_>>();
            let missing_coverage_ids = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/stream_discipline_artifact.json", &json!({
                                        "generator":"bijux-dev-cli",
                                        "scope":"stdout-stderr discipline",
                                        "status": if drift_items.is_empty() && missing_coverage_ids.is_empty() { "complete" } else { "partial" },
                                        "coverage_ids": (41..59).collect::<Vec<_>>(),
                                        "release_blocking": true,
                                        "rows": rows,
                                        "coverage_rows": coverage_rows,
                                        "summary": {
                                            "covered_rows": rows.len().saturating_sub(drift_items.len()),
                                            "drift_rows": drift_items.len(),
                                            "covered_requirements": coverage_rows.len().saturating_sub(missing_coverage_ids.len()),
                                            "missing_coverage_ids": missing_coverage_ids.len(),
                                        },
                                    })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/stream_drift_artifact.json",
                &json!({
                    "generator":"bijux-dev-cli",
                    "scope":"stdout-stderr discipline drift",
                    "status": if drift_items.is_empty() { "clean" } else { "drift-detected" },
                    "coverage_ids":[59,60],
                    "drift_count": drift_items.len(),
                    "drift_items": drift_items,
                    "missing_coverage_ids": missing_coverage_ids,
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/stream_discipline_artifact.json",
                "artifacts/status/stream_drift_artifact.json"
            ]}))
        }
        _ => None,
    }
}
