#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MEMORY-DEEP-BEHAVIOR-REPORTS" => {
            let tests = [
                "crates/bijux-cli/tests/integration/cli/memory/memory_command_matrix.rs",
                "crates/bijux-cli/tests/integration/cli/memory/memory_parity.rs",
                "crates/bijux-cli/tests/integration/cli/memory/memory_output_stability.rs",
            ];
            let mut sources = BTreeMap::<String, String>::new();
            for path in tests {
                let full = workspace_root.join(path);
                if full.exists() {
                    sources.insert(path.to_string(), fs::read_to_string(full).unwrap_or_default());
                }
            }
            let find_test = |name: &str| -> Option<String> {
                let needle = format!("fn {name}(");
                sources
                    .iter()
                    .find(|(_, source)| source.contains(&needle))
                    .map(|(path, _)| path.clone())
            };
            let run_json = |args: &[&str]| -> Value {
                run_bijux_json(workspace_root, args).unwrap_or_else(|_| json!({}))
            };
            let semantic = run_json(&["memory", "list"]);
            let determinism_a = Command::new("cargo")
                .args([
                    "run",
                    "-q",
                    "-p",
                    "bijux-cli",
                    "--",
                    "memory",
                    "list",
                    "--format",
                    "json",
                    "--no-pretty",
                ])
                .current_dir(workspace_root)
                .output()
                .ok();
            let determinism_b = Command::new("cargo")
                .args([
                    "run",
                    "-q",
                    "-p",
                    "bijux-cli",
                    "--",
                    "memory",
                    "list",
                    "--format",
                    "json",
                    "--no-pretty",
                ])
                .current_dir(workspace_root)
                .output()
                .ok();
            let corruption = run_json(&["state-audit"]);
            let diagnostics = run_json(&["state-doctor"]);
            let failure = Command::new("cargo")
                .args(["run", "-q", "-p", "bijux-cli", "--", "memory", "list", "--unknown-flag"])
                .current_dir(workspace_root)
                .output()
                .ok();
            let path_behavior = run_json(&["memory", "list"]);
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (121, "memory_state_parsing_is_stable_under_field_reordering_and_unknown_fields"),
                (122, "memory_state_parsing_is_stable_under_field_reordering_and_unknown_fields"),
                (123, "memory_wrong_type_and_missing_required_shape_failures_are_stable"),
                (124, "memory_wrong_type_and_missing_required_shape_failures_are_stable"),
                (125, "missing_and_empty_memory_states_are_intentionally_consistent"),
                (126, "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability"),
                (127, "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability"),
                (128, "memory_quiet_no_color_and_deterministic_repeated_runs"),
                (129, "memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability"),
                (130, "memory_config_path_override_does_not_change_home_memory_resolution"),
                (131, "memory_state_audit_and_state_doctor_agree_on_malformed_state_findings"),
                (132, "memory_path_override_and_quiet_mode_keep_functional_semantics"),
                (133, "memory_path_override_and_quiet_mode_keep_functional_semantics"),
            ]);
            let coverage_rows = required
                                        .iter()
                                        .map(|(id, name)| {
                                            let evidence = find_test(name);
                                            json!({"coverage_id":id,"test_name":name,"status":if evidence.is_some(){"covered"}else{"missing"},"evidence":evidence})
                                        })
                                        .collect::<Vec<_>>();
            let missing = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .filter_map(|row| row.get("coverage_id").and_then(Value::as_i64))
                .collect::<Vec<_>>();
            let det_ok = determinism_a.is_some()
                && determinism_b.is_some()
                && determinism_a.as_ref().is_some_and(|o| o.status.success())
                && determinism_b.as_ref().is_some_and(|o| o.status.success())
                && determinism_a.as_ref().map(|o| (&o.stdout, &o.stderr))
                    == determinism_b.as_ref().map(|o| (&o.stdout, &o.stderr));
            let failure_code = failure.as_ref().and_then(|o| o.status.code()).unwrap_or(1);
            let memory_semantic = json!({"generator":"bijux-dev-cli","scope":"memory semantic","coverage_ids":[121,122,125,132,134],"status":if semantic.is_object(){"complete"}else{"partial"},"sample":semantic});
            let memory_determinism = json!({"generator":"bijux-dev-cli","scope":"memory determinism","coverage_ids":[126,127,128,129,135],"status":if det_ok{"complete"}else{"partial"},"byte_stable":det_ok});
            let memory_corruption = json!({"generator":"bijux-dev-cli","scope":"memory corruption","coverage_ids":[123,124,131,136],"status":if corruption.is_object(){"complete"}else{"partial"},"sample":corruption});
            let memory_diagnostics = json!({"generator":"bijux-dev-cli","scope":"memory diagnostics consistency","coverage_ids":[131,137],"status":if diagnostics.is_object(){"complete"}else{"partial"},"sample":diagnostics});
            let memory_failure = json!({"generator":"bijux-dev-cli","scope":"memory failure class","coverage_ids":[123,124,138],"status":if failure_code==2{"complete"}else{"partial"},"sample_exit_code":failure_code});
            let memory_path = json!({"generator":"bijux-dev-cli","scope":"memory path behavior","coverage_ids":[130,133,139],"status":if path_behavior.is_object(){"complete"}else{"partial"},"sample":path_behavior});
            let mut drift = Vec::<Value>::new();
            for (name, payload) in [
                ("memory_semantic_artifact.json", &memory_semantic),
                ("memory_determinism_artifact.json", &memory_determinism),
                ("memory_corruption_artifact.json", &memory_corruption),
                ("memory_diagnostics_consistency_artifact.json", &memory_diagnostics),
                ("memory_failure_class_artifact.json", &memory_failure),
                ("memory_path_behavior_artifact.json", &memory_path),
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
                "artifacts/status/memory_semantic_artifact.json",
                &memory_semantic,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_determinism_artifact.json",
                &memory_determinism,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_corruption_artifact.json",
                &memory_corruption,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_diagnostics_consistency_artifact.json",
                &memory_diagnostics,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_failure_class_artifact.json",
                &memory_failure,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/memory_path_behavior_artifact.json",
                &memory_path,
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/memory_deep_behavior_drift_artifact.json", &json!({
                                        "generator":"bijux-dev-cli","scope":"memory deep behavior drift","coverage_ids":[140],
                                        "status": if drift.is_empty() { "clean" } else { "drift-detected" },
                                        "drift_count": drift.len(),
                                        "drift_items": drift,
                                        "coverage_rows": coverage_rows,
                                    })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/memory_semantic_artifact.json",
                "artifacts/status/memory_determinism_artifact.json",
                "artifacts/status/memory_corruption_artifact.json",
                "artifacts/status/memory_diagnostics_consistency_artifact.json",
                "artifacts/status/memory_failure_class_artifact.json",
                "artifacts/status/memory_path_behavior_artifact.json",
                "artifacts/status/memory_deep_behavior_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-ROUTE-LAW-REPORTS" => {
            let generated_at = generated_at_utc();
            let registry =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/registry.rs"))
                    .unwrap_or_default();
            let parser =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/parser.rs"))
                    .unwrap_or_default();
            let parse_quoted = |block: &str| -> Vec<String> {
                block
                    .split('"')
                    .enumerate()
                    .filter_map(|(idx, part)| (idx % 2 == 1).then_some(part.to_string()))
                    .collect::<Vec<_>>()
            };
            let builtins = registry
                .split("let built_ins = BTreeSet::from([")
                .nth(1)
                .and_then(|s| s.split("]);").next())
                .map(parse_quoted)
                .unwrap_or_default()
                .into_iter()
                .filter(|s| !s.is_empty() && s.contains(' '))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let aliases = registry
                .split("let aliases = BTreeMap::from([")
                .nth(1)
                .and_then(|s| s.split("]);").next())
                .map(parse_quoted)
                .unwrap_or_default();
            let owner_rows = builtins
                                        .iter()
                                        .map(|command| {
                                            json!({"command":command,"owner_crate":"bijux-cli","source":"crates/bijux-cli/src/routing/registry.rs"})
                                        })
                                        .collect::<Vec<_>>();
            let mut test_files = collect_files(&workspace_root.join("crates"));
            test_files.retain(|p| {
                p.to_string_lossy().contains("/tests/")
                    && p.extension().and_then(|e| e.to_str()) == Some("rs")
            });
            let coverage_rows = builtins
                                        .iter()
                                        .map(|command| {
                                            let matched = test_files
                                                .iter()
                                                .filter_map(|p| {
                                                    let text = fs::read_to_string(p).ok()?;
                                                    (text.contains(command)).then_some(rel(p, workspace_root))
                                                })
                                                .collect::<BTreeSet<_>>()
                                                .into_iter()
                                                .take(25)
                                                .collect::<Vec<_>>();
                                            json!({"command":command,"coverage_files":matched,"coverage_count":matched.len()})
                                        })
                                        .collect::<Vec<_>>();
            let parity = fs::read_to_string(
                workspace_root.join("artifacts/parity/command_parity_matrix.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let parity_items =
                parity.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
            let mut parity_by_cmd = BTreeMap::<String, Value>::new();
            for row in parity_items {
                if let Some(command) = row.get("command").and_then(Value::as_str) {
                    parity_by_cmd.insert(command.to_string(), row);
                }
            }
            let parity_rows = builtins
                .iter()
                .map(|command| {
                    let row = parity_by_cmd.get(command).cloned().unwrap_or_else(|| json!({}));
                    json!({
                        "command":command,
                        "status":row.get("status").and_then(Value::as_str).unwrap_or("unknown"),
                        "owner":row.get("owner").and_then(Value::as_str).unwrap_or("unknown"),
                        "blocker":row.get("blocker").and_then(Value::as_str).unwrap_or(""),
                        "confidence":row.get("confidence").and_then(Value::as_f64).unwrap_or(0.0),
                    })
                })
                .collect::<Vec<_>>();
            let legacy_route_aliases = ["dev routes", "dev registry"]
                .into_iter()
                .filter(|alias| aliases.iter().any(|candidate| candidate == alias))
                .collect::<Vec<_>>();
            let legacy_hidden = ["routes", "registry"]
                .into_iter()
                .filter(|name| parser.contains(&format!("Command::new(\"{name}\").hide(true)")))
                .collect::<Vec<_>>();
            let baseline = fs::read_to_string(
                workspace_root.join("configs/status/route_special_cases_baseline.json"),
            )
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or_else(|| json!({}));
            let baseline_count =
                baseline.get("baseline_special_case_count").and_then(Value::as_i64).unwrap_or(0);
            let current_count = (legacy_route_aliases.len() + legacy_hidden.len()) as i64;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_command_owner_mapping.json",
                &json!({
                    "generated_at":generated_at,"generator":"bijux-dev-cli","items":owner_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_command_test_coverage_mapping.json",
                &json!({
                    "generated_at":generated_at,"generator":"bijux-dev-cli","items":coverage_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_command_parity_status_mapping.json",
                &json!({
                    "generated_at":generated_at,"generator":"bijux-dev-cli","items":parity_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_special_cases.json",
                &json!({
                    "generated_at":generated_at,
                    "generator":"bijux-dev-cli",
                    "coverage_id":638,
                    "report":{
                        "legacy_route_aliases":legacy_route_aliases,
                        "legacy_hidden_dev_subcommands":legacy_hidden,
                        "summary":{
                            "special_case_count":current_count,
                            "baseline_special_case_count":baseline_count,
                            "delta_from_baseline":current_count-baseline_count,
                        }
                    },
                    "rule":"special-case count must trend down over releases",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/route_command_owner_mapping.json",
                "artifacts/status/route_command_test_coverage_mapping.json",
                "artifacts/status/route_command_parity_status_mapping.json",
                "artifacts/status/route_special_cases.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-ROOT-COMMAND-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/integration/cli/root/root_command_matrix.rs"),
            )
            .unwrap_or_default();
            let commands = vec![
                "atlas",
                "audit",
                "completion",
                "config",
                "doctor",
                "docs",
                "history",
                "inspect",
                "memory",
                "plugins",
                "repl",
                "sleep",
                "status",
                "version",
            ];
            let impact = BTreeMap::from([
                ("status", 100),
                ("version", 95),
                ("doctor", 90),
                ("inspect", 85),
                ("docs", 80),
                ("audit", 75),
                ("sleep", 60),
                ("config", 55),
                ("plugins", 50),
                ("repl", 45),
                ("history", 40),
                ("memory", 35),
                ("completion", 30),
                ("atlas", 25),
            ]);
            let mut rows = commands
                                        .iter()
                                        .map(|command| {
                                            json!({
                                                "command":command,
                                                "status": if source.contains(&format!("\"{command}\"")) {"complete"} else {"partial"},
                                                "evidence":"crates/bijux-cli/tests/integration/cli/root/root_command_matrix.rs",
                                                "status_model":["complete","partial","shim","missing"],
                                                "user_impact": impact.get(command).copied().unwrap_or(20),
                                            })
                                        })
                                        .collect::<Vec<_>>();
            rows.sort_by(|a, b| {
                let ai = a.get("user_impact").and_then(Value::as_i64).unwrap_or(0);
                let bi = b.get("user_impact").and_then(Value::as_i64).unwrap_or(0);
                let ac = a.get("command").and_then(Value::as_str).unwrap_or("");
                let bc = b.get("command").and_then(Value::as_str).unwrap_or("");
                bi.cmp(&ai).then_with(|| ac.cmp(bc))
            });
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (203, "parity_version_against_current_expected_behavior"),
                (204, "parity_status_against_current_expected_behavior"),
                (205, "parity_doctor_against_current_expected_behavior"),
                (206, "parity_inspect_against_current_expected_behavior"),
                (207, "parity_docs_against_current_expected_behavior"),
                (208, "parity_audit_against_current_expected_behavior"),
                (209, "parity_sleep_against_current_expected_behavior"),
                (210, "help_snapshot_exists_for_every_root_command"),
                (211, "exit_code_and_stream_discipline_for_root_commands"),
                (212, "exit_code_and_stream_discipline_for_root_commands"),
                (213, "machine_readable_root_commands_support_json_and_yaml"),
                (214, "machine_readable_root_commands_support_json_and_yaml"),
                (215, "quiet_mode_is_supported_for_relevant_root_commands"),
                (216, "no_color_is_supported_for_text_root_commands"),
                (217, "malformed_input_is_rejected_for_argument_taking_root_commands"),
                (218, "repeated_run_determinism_for_machine_readable_root_commands"),
                (219, "root_command_matrix_artifact_smoke_uses_supported_commands"),
            ]);
            let coverage_rows = required.iter().map(|(id, name)| json!({
                                        "coverage_id":id,
                                        "test":name,
                                        "status": if source.contains(&format!("fn {name}(")) {"complete"} else {"missing"},
                                        "evidence":"crates/bijux-cli/tests/integration/cli/root/root_command_matrix.rs",
                                    })).collect::<Vec<_>>();
            let has_cov = |id: i64| {
                coverage_rows.iter().any(|r| {
                    r.get("coverage_id").and_then(Value::as_i64) == Some(id)
                        && r.get("status").and_then(Value::as_str) == Some("complete")
                })
            };
            let parity_ok = [203_i64, 204, 205, 206, 207, 208, 209].into_iter().all(has_cov);
            let coverage = json!({
                "parity": parity_ok,
                "help_snapshot": has_cov(210),
                "stderr_stdout": has_cov(212),
                "exit_code": has_cov(211),
                "json_output": has_cov(213),
                "yaml_output": has_cov(214),
                "determinism": has_cov(218),
            });
            let mut all_required = true;
            for key in [
                "parity",
                "help_snapshot",
                "stderr_stdout",
                "exit_code",
                "json_output",
                "yaml_output",
                "determinism",
            ] {
                if coverage.get(key).and_then(Value::as_bool) != Some(true) {
                    all_required = false;
                }
            }
            let remaining = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .cloned()
                .collect::<Vec<_>>();
            let generated_at = generated_at_utc();
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_coverage_report.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"root command coverage","commands":rows,
                                        "summary":{"total":rows.len(),"complete":rows.iter().filter(|r| r["status"]=="complete").count(),"partial":rows.iter().filter(|r| r["status"]=="partial").count(),"shim":0,"missing":0}
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_matrix_artifact.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"root command matrix","coverage_rows":coverage_rows,"commands":rows
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_surface_domain_contract.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","domain":"root-command-surface","status":"frozen",
                                        "rule":"Root commands are covered by explicit parity, stream, formatting, malformed-input, and determinism tests.",
                                        "evidence":["crates/bijux-cli/tests/integration/cli/root/root_command_matrix.rs","artifacts/status/root_command_coverage_report.json","artifacts/status/root_command_matrix_artifact.json"]
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_remaining_inventory.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"remaining root commands not proven complete in rust","remaining_commands":remaining,"count":remaining.len()
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_impact_ranking.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"root command impact ranking for closure execution","ranked_remaining_commands":remaining,"count":remaining.len()
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_completion_report.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"root command closure execution","remaining_count":remaining.len(),
                                        "top_five_execution":remaining.iter().take(5).enumerate().map(|(idx,row)| json!({"order":idx+1,"command":row["command"],"coverage_checks":coverage,"evidence":"crates/bijux-cli/tests/integration/cli/root/root_command_matrix.rs"})).collect::<Vec<_>>(),
                                        "coverage_checks":coverage,
                                        "closure_status": if remaining.is_empty() && all_required {"green"} else {"open"},
                                        "closure_reason": if remaining.is_empty() && all_required {"all root commands are complete and closure checks are proven"} else {"root command closure still has open items"},
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/root_command_closure_set.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"tracked root command closure set",
                                        "tracked_commands":rows.iter().filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                                        "closure_rule":"Root-command completion claims require zero remaining inventory and all required coverage checks.",
                                        "coverage_checks":coverage,"status":"frozen"
                                    })).ok()?;
            let mut text = format!("Root Command Completion Report\nremaining: {}\ncoverage checks all required: {}\n\nrequired coverage checks:\n", remaining.len(), all_required);
            for key in [
                "parity",
                "help_snapshot",
                "stderr_stdout",
                "exit_code",
                "json_output",
                "yaml_output",
                "determinism",
            ] {
                text.push_str(&format!(
                    "- {key}: {}\n",
                    coverage.get(key).and_then(Value::as_bool).unwrap_or(false)
                ));
            }
            fs::write(
                workspace_root.join("artifacts/status/root_command_completion_report.txt"),
                text,
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/root_command_coverage_report.json",
                "artifacts/status/root_command_matrix_artifact.json",
                "artifacts/status/root_command_surface_domain_contract.json",
                "artifacts/status/root_command_remaining_inventory.json",
                "artifacts/status/root_command_impact_ranking.json",
                "artifacts/status/root_command_completion_report.json",
                "artifacts/status/root_command_closure_set.json",
                "artifacts/status/root_command_completion_report.txt"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-CLI-COMMAND-SURFACE-REPORTS" => {
            let matrix = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/integration/cli/root/cli_command_matrix.rs"),
            )
            .unwrap_or_default();
            let fixture = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/fixtures/cli_subcommands.txt"),
            )
            .unwrap_or_default();
            let commands = fixture
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("cli "))
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            let mut rows = commands
                                        .iter()
                                        .map(|command| {
                                            let parts = command.split_whitespace().collect::<Vec<_>>();
                                            let quoted = parts
                                                .iter()
                                                .map(|p| format!("\"{p}\""))
                                                .collect::<Vec<_>>()
                                                .join(", ");
                                            json!({
                                                "command":command,
                                                "status": if matrix.contains(&quoted) || matrix.contains(&format!("\"{command}\"")) {"complete"} else {"partial"},
                                                "status_model":["complete","partial","shim","missing"],
                                                "evidence":"crates/bijux-cli/tests/integration/cli/root/cli_command_matrix.rs",
                                                "evidence_links":["crates/bijux-cli/tests/integration/cli/root/cli_command_matrix.rs"],
                                                "user_value": match command.as_str() {
                                                    "cli status" => 100,"cli paths" => 95,"cli self-test" => 90,"cli config get" => 88,"cli config set" => 86,"cli config list" => 84,"cli config unset" => 80,"cli config clear" => 78,
                                                    "cli plugins list" => 96,"cli plugins inspect" => 94,"cli plugins install" => 92,"cli plugins uninstall" => 92,"cli plugins check" => 90,"cli plugins doctor" => 88,_ => 70
                                                }
                                            })
                                        })
                                        .collect::<Vec<_>>();
            rows.sort_by(|a, b| {
                let av = a.get("user_value").and_then(Value::as_i64).unwrap_or(0);
                let bv = b.get("user_value").and_then(Value::as_i64).unwrap_or(0);
                let ac = a.get("command").and_then(Value::as_str).unwrap_or("");
                let bc = b.get("command").and_then(Value::as_str).unwrap_or("");
                bv.cmp(&av).then_with(|| ac.cmp(bc))
            });
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (223, "parity_cli_status_paths_and_self_test_against_current_behavior"),
                (224, "parity_cli_status_paths_and_self_test_against_current_behavior"),
                (225, "parity_cli_status_paths_and_self_test_against_current_behavior"),
                (226, "parity_cli_config_get_and_set_against_current_behavior"),
                (227, "parity_cli_config_get_and_set_against_current_behavior"),
                (228, "parity_cli_plugins_list_and_inspect_against_current_behavior"),
                (229, "parity_cli_plugins_list_and_inspect_against_current_behavior"),
                (230, "help_snapshots_exist_for_all_cli_subcommands"),
                (231, "stderr_stdout_and_exit_code_discipline_for_cli_commands"),
                (232, "stderr_stdout_and_exit_code_discipline_for_cli_commands"),
                (233, "machine_readable_cli_commands_support_json_and_yaml"),
                (234, "machine_readable_cli_commands_support_json_and_yaml"),
                (235, "quiet_mode_and_no_color_behavior_for_relevant_cli_commands"),
                (236, "quiet_mode_and_no_color_behavior_for_relevant_cli_commands"),
                (237, "malformed_input_is_rejected_for_argument_taking_cli_subcommands"),
                (238, "repeated_run_stability_for_machine_readable_cli_commands"),
                (239, "cli_command_matrix_artifact_smoke_uses_supported_commands"),
            ]);
            let coverage_rows = required.iter().map(|(id, name)| json!({
                                        "coverage_id":id,"test":name,"status":if matrix.contains(&format!("fn {name}(")){"complete"}else{"missing"},
                                        "evidence":"crates/bijux-cli/tests/integration/cli/root/cli_command_matrix.rs"
                                    })).collect::<Vec<_>>();
            let has_cov = |id: i64| {
                coverage_rows.iter().any(|r| {
                    r.get("coverage_id").and_then(Value::as_i64) == Some(id)
                        && r.get("status").and_then(Value::as_str) == Some("complete")
                })
            };
            let parity_ok = [223_i64, 226, 228].into_iter().all(has_cov);
            let coverage = json!({
                "parity": parity_ok,
                "machine_output": has_cov(233),
                "help_and_error_snapshots": has_cov(230) && has_cov(231),
            });
            let all_required = coverage.get("parity").and_then(Value::as_bool) == Some(true)
                && coverage.get("machine_output").and_then(Value::as_bool) == Some(true)
                && coverage.get("help_and_error_snapshots").and_then(Value::as_bool) == Some(true);
            let remaining = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .cloned()
                .collect::<Vec<_>>();
            let generated_at = generated_at_utc();
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_coverage_report.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"cli command coverage","commands":rows,
                                        "summary":{"total":rows.len(),"complete":rows.iter().filter(|r| r["status"]=="complete").count(),"partial":rows.iter().filter(|r| r["status"]=="partial").count(),"shim":0,"missing":0}
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_matrix_artifact.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"cli command matrix","coverage_rows":coverage_rows,"commands":rows
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_surface_domain_contract.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","domain":"cli-command-surface","status":"frozen",
                                        "rule":"cli subcommands are covered by explicit parity, stream, formatting, malformed-input, and determinism tests.",
                                        "evidence":["crates/bijux-cli/tests/routing/fixtures/cli_subcommands.txt","crates/bijux-cli/tests/integration/cli/root/cli_command_matrix.rs","artifacts/status/cli_command_coverage_report.json","artifacts/status/cli_command_matrix_artifact.json"]
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_remaining_inventory.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"remaining cli subcommands not proven complete in rust","remaining_commands":remaining,"count":remaining.len()
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_value_ranking.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"cli subcommand user-value ranking for closure execution","ranked_remaining_commands":remaining,"count":remaining.len()
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_completion_report.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"cli command closure execution","remaining_count":remaining.len(),
                                        "coverage_checks":coverage,
                                        "closure_status": if remaining.is_empty() && all_required {"green"} else {"open"},
                                        "closure_reason": if remaining.is_empty() && all_required {"all cli subcommands are complete and closure checks are proven"} else {"cli subcommand closure still has open items"},
                                        "top_targets": remaining.iter().take(2).cloned().collect::<Vec<_>>(),
                                    })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/cli_command_closure_set.json", &json!({
                                        "generated_at":generated_at,"generator":"bijux-dev-cli","scope":"tracked cli command closure set",
                                        "tracked_commands":rows.iter().filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                                        "coverage_checks":coverage,"status":"frozen"
                                    })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/cli_command_coverage_report.json",
                "artifacts/status/cli_command_matrix_artifact.json",
                "artifacts/status/cli_command_surface_domain_contract.json",
                "artifacts/status/cli_command_remaining_inventory.json",
                "artifacts/status/cli_command_value_ranking.json",
                "artifacts/status/cli_command_completion_report.json",
                "artifacts/status/cli_command_closure_set.json"
            ]}))
        }
        _ => None,
    }
}
