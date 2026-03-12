#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE" => {
            let commands = [
                ("status", ["dev", "cli", "release", "status"]),
                ("evidence", ["dev", "cli", "release", "evidence"]),
                ("readiness", ["dev", "cli", "release", "readiness"]),
                ("diff", ["dev", "cli", "release", "diff"]),
                ("gaps", ["dev", "cli", "release", "gaps"]),
                (
                    "behavior_changes",
                    ["dev", "cli", "release", "behavior-changes"],
                ),
                (
                    "intentional_differences",
                    ["dev", "cli", "release", "intentional-differences"],
                ),
                (
                    "unresolved_gaps",
                    ["dev", "cli", "release", "unresolved-gaps"],
                ),
                (
                    "compatibility_leftovers",
                    ["dev", "cli", "release", "compatibility-leftovers"],
                ),
            ];
            let mut reports = serde_json::Map::new();
            for (name, cmd) in commands {
                reports.insert(name.to_string(), run_bijux_json(workspace_root, &cmd).ok()?);
            }
            let gaps = reports.get("gaps").cloned().unwrap_or_else(|| json!({}));
            let unresolved = gaps
                .get("unresolved_gaps")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let missing = gaps
                .get("missing_evidence")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_release_truth_bundle.json",
                &json!({
                    "source": "dev cli release *",
                    "reports": reports,
                    "summary": {
                        "unresolved_gaps": unresolved,
                        "missing_evidence": missing,
                    }
                }),
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_release_truth_bundle.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE" => {
            let commands = [
                "dev cli status",
                "dev cli parity",
                "dev cli runtime-identity",
                "dev cli state-audit",
                "dev cli package-health",
                "dev cli maintenance-audit",
                "dev cli rustdoc audit",
                "dev cli release status",
                "dev cli docs-audit",
                "dev cli crate-health",
            ];
            let mut payload = serde_json::Map::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let row = run_bijux_json(workspace_root, &argv).ok()?;
                payload.insert(command.to_string(), json!({
                                    "top_level_keys": row.as_object().map(|obj| obj.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                                    "payload": row,
                                }));
            }
            let ownership_path =
                workspace_root.join("artifacts/status/dev_cli_ownership_report.json");
            let ownership = fs::read_to_string(&ownership_path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok());
            let mut out = json!({
                "scope": "bijux-dev-cli control-plane bundle",
                "commands": payload,
            });
            if let Some(ownership) = ownership {
                out["ownership_report"] = ownership;
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_control_plane_bundle.json",
                &out,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_control_plane_bundle.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP" => {
            let commands = [
                "dev cli env",
                "dev cli contracts",
                "dev cli parity",
                "dev cli status",
            ];
            let mut input_map = BTreeMap::<&str, Vec<&str>>::new();
            input_map.insert(
                "dev cli env",
                vec![
                    "process environment",
                    "resolved config/history/plugins paths",
                ],
            );
            input_map.insert(
                "dev cli contracts",
                vec!["static schema contract declarations", "runtime version"],
            );
            input_map.insert(
                "dev cli parity",
                vec!["artifacts/parity/*.json", "artifacts/parity/*.txt"],
            );
            input_map.insert(
                "dev cli status",
                vec![
                    "artifacts/status/*.json",
                    "artifacts/status/*.txt",
                    "artifacts/parity/rust_python_parity_report.json",
                    "dev-cli inventory payload",
                ],
            );
            let mut reports = Vec::<Value>::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let payload = run_bijux_json(workspace_root, &argv).ok()?;
                let output_top_level_keys = payload
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                reports.push(json!({
                    "command": command,
                    "inputs": input_map.get(command).cloned().unwrap_or_default(),
                    "output_top_level_keys": output_top_level_keys,
                    "output_kind": if payload.is_object() { "json-object" } else { "non-object" },
                }));
            }
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "dev-cli maintainer report inputs vs outputs",
                "reports": reports,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintainer_report_io_map.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_maintainer_report_io_map.json"]}),
            )
        }
        "STATUS-CONTRACT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS" => {
            let parity_first = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_second = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let status_payload = run_bijux_json(workspace_root, &["dev", "cli", "status"]).ok()?;
            let parity_text_first =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_text_second =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;

            let valid_statuses = BTreeSet::from([
                "rust-complete",
                "rust-partial",
                "python-only",
                "intentionally-different",
            ]);
            let migration_rows = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let parity_rows = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let invalid_status_rows: Vec<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    let status = row.get("status").and_then(Value::as_str)?;
                    if valid_statuses.contains(status) {
                        None
                    } else {
                        Some(
                            row.get("command")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        )
                    }
                })
                .collect();
            let partial_without_blocker: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-partial"))
                .filter_map(|row| {
                    let blocker = row
                        .get("blocker")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let shim_alias = row
                        .get("shim_alias_dependency")
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default();
                    let has_shim_alias = shim_alias
                        .get("aliases")
                        .and_then(Value::as_array)
                        .is_some_and(|items| !items.is_empty())
                        || shim_alias
                            .get("shims")
                            .and_then(Value::as_array)
                            .is_some_and(|items| !items.is_empty());
                    let has_parity_mismatch = row
                        .get("parity_coverage")
                        .and_then(Value::as_object)
                        .is_some_and(|obj| obj.values().any(|v| v == &Value::Bool(false)));
                    if blocker.is_empty() && !has_shim_alias && !has_parity_mismatch {
                        Some(
                            row.get("command")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
                .collect();
            let intentional_without_reason: Vec<String> = migration_rows
                .iter()
                .filter(|row| {
                    row.get("status").and_then(Value::as_str) == Some("intentionally-different")
                })
                .filter_map(|row| {
                    let reason = row
                        .get("reason")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .trim();
                    if reason.is_empty() {
                        Some(
                            row.get("command")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
                .collect();
            let complete_without_evidence: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-complete"))
                .filter_map(|row| {
                    if row
                        .get("evidence_links")
                        .and_then(Value::as_array)
                        .is_none_or(Vec::is_empty)
                    {
                        Some(
                            row.get("command")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
                .collect();
            let parity_commands: BTreeSet<String> = parity_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let migration_commands: BTreeSet<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let missing_from_migration: Vec<String> = parity_commands
                .difference(&migration_commands)
                .cloned()
                .collect();
            let parity_complete = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let migration_complete = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("rust-complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let consistency_checks = json!({
                "migration_rows_have_valid_status": invalid_status_rows.is_empty(),
                "partial_rows_have_blockers": partial_without_blocker.is_empty(),
                "intentional_rows_have_reasons": intentional_without_reason.is_empty(),
                "complete_rows_have_evidence_links": complete_without_evidence.is_empty(),
                "parity_commands_exist_in_migration_matrix": missing_from_migration.is_empty(),
                "parity_and_status_complete_counts_align": parity_complete == migration_complete,
                "parity_json_is_deterministic": parity_first == parity_second,
                "parity_text_is_deterministic": parity_text_first == parity_text_second,
            });
            let migration_truth_artifact = json!({
                "scope": "migration truth",
                "generator": "bijux-dev-cli",
                "rows_total": migration_rows.len(),
                "checks": {
                    "valid_status_rows": consistency_checks["migration_rows_have_valid_status"],
                    "partial_rows_with_blockers": consistency_checks["partial_rows_have_blockers"],
                    "intentional_rows_with_reasons": consistency_checks["intentional_rows_have_reasons"],
                    "complete_rows_with_evidence_links": consistency_checks["complete_rows_have_evidence_links"],
                },
                "status": if consistency_checks["migration_rows_have_valid_status"] == true
                    && consistency_checks["partial_rows_have_blockers"] == true
                    && consistency_checks["intentional_rows_have_reasons"] == true
                    && consistency_checks["complete_rows_have_evidence_links"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let parity_evidence_consistency_artifact = json!({
                "scope": "parity evidence consistency",
                "generator": "bijux-dev-cli",
                "checks": {
                    "parity_commands_exist_in_migration_matrix": consistency_checks["parity_commands_exist_in_migration_matrix"],
                    "parity_and_status_complete_counts_align": consistency_checks["parity_and_status_complete_counts_align"],
                    "parity_json_is_deterministic": consistency_checks["parity_json_is_deterministic"],
                    "parity_text_is_deterministic": consistency_checks["parity_text_is_deterministic"],
                },
                "status": if consistency_checks["parity_commands_exist_in_migration_matrix"] == true
                    && consistency_checks["parity_and_status_complete_counts_align"] == true
                    && consistency_checks["parity_json_is_deterministic"] == true
                    && consistency_checks["parity_text_is_deterministic"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let drift_checks: Vec<String> = consistency_checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let parity_drift_artifact = json!({
                "scope": "parity and migration drift",
                "generator": "bijux-dev-cli",
                "drift_checks": drift_checks,
                "drift_count": drift_checks.len(),
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                "details": {
                    "invalid_status_rows": invalid_status_rows,
                    "partial_without_blocker": partial_without_blocker,
                    "intentional_without_reason": intentional_without_reason,
                    "complete_without_evidence": complete_without_evidence,
                    "parity_missing_from_migration": missing_from_migration,
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/migration_truth_artifact.json",
                &migration_truth_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_evidence_consistency_artifact.json",
                &parity_evidence_consistency_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_drift_artifact.json",
                &parity_drift_artifact,
            )
            .ok()?;
            Some(json!({
                "status":"ok",
                "contract_id":contract_id,
                "implementation":"rust",
                "outputs":[
                    "artifacts/status/migration_truth_artifact.json",
                    "artifacts/status/parity_evidence_consistency_artifact.json",
                    "artifacts/status/parity_drift_artifact.json"
                ]
            }))
        }
        _ => None,
    }
}
