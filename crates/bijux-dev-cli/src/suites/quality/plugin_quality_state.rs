#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-STATE-AUDIT-REPORTS" => {
            let read = |path: &str| -> Value {
                fs::read_to_string(workspace_root.join(path))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let module_status_from_matrix = |matrix: &Value, prefixes: &[&str]| -> Value {
                let rows = matrix
                    .get("commands")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let matched = rows
                    .into_iter()
                    .filter(|row| {
                        let cmd = row.get("command").and_then(Value::as_str).unwrap_or("");
                        prefixes.iter().any(|prefix| cmd.starts_with(prefix))
                    })
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    return json!({"status":"still-changing","reason":"no command rows found","counts":{}});
                }
                let mut counts = BTreeMap::from([
                    ("rust-complete".to_string(), 0usize),
                    ("rust-partial".to_string(), 0usize),
                    ("python-only".to_string(), 0usize),
                    ("intentionally-different".to_string(), 0usize),
                ]);
                for row in &matched {
                    if let Some(status) = row.get("status").and_then(Value::as_str) {
                        if let Some(slot) = counts.get_mut(status) {
                            *slot += 1;
                        }
                    }
                }
                let status = if counts["python-only"] > 0 {
                    "still-changing"
                } else if counts["rust-partial"] > 0 {
                    "partial"
                } else {
                    "complete"
                };
                let reason = if status == "still-changing" {
                    "python-only commands remain"
                } else if status == "partial" {
                    "rust-partial commands remain"
                } else {
                    "all command rows are rust-complete or intentionally-different"
                };
                json!({"status":status,"reason":reason,"counts":counts,"total":matched.len()})
            };
            let migration = read("artifacts/status/command_migration_matrix.json");
            let state_behavior = read("artifacts/status/status_state_behavior_coverage.json");
            let state_paths = read("artifacts/status/status_state_paths_report.json");
            let state_corruption =
                read("artifacts/status/status_state_corruption_health_report.json");
            let state_audit = read("artifacts/status/state_audit_report.json");
            let state_doctor = read("artifacts/status/state_doctor_report.json");
            let state_write_guarantees = read("artifacts/status/state_write_guarantees.json");
            let state_recovery_guarantees = read("artifacts/status/state_recovery_guarantees.json");
            let state_inventory = read("artifacts/status/state_file_inventory.json");
            let parity_matrix = read("artifacts/parity/state_behavior_parity_matrix.json");
            let module_status = json!({
                "config": module_status_from_matrix(&migration, &["config", "cli config"]),
                "history": module_status_from_matrix(&migration, &["history", "cli history"]),
                "memory": module_status_from_matrix(&migration, &["memory", "cli memory"]),
                "plugin_registry_behavior": module_status_from_matrix(&migration, &["plugins", "cli plugins"]),
            });
            let base = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_migration_status.json",
                &json!({
                    "generated_at": base["generated_at"],
                    "generator": base["generator"],
                    "modules": module_status,
                    "source_matrix": "artifacts/status/command_migration_matrix.json",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/unified_state_behavior_report.json",
                &json!({
                    "generated_at": base["generated_at"],
                    "generator": base["generator"],
                    "module_status": module_status,
                    "state_behavior_coverage": state_behavior,
                    "state_behavior_parity_matrix": parity_matrix,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                        workspace_root,
                                        "artifacts/status/unified_state_corruption_report.json",
                                        &json!({
                                            "generated_at": base["generated_at"],
                                            "generator": base["generator"],
                                            "status_corruption_health": state_corruption,
                                            "runtime_state_audit": state_audit.get("corruption_health").cloned().unwrap_or_else(|| json!({})),
                                        }),
                                    )
                                    .ok()?;
            write_status_artifact_json(
                                        workspace_root,
                                        "artifacts/status/unified_state_rollback_report.json",
                                        &json!({
                                            "generated_at": base["generated_at"],
                                            "generator": base["generator"],
                                            "recovery_guarantees": state_recovery_guarantees,
                                            "write_guarantees": state_write_guarantees,
                                            "doctor_repairs": state_doctor.get("doctor").and_then(Value::as_object).and_then(|d| d.get("repairs")).cloned().unwrap_or_else(|| json!([])),
                                        }),
                                    )
                                    .ok()?;
            write_status_artifact_json(
                                        workspace_root,
                                        "artifacts/status/unified_state_path_resolution_report.json",
                                        &json!({
                                            "generated_at": base["generated_at"],
                                            "generator": base["generator"],
                                            "path_resolution": state_paths,
                                            "runtime_paths": state_audit.get("paths").cloned().unwrap_or_else(|| json!({})),
                                            "inventory": state_inventory.get("state_files").cloned().unwrap_or_else(|| json!([])),
                                        }),
                                    )
                                    .ok()?;
            let mut snapshots = Vec::<String>::new();
            for name in [
                "dev_cli_state_doctor_text.txt",
                "dev_cli_state_doctor_no_color.txt",
                "dev_cli_state_audit_text.txt",
                "dev_cli_state_audit_no_color.txt",
            ] {
                let p = workspace_root
                    .join("crates/bijux-cli/tests/snapshots")
                    .join(name);
                if p.exists() {
                    snapshots.push(format!("crates/bijux-cli/tests/snapshots/{name}"));
                }
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/unified_state_doctor_snapshots.json",
                &json!({
                    "generated_at": base["generated_at"],
                    "generator": base["generator"],
                    "snapshots": snapshots,
                    "runtime_reports": [
                        "artifacts/status/state_audit_report.json",
                        "artifacts/status/state_doctor_report.json",
                        "artifacts/status/state_doctor_report.txt",
                    ],
                }),
            )
            .ok()?;
            let payload = json!({
                "generated_at": base["generated_at"],
                "generator": base["generator"],
                "behavior_report": read("artifacts/status/unified_state_behavior_report.json"),
                "corruption_report": read("artifacts/status/unified_state_corruption_report.json"),
                "rollback_report": read("artifacts/status/unified_state_rollback_report.json"),
                "path_resolution_report": read("artifacts/status/unified_state_path_resolution_report.json"),
                "doctor_snapshots": read("artifacts/status/unified_state_doctor_snapshots.json"),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/unified_state_audit_payload.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                    "artifacts/status/state_migration_status.json",
                    "artifacts/status/unified_state_behavior_report.json",
                    "artifacts/status/unified_state_corruption_report.json",
                    "artifacts/status/unified_state_rollback_report.json",
                    "artifacts/status/unified_state_path_resolution_report.json",
                    "artifacts/status/unified_state_doctor_snapshots.json",
                    "artifacts/status/unified_state_audit_payload.json"
                ]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEEP-TEST-QUALITY-REPORTS" => {
            let test_root = workspace_root.join("crates/bijux-cli/tests/bin_surface");
            let mut rows = Vec::<(String, String, i64, i64)>::new();
            for path in collect_files(&test_root) {
                if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let rel_path = rel(&path, workspace_root);
                let text = fs::read_to_string(&path).unwrap_or_default();
                let lower = text.to_lowercase();
                let assert_count =
                    (text.matches("assert!(").count() + text.matches("assert_eq!(").count()) as i64;
                let score = assert_count
                    + if [
                        "failure",
                        "error",
                        "malformed",
                        "missing",
                        "invalid",
                        "usage",
                    ]
                    .iter()
                    .any(|k| lower.contains(k))
                    {
                        3
                    } else {
                        0
                    }
                    + if lower.contains("repeat") || lower.contains("determin") {
                        2
                    } else {
                        0
                    }
                    + if lower.contains("consisten")
                        || lower.contains("schema")
                        || lower.contains("shape")
                    {
                        2
                    } else {
                        0
                    }
                    + if lower.contains("corrupt") || lower.contains("rollback") {
                        2
                    } else {
                        0
                    };
                rows.push((rel_path, text, score, assert_count));
            }
            let domains: [(&str, fn(&str) -> bool); 5] = [
                ("commands", |rel| {
                    ["command", "root", "cli_", "ported", "help"]
                        .iter()
                        .any(|k| rel.contains(k))
                }),
                ("config", |rel| rel.contains("config")),
                ("history", |rel| rel.contains("history")),
                ("memory", |rel| rel.contains("memory")),
                ("diagnostics", |rel| {
                    [
                        "diagnostics",
                        "doctor",
                        "inspect",
                        "dev_cli_output_contracts",
                    ]
                    .iter()
                    .any(|k| rel.contains(k))
                }),
            ];
            let mut by_value = serde_json::Map::<String, Value>::new();
            let mut missing_cases = serde_json::Map::<String, Value>::new();
            let mut weak_replace = serde_json::Map::<String, Value>::new();
            for (domain, predicate) in domains {
                let mut tests = rows
                                            .iter()
                                            .filter(|(path, _, _, _)| predicate(&path.to_lowercase()))
                                            .map(|(path, text, score, assert_count)| {
                                                json!({"path": path, "text": text, "score": score, "assert_count": assert_count})
                                            })
                                            .collect::<Vec<_>>();
                tests.sort_by(|a, b| {
                    let ascore = a.get("score").and_then(Value::as_i64).unwrap_or(0);
                    let bscore = b.get("score").and_then(Value::as_i64).unwrap_or(0);
                    bscore.cmp(&ascore)
                });
                by_value.insert(
                                            domain.to_string(),
                                            json!({
                                                "count": tests.len(),
                                                "top_by_value": tests.iter().take(20).map(|t| json!({"path": t["path"], "value_score": t["score"]})).collect::<Vec<_>>()
                                            }),
                                        );
                let merged = tests
                    .iter()
                    .filter_map(|t| t.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n")
                    .to_lowercase();
                let reqs = match domain {
                    "commands" => vec![
                        "unknown command usage",
                        "deterministic repeated run",
                        "stderr stdout separation",
                    ],
                    "config" => vec![
                        "rollback on invalid mutation",
                        "corruption recovery",
                        "precedence consistency",
                    ],
                    "history" => vec![
                        "malformed interleaving resilience",
                        "deterministic ordering",
                        "state doctor consistency",
                    ],
                    "memory" => vec![
                        "wrong type field handling",
                        "missing state handling",
                        "corruption diagnostics consistency",
                    ],
                    _ => vec![
                        "findings order determinism",
                        "schema consistency",
                        "source of truth consistency",
                    ],
                };
                let cues = |name: &str| -> Vec<&str> {
                    match name {
                        "unknown command usage" => {
                            vec!["unknown-command", "unknown command", "usage"]
                        }
                        "deterministic repeated run" => vec!["repeat", "repeated", "determin"],
                        "stderr stdout separation" => vec!["stderr", "stdout"],
                        "rollback on invalid mutation" => vec!["rollback", "invalid"],
                        "corruption recovery" => vec!["corrupt", "malformed", "recovery"],
                        "precedence consistency" => vec!["precedence", "source_precedence"],
                        "malformed interleaving resilience" => {
                            vec!["malformed", "interleav", "resilience"]
                        }
                        "deterministic ordering" => vec!["ordering", "determin"],
                        "state doctor consistency" => vec!["state-doctor", "doctor"],
                        "wrong type field handling" => vec!["wrong-type", "wrong type"],
                        "missing state handling" => vec!["missing", "count"],
                        "corruption diagnostics consistency" => {
                            vec!["corrupt", "doctor", "consisten"]
                        }
                        "findings order determinism" => vec!["findings", "issues", "determin"],
                        "schema consistency" => vec!["schema", "shape", "contracts"],
                        _ => vec!["source", "routes", "registry", "env"],
                    }
                };
                let missing = reqs
                    .into_iter()
                    .filter(|item| !cues(item).iter().any(|cue| merged.contains(cue)))
                    .collect::<Vec<_>>();
                missing_cases.insert(domain.to_string(), json!(missing));
                let mut weakest = tests;
                weakest.sort_by(|a, b| {
                    let ascore = a.get("score").and_then(Value::as_i64).unwrap_or(0);
                    let bscore = b.get("score").and_then(Value::as_i64).unwrap_or(0);
                    ascore.cmp(&bscore)
                });
                weak_replace.insert(
                                            domain.to_string(),
                                            json!(weakest
                                                .iter()
                                                .take(8)
                                                .map(|t| json!({"path": t["path"], "value_score": t["score"], "replacement_goal": "add failure-path or determinism proof"}))
                                                .collect::<Vec<_>>()),
                                        );
            }
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/deep_tests_by_value_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "domains": by_value,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/deep_missing_behavior_cases_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "domains": missing_cases,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/deep_weak_tests_replacement_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "domains": weak_replace,
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/deep_test_first_domains_contract.json", &json!({
                                        "generated_at": generated_at,
                                        "generator": "bijux-dev-cli",
                                        "status": "frozen",
                                        "domains": ["commands","config","history","memory","diagnostics"],
                                        "rules": [
                                            "new command features require at least one deep failure-path or determinism test",
                                            "new diagnostics features require at least one consistency or shape test",
                                            "new stateful features require at least one corruption or rollback test",
                                        ],
                                    })).ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                    "artifacts/status/deep_tests_by_value_report.json",
                    "artifacts/status/deep_missing_behavior_cases_report.json",
                    "artifacts/status/deep_weak_tests_replacement_report.json",
                    "artifacts/status/deep_test_first_domains_contract.json"
                ]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-PERFORMANCE-REPORTS" => {
            let generated_at = generated_at_utc();
            let startup = vec![
                "version",
                "status",
                "doctor",
                "plugins list",
                "cli config get",
                "dev cli status",
                "plugins list (broken registry)",
                "plugins list (large registry)",
                "cli config get (large config)",
                "history (large history)",
            ];
            let memory = vec![
                "version payload-size",
                "status payload-size",
                "plugins list payload-size",
                "repl startup memory estimate",
            ];
            let rendering = vec![
                "output json render (large payload)",
                "output yaml render (large payload)",
            ];
            let thresholds = json!({
                "mode":"critical-path-only",
                "why":"guard user-visible regressions first; avoid vanity microbenchmarks",
                "startup_ms":{"version":120,"status":250,"doctor":500,"plugins list":400,"cli config get":200,"dev cli status":900,"plugins list (broken registry)":500,"plugins list (large registry)":900,"cli config get (large config)":650,"history (large history)":1200},
                "payload_bytes":{"version":4096,"status":24576,"plugins list":32768,"repl startup memory estimate":524288},
                "rendering_budget_ms":{"json_large_payload_total":3000,"yaml_large_payload_total":3000}
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/performance_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator":"bijux-dev-cli",
                    "scope":"performance realism",
                    "status":"complete",
                    "coverage_ids":[557],
                    "benchmark_sets":{"startup":startup,"memory":memory,"rendering":rendering},
                    "evidence_tests":[
                        "crates/bijux-cli/tests/bin_surface/performance_realism_hardening.rs",
                        "crates/bijux-cli-output/tests/output_rendering_performance.rs",
                        "crates/bijux-cli-repl/tests/repl_startup_performance_budget.rs"
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/performance_regression_budget.json",
                &json!({
                    "generated_at": generated_at,
                    "generator":"bijux-dev-cli",
                    "scope":"regression budgets",
                    "status":"complete",
                    "coverage_ids":[558,560],
                    "thresholds":thresholds,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/performance_benchmark_policy.json",
                &json!({
                    "generated_at": generated_at,
                    "generator":"bijux-dev-cli",
                    "scope":"benchmark policy",
                    "status":"complete",
                    "coverage_ids":[559],
                    "rules":[
                        "benchmark additions must target user-visible commands or rendering paths",
                        "regression thresholds apply to critical-path commands only",
                        "new microbenchmarks without user impact are rejected in CI",
                    ],
                }),
            )
            .ok()?;
            let mut text = String::from("Performance Report\n\ncritical_path_benchmarks:\n");
            for s in &startup {
                text.push_str(&format!("  - {s}\n"));
            }
            text.push_str("\nmemory_benchmarks:\n");
            for s in &memory {
                text.push_str(&format!("  - {s}\n"));
            }
            text.push_str("\nrendering_benchmarks:\n");
            for s in &rendering {
                text.push_str(&format!("  - {s}\n"));
            }
            fs::write(
                workspace_root.join("artifacts/status/performance_report.txt"),
                text,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                    "artifacts/status/performance_report.json",
                    "artifacts/status/performance_regression_budget.json",
                    "artifacts/status/performance_benchmark_policy.json",
                    "artifacts/status/performance_report.txt"
                ]}),
            )
        }
        _ => None,
    }
}
