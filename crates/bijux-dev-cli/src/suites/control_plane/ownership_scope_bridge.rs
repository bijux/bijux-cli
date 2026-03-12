#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DEV-CLI-RESILIENCE-REPORTS" => {
            let run_cmd =
                |args: &[&str], envs: &[(&str, String)]| -> Result<std::process::Output, String> {
                    let mut cmd = Command::new("cargo");
                    cmd.args(["run", "-q", "-p", "bijux-cli", "--bin", "bijux", "--"])
                        .args(args)
                        .current_dir(workspace_root);
                    for (k, v) in envs {
                        cmd.env(k, v);
                    }
                    cmd.output().map_err(|error| {
                        format!("failed to execute cargo run for resilience report: {error}")
                    })
                };
            let summary_commands: Vec<Vec<&str>> = vec![
                vec!["dev", "cli", "status"],
                vec!["dev", "cli", "dashboard"],
                vec!["dev", "cli", "truth"],
                vec!["dev", "cli", "blockers"],
                vec!["dev", "cli", "next"],
            ];
            let machine_commands: Vec<Vec<&str>> = vec![
                vec!["dev", "cli", "parity"],
                vec!["dev", "cli", "evidence", "audit"],
                vec!["dev", "cli", "routes"],
                vec!["dev", "cli", "registry"],
                vec!["dev", "cli", "env"],
                vec!["dev", "cli", "contracts"],
                vec!["dev", "cli", "state-audit"],
                vec!["dev", "cli", "state-doctor"],
                vec!["dev", "cli", "runtime-identity"],
                vec!["dev", "cli", "package-health"],
            ];
            let mut determinism_rows = Vec::<Value>::new();
            for command in summary_commands.iter().chain(machine_commands.iter()) {
                let mut first = command.clone();
                first.extend(["--format", "json", "--no-pretty"]);
                let mut second = command.clone();
                second.extend(["--format", "json", "--no-pretty"]);
                let a = run_cmd(&first, &[]);
                let b = run_cmd(&second, &[]);
                let stable = matches!((&a, &b), (Ok(left), Ok(right))
                    if left.status.code() == right.status.code() && left.stdout == right.stdout);
                determinism_rows.push(json!({
                    "command": command.join(" "),
                    "stable": stable,
                    "first_exit": a.as_ref().ok().and_then(|out| out.status.code()),
                    "second_exit": b.as_ref().ok().and_then(|out| out.status.code()),
                    "first_error": a.as_ref().err(),
                    "second_error": b.as_ref().err(),
                }));
            }
            let tmp = std::env::temp_dir()
                .join(format!("bijux-dev-cli-side-effects-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            let _ = fs::create_dir_all(tmp.join("plugins"));
            let config = tmp.join("config.env");
            let history = tmp.join("history.json");
            let memory = tmp.join("memory.json");
            let plugins = tmp.join("plugins");
            let _ = fs::write(&config, "BIJUXCLI_SAMPLE=1\n");
            let _ = fs::write(&history, "[]");
            let _ = fs::write(&memory, "{}");
            let _ = fs::write(
                plugins.join("healthy.toml"),
                "[plugin]\nname='healthy'\nentry='plugin:main'\n",
            );
            let digest = |p: &Path| -> String {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let data = fs::read(p).unwrap_or_default();
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                format!("{:016x}", hasher.finish())
            };
            let before = json!({"config":digest(&config),"history":digest(&history),"memory":digest(&memory)});
            let envs = vec![
                ("BIJUX_CONFIG_PATH", config.display().to_string()),
                ("BIJUX_HISTORY_PATH", history.display().to_string()),
                ("BIJUX_MEMORY_PATH", memory.display().to_string()),
                ("BIJUX_PLUGINS_DIR", plugins.display().to_string()),
            ];
            let mut side_effect_run_errors = Vec::<String>::new();
            for command in summary_commands.iter().chain(machine_commands.iter()) {
                if let Err(error) = run_cmd(command, &envs) {
                    side_effect_run_errors.push(format!("{}: {error}", command.join(" ")));
                }
            }
            let after = json!({"config":digest(&config),"history":digest(&history),"memory":digest(&memory)});
            let _ = fs::remove_dir_all(&tmp);
            let failure_cases: Vec<(&str, Vec<&str>, Vec<(&str, String)>)> = vec![
                (
                    "status_unreadable_input",
                    vec!["dev", "cli", "status"],
                    vec![("BIJUX_HISTORY_PATH", "/root/forbidden/history.json".to_string())],
                ),
                (
                    "parity_corrupted_input",
                    vec!["dev", "cli", "parity"],
                    vec![("BIJUX_MEMORY_PATH", "/dev/null/not-json".to_string())],
                ),
                (
                    "contracts_missing_snapshot_context",
                    vec!["dev", "cli", "contracts"],
                    vec![("PWD", "/definitely/missing/contracts/root".to_string())],
                ),
                (
                    "runtime_identity_path_ambiguity",
                    vec!["dev", "cli", "runtime-identity"],
                    vec![(
                        "PATH",
                        format!(
                            "/tmp/bijux-a:/tmp/bijux-b:{}",
                            std::env::var("PATH").unwrap_or_default()
                        ),
                    )],
                ),
                (
                    "package_health_metadata_mismatch",
                    vec!["dev", "cli", "package-health"],
                    vec![
                        ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
                        ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string()),
                    ],
                ),
            ];
            let mut failure_rows = Vec::<Value>::new();
            for (case_id, command, env) in &failure_cases {
                let mut args = command.clone();
                args.extend(["--format", "json", "--no-pretty"]);
                match run_cmd(&args, env) {
                    Ok(out) => {
                        let payload = serde_json::from_slice::<Value>(&out.stdout)
                            .unwrap_or_else(|_| json!({}));
                        failure_rows.push(json!({
                            "case_id": case_id,
                            "command": command.join(" "),
                            "exit_code": out.status.code().unwrap_or(1),
                            "json_object": payload.is_object(),
                        }));
                    }
                    Err(error) => {
                        failure_rows.push(json!({
                            "case_id": case_id,
                            "command": command.join(" "),
                            "exit_code": Value::Null,
                            "json_object": false,
                            "spawn_error": error,
                        }));
                    }
                }
            }
            let summary_set: BTreeSet<String> =
                summary_commands.iter().map(|c| c.join(" ")).collect();
            let machine_set: BTreeSet<String> =
                machine_commands.iter().map(|c| c.join(" ")).collect();
            let checks = json!({
                "failure_injection_cases_reported": failure_rows.len() == failure_cases.len(),
                "determinism_rows_present": determinism_rows.len() == summary_commands.len()+machine_commands.len(),
                "summary_commands_deterministic": determinism_rows.iter().filter(|r| summary_set.contains(r.get("command").and_then(Value::as_str).unwrap_or(""))).all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)),
                "machine_commands_deterministic": determinism_rows.iter().filter(|r| machine_set.contains(r.get("command").and_then(Value::as_str).unwrap_or(""))).all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)),
                "read_only_commands_did_not_mutate_state": before == after,
                "side_effect_runs_executed": side_effect_run_errors.is_empty(),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_control_plane_resilience_artifact.json", &json!({
                                "scope":"dev cli control-plane resilience","generator":"bijux-dev-cli","failure_injection_cases":failure_rows,"checks":checks,"side_effect_run_errors":side_effect_run_errors,
                                "status": if drift_checks.is_empty() {"complete"} else {"partial"}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_determinism_artifact.json", &json!({
                                "scope":"dev cli determinism","generator":"bijux-dev-cli","rows":determinism_rows,
                                "status": if determinism_rows.iter().all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)) {"clean"} else {"drift"}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_side_effect_audit_artifact.json", &json!({
                                "scope":"dev cli side-effect audit","generator":"bijux-dev-cli","before":before,"after":after,
                                "status": if before == after {"clean"} else {"drift"}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_resilience_drift_artifact.json", &json!({
                                "scope":"dev cli resilience drift","generator":"bijux-dev-cli","drift_checks":drift_checks,"drift_count":drift_checks.len(),
                                "status": if drift_checks.is_empty() {"clean"} else {"drift"}
                            })).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_control_plane_resilience_artifact.json",
                "artifacts/status/dev_cli_determinism_artifact.json",
                "artifacts/status/dev_cli_side_effect_audit_artifact.json",
                "artifacts/status/dev_cli_resilience_drift_artifact.json"
            ]}))
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT" => {
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let runtime_leakage = read_json("artifacts/status/runtime_dev_leakage_report.json");
            let interface_bridge =
                read_json("artifacts/status/dev_cli_interface_bridge_report.json");
            let dispatch = read_json("artifacts/status/dev_cli_dispatch_ownership_report.json");
            let mut violations = Vec::<String>::new();
            if runtime_leakage.get("status").and_then(Value::as_str) != Some("ok") {
                violations.push("runtime leakage report is not green".to_string());
            }
            if interface_bridge.get("interfaces").and_then(Value::as_array).is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("contains_json_assembly").and_then(Value::as_bool) == Some(true)
                })
            }) {
                violations.push("query bridge still assembles presentation json".to_string());
            }
            if dispatch
                .get("checks")
                .and_then(|v| v.get("bin_has_direct_dispatch_match_arms"))
                .and_then(Value::as_bool)
                == Some(true)
            {
                violations.push("bin owns direct dispatch match arms".to_string());
            }
            let payload = json!({
                "scope":"runtime responsibility reassessment",
                "status": if violations.is_empty() {"ok"} else {"degraded"},
                "violations": violations,
                "decision": if violations.is_empty() {
                    "no remaining runtime responsibilities violate the current dev-cli control-plane standard"
                } else {
                    "runtime responsibilities still violate control-plane standard"
                }
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_responsibility_reassessment.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/runtime_responsibility_reassessment.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS" => {
            let bridge_duplicate = fs::read_to_string(
                workspace_root.join("artifacts/status/bridge_duplicate_law_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let duplicate_count = bridge_duplicate
                .get("summary")
                .and_then(|v| v.get("duplicate_rule_count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let bridge_source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-python/tests/bridge_bindings.rs"),
            )
            .unwrap_or_default();
            let cross_surface_source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs"),
            )
            .unwrap_or_default();
            let proof_tests = vec![
                                (
                                    "same_route_graph",
                                    vec![
                                        "binary_and_bridge_use_same_command_registry_contract",
                                        "route_registry_snapshots_match_across_binary_core_and_bridge",
                                    ],
                                ),
                                (
                                    "same_command_registry",
                                    vec!["binary_and_bridge_use_same_command_registry_contract"],
                                ),
                                (
                                    "same_output_envelope",
                                    vec!["binary_and_bridge_use_same_output_envelope_shape"],
                                ),
                                (
                                    "same_exit_mappings",
                                    vec!["binary_and_bridge_use_same_exit_mapping_for_unknown_route"],
                                ),
                                (
                                    "same_namespace_law",
                                    vec!["binary_and_bridge_use_same_namespace_rejection_logic"],
                                ),
                                (
                                    "same_config_precedence",
                                    vec!["execution_path_keeps_config_precedence_identical_between_binary_and_bridge"],
                                ),
                            ];
            let mut proof_map = serde_json::Map::new();
            for (key, names) in proof_tests {
                let present: Vec<String> = names
                    .iter()
                    .filter(|name| {
                        bridge_source.contains(&format!("fn {name}("))
                            || cross_surface_source.contains(&format!("fn {name}("))
                    })
                    .map(|name| (*name).to_string())
                    .collect();
                proof_map.insert(
                                    key.to_string(),
                                    json!({"required": names, "present": present, "ok": present.len()==names.len()}),
                                );
            }
            let all_proofs_ok = proof_map
                .values()
                .all(|item| item.get("ok").and_then(Value::as_bool) == Some(true));
            let wrapper_ok = duplicate_count == 0 && all_proofs_ok;
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "bridge wrapper-only closure",
                "duplicate_law": {
                    "duplicate_rule_count": duplicate_count,
                    "status": if duplicate_count == 0 { "clean" } else { "duplicates-found" }
                },
                "proof_tests": proof_map,
                "status": if wrapper_ok { "green" } else { "open" },
                "wrapper_only_frozen": wrapper_ok
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                &payload,
            )
            .ok()?;
            let mut lines = vec![
                "Bridge Wrapper-Only Closure Report".to_string(),
                format!(
                    "status: {}",
                    payload.get("status").and_then(Value::as_str).unwrap_or("open")
                ),
                format!(
                    "wrapper-only frozen: {}",
                    payload.get("wrapper_only_frozen").and_then(Value::as_bool).unwrap_or(false)
                ),
                format!("duplicate rule count: {duplicate_count}"),
            ];
            if let Some(obj) = payload.get("proof_tests").and_then(Value::as_object) {
                for (key, item) in obj {
                    lines.push(format!(
                        "- {key}: {}",
                        item.get("ok").and_then(Value::as_bool).unwrap_or(false)
                    ));
                }
            }
            fs::write(
                workspace_root.join("artifacts/status/bridge_wrapper_only_closure_report.txt"),
                lines.join("\n") + "\n",
            )
            .ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                "artifacts/status/bridge_wrapper_only_closure_report.txt"
            ]}))
        }
        _ => None,
    }
}
