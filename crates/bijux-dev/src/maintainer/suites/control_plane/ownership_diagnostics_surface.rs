#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MAINTAINER-STATE-DIAGNOSTICS-REPORTS" => {
            let read_json = |rel_path: &str| -> Value {
                fs::read_to_string(workspace_root.join(rel_path))
                    .ok()
                    .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let state_audit = read_json("artifacts/status/state_audit_report.json");
            let state_doctor = read_json("artifacts/status/state_doctor_report.json");
            let unified_corruption =
                read_json("artifacts/status/unified_state_corruption_report.json");
            let repeated_harness =
                read_json("artifacts/status/repeated_run_corruption_harness.json");
            let audit_checks = json!({
                "paths_present": state_audit.get("paths").is_some_and(Value::is_object),
                "corruption_health_present": state_audit.get("corruption_health").is_some_and(Value::is_object),
                "config_path_present": state_audit.get("paths").and_then(|v| v.get("config")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "plugin_registry_path_present": state_audit.get("paths").and_then(|v| v.get("plugins_registry")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "history_path_present": state_audit.get("paths").and_then(|v| v.get("history")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "memory_path_present": state_audit.get("paths").and_then(|v| v.get("memory")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
            });
            let doctor_checks = json!({
                "doctor_object_present": state_doctor.get("doctor").is_some_and(Value::is_object),
                "issues_list_present": state_doctor.get("doctor").and_then(|v| v.get("issues")).is_some_and(Value::is_array),
                "repairs_list_present": state_doctor.get("doctor").and_then(|v| v.get("repairs")).is_some_and(Value::is_array),
                "runtime_marker_present": state_doctor.get("runtime").is_some_and(Value::is_string),
            });
            let harness_results = repeated_harness
                .get("results")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let has_corrupt_config_probe = harness_results.iter().any(|row| {
                row.get("name").and_then(Value::as_str) == Some("state_doctor_json_corrupt_config")
            });
            let all_harness_stable = !harness_results.is_empty()
                && harness_results
                    .iter()
                    .all(|row| row.get("stable").and_then(Value::as_bool) == Some(true));
            let harness_checks = json!({
                "corrupt_config_probe_present": has_corrupt_config_probe,
                "harness_results_stable": all_harness_stable,
                "unified_corruption_report_present": !unified_corruption.as_object().is_some_and(|obj| obj.is_empty()),
            });
            let all_checks = [audit_checks.clone(), doctor_checks.clone(), harness_checks.clone()]
                .into_iter()
                .filter_map(|v| v.as_object().cloned())
                .fold(serde_json::Map::new(), |mut acc, map| {
                    acc.extend(map);
                    acc
                });
            let drift_checks: Vec<String> = all_checks
                .iter()
                .filter(|(_, v)| v.as_bool() != Some(true))
                .map(|(k, _)| k.to_string())
                .collect();
            write_status_artifact_json(workspace_root, "artifacts/status/state_audit_truth_artifact.json", &json!({
                                "scope": "state audit truth",
                                "generator": "bijux-dev-cli",
                                "checks": audit_checks,
                                "status": if audit_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/state_doctor_truth_artifact.json", &json!({
                                "scope": "state doctor truth",
                                "generator": "bijux-dev-cli",
                                "checks": doctor_checks,
                                "status": if doctor_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/corrupted_state_truth_artifact.json", &json!({
                                "scope": "corrupted state truth",
                                "generator": "bijux-dev-cli",
                                "checks": harness_checks,
                                "status": if harness_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
                            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_diagnostics_drift_artifact.json",
                &json!({
                    "scope": "state diagnostics drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/state_audit_truth_artifact.json",
                "artifacts/status/state_doctor_truth_artifact.json",
                "artifacts/status/corrupted_state_truth_artifact.json",
                "artifacts/status/state_diagnostics_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-BOUNDARY-REPORTS" => {
            let maintainer_fixture = workspace_root.join(
                "crates/bijux-dev/tests/maintainer/data/fixtures/routing/maintainer_subcommands.txt",
            );
            let runtime_dispatch =
                workspace_root.join("crates/bijux-cli/src/interface/cli/dispatch.rs");
            let route_files = [
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/root.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/docs.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/config.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/evidence.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/maintenance.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/python.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/release.rs"),
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/rustdoc.rs"),
            ];
            let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
            let runtime_dispatch_source = read(&runtime_dispatch);
            let maintainer_route_source =
                route_files.iter().map(|path| read(path)).collect::<Vec<_>>().join("\n");
            let commands: Vec<String> = read(&maintainer_fixture)
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("bijux-dev-cli "))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let maintainer_diag = BTreeSet::from([
                "bijux-dev-cli routes",
                "bijux-dev-cli route-audit",
                "bijux-dev-cli registry",
                "bijux-dev-cli parity",
                "bijux-dev-cli status",
                "bijux-dev-cli maintenance-audit",
                "bijux-dev-cli crate-health",
                "bijux-dev-cli package-health",
                "bijux-dev-cli env",
                "bijux-dev-cli doctor",
                "bijux-dev-cli contracts",
                "bijux-dev-cli runtime-identity",
                "bijux-dev-cli state-audit",
                "bijux-dev-cli state-doctor",
                "bijux-dev-cli docs-audit",
            ]);
            let runtime_query_consumers = BTreeSet::from([
                "bijux-dev-cli routes",
                "bijux-dev-cli route-audit",
                "bijux-dev-cli registry",
                "bijux-dev-cli env",
                "bijux-dev-cli doctor",
                "bijux-dev-cli contracts",
                "bijux-dev-cli runtime-identity",
                "bijux-dev-cli package-health",
                "bijux-dev-cli state-audit",
                "bijux-dev-cli state-doctor",
                "bijux-dev-cli plugin-health",
                "bijux-dev-cli list-products",
                "bijux-dev-cli list-plugins",
            ]);
            let route_markers = BTreeMap::from([
                ("bijux-dev-cli maintenance", "group == \"maintenance\""),
                ("bijux-dev-cli rustdoc", "group == \"rustdoc\""),
                ("bijux-dev-cli release", "group == \"release\""),
                ("bijux-dev-cli evidence", "group == \"evidence\""),
                ("bijux-dev-cli config", "group == \"config\""),
                ("bijux-dev-cli python", "group == \"python\""),
                ("bijux-dev-cli repo", "group == \"repo\""),
                ("bijux-dev-cli dashboard", "command == \"dashboard\""),
                ("bijux-dev-cli quickcheck", "command == \"quickcheck\""),
                ("bijux-dev-cli truth", "command == \"truth\""),
                ("bijux-dev-cli blockers", "command == \"blockers\""),
                ("bijux-dev-cli next", "command == \"next\""),
                ("bijux-dev-cli inventory", "command == \"inventory\""),
                ("bijux-dev-cli routes", "command == \"routes\""),
                ("bijux-dev-cli route-audit", "command == \"route-audit\""),
                ("bijux-dev-cli registry", "command == \"registry\""),
                ("bijux-dev-cli parity", "command == \"parity\""),
                ("bijux-dev-cli docs", "command == \"docs\""),
                (
                    "bijux-dev-cli docs publish-contract-assets",
                    "group == \"docs\" && command == \"publish-contract-assets\"",
                ),
                ("bijux-dev-cli docs-audit", "command == \"docs-audit\""),
                ("bijux-dev-cli plugin-health", "command == \"plugin-health\""),
                ("bijux-dev-cli status", "command == \"status\""),
                ("bijux-dev-cli maintenance-audit", "command == \"maintenance-audit\""),
                ("bijux-dev-cli snapshots-audit", "command == \"snapshots-audit\""),
                ("bijux-dev-cli fixture-audit", "command == \"fixture-audit\""),
                ("bijux-dev-cli crate-health", "command == \"crate-health\""),
                ("bijux-dev-cli package-health", "command == \"package-health\""),
                ("bijux-dev-cli env", "command == \"env\""),
                ("bijux-dev-cli doctor", "command == \"doctor\""),
                ("bijux-dev-cli contracts", "command == \"contracts\""),
                ("bijux-dev-cli runtime-identity", "command == \"runtime-identity\""),
                ("bijux-dev-cli docs-prune-plan", "command == \"docs-prune-plan\""),
                ("bijux-dev-cli state-audit", "command == \"state-audit\""),
                ("bijux-dev-cli state-doctor", "command == \"state-doctor\""),
                ("bijux-dev-cli atlas", "command == \"atlas\""),
                ("bijux-dev-cli di", "command == \"di\""),
                ("bijux-dev-cli list-products", "command == \"list-products\""),
                ("bijux-dev-cli list-plugins", "command == \"list-plugins\""),
            ]);
            let runtime_mentions_maintainer = runtime_dispatch_source.contains("bijux-dev-cli");
            let mut dev_rows = Vec::<Value>::new();
            let mut misplaced = Vec::<Value>::new();
            let mut missing_impl = Vec::<String>::new();
            for command in commands {
                let owner = if runtime_query_consumers.contains(command.as_str()) {
                    "bijux-dev-cli + runtime-data-providers".to_string()
                } else {
                    "bijux-dev-cli".to_string()
                };
                let implemented = route_markers
                    .get(command.as_str())
                    .is_some_and(|marker| maintainer_route_source.contains(marker));
                if !implemented {
                    missing_impl.push(command.clone());
                }
                let leaks = runtime_mentions_maintainer;
                let behavior_kind = if maintainer_diag.contains(command.as_str()) {
                    "diagnostic"
                } else {
                    "automation"
                };
                dev_rows.push(json!({
                    "command": command,
                    "behavior_kind": behavior_kind,
                    "intended_owner": "maintainer-control-plane",
                    "current_owner": owner,
                    "leaks_through_runtime": leaks,
                    "exposed_through_binary": true,
                    "evidence": [
                        "crates/bijux-dev/tests/maintainer/data/fixtures/routing/maintainer_subcommands.txt",
                        "crates/bijux-dev/src/maintainer/cli/routes",
                        "crates/bijux-cli/src/interface/cli/dispatch.rs"
                    ],
                }));
                if leaks {
                    misplaced.push(json!({
                        "behavior": command,
                        "expected_owner": "bijux-dev-cli",
                        "current_owner": owner,
                        "reason": "maintainer behavior still implemented in runtime crates",
                        "severity": "must-move",
                    }));
                }
            }
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_owned_behaviors_inventory.json", &json!({
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "scope": "maintainer-owned behavior inventory",
                                "commands": dev_rows,
                                "maintainer_only_commands_implemented_in_runtime_crates": dev_rows.iter().filter(|row| row.get("leaks_through_runtime").and_then(Value::as_bool)==Some(true)).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                                "maintainer_only_diagnostics_exposed_from_bin": maintainer_diag,
                                "maintenance_replacements_already_covered_by_control_plane": Value::Array(vec![]),
                                "remaining_maintenance_to_move_into_control_plane": Value::Array(vec![]),
                                "boundary_rules": {
                                    "control_plane_owner": "bijux-dev-cli owns maintainer automation and report assembly",
                                    "runtime_scope": "runtime crates own runtime law and structured-data services, not maintainer workflows",
                                    "canonical_surface": "bijux-dev-cli remains the canonical maintainer command surface",
                                    "distribution": "bijux-dev-cli is a workspace crate, not a second public binary package",
                                    "binary_identity": "bijux remains the only canonical executable",
                                    "law_center": "bijux-dev-cli does not become a second runtime law center"
                                },
                                "boundary_frozen": true,
                                "missing_implementation_mappings": missing_impl,
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/runtime_owned_behaviors_inventory.json", &json!({
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "scope": "runtime-owned behaviors",
                                "behaviors": [
                                    {"behavior":"command routing and normalization","owner":"bijux-cli","evidence":"crates/bijux-cli/src/routing/catalog.rs"},
                                    {"behavior":"runtime command execution kernel","owner":"bijux-cli","evidence":"crates/bijux-cli/src/interface/cli/dispatch.rs"},
                                    {"behavior":"config persistence and state law","owner":"bijux-cli","evidence":"crates/bijux-cli/src/features/config"},
                                    {"behavior":"plugin registry lifecycle","owner":"bijux-cli","evidence":"crates/bijux-cli/src/features/plugins"},
                                    {"behavior":"install and runtime identity primitives","owner":"bijux-cli","evidence":"crates/bijux-cli/src/features/install"},
                                    {"behavior":"output envelope and rendering","owner":"bijux-cli","evidence":"crates/bijux-cli/src/shared/output.rs"}
                                ],
                                "rules": {
                                    "runtime_crates_do_not_own_maintainer_workflows": true,
                                    "runtime_crates_expose_structured_data_only_for_maintainer_reports": true
                                }
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/misplaced_maintainer_behaviors_report.json", &json!({
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "scope": "misplaced maintainer behavior still implemented in runtime crates",
                                "misplaced_behaviors": misplaced,
                                "summary": {"total_maintainer_commands": dev_rows.len(), "misplaced_count": misplaced.len()},
                                "boundary_freeze": {"status":"frozen-before-extraction","rule":"boundary inventory must be generated and reviewed before moving implementation"},
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_ownership_report.json", &json!({
                                "generated_at": generated_at_utc(),
                                "generator": "bijux-dev-cli",
                                "scope": "maintainer inventory command ownership",
                                "maintainer_inventory_commands": [
                                    "bijux-dev-cli inventory","bijux-dev-cli maintenance-audit","bijux-dev-cli docs-audit","bijux-dev-cli crate-health",
                                    "bijux-dev-cli package-health","bijux-dev-cli runtime-identity","bijux-dev-cli state-audit","bijux-dev-cli state-doctor"
                                ],
                                "owned_by_maintainer_control_plane": dev_rows.iter().filter(|row| row.get("current_owner").and_then(Value::as_str).is_some_and(|s| s.starts_with("bijux-dev-cli"))).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                                "not_yet_owned_by_maintainer_control_plane": dev_rows.iter().filter(|row| row.get("current_owner").and_then(Value::as_str).is_none_or(|s| !s.starts_with("bijux-dev-cli"))).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/maintainer_owned_behaviors_inventory.json",
                "artifacts/status/runtime_owned_behaviors_inventory.json",
                "artifacts/status/misplaced_maintainer_behaviors_report.json",
                "artifacts/status/maintainer_command_ownership_report.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-COMMAND-SURFACE-REPORTS" => {
            let fixture = workspace_root.join(
                "crates/bijux-dev/tests/maintainer/data/fixtures/routing/maintainer_subcommands.txt",
            );
            let tests_root = workspace_root.join("crates/bijux-cli/tests");
            let test_sources: BTreeMap<String, String> = collect_files(&tests_root)
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
                .map(|p| (rel(&p, workspace_root), fs::read_to_string(p).unwrap_or_default()))
                .collect();
            let source = test_sources.values().cloned().collect::<Vec<_>>().join("\n");
            let commands: Vec<String> = fs::read_to_string(&fixture)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("bijux-dev-cli "))
                .map(ToString::to_string)
                .collect();
            let dev_values: BTreeMap<String, i64> = BTreeMap::from([
                ("bijux-dev-cli status".to_string(), 100),
                ("bijux-dev-cli routes".to_string(), 98),
                ("bijux-dev-cli registry".to_string(), 98),
                ("bijux-dev-cli env".to_string(), 96),
                ("bijux-dev-cli doctor".to_string(), 95),
                ("bijux-dev-cli contracts".to_string(), 93),
                ("bijux-dev-cli parity".to_string(), 91),
                ("bijux-dev-cli runtime-identity".to_string(), 90),
                ("bijux-dev-cli state-audit".to_string(), 90),
                ("bijux-dev-cli state-doctor".to_string(), 90),
            ]);
            let mut rows = Vec::<Value>::new();
            for command in commands {
                let parts = command.split(' ').collect::<Vec<_>>();
                let quoted =
                    parts.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
                let evidence_links: Vec<String> = test_sources
                    .iter()
                    .filter(|(_, src)| {
                        src.contains(&quoted) || src.contains(&format!("\"{command}\""))
                    })
                    .map(|(path, _)| path.to_string())
                    .collect();
                let status = if !evidence_links.is_empty()
                    || source.contains(&quoted)
                    || source.contains(&format!("\"{command}\""))
                {
                    "complete"
                } else {
                    "partial"
                };
                rows.push(json!({
                                    "command": command,
                                    "status": status,
                                    "status_model": ["complete","partial","shim","missing"],
                                    "evidence": evidence_links.first().cloned().unwrap_or_else(|| rel(&fixture, workspace_root)),
                                    "evidence_links": evidence_links,
                                    "maintainer_value": dev_values.get(&command).copied().unwrap_or(75),
                                }));
            }
            rows.sort_by(|l, r| {
                let lv = l.get("maintainer_value").and_then(Value::as_i64).unwrap_or(0);
                let rv = r.get("maintainer_value").and_then(Value::as_i64).unwrap_or(0);
                rv.cmp(&lv).then_with(|| {
                    l.get("command")
                        .and_then(Value::as_str)
                        .cmp(&r.get("command").and_then(Value::as_str))
                })
            });
            let req: BTreeMap<i64, &str> = BTreeMap::from([
                                (243,"parity_for_key_maintainer_commands_against_current_behavior"),
                                (250,"help_snapshots_exist_for_all_maintainer_subcommands"),
                                (251,"json_and_text_outputs_are_available_for_machine_and_text_heavy_maintainer_commands"),
                                (253,"stderr_stdout_and_exit_code_discipline_for_maintainer_commands"),
                                (255,"malformed_input_is_rejected_for_maintainer_subcommands"),
                                (256,"repeated_run_determinism_for_machine_readable_maintainer_commands"),
                                (257,"consistency_across_maintainer_routes_inspect_and_registry_state"),
                                (258,"consistency_across_maintainer_env_and_config_resolution_paths"),
                            ]);
            let coverage_checks = json!({
                "parity": source.contains("fn parity_for_key_maintainer_commands_against_current_behavior("),
                "contract_shape": source.contains("fn json_and_text_outputs_are_available_for_machine_and_text_heavy_maintainer_commands("),
                "help_snapshots": source.contains("fn help_snapshots_exist_for_all_maintainer_subcommands("),
                "stderr_stdout_exit_code": source.contains("fn stderr_stdout_and_exit_code_discipline_for_maintainer_commands("),
                "malformed_input": source.contains("fn malformed_input_is_rejected_for_maintainer_subcommands("),
                "determinism": source.contains("fn repeated_run_determinism_for_machine_readable_maintainer_commands("),
                "consistency_inspect_routes_registry": source.contains("fn consistency_across_maintainer_routes_inspect_and_registry_state("),
                "consistency_config_env_resolution": source.contains("fn consistency_across_maintainer_env_and_config_resolution_paths("),
                "consistency_plugin_registry_state": source.contains("fn consistency_across_maintainer_routes_inspect_and_registry_state("),
            });
            let all_required = coverage_checks
                .as_object()
                .is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true)));
            let summary = json!({
                "total": rows.len(),
                "complete": rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("complete")).count(),
                "partial": rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("partial")).count(),
                "shim": 0, "missing": 0
            });
            let remaining: Vec<Value> = rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("complete"))
                .cloned()
                .collect();
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_coverage_report.json", &json!({
                                "generated_at": generated_at_utc(), "generator":"bijux-dev-cli","scope":"bijux-dev-cli command coverage","commands":rows,"summary":summary
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_matrix_artifact.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"bijux-dev-cli command matrix",
                                "coverage_rows": req.into_iter().map(|(id,name)| {
                                    let evidence = test_sources.iter().find(|(_, src)| src.contains(&format!("fn {name}("))).map(|(path, _)| path.clone());
                                    json!({"coverage_id":id,"test":name,"status": if evidence.is_some() {"complete"} else {"missing"},"evidence":evidence})
                                }).collect::<Vec<_>>(),
                                "commands": rows
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_surface_domain_contract.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","domain":"maintainer-command-surface","status":"frozen",
                                "rule":"bijux-dev-cli commands are the maintainer control surface and must keep parity, diagnostics, and deterministic output law."
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_remaining_inventory.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"remaining bijux-dev-cli subcommands not proven complete in rust","remaining_commands":remaining,"count":remaining.len()
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_value_ranking.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"maintainer value ranking for closure execution","ranked_remaining_commands":remaining,"count":remaining.len()
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_completion_report.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"bijux-dev-cli command closure execution","remaining_count":remaining.len(),"coverage_checks":coverage_checks,
                                "closure_status": if remaining.is_empty() && all_required {"green"} else {"open"},
                                "top_targets": remaining.iter().take(2).cloned().collect::<Vec<_>>()
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_command_closure_set.json", &json!({
                                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"tracked bijux-dev-cli closure set","tracked_commands":rows.iter().filter_map(|r| r.get("command").cloned()).collect::<Vec<_>>(),
                                "coverage_checks":coverage_checks,"status":"frozen"
                            })).ok()?;
            let cli_completion = fs::read_to_string(
                workspace_root.join("artifacts/status/cli_command_completion_report.json"),
            )
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .unwrap_or_else(|| json!({}));
            let cli_remaining =
                cli_completion.get("remaining_count").and_then(Value::as_i64).unwrap_or(0);
            let cli_green =
                cli_completion.get("closure_status").and_then(Value::as_str) == Some("green");
            let dev_green = remaining.is_empty() && all_required;
            let combined = json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"cli and bijux-dev-cli command closure",
                "cli":{"remaining_count":cli_remaining,"closure_status":cli_completion.get("closure_status").cloned().unwrap_or_else(|| json!("open")),"top_targets":cli_completion.get("top_targets").cloned().unwrap_or_else(|| json!([]))},
                "maintainer":{"remaining_count":remaining.len(),"closure_status":if dev_green {"green"} else {"open"},"top_targets":remaining.iter().take(2).cloned().collect::<Vec<_>>()},
                "cross_command_consistency":{"inspect_routes_registry":coverage_checks["consistency_inspect_routes_registry"],"config_env_resolution":coverage_checks["consistency_config_env_resolution"],"plugin_registry_state":coverage_checks["consistency_plugin_registry_state"]},
                "closure_status": if cli_green && dev_green {"green"} else {"open"},
                "complete_language_allowed": cli_green && dev_green
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cli_maintainer_command_closure_report.json",
                &combined,
            )
            .ok()?;
            let txt = format!(
                                "CLI and Maintainer Command Closure Report\noverall: {}\ncomplete language allowed: {}\n\ncli remaining: {}\nbijux-dev-cli remaining: {}\n",
                                combined.get("closure_status").and_then(Value::as_str).unwrap_or("open"),
                                combined.get("complete_language_allowed").and_then(Value::as_bool).unwrap_or(false),
                                cli_remaining,
                                remaining.len()
                            );
            fs::write(
                workspace_root.join("artifacts/status/cli_maintainer_command_closure_report.txt"),
                txt,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/maintainer_command_coverage_report.json",
                "artifacts/status/maintainer_command_matrix_artifact.json",
                "artifacts/status/maintainer_command_surface_domain_contract.json",
                "artifacts/status/maintainer_command_remaining_inventory.json",
                "artifacts/status/maintainer_command_value_ranking.json",
                "artifacts/status/maintainer_command_completion_report.json",
                "artifacts/status/maintainer_command_closure_set.json",
                "artifacts/status/cli_maintainer_command_closure_report.json",
                "artifacts/status/cli_maintainer_command_closure_report.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-MAINTAINER-DISPATCH-OWNERSHIP-REPORTS" => {
            let main_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/bin/bijux.rs"))
                    .unwrap_or_default();
            let runtime_entry =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/bootstrap/run.rs"))
                    .unwrap_or_default();
            let runtime_dispatch = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/src/interface/cli/dispatch.rs"),
            )
            .unwrap_or_default();
            let parser_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/parser.rs"))
                    .unwrap_or_default();
            let registry_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/registry.rs"))
                    .unwrap_or_default();
            let maintainer_root = fs::read_to_string(
                workspace_root.join("crates/bijux-dev/src/maintainer/cli/routes/root.rs"),
            )
            .unwrap_or_default();
            let runtime_maintainer_literal_count =
                runtime_dispatch.matches("bijux-dev-cli").count();
            let maintainer_builder_call_count = [
                "dev_routes::build_report(",
                "dev_registry::build_report(",
                "dev_env::build_report(",
                "dev_contracts::build_report(",
                "dev_parity::build_report(",
                "dev_status::build_report(",
                "dev_maintenance_audit::build_inventory_report(",
                "dev_maintenance_audit::build_report(",
                "dev_docs_audit::build_report(",
                "dev_crate_health::build_report(",
                "dev_runtime_identity::build_report(",
                "dev_package_health::build_report(",
                "dev_state_audit::build_report(",
                "dev_state_audit::build_doctor_report(",
            ]
            .iter()
            .map(|token| maintainer_root.matches(token).count())
            .sum::<usize>();
            write_status_artifact_json(workspace_root, "artifacts/status/maintainer_dispatch_ownership_report.json", &json!({
                                "scope":"bijux-dev-cli dispatch ownership","status":"ok",
                                "dispatch_chain":[
                                    {"crate":"bijux-cli","role":"entrypoint-only","evidence":"src/bin/bijux.rs delegates to bijux_cli::api::runtime::run_cli_from_env"},
                                    {"crate":"bijux-cli","role":"runtime-only dispatch","evidence":"src/interface/cli/dispatch.rs routes runtime commands and delegates only known product tools"},
                                    {"crate":"bijux-dev-cli","role":"maintainer-workflow-implementation-owner","evidence":"src/cli/routes/root.rs assembles maintainer report payloads"}
                                ],
                                "checks":{
                                    "bin_mentions_maintainer_literals": main_rs.contains("bijux-dev-cli"),
                                    "runtime_entrypoint_calls_runtime_dispatch": runtime_entry.contains("run_app(&argv)"),
                                    "runtime_dispatch_is_free_of_maintainer_literals": runtime_maintainer_literal_count == 0,
                                    "runtime_dispatch_uses_known_product_delegation": runtime_dispatch.contains("try_delegate_known_bijux_tool"),
                                    "maintainer_builder_call_count": maintainer_builder_call_count
                                },
                                "rules":[
                                    "bin must remain entrypoint-only",
                                    "routing must remain command identity only",
                                    "maintainer workflows must be implemented in bijux-dev-cli"
                                ]
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/bin_entrypoint_responsibility_diff.json", &json!({
                                "scope":"bin responsibility diff","status":"ok",
                                "current":{
                                    "file":"crates/bijux-cli/src/bin/bijux.rs",
                                    "line_count": main_rs.lines().count(),
                                    "maintainer_literal_mentions": main_rs.matches("bijux-dev-cli").count(),
                                    "core_entrypoint_calls": main_rs.matches("run_cli_from_env").count(),
                                    "runtime_dispatch_mentions": runtime_dispatch.matches("run_app(").count(),
                                    "parser_dependency_mentions": main_rs.matches("bijux_cli::routing::parser").count()
                                },
                                "routing_identity_checks":{
                                    "parser_build_report_mentions": parser_rs.matches("build_report(").count(),
                                    "registry_build_report_mentions": registry_rs.matches("build_report(").count(),
                                    "parser_json_assembly_mentions": parser_rs.matches("serde_json::json!").count(),
                                    "registry_json_assembly_mentions": registry_rs.matches("serde_json::json!").count(),
                                    "runtime_dispatch_maintainer_literal_mentions": runtime_maintainer_literal_count
                                }
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/maintainer_dispatch_ownership_report.json",
                "artifacts/status/bin_entrypoint_responsibility_diff.json"
            ]}))
        }
        _ => None,
    }
}
