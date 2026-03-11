#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-MIGRATION-NOTES" => {
            let generated_at = generated_at_utc();
            let parity_matrix = fs::read_to_string(
                workspace_root.join("artifacts/parity/command_parity_matrix.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let command_rows = parity_matrix
                .get("commands")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let changed: Vec<Value> = command_rows
                .into_iter()
                .filter(|row| {
                    row.get("status").and_then(Value::as_str).is_some_and(|s| {
                        matches!(
                            s,
                            "partial" | "intentionally-different" | "different-by-decision"
                        )
                    })
                })
                .map(|row| {
                    json!({
                        "command": row.get("command").cloned().unwrap_or(Value::Null),
                        "status": row.get("status").cloned().unwrap_or(Value::Null),
                        "reason": row.get("reason").cloned().unwrap_or_else(|| json!("")),
                        "blocker": row.get("blocker").cloned().unwrap_or_else(|| json!("")),
                    })
                })
                .collect();
            let package_health = fs::read_to_string(
                workspace_root.join("artifacts/status/package_health_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let assumptions = package_health
                .get("payload")
                .and_then(|v| v.get("install_state_assumptions"))
                .cloned()
                .unwrap_or_else(|| json!([]));
            let runtime_unity = fs::read_to_string(
                workspace_root.join("artifacts/status/runtime_unity_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let plugin_failures = fs::read_to_string(
                workspace_root
                    .join("artifacts/status/plugin_lifecycle_failure_injection_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let rollback = fs::read_to_string(
                workspace_root.join("artifacts/status/plugin_rollback_proof_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let config = fs::read_to_string(
                workspace_root.join("artifacts/status/config_corruption_matrix.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let state = fs::read_to_string(
                workspace_root.join("artifacts/status/state_resilience_summary.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let guidance = fs::read_to_string(
                workspace_root.join("artifacts/status/state_recovery_guidance.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/migration_notes_commands.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "commands",
                    "coverage_ids": [574],
                    "items": changed.into_iter().take(250).collect::<Vec<_>>(),
                    "source": "artifacts/parity/command_parity_matrix.json",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_packaging.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "packaging",
                                    "coverage_ids": [575],
                                    "runtime_unity_ok": runtime_unity.get("ok").and_then(Value::as_bool).unwrap_or(false),
                                    "items": [
                                        {
                                            "area": "runtime-identity",
                                            "note": "verify active binary and PATH shadowing behavior before cutover",
                                            "evidence": "artifacts/status/runtime_unity_report.json",
                                        },
                                        {
                                            "area": "install-assumptions",
                                            "note": "review install-state assumptions and shell completion target paths",
                                            "assumptions": assumptions,
                                            "evidence": "artifacts/status/package_health_report.json",
                                        },
                                    ],
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_plugin_lifecycle.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "plugin-lifecycle",
                                    "coverage_ids": [576],
                                    "items": [
                                        {
                                            "area": "plugin-install-write-path",
                                            "note": "validate rollback and retry behavior before enabling new plugin capabilities",
                                            "evidence": [
                                                "artifacts/status/plugin_lifecycle_failure_injection_report.json",
                                                "artifacts/status/plugin_rollback_proof_report.json",
                                            ],
                                        },
                                        {
                                            "area": "plugin-runtime-diagnostics",
                                            "note": "verify reserved-name and registry diagnostics surface expected errors",
                                            "evidence": "artifacts/status/namespace_abuse_report.json",
                                        },
                                    ],
                                    "plugin_report_status": plugin_failures.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "rollback_report_status": rollback.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                }),
                            )
                            .ok()?;
            write_status_artifact_json(
                                workspace_root,
                                "artifacts/status/migration_notes_state_behavior.json",
                                &json!({
                                    "generated_at": generated_at,
                                    "generator": "bijux-dev-cli",
                                    "scope": "state-behavior",
                                    "coverage_ids": [577],
                                    "items": [
                                        {
                                            "area": "config",
                                            "note": "backup and validate config before mutating across runtime upgrades",
                                            "evidence": "artifacts/status/config_corruption_matrix.json",
                                        },
                                        {
                                            "area": "history-memory",
                                            "note": "run state doctor when corrupted history or memory payloads are detected",
                                            "evidence": "artifacts/status/state_resilience_summary.json",
                                        },
                                        {
                                            "area": "recovery",
                                            "note": "follow machine-readable state recovery guidance for rollback paths",
                                            "evidence": "artifacts/status/state_recovery_guidance.json",
                                        },
                                    ],
                                    "config_status": config.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "state_status": state.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                    "guidance_status": guidance.get("status").cloned().unwrap_or_else(|| json!("unknown")),
                                }),
                            )
                            .ok()?;
            let migration_cmds = fs::read_to_string(
                workspace_root.join("artifacts/status/migration_notes_commands.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .and_then(|v| v.get("items").cloned())
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default();
            let mut text = String::from("Migration Notes\n\nCommands:\n");
            for item in migration_cmds.into_iter().take(40) {
                let command = item.get("command").and_then(Value::as_str).unwrap_or("");
                let status = item.get("status").and_then(Value::as_str).unwrap_or("");
                let reason = item.get("reason").and_then(Value::as_str).unwrap_or("");
                text.push_str(&format!("- {command}: status={status} reason={reason}\n"));
            }
            text.push_str(
                                "\nPackaging:\n- runtime-identity: verify active binary and PATH shadowing behavior before cutover\n- install-assumptions: review install-state assumptions and shell completion target paths\n\nPlugin lifecycle:\n- plugin-install-write-path: validate rollback and retry behavior before enabling new plugin capabilities\n- plugin-runtime-diagnostics: verify reserved-name and registry diagnostics surface expected errors\n\nState behavior:\n- config: backup and validate config before mutating across runtime upgrades\n- history-memory: run state doctor when corrupted history or memory payloads are detected\n- recovery: follow machine-readable state recovery guidance for rollback paths\n",
                            );
            fs::write(
                workspace_root.join("artifacts/status/migration_notes.txt"),
                text,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                    "artifacts/status/migration_notes_commands.json",
                    "artifacts/status/migration_notes_packaging.json",
                    "artifacts/status/migration_notes_plugin_lifecycle.json",
                    "artifacts/status/migration_notes_state_behavior.json",
                    "artifacts/status/migration_notes.txt"
                ]}),
            )
        }
        _ => None,
    }
}
