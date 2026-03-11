#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-COMPATIBILITY-SHIM-REPORTS" => {
            let generated_at = generated_at_utc();
            let baseline: Value = fs::read_to_string(
                workspace_root.join("configs/status/compatibility_baseline.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let status: Value =
                fs::read_to_string(workspace_root.join("artifacts/status/status.json"))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}));
            let registry =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/registry.rs"))
                    .unwrap_or_default();
            let mut alias_pairs = Vec::<(String, String)>::new();
            for line in registry.lines() {
                if line.contains(".to_string()") && line.contains("\", \"") {
                    let parts = line.split('"').collect::<Vec<_>>();
                    if parts.len() >= 4 {
                        alias_pairs.push((parts[1].to_string(), parts[3].to_string()));
                    }
                }
            }
            alias_pairs.sort();
            alias_pairs.dedup();
            let rows =
                status.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let shims = rows
                                        .iter()
                                        .filter(|row| row.get("status").and_then(Value::as_str) == Some("shim"))
                                        .map(|row| {
                                            let command = row.get("command").and_then(Value::as_str).unwrap_or("").to_string();
                                            let matrix_status = row.get("matrix_status").and_then(Value::as_str).unwrap_or("");
                                            let confidence = row.get("confidence").and_then(Value::as_f64).unwrap_or(0.0);
                                            let blocker = row.get("blocker").and_then(Value::as_str).unwrap_or("").to_string();
                                            if matrix_status == "complete" && confidence >= 0.9 {
                                                json!({"command":command,"classification":"delete-now","justification":"parity coverage is complete and confidence is high","removal_condition":"remove once canonical route regression tests remain green","evidence_links":["artifacts/parity/command_parity_matrix.json","artifacts/parity/command_parity_diffs.json"],"matrix_status":matrix_status,"confidence":confidence,"blocker":blocker})
                                            } else if !blocker.is_empty() {
                                                json!({"command":command,"classification":"needed","justification":format!("blocked by {blocker}"),"removal_condition":"remove after blocker closes and regression tests stay green","evidence_links":["artifacts/status/status_known_parity_gaps.json","artifacts/parity/command_parity_matrix.json"],"matrix_status":matrix_status,"confidence":confidence,"blocker":blocker})
                                            } else {
                                                json!({"command":command,"classification":"temporary","justification":"legacy entrypoint remains for current user-compatibility contract","removal_condition":"remove when parity matrix status for canonical path is rust-complete","evidence_links":["artifacts/status/command_migration_matrix.json","artifacts/parity/command_parity_matrix.json"],"matrix_status":matrix_status,"confidence":confidence,"blocker":blocker})
                                            }
                                        })
                                        .collect::<Vec<_>>();
            let aliases = alias_pairs
                                        .iter()
                                        .map(|(alias, canonical)| {
                                            if alias.starts_with("dev ") {
                                                json!({"alias":alias,"canonical":canonical,"classification":"temporary","justification":"legacy developer shortcut remains for compatibility contract","removal_condition":"remove when canonical dev cli path has stable parity coverage","evidence_links":["artifacts/status/command_migration_matrix.json","artifacts/parity/command_parity_matrix.json"]})
                                            } else if alias.starts_with("config ") || alias.starts_with("plugins ") {
                                                json!({"alias":alias,"canonical":canonical,"classification":"needed","justification":"legacy compatibility for core operator workflows","removal_condition":"remove when compatibility policy no longer requires shorthand","evidence_links":["artifacts/status/compatibility_alias_inventory.json","artifacts/status/status_known_parity_gaps.json"]})
                                            } else {
                                                json!({"alias":alias,"canonical":canonical,"classification":"temporary","justification":"legacy root shorthand remains for compatibility contract","removal_condition":"remove when canonical route adoption is complete and tested","evidence_links":["artifacts/status/command_migration_matrix.json","artifacts/parity/command_parity_matrix.json"]})
                                            }
                                        })
                                        .collect::<Vec<_>>();
            let hidden_aliases = aliases
                                        .iter()
                                        .filter(|item| item.get("alias").and_then(Value::as_str).is_some_and(|a| a.starts_with("dev ")))
                                        .map(|item| json!({"alias":item["alias"],"canonical":item["canonical"],"justification":item["justification"],"removal_condition":item["removal_condition"],"evidence_links":item["evidence_links"]}))
                                        .collect::<Vec<_>>();
            let old_python = aliases
                                        .iter()
                                        .filter(|item| item.get("alias").and_then(Value::as_str).is_some_and(|a| a.starts_with("config ") || a.starts_with("plugins ") || ["doctor","version","repl","completion","inspect"].iter().any(|k| a.starts_with(k))))
                                        .map(|item| json!({"legacy_path":item["alias"],"canonical":item["canonical"],"justification":item["justification"],"removal_condition":item["removal_condition"],"evidence_links":item["evidence_links"]}))
                                        .collect::<Vec<_>>();
            let before_shim =
                baseline.get("baseline_shim_count").and_then(Value::as_i64).unwrap_or(0);
            let before_alias =
                baseline.get("baseline_alias_count").and_then(Value::as_i64).unwrap_or(0);
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_shim_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","rule":"remaining shims require justification and removal plan","items":shims,"summary":{"count":shims.len(),"baseline_count":before_shim,"removed_since_baseline":before_shim - shims.len() as i64}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_alias_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","rule":"remaining aliases require justification and removal plan","items":aliases,"summary":{"count":aliases.len(),"baseline_count":before_alias,"removed_since_baseline":before_alias - aliases.len() as i64}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/hidden_alias_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","items":hidden_aliases,"summary":{"count":hidden_aliases.len()}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/old_python_path_tolerance_inventory.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","items":old_python,"summary":{"count":old_python.len()}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_shim_count_delta.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","before":before_shim,"after":shims.len(),"delta":shims.len() as i64 - before_shim})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_alias_count_delta.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","before":before_alias,"after":aliases.len(),"delta":aliases.len() as i64 - before_alias})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_shim_count_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","baseline_count":before_shim,"current_count":shims.len(),"removed_since_baseline":before_shim - shims.len() as i64})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/compatibility_alias_count_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","baseline_count":before_alias,"current_count":aliases.len(),"removed_since_baseline":before_alias - aliases.len() as i64})).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/live_compatibility_shims.json",
                &json!({"generated_at":generated_at,"items":shims}),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/live_compatibility_aliases.json",
                &json!({"generated_at":generated_at,"items":aliases}),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/compatibility_shim_inventory.json","artifacts/status/compatibility_alias_inventory.json","artifacts/status/hidden_alias_inventory.json","artifacts/status/old_python_path_tolerance_inventory.json","artifacts/status/compatibility_shim_count_delta.json","artifacts/status/compatibility_alias_count_delta.json","artifacts/status/compatibility_shim_count_report.json","artifacts/status/compatibility_alias_count_report.json","artifacts/status/live_compatibility_shims.json","artifacts/status/live_compatibility_aliases.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-METADATA-CONSISTENCY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/metadata_inspection_matrix.rs"),
            )
            .unwrap_or_default();
            let inspect =
                run_bijux_json(workspace_root, &["inspect"]).unwrap_or_else(|_| json!({}));
            let routes = run_bijux_json(workspace_root, &["dev", "cli", "routes"])
                .unwrap_or_else(|_| json!({}));
            let registry = run_bijux_json(workspace_root, &["dev", "cli", "registry"])
                .unwrap_or_else(|_| json!({}));
            let route_key = |row: &Value| -> String {
                row.get("segments")
                    .and_then(Value::as_array)
                    .map(|seg| seg.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" "))
                    .unwrap_or_default()
            };
            let inspect_route_set = inspect
                .get("route_sources")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(route_key)
                .collect::<BTreeSet<_>>();
            let dev_route_set = routes
                .get("routes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(route_key)
                .collect::<BTreeSet<_>>();
            let required_keys = vec![
                "status",
                "builtins",
                "route_sources",
                "reserved_namespaces",
                "plugin_origins",
                "alias_rewrites",
                "contracts",
            ];
            let missing_keys = required_keys
                .iter()
                .filter(|k| inspect.get(**k).is_none())
                .copied()
                .collect::<Vec<_>>();
            let reserved_inspect = inspect
                .get("reserved_namespaces")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter(|r| r.get("reserved").and_then(Value::as_bool) == Some(true))
                .filter_map(|r| r.get("name").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>();
            let reserved_registry = registry
                .get("registry")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter(|r| r.get("reserved").and_then(Value::as_bool) == Some(true))
                .filter_map(|r| r.get("name").and_then(Value::as_str))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                                        (61, "every_routable_command_has_inspectable_metadata_and_stable_route_identity"),(62, "every_routable_command_has_inspectable_metadata_and_stable_route_identity"),
                                        (63, "inspect_exposes_builtin_and_plugin_metadata_consistently"),(64, "inspect_exposes_builtin_and_plugin_metadata_consistently"),
                                        (65, "inspect_routes_and_registry_agree_on_namespace_ownership_and_plugin_source_metadata"),(66, "inspect_routes_and_registry_agree_on_namespace_ownership_and_plugin_source_metadata"),
                                        (67, "route_metadata_is_stable_and_json_serializable_for_covered_commands"),(68, "route_metadata_is_stable_and_json_serializable_for_covered_commands"),
                                        (69, "command_metadata_fields_do_not_disappear_or_rename_silently"),(70, "command_metadata_fields_do_not_disappear_or_rename_silently"),
                                        (71, "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical"),(72, "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical"),(73, "reserved_namespaces_and_alias_metadata_are_consistent_and_non_canonical"),
                                        (74, "help_output_and_inspect_metadata_agree_on_command_names_and_grouping"),(75, "help_output_and_inspect_metadata_agree_on_command_names_and_grouping"),
                                    ]);
            let coverage_rows = required.iter().map(|(id,name)| json!({"coverage_id":id,"test_name":name,"status":if source.contains(&format!("fn {name}(")){"covered"}else{"missing"},"evidence":"crates/bijux-cli/tests/bin_surface/metadata_inspection_matrix.rs"})).collect::<Vec<_>>();
            let missing_cov = coverage_rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|r| r.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let cmd_meta = json!({"generator":"bijux-dev-cli","scope":"command metadata consistency","coverage_ids":[61,63,64,68,69,70,71,72,73,74,75,76,80],"release_blocking":true,"required_keys":required_keys,"missing_keys":missing_keys,"status":if missing_keys.is_empty(){"complete"}else{"partial"}});
            let route_meta = json!({"generator":"bijux-dev-cli","scope":"route metadata consistency","coverage_ids":[62,65,67,77,79],"inspect_route_count":inspect_route_set.len(),"dev_route_count":dev_route_set.len(),"route_identity_match":inspect_route_set==dev_route_set,"status":if inspect_route_set==dev_route_set{"complete"}else{"partial"}});
            let ownership = json!({"generator":"bijux-dev-cli","scope":"command ownership","coverage_ids":[66,79],"registry_owners":registry.get("registry").and_then(Value::as_array).cloned().unwrap_or_default().iter().filter_map(|r| r.get("owner").and_then(Value::as_str)).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),"plugin_origin_owners":inspect.get("plugin_origins").and_then(Value::as_array).cloned().unwrap_or_default().iter().filter_map(|r| r.get("owner").and_then(Value::as_str)).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),"reserved_namespace_match":reserved_inspect==reserved_registry,"status":if reserved_inspect==reserved_registry{"complete"}else{"partial"}});
            let mut drift = Vec::<Value>::new();
            if !missing_keys.is_empty() {
                drift.push(json!({"kind":"missing-inspect-keys","keys":missing_keys}));
            }
            if inspect_route_set != dev_route_set {
                drift.push(json!({"kind":"route-identity-mismatch"}));
            }
            if reserved_inspect != reserved_registry {
                drift.push(json!({"kind":"reserved-namespace-mismatch"}));
            }
            if !missing_cov.is_empty() {
                drift.push(
                    json!({"kind":"missing-coverage_id-coverage","coverage_ids":missing_cov}),
                );
            }
            let drift_artifact = json!({"generator":"bijux-dev-cli","scope":"metadata drift","coverage_ids":[78,80],"status":if drift.is_empty(){"clean"}else{"drift-detected"},"drift_count":drift.len(),"drift_items":drift,"coverage_rows":coverage_rows});
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_metadata_artifact.json",
                &cmd_meta,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_metadata_artifact.json",
                &route_meta,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/metadata_drift_artifact.json",
                &drift_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_ownership_artifact.json",
                &ownership,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/command_metadata_artifact.json","artifacts/status/route_metadata_artifact.json","artifacts/status/metadata_drift_artifact.json","artifacts/status/command_ownership_artifact.json"
            ]}))
        }
        _ => None,
    }
}
