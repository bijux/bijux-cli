#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-STATUS-REPORTS" => {
            let generated_at = generated_at_utc();
            let read = |p: &str| {
                fs::read_to_string(workspace_root.join(p))
                    .ok()
                    .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let current_state = read("artifacts/status/current_rust_state.json");
            let parity_matrix = read("artifacts/parity/command_parity_matrix.json");
            let bridge_report = read("artifacts/parity/binary_vs_python_bridge_parity_report.json");
            let runtime_unity = read("artifacts/status/runtime_unity_report.json");
            let state_config = read("artifacts/parity/config_parity_report.json");
            let state_history = read("artifacts/parity/history_parity_report.json");
            let state_memory = read("artifacts/parity/memory_parity_report.json");
            let plugin_state = read("artifacts/status/plugin_state_report.json");
            let intentional = read("docs/architecture/parity/intentional_differences.json");
            let aliases = current_state
                .get("rust_routed_commands")
                .and_then(|r| r.get("aliases"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect::<BTreeSet<_>>();
            let rows = parity_matrix
                .get("commands")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut command_rows = rows
                                .into_iter()
                                .filter_map(|row| row.as_object().cloned())
                                .filter_map(|row| {
                                    let command = row.get("command")?.as_str()?.trim().to_string();
                                    if command.is_empty() {
                                        return None;
                                    }
                                    let matrix_status = row.get("status").and_then(Value::as_str).unwrap_or("missing");
                                    let status = if aliases.contains(&command) {
                                        "shim"
                                    } else if matrix_status == "missing" {
                                        "missing"
                                    } else if matrix_status == "partial" {
                                        "partial"
                                    } else {
                                        "complete"
                                    };
                                    Some(json!({
                                        "command":command,"group":row.get("group").and_then(Value::as_str).unwrap_or("unknown"),
                                        "status":status,"matrix_status":matrix_status,
                                        "owner":row.get("owner").and_then(Value::as_str).unwrap_or(""),
                                        "reason":row.get("reason").and_then(Value::as_str).unwrap_or(""),
                                        "blocker":row.get("blocker").and_then(Value::as_str).unwrap_or(""),
                                        "confidence":row.get("confidence").cloned().unwrap_or_else(|| json!(0.0))
                                    }))
                                })
                                .collect::<Vec<_>>();
            command_rows.sort_by(|a, b| {
                a.get("command")
                    .and_then(Value::as_str)
                    .cmp(&b.get("command").and_then(Value::as_str))
            });
            let root_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter_map(|c| c.split_whitespace().next().map(ToString::to_string))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let cli_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter(|c| c.starts_with("cli "))
                .map(|c| c.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let maintainer_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter(|c| c.starts_with("bijux-dev-cli "))
                .map(|c| c.split_whitespace().take(4).collect::<Vec<_>>().join(" "))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let plugin_commands = command_rows
                .iter()
                .filter_map(|r| r.get("command").and_then(Value::as_str))
                .filter_map(|c| {
                    if c.starts_with("plugins ") {
                        Some(c.split_whitespace().take(2).collect::<Vec<_>>().join(" "))
                    } else if c.starts_with("cli plugins ") {
                        Some(c.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let snapshot_covered = current_state
                .get("snapshot_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let stream_covered = current_state
                .get("stderr_stdout_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let exit_covered = current_state
                .get("exit_code_covered_commands")
                .cloned()
                .unwrap_or_else(|| json!([]));
            let fail_covered = collect_files(&workspace_root.join("crates"))
                .into_iter()
                .filter(|p| {
                    p.to_string_lossy().contains("/tests/")
                        && p.extension().and_then(|e| e.to_str()) == Some("rs")
                })
                .filter_map(|p| fs::read_to_string(&p).ok())
                .flat_map(|txt| txt.lines().map(ToString::to_string).collect::<Vec<_>>())
                .filter(|line| {
                    line.contains("[\"")
                        && [
                            "error",
                            "failure",
                            "invalid",
                            "malformed",
                            "missing",
                            "reject",
                            "rollback",
                            "corrupt",
                            "unsafe",
                            "duplicate",
                            "conflict",
                            "shadow",
                        ]
                        .iter()
                        .any(|k| line.to_lowercase().contains(k))
                })
                .filter_map(|line| {
                    let quoted = line.split('"').collect::<Vec<_>>();
                    let vals = quoted
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| (i % 2 == 1).then_some((*v).to_string()))
                        .collect::<Vec<_>>();
                    (!vals.is_empty()).then_some(vals.join(" "))
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let known_gaps = command_rows.iter().filter(|row| row.get("status").and_then(Value::as_str).is_some_and(|s| ["missing","partial","shim"].contains(&s))).map(|row| json!({"command":row["command"],"status":row["status"],"blocker":row["blocker"],"owner":row["owner"]})).collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/status.json", &json!({
                                "generated_at":generated_at,"generator":"bijux-dev-cli","commands":command_rows,
                                "summary":{"total":command_rows.len(),"complete":command_rows.iter().filter(|r| r["status"]=="complete").count(),"partial":command_rows.iter().filter(|r| r["status"]=="partial").count(),"shim":command_rows.iter().filter(|r| r["status"]=="shim").count(),"missing":command_rows.iter().filter(|r| r["status"]=="missing").count()}
                            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_root_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":root_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_cli_subcommands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":cli_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_maintainer_subcommands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":maintainer_commands})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_plugin_commands.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":plugin_commands})).ok()?;
            let repl = command_rows
                .iter()
                .filter(|r| {
                    r.get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|c| c.split_whitespace().any(|p| p == "repl"))
                })
                .cloned()
                .collect::<Vec<_>>();
            write_status_artifact_json(workspace_root, "artifacts/status/status_repl_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","summary":{"count":repl.len(),"statuses":{"complete":repl.iter().filter(|r| r["status"]=="complete").count(),"partial":repl.iter().filter(|r| r["status"]=="partial").count(),"shim":repl.iter().filter(|r| r["status"]=="shim").count(),"missing":repl.iter().filter(|r| r["status"]=="missing").count()}},"commands":repl,"evidence_files":["crates/bijux-cli/tests/integration/repl/transcript_parity.rs","crates/bijux-cli/tests/integration/repl/repl_transcript_contracts.rs"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_python_bridge_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","report":bridge_report})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_install_packaging_parity_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","runtime_unity":runtime_unity,"runtime_identity_rules":current_state.get("runtime_identity_rules").cloned().unwrap_or_else(|| json!({})),"package_entrypoints":current_state.get("package_entrypoints").cloned().unwrap_or_else(|| json!([]))})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_behavior_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","config":state_config,"history":state_history,"memory":state_memory,"plugin_state":plugin_state})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_paths_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","state_paths":{"config":"BIJUXCLI_CONFIG or <HOME>/.bijux/.env","history":"BIJUXCLI_HISTORY_FILE or <HOME>/.bijux/.history","plugins_dir":"BIJUXCLI_PLUGINS_DIR or <HOME>/.bijux/.plugins","plugins_registry":"<plugins_dir>/registry.json","memory":"<HOME>/.bijux/.memory.json"},"source_precedence":["flags","env","config","defaults"]})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_state_corruption_health_report.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","areas":{"config":{"report":state_config,"focus":["malformed file","duplicate key","partial-write rollback"]},"history":{"report":state_history,"focus":["malformed array entries","line-format compatibility","oversized budget"]},"memory":{"report":state_memory,"focus":["malformed json","wrong-type object rejection"]},"plugin_registry":{"report":plugin_state,"focus":["malformed registry json","partial-write self-repair","stale backup cleanup"]}}})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_snapshot_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":snapshot_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_stream_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":stream_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_exit_code_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":exit_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_failure_path_coverage.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":fail_covered})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_compatibility_aliases.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","aliases":aliases.into_iter().collect::<Vec<_>>()})).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/status_known_parity_gaps.json",
                &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","gaps":known_gaps}),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_intentional_differences.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","commands":intentional})).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/status_unowned_maintenance.json", &json!({"generated_at":generated_at,"generator":"bijux-dev-cli","maintenance":fs::read_to_string(workspace_root.join("artifacts/status/maintainer_maintenance_outside_control_plane.json")).ok().and_then(|text| serde_json::from_str::<Value>(&text).ok()).and_then(|report| report.get("maintenance").cloned()).unwrap_or_else(|| json!([]))})).ok()?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/status.json","artifacts/status/status_root_commands.json","artifacts/status/status_cli_subcommands.json","artifacts/status/status_maintainer_subcommands.json","artifacts/status/status_plugin_commands.json","artifacts/status/status_repl_parity_coverage.json","artifacts/status/status_python_bridge_parity_coverage.json","artifacts/status/status_install_packaging_parity_coverage.json","artifacts/status/status_state_behavior_coverage.json","artifacts/status/status_state_paths_report.json","artifacts/status/status_state_corruption_health_report.json","artifacts/status/status_snapshot_coverage.json","artifacts/status/status_stream_coverage.json","artifacts/status/status_exit_code_coverage.json","artifacts/status/status_failure_path_coverage.json","artifacts/status/status_compatibility_aliases.json","artifacts/status/status_known_parity_gaps.json","artifacts/status/status_intentional_differences.json","artifacts/status/status_unowned_maintenance.json"
            ]}))
        }
        _ => None,
    }
}
