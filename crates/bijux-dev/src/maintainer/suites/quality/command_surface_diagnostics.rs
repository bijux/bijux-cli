#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-DEEP-BEHAVIOR-REPORTS" => {
            let mut sources = BTreeMap::<String, String>::new();
            for root in ["crates/bijux-cli/tests", "crates/bijux-dev/tests/maintainer"] {
                for path in collect_files(&workspace_root.join(root)) {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                        sources.insert(
                            rel(&path, workspace_root),
                            fs::read_to_string(path).unwrap_or_default(),
                        );
                    }
                }
            }
            let find_test = |name: &str| -> Option<String> {
                let needle = format!("fn {name}(");
                sources.iter().find(|(_, src)| src.contains(&needle)).map(|(p, _)| p.clone())
            };
            let run_json =
                |args: &[&str]| run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}));
            let doctor_a = run_json(&["doctor"]);
            let doctor_b = run_json(&["doctor"]);
            let state_doctor_a = run_json(&["state-doctor"]);
            let state_doctor_b = run_json(&["state-doctor"]);
            let inspect = run_json(&["inspect"]);
            let env = run_json(&["env"]);
            let contracts = run_json(&["contracts"]);
            let routes = run_json(&["routes"]);
            let registry = run_json(&["registry"]);
            let plugin_health = run_json(&["plugin-health"]);
            let package_health = run_json(&["package-health"]);
            let runtime_identity = run_json(&["runtime-identity"]);
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (141, "doctor_findings_are_stable_and_do_not_reorder_nondeterministically"),
                                (142, "doctor_findings_are_stable_and_do_not_reorder_nondeterministically"),
                                (143, "doctor_json_and_text_are_stable_with_no_color_mode"),
                                (144, "doctor_json_and_text_are_stable_with_no_color_mode"),
                                (145, "inspect_and_doctor_agree_on_route_state_overlap_signals"),
                                (146, "maintainer_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (147, "maintainer_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (148, "maintainer_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (149, "maintainer_env_contracts_routes_and_registry_match_current_snapshots_and_resolution"),
                                (150, "state_doctor_and_plugin_health_match_corruption_harness_findings"),
                                (151, "state_doctor_and_plugin_health_match_corruption_harness_findings"),
                                (152, "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions"),
                                (153, "package_health_and_runtime_identity_are_consistent_with_active_binary_conditions"),
                            ]);
            let coverage = required
                                .iter()
                                .map(|(id, name)| {
                                    let evidence = find_test(name);
                                    json!({"coverage_id":id,"test_name":name,"status":if evidence.is_some(){"covered"}else{"missing"},"evidence":evidence})
                                })
                                .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let expected_contracts = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-dev/tests/maintainer/data/golden/runtime_contracts.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let expected_routes = fs::read_to_string(
                workspace_root.join("crates/bijux-dev/tests/maintainer/data/golden/runtime_routes.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let current_contracts = json!({
                "contracts": contracts.get("contracts").cloned().unwrap_or(Value::Null),
                "schema_version": contracts.get("schema_version").cloned().unwrap_or(Value::Null),
            });
            let route_set = |value: &Value| -> BTreeSet<String> {
                value
                    .get("routes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|row| row.get("segments").and_then(Value::as_array).cloned())
                    .map(|segments| {
                        segments
                            .into_iter()
                            .filter_map(|s| s.as_str().map(ToString::to_string))
                            .collect::<Vec<_>>()
                            .join(" ")
                    })
                    .collect()
            };
            let expected_route_set = route_set(&expected_routes);
            let current_route_set = route_set(&routes);
            let diagnostics_consistency = json!({"generator":"bijux-dev-cli","scope":"diagnostics consistency","coverage_ids":[145,146,149,150,151,152,154],"status":if inspect.is_object()&&doctor_a.is_object()&&env.is_object()&&routes.is_object()&&registry.is_object()&&package_health.is_object()&&runtime_identity.is_object(){"complete"}else{"partial"},"sample":{"inspect_status":inspect.get("status"),"doctor_status":doctor_a.get("status"),"env_keys":env.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}});
            let doctor_determinism = json!({"generator":"bijux-dev-cli","scope":"doctor determinism","coverage_ids":[141,142,143,144,155,158],"status":if doctor_a==doctor_b && state_doctor_a==state_doctor_b && state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues")){"complete"}else{"partial"},"byte_stable":doctor_a==doctor_b && state_doctor_a==state_doctor_b});
            let schema_drift = json!({"generator":"bijux-dev-cli","scope":"diagnostics schema drift","coverage_ids":[147,148,156],"status":if current_contracts==expected_contracts && expected_route_set.is_subset(&current_route_set){"complete"}else{"partial"},"contracts_matches_snapshot":current_contracts==expected_contracts,"routes_matches_snapshot":expected_route_set.is_subset(&current_route_set)});
            let source_of_truth = json!({"generator":"bijux-dev-cli","scope":"diagnostics source of truth","coverage_ids":[146,147,148,149,157],"status":if env.is_object()&&contracts.is_object()&&routes.is_object()&&registry.is_object(){"complete"}else{"partial"},"source_commands":["bijux-dev-cli env","bijux-dev-cli contracts","bijux-dev-cli routes","bijux-dev-cli registry"]});
            let findings_order = json!({"generator":"bijux-dev-cli","scope":"findings order","coverage_ids":[141,142,150,158],"status":if state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues")){"complete"}else{"partial"},"stable_order":state_doctor_a.get("doctor").and_then(|d| d.get("issues"))==state_doctor_b.get("doctor").and_then(|d| d.get("issues"))});
            let contract = json!({"generator":"bijux-dev-cli","scope":"diagnostics contract","coverage_ids":[143,144,145,152,153,159],"status":if doctor_a.is_object()&&plugin_health.is_object()&&package_health.is_object()&&runtime_identity.is_object(){"complete"}else{"partial"},"contract_keys":{"doctor":doctor_a.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"plugin_health":plugin_health.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"package_health":package_health.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),"runtime_identity":runtime_identity.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default()}});
            let mut drift = Vec::<Value>::new();
            for (name, payload) in [
                ("diagnostics_consistency_artifact.json", &diagnostics_consistency),
                ("doctor_determinism_artifact.json", &doctor_determinism),
                ("diagnostics_schema_drift_artifact.json", &schema_drift),
                ("diagnostics_source_of_truth_artifact.json", &source_of_truth),
                ("findings_order_artifact.json", &findings_order),
                ("diagnostics_contract_artifact.json", &contract),
            ] {
                if payload.get("status").and_then(Value::as_str) != Some("complete") {
                    drift.push(json!({"artifact":name,"reason":"status-not-complete"}));
                }
            }
            if !missing.is_empty() {
                drift.push(json!({"reason":"missing-coverage_id-coverage","coverage_ids":missing}));
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_consistency_artifact.json",
                &diagnostics_consistency,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/doctor_determinism_artifact.json",
                &doctor_determinism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_schema_drift_artifact.json",
                &schema_drift,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_source_of_truth_artifact.json",
                &source_of_truth,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/findings_order_artifact.json",
                &findings_order,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_contract_artifact.json",
                &contract,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/diagnostics_deep_behavior_drift_artifact.json", &json!({"generator":"bijux-dev-cli","scope":"diagnostics deep behavior drift","coverage_ids":[160],"status":if drift.is_empty(){"clean"}else{"drift-detected"},"drift_count":drift.len(),"drift_items":drift,"coverage_rows":coverage})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_consistency_artifact.json",
                "artifacts/status/doctor_determinism_artifact.json",
                "artifacts/status/diagnostics_schema_drift_artifact.json",
                "artifacts/status/diagnostics_source_of_truth_artifact.json",
                "artifacts/status/findings_order_artifact.json",
                "artifacts/status/diagnostics_contract_artifact.json",
                "artifacts/status/diagnostics_deep_behavior_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DIAGNOSTICS-TRUST-REPORTS" => {
            let mut sources = BTreeMap::<String, String>::new();
            for root in ["crates/bijux-cli/tests", "crates/bijux-dev/tests/maintainer"] {
                for path in collect_files(&workspace_root.join(root)) {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                        sources.insert(
                            rel(&path, workspace_root),
                            fs::read_to_string(path).unwrap_or_default(),
                        );
                    }
                }
            }
            let find_test = |name: &str| -> Option<String> {
                let needle = format!("fn {name}(");
                sources.iter().find(|(_, src)| src.contains(&needle)).map(|(p, _)| p.clone())
            };
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                (361, "maintainer_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable"),
                                (362, "maintainer_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable"),
                                (363, "maintainer_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (364, "maintainer_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (365, "maintainer_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (366, "maintainer_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (367, "maintainer_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth"),
                                (368, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (369, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (370, "doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases"),
                                (371, "diagnostics_do_not_invent_unsupported_remediation_steps"),
                                (372, "diagnostics_text_is_boring_and_json_is_machine_friendly"),
                                (373, "diagnostics_text_is_boring_and_json_is_machine_friendly"),
                                (374, "diagnostics_runs_are_deterministic_for_covered_commands"),
                            ]);
            let coverage = required
                .iter()
                .map(|(id, t)| {
                    let evidence = find_test(t);
                    json!({"coverage_id":id,"test":t,"status":if evidence.is_some(){"covered"}else{"missing"},"evidence":evidence})
                })
                .collect::<Vec<_>>();
            let missing = coverage
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let expected_keys: BTreeMap<&str, Vec<&str>> = BTreeMap::from([
                ("bijux-dev-cli contracts", vec!["contracts", "runtime_version", "schema_version"]),
                ("bijux-dev-cli routes", vec!["aliases", "routes"]),
                ("bijux-dev-cli registry", vec!["ownership", "precedence", "registry"]),
                ("bijux-dev-cli env", vec!["active", "env", "source_precedence"]),
                (
                    "bijux-dev-cli parity",
                    vec![
                        "binary_bridge",
                        "command_matrix",
                        "commands_fully_rust_owned",
                        "commands_python_only",
                        "commands_using_compatibility_shims",
                        "coverage",
                        "diffs",
                        "exit_code_report",
                        "flag_normalization_report",
                        "help_diff_report",
                        "machine_output_diff_report",
                        "parity_dashboard",
                        "parity_dashboard_text",
                        "plugin_lifecycle",
                        "plugin_matrix",
                        "precedence_report",
                        "python_bridge_matrix",
                        "repl_cli_output_diff",
                        "repl_matrix",
                        "rust_python",
                        "state_behavior_matrix",
                        "state_parity",
                        "stream_report",
                        "text_summary",
                    ],
                ),
                (
                    "bijux-dev-cli crate-health",
                    vec![
                        "crate_metrics",
                        "crate_report",
                        "cross_crate_api_usage",
                        "dependency_edges",
                        "duplication_hotspots",
                        "internal_only_candidates_by_crate",
                        "public_api_by_crate",
                        "public_api_counts",
                    ],
                ),
                ("bijux-dev-cli docs-audit", vec!["docs", "docs_audit", "docs_count"]),
                ("bijux-dev-cli doctor", vec!["issues", "runtime", "status"]),
                (
                    "bijux-dev-cli runtime-identity",
                    vec![
                        "active_binary",
                        "active_binary_selection_is_ambiguous",
                        "active_path_is_canonical_name",
                        "active_path_is_shadowed",
                        "canonical_user_binary",
                        "diagnostics",
                        "entrypoints",
                        "install_source",
                        "package_channels",
                        "path_binaries",
                        "public_runtime_binary_names",
                        "runtime",
                        "schema",
                        "secondary_public_runtime_binary_names",
                        "text_summary",
                    ],
                ),
            ]);
            let mut schema_rows = Vec::<Value>::new();
            for (command, expected) in &expected_keys {
                let parts = command.split_whitespace().collect::<Vec<_>>();
                let payload = run_bijux_json(workspace_root, &parts).unwrap_or_else(|_| json!({}));
                let actual = payload
                    .as_object()
                    .map(|o| o.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                let mut sorted_actual = actual.clone();
                sorted_actual.sort();
                let mut sorted_expected =
                    expected.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
                sorted_expected.sort();
                schema_rows.push(json!({"command":command,"expected_keys":expected,"actual_keys":sorted_actual,"status":if sorted_actual==sorted_expected{"match"}else{"drift"}}));
            }
            let schema_drift = schema_rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("match"))
                .count();
            let plugin_health =
                run_bijux_json(workspace_root, &["plugin-health"]).unwrap_or_else(|_| json!({}));
            let trust = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust","coverage_ids":[361,362,363,364,365,366,367,374,375],"status":if missing.is_empty(){"complete"}else{"partial"},"coverage_rows":coverage});
            let actionable = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"actionable diagnostics","coverage_ids":[368,369,370,371,376],"status":if missing.is_empty(){"complete"}else{"partial"},"checks":{"plugin_health_has_guidance":serde_json::to_string(&plugin_health).unwrap_or_default().contains("Use `bijux-dev-cli plugin-health --format json`"),"doctor_payload_present":run_bijux_json(workspace_root,&["doctor"]).map(|v|v.is_object()).unwrap_or(false),"runtime_identity_payload_present":run_bijux_json(workspace_root,&["runtime-identity"]).map(|v|v.is_object()).unwrap_or(false)}});
            let minimalism = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics minimalism","coverage_ids":[372,373,377],"status":if missing.is_empty(){"complete"}else{"partial"},"json_commands_checked":expected_keys.keys().collect::<Vec<_>>(),"json_schema_drift_count":schema_drift});
            let schema = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust schema drift","coverage_ids":[378],"status":if schema_drift==0 && missing.is_empty(){"clean"}else{"drift"},"drift_count":schema_drift + missing.len(),"schema_rows":schema_rows,"missing_coverage_ids":missing});
            let contract = json!({"generated_at":generated_at_utc(),"generator":"bijux-dev-cli","scope":"diagnostics trust contract","coverage_ids":[380],"status":if schema_drift==0 && missing.is_empty(){"frozen"}else{"not-frozen"},"law":"diagnostics are credible operator output"});
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_artifact.json",
                &trust,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/actionable_diagnostics_artifact.json",
                &actionable,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_minimalism_artifact.json",
                &minimalism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_schema_drift_artifact.json",
                &schema,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_trust_contract.json",
                &contract,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_trust_artifact.json",
                "artifacts/status/actionable_diagnostics_artifact.json",
                "artifacts/status/diagnostics_minimalism_artifact.json",
                "artifacts/status/diagnostics_trust_schema_drift_artifact.json",
                "artifacts/status/diagnostics_trust_contract.json"
            ]}))
        }
        _ => None,
    }
}
