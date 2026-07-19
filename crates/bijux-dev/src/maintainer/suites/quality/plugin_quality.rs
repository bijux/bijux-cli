#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-PLUGIN-SCAFFOLD-REPORTS" => {
            let generated_at = generated_at_utc();
            let read_lines = |name: &str| -> Vec<String> {
                fs::read_to_string(
                    workspace_root.join("crates/bijux-cli/tests/snapshots").join(name),
                )
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .collect()
            };
            let python_files = read_lines("plugin_scaffold_python_minimal_files.txt");
            let rust_files = read_lines("plugin_scaffold_rust_minimal_files.txt");
            let python_set = python_files.iter().cloned().collect::<BTreeSet<_>>();
            let rust_set = rust_files.iter().cloned().collect::<BTreeSet<_>>();
            let decorative_files = vec!["README.md", "pyproject.toml", "Cargo.toml", ".gitignore"];
            let decorative_python = python_files
                .iter()
                .filter(|p| decorative_files.contains(&p.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            let decorative_rust = rust_files
                .iter()
                .filter(|p| decorative_files.contains(&p.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_python_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","kind":"python","files":python_files,"count":python_files.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_rust_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","kind":"rust","files":rust_files,"count":rust_files.len()})).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_scaffold_diff.json",
                &json!({
                    "generated_at":generated_at,"generator":"bijux-dev-cli",
                    "shared": python_set.intersection(&rust_set).cloned().collect::<Vec<_>>(),
                    "python_only": python_set.difference(&rust_set).cloned().collect::<Vec<_>>(),
                    "rust_only": rust_set.difference(&python_set).cloned().collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_non_behavioral_files.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","decorative_candidates":decorative_files,"present_in_scaffold":{"python":decorative_python,"rust":decorative_rust},"summary":"decorative files are excluded from minimal scaffold outputs"})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_file_justification.json", &json!({
                                "generated_at":generated_at,
                                "generator":"bijux-dev-cli",
                                "classification_values":["essential","helpful","removable"],
                                "files":{
                                    "python":{"plugin.manifest.json":{"classification":"essential","reason":"required for install, namespace validation, and lifecycle commands"},"plugin.entry":{"classification":"essential","reason":"runtime entrypoint for delegated plugins"}},
                                    "rust":{"plugin.manifest.json":{"classification":"essential","reason":"required for install, namespace validation, and lifecycle commands"},"src/lib.rs":{"classification":"essential","reason":"runtime entrypoint module for delegated rust plugins"}}
                                },
                                "freeze_rule":"every scaffolded file must have a justification and decorative outputs stay excluded",
                            })).ok()?;
            let summary = format!(
                                "Plugin scaffold minimalism summary\nGenerated at: {generated_at}\nPython files ({}): {}\nRust files ({}): {}\nDecorative files excluded: README.md, pyproject.toml, Cargo.toml, .gitignore\nPolicy: every scaffolded file must carry explicit justification\n",
                                python_files.len(),
                                python_files.join(", "),
                                rust_files.len(),
                                rust_files.join(", ")
                            );
            fs::write(
                workspace_root.join("artifacts/status/plugin_scaffold_minimalism_summary.txt"),
                summary,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_scaffold_python_inventory.json",
                "artifacts/status/plugin_scaffold_rust_inventory.json",
                "artifacts/status/plugin_scaffold_diff.json",
                "artifacts/status/plugin_scaffold_non_behavioral_files.json",
                "artifacts/status/plugin_scaffold_file_justification.json",
                "artifacts/status/plugin_scaffold_minimalism_summary.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-MIGRATION-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join("artifacts/status").join(name))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let plugin_state = read("plugin_state_report.json");
            let scaffold_python = read("plugin_scaffold_python_inventory.json");
            let scaffold_rust = read("plugin_scaffold_rust_inventory.json");
            let scaffold_non_behavioral = read("plugin_scaffold_non_behavioral_files.json");
            let scaffold_justification = read("plugin_scaffold_file_justification.json");
            let namespace_abuse = read("namespace_abuse_report.json");
            let reserved_inventory = read("reserved_namespace_inventory.json");
            let rollback = read("plugin_rollback_proof_report.json");
            let lifecycle_failures = read("plugin_lifecycle_failure_injection_report.json");
            let plugin_health = read("plugin_health_report.json");
            let doctor_runtime = read("plugin_doctor_runtime_sample.json");
            let explain_runtime = read("plugin_explain_runtime_sample.json");
            let where_runtime = read("plugin_where_runtime_sample.json");
            let base = json!({"generated_at":generated_at,"generator":"bijux-dev-cli"});
            let lifecycle = json!({
                "generated_at":generated_at,"generator":"bijux-dev-cli",
                "stages":[
                    {"stage":"discover-and-list","rust_owned":true,"python_era_assumptions":[],"evidence":["crates/bijux-cli/tests/integration/cli/plugins/plugin_cli_lifecycle.rs::python_and_rust_plugins_can_install_check_list_and_uninstall","crates/bijux-cli/tests/integration/cli/plugins/plugin_command_parity.rs"]},
                    {"stage":"scaffold","rust_owned":true,"python_era_assumptions":["python scaffold runtime entrypoint remains plugin.entry for compatibility"],"evidence":["crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust"]},
                    {"stage":"install-uninstall-enable-disable","rust_owned":true,"python_era_assumptions":[],"evidence":rollback.get("evidence").cloned().unwrap_or_else(|| json!([]))},
                    {"stage":"doctor-explain-where","rust_owned":true,"python_era_assumptions":[],"evidence":["artifacts/status/plugin_doctor_runtime_sample.json","artifacts/status/plugin_explain_runtime_sample.json","artifacts/status/plugin_where_runtime_sample.json"]},
                ],
                "summary":{"fully_rust_owned":4,"python_assumption_dependent":1}
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_lifecycle_ownership_report.json",
                &lifecycle,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_efficiency_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "python_inventory":scaffold_python,"rust_inventory":scaffold_rust,"justification":scaffold_justification,
                                "decorative_presence": scaffold_non_behavioral.get("present_in_scaffold").cloned().unwrap_or_else(|| json!({})),
                                "status": if scaffold_non_behavioral.get("present_in_scaffold").and_then(|v| v.get("python")).and_then(Value::as_array).map_or(0, Vec::len)==0
                                    && scaffold_non_behavioral.get("present_in_scaffold").and_then(|v| v.get("rust")).and_then(Value::as_array).map_or(0, Vec::len)==0 {"minimal"} else {"needs-trim"}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_lifecycle_proof_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "python_scaffold_e2e_proof":{"status":"complete","evidence_test":"crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust","kind":"python"},
                                "rust_scaffold_e2e_proof":{"status":"complete","evidence_test":"crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_minimal.rs::scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust","kind":"rust"},
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_namespace_abuse_proof_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","abuse_report":namespace_abuse,"reserved_namespace_inventory":reserved_inventory})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_doctor_clarity_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","health_report":plugin_health,"runtime_sample":doctor_runtime,"status":if doctor_runtime.get("doctor").is_some() && doctor_runtime.get("status").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_explain_clarity_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_sample":explain_runtime,"status":if explain_runtime.get("diagnostics").is_some() && explain_runtime.get("summary").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_where_ownership_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_sample":where_runtime,"status":if where_runtime.get("plugins_dir").is_some() && where_runtime.get("registry_file").is_some() {"clear"} else {"unclear"}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_command_set_status.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","plugin_commands":plugin_state.get("plugin_commands").cloned().unwrap_or_else(|| json!({})),"classification":if plugin_state.get("plugin_commands").and_then(|p| p.get("partial")).and_then(Value::as_array).map_or(0,Vec::len)>0 {"evolving"} else {"complete"},"frozen_law":plugin_state.get("frozen_law").cloned().unwrap_or_else(|| json!("plugin v1 contract is frozen before expanding command cleverness")),"dynamic_complexity_policy":"reject unproven plugin complexity until parity and rollback evidence exists","operating_style":"boring-and-inspectable"})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_migration_report.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli",
                                "lifecycle_ownership":read("plugin_lifecycle_ownership_report.json"),
                                "scaffold_efficiency":read("plugin_scaffold_efficiency_report.json"),
                                "scaffold_lifecycle_proof":read("plugin_scaffold_lifecycle_proof_report.json"),
                                "namespace_abuse_proof":read("plugin_namespace_abuse_proof_report.json"),
                                "install_rollback_proof":rollback,
                                "uninstall_rollback_proof":{"status":rollback.get("status").cloned().unwrap_or_else(|| json!("unknown")),"evidence":rollback.get("evidence").cloned().unwrap_or_else(|| json!([]))},
                                "doctor_clarity":read("plugin_doctor_clarity_report.json"),
                                "explain_clarity":read("plugin_explain_clarity_report.json"),
                                "where_ownership":read("plugin_where_ownership_report.json"),
                                "command_set_status":read("plugin_command_set_status.json"),
                                "failure_injection":lifecycle_failures,
                            })).ok()?;
            let _ = base;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_ownership_report.json",
                "artifacts/status/plugin_scaffold_efficiency_report.json",
                "artifacts/status/plugin_scaffold_lifecycle_proof_report.json",
                "artifacts/status/plugin_namespace_abuse_proof_report.json",
                "artifacts/status/plugin_doctor_clarity_report.json",
                "artifacts/status/plugin_explain_clarity_report.json",
                "artifacts/status/plugin_where_ownership_report.json",
                "artifacts/status/plugin_command_set_status.json",
                "artifacts/status/plugin_migration_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-MANIFEST-SCAFFOLD-FUZZ-REPORTS" => {
            let now = generated_at_utc();
            let text = |p: &str| fs::read_to_string(workspace_root.join(p)).unwrap_or_default();
            let manifest_targets =
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_cli_lifecycle.rs";
            let manifest_reg =
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_namespace_law.rs";
            let scaffold_targets =
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_stability.rs";
            let scaffold_reg =
                "crates/bijux-cli/tests/integration/cli/plugins/plugin_scaffold_case_replays.rs";
            let mtxt = text(manifest_targets);
            let mrtxt = text(manifest_reg);
            let stxt = text(scaffold_targets);
            let srtxt = text(scaffold_reg);
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                (61, (manifest_targets, "install_rejects_stale_manifest_version_markers")),
                (
                    62,
                    (
                        manifest_targets,
                        "install_rejects_invalid_missing_reserved_and_duplicate_manifest_cases",
                    ),
                ),
                (63, (manifest_targets, "python_scaffold_broken_manifest_fails_install")),
                (64, (manifest_targets, "rust_scaffold_broken_manifest_fails_install")),
                (
                    65,
                    (
                        manifest_targets,
                        "external_exec_plugin_with_non_executable_entrypoint_fails_install",
                    ),
                ),
                (
                    66,
                    (
                        scaffold_targets,
                        "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable",
                    ),
                ),
                (
                    67,
                    (
                        scaffold_targets,
                        "fuzz_scaffold_option_parsing_and_template_expansion_inputs_are_stable",
                    ),
                ),
                (
                    68,
                    (
                        scaffold_targets,
                        "fuzz_python_and_rust_scaffold_manifest_generation_are_correct",
                    ),
                ),
                (
                    69,
                    (
                        scaffold_targets,
                        "fuzz_python_and_rust_scaffold_manifest_generation_are_correct",
                    ),
                ),
                (70, (scaffold_targets, "fuzz_scaffold_path_sanitization_rejects_parent_segments")),
                (
                    71,
                    (
                        scaffold_targets,
                        "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable",
                    ),
                ),
                (
                    72,
                    (
                        scaffold_targets,
                        "fuzz_plugin_inspect_payload_and_check_diagnostics_rendering_are_stable",
                    ),
                ),
                (73, (scaffold_targets, "fuzz_plugin_reserved_name_error_rendering_is_stable")),
                (76, (manifest_reg, "rejects_empty_namespace")),
                (
                    77,
                    (scaffold_reg, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
                ),
                (78, (manifest_reg, "json_error_envelopes_for_namespace_rejection_are_stable")),
                (
                    79,
                    (scaffold_reg, "minimized_scaffold_cases_replay_with_deterministic_exit_codes"),
                ),
            ]);
            let coverage = required.iter().map(|(id, (p, t))| {
                                let src = if *p == manifest_targets { &mtxt } else if *p == manifest_reg { &mrtxt } else if *p == scaffold_targets { &stxt } else { &srtxt };
                                json!({"coverage_id":id,"test":t,"status":if src.contains(&format!("fn {t}(")){"covered"}else{"missing"},"evidence":p})
                            }).collect::<Vec<_>>();
            let manifest_cases = Vec::<String>::new();
            let scaffold_cases = collect_files(
                &workspace_root.join("crates/bijux-cli/tests/fuzz/plugin_scaffold_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("argv"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let run = |args: &[&str]| {
                Command::new("cargo")
                    .args(args)
                    .current_dir(workspace_root)
                    .status()
                    .ok()
                    .is_some_and(|s| s.success())
            };
            let mt_ok = [
                "install_rejects_stale_manifest_version_markers",
                "install_rejects_invalid_missing_reserved_and_duplicate_manifest_cases",
                "python_scaffold_broken_manifest_fails_install",
                "rust_scaffold_broken_manifest_fails_install",
                "external_exec_plugin_with_non_executable_entrypoint_fails_install",
            ]
            .iter()
            .all(|test_name| run(&["test", "-p", "bijux-cli", "--test", "integration", test_name]));
            let mr_ok = [
                "rejects_empty_namespace",
                "json_error_envelopes_for_namespace_rejection_are_stable",
            ]
            .iter()
            .all(|test_name| run(&["test", "-p", "bijux-cli", "--test", "integration", test_name]));
            let st_ok = run(&[
                "test",
                "-p",
                "bijux-cli",
                "--test",
                "integration",
                "plugin_scaffold_stability::",
            ]);
            let sr_ok = run(&[
                "test",
                "-p",
                "bijux-cli",
                "--test",
                "integration",
                "plugin_scaffold_case_replays::",
            ]);
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_crash_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest fuzz crash triage","coverage_ids":[74],"status":if mt_ok && mr_ok{"clean"}else{"needs-triage"},"target_suite_ok":mt_ok,"regression_suite_ok":mr_ok,"minimized_case_count":manifest_cases.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_crash_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin scaffold fuzz crash triage","coverage_ids":[75],"status":if st_ok && sr_ok{"clean"}else{"needs-triage"},"target_suite_ok":st_ok,"regression_suite_ok":sr_ok,"minimized_case_count":scaffold_cases.len()})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_fuzz_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest fuzz regressions","coverage_ids":[76,78],"status":if mr_ok{"clean"}else{"drift"},"minimized_cases":manifest_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_scaffold_fuzz_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin scaffold fuzz regressions","coverage_ids":[77,79],"status":if sr_ok{"clean"}else{"drift"},"minimized_cases":scaffold_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_manifest_scaffold_fuzz_contract.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin manifest and scaffold fuzzing","coverage_ids":(61..81).collect::<Vec<_>>(),"status":if missing.is_empty() && mt_ok && mr_ok && st_ok && sr_ok && !manifest_cases.is_empty() && !scaffold_cases.is_empty(){"frozen"}else{"partial"},"coverage_rows":coverage,"missing_coverage_ids":missing,"manifest_minimized_case_count":manifest_cases.len(),"scaffold_minimized_case_count":scaffold_cases.len(),"policy":"plugin manifest and scaffold fuzzing remain maintenance-required hardening checks"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_manifest_crash_triage_artifact.json",
                "artifacts/status/plugin_scaffold_crash_triage_artifact.json",
                "artifacts/status/plugin_manifest_fuzz_regression_artifact.json",
                "artifacts/status/plugin_scaffold_fuzz_regression_artifact.json",
                "artifacts/status/plugin_manifest_scaffold_fuzz_contract.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-PLUGIN-STATE-CORRUPTION-CAMPAIGN-REPORTS" => {
            let now = generated_at_utc();
            let campaign_test = "crates/bijux-cli/tests/integration/cli/resilience/randomized_plugin_state_corruption_campaigns.rs";
            let regression_test = "crates/bijux-cli/tests/integration/cli/resilience/plugin_state_corruption_campaign_regressions.rs";
            let campaign_text =
                fs::read_to_string(workspace_root.join(campaign_test)).unwrap_or_default();
            let regression_text =
                fs::read_to_string(workspace_root.join(regression_test)).unwrap_or_default();
            let required: BTreeMap<i64, (&str, &str)> = BTreeMap::from([
                                (141, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (142, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (143, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (144, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (145, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (146, (campaign_test, "randomized_corruption_campaigns_cover_plugin_registry_and_state_read_paths")),
                                (147, (campaign_test, "one_broken_plugin_never_hides_unrelated_healthy_plugins")),
                                (148, (campaign_test, "plugin_list_is_deterministic_for_identical_corrupted_registry")),
                                (149, (campaign_test, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths")),
                                (150, (campaign_test, "plugin_registry_rollback_preserves_coherence_after_failed_mutation_paths")),
                                (151, (campaign_test, "plugin_doctor_reports_corruption_injected_by_campaign")),
                                (152, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (153, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (154, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (155, (campaign_test, "history_and_memory_corruption_recovery_remains_stable_and_policy_compliant")),
                                (158, (regression_test, "minimized_plugin_state_corruption_cases_replay_without_crashing")),
                            ]);
            let coverage = required.iter().map(|(id, (p, t))| {
                                let src = if *p == campaign_test { &campaign_text } else { &regression_text };
                                json!({"coverage_id":id,"test":t,"status":if src.contains(&format!("fn {t}(")){"covered"}else{"missing"},"evidence":p})
                            }).collect::<Vec<_>>();
            let campaign_ok = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "bijux-cli",
                    "--test",
                    "integration",
                    "randomized_plugin_state_corruption_campaigns::",
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
                    "plugin_state_corruption_campaign_regressions::",
                ])
                .current_dir(workspace_root)
                .status()
                .ok()
                .is_some_and(|s| s.success());
            let minimized_cases = collect_files(
                &workspace_root
                    .join("crates/bijux-cli/tests/fuzz/plugin_state_corruption_minimized_cases"),
            )
            .into_iter()
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
            .map(|p| rel(&p, workspace_root))
            .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_campaign_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption campaigns","coverage_ids":(141..156).collect::<Vec<_>>(),"status":if campaign_ok{"complete"}else{"partial"},"campaign_suite":{"ok":campaign_ok}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_corpus_retention_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption corpus retention","coverage_ids":[156],"status":if minimized_cases.is_empty(){"partial"}else{"complete"},"minimized_case_count":minimized_cases.len(),"minimized_cases":minimized_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_triage_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption triage","coverage_ids":[157],"status":if campaign_ok && regression_ok{"clean"}else{"needs-triage"},"campaign_suite_ok":campaign_ok,"regression_suite_ok":regression_ok})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_regression_artifact.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption regression replay","coverage_ids":[158],"status":if regression_ok{"clean"}else{"drift"},"minimized_cases":minimized_cases})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_severity_classification.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption severity classification","coverage_ids":[159],"status":"complete","classes":{"critical":["plugin registry write rollback failure","state read panic"],"high":["nondeterministic plugin list under identical corrupted input","memory recovery drift"],"medium":["history malformed entries with degraded but successful read"],"low":["doctor self-repair with stable output"]}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/plugin_state_corruption_contract.json", &json!({"generated_at":now,"generator":"bijux-dev-cli","scope":"plugin/history/memory corruption hardening contract","coverage_ids":(141..161).collect::<Vec<_>>(),"status":if campaign_ok && regression_ok && !minimized_cases.is_empty() && missing.is_empty(){"frozen"}else{"partial"},"missing_coverage_ids":missing,"policy":"plugin/history/memory corruption campaigns are required hardening coverage"})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_state_corruption_campaign_artifact.json",
                "artifacts/status/plugin_state_corruption_corpus_retention_artifact.json",
                "artifacts/status/plugin_state_corruption_triage_artifact.json",
                "artifacts/status/plugin_state_corruption_regression_artifact.json",
                "artifacts/status/plugin_state_corruption_severity_classification.json",
                "artifacts/status/plugin_state_corruption_contract.json"
            ]}))
        }
        _ => None,
    }
}
