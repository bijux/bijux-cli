#[allow(clippy::wildcard_imports)]
use crate::contracts::maintenance::*;

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    match contract_id {
        "STATUS-CONTRACT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS" => {
            let stale_root = std::env::var("DEV_CLI_STALE_ARTIFACT_ROOT")
                .ok()
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_root.to_path_buf());
            let stale_write = |artifact: &str, payload: &Value| -> Option<()> {
                let path = stale_root.join(artifact);
                write_json(&path, payload).ok()
            };
            let now_epoch = std::env::var("DEV_CLI_STALE_NOW_EPOCH")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|dur| dur.as_secs())
                        .unwrap_or(0)
                });
            let max_age_seconds = std::env::var("DEV_CLI_STALE_MAX_SECONDS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(86_400);
            let forced_raw = std::env::var("DEV_CLI_FORCE_STALE_FILES").unwrap_or_default();
            let mut forced: BTreeSet<String> = forced_raw
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect();
            if std::env::var("DEV_CLI_INJECT_STALE_ARTIFACT").is_ok_and(|raw| raw == "1") {
                forced.insert("artifacts/status/parity_drift_artifact.json".to_string());
            }
            let specs = vec![
                (
                    "evidence_deleted_before_evidence_audit",
                    "bijux-dev-cli evidence audit",
                    "artifacts/status/evidence_integrity_artifact.json",
                    "critical",
                    "Detect missing evidence artifact before evidence audit.",
                ),
                (
                    "evidence_stale_before_evidence_stale",
                    "bijux-dev-cli evidence stale",
                    "artifacts/status/evidence_integrity_artifact.json",
                    "critical",
                    "Detect stale evidence artifact before evidence stale command.",
                ),
                (
                    "parity_stale_before_status",
                    "bijux-dev-cli status",
                    "artifacts/status/parity_drift_artifact.json",
                    "critical",
                    "Detect stale parity artifact before status command.",
                ),
                (
                    "migration_stale_before_truth",
                    "bijux-dev-cli truth",
                    "artifacts/status/migration_truth_artifact.json",
                    "critical",
                    "Detect stale migration artifact before truth command.",
                ),
                (
                    "package_health_stale_before_dashboard",
                    "bijux-dev-cli dashboard",
                    "artifacts/status/package_health_diagnostics_artifact.json",
                    "critical",
                    "Detect stale package health artifact before dashboard command.",
                ),
                (
                    "state_audit_stale_before_blockers",
                    "bijux-dev-cli blockers",
                    "artifacts/status/state_audit_truth_artifact.json",
                    "critical",
                    "Detect stale state audit artifact before blockers command.",
                ),
                (
                    "docs_audit_stale_before_repo_health",
                    "bijux-dev-cli repo health",
                    "artifacts/status/docs_audit.json",
                    "critical",
                    "Detect stale docs-audit artifact before repo health command.",
                ),
                (
                    "maintenance_audit_stale_before_repo_health",
                    "bijux-dev-cli repo health",
                    "artifacts/status/maintenance_gap_behaviors.json",
                    "critical",
                    "Detect stale maintenance-audit artifact before repo health command.",
                ),
                (
                    "crate_health_stale_before_crate_health",
                    "bijux-dev-cli crate-health",
                    "artifacts/status/duplication_hotspots.json",
                    "critical",
                    "Detect stale crate-health artifact before crate-health command.",
                ),
                (
                    "optional_next_report_stale_warning",
                    "bijux-dev-cli next",
                    "artifacts/status/dev_cli_next_report.json",
                    "warning",
                    "Stale optional report is tolerated with warning.",
                ),
            ];
            let checks: Vec<Value> = specs
                .iter()
                .map(|(scenario_id, command, relative_path, severity, description)| {
                    let path = stale_root.join(relative_path);
                    let exists = path.exists();
                    let mut state = "fresh".to_string();
                    let mut age_seconds = None::<u64>;
                    if !exists {
                        state = "missing".to_string();
                    } else {
                        let modified = path
                            .metadata()
                            .ok()
                            .and_then(|meta| meta.modified().ok())
                            .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|dur| dur.as_secs())
                            .unwrap_or(now_epoch);
                        let age = now_epoch.saturating_sub(modified);
                        age_seconds = Some(age);
                        if forced.contains(*relative_path) || age > max_age_seconds {
                            state = "stale".to_string();
                        }
                    }
                    json!({
                        "scenario_id": scenario_id,
                        "command": command,
                        "path": relative_path,
                        "severity": severity,
                        "description": description,
                        "exists": exists,
                        "state": state,
                        "age_seconds": age_seconds,
                        "max_age_seconds": max_age_seconds,
                    })
                })
                .collect();
            let stale_or_missing: Vec<Value> = checks
                .iter()
                .filter(|row| {
                    row.get("state")
                        .and_then(Value::as_str)
                        .is_some_and(|s| s == "stale" || s == "missing")
                })
                .cloned()
                .collect();
            let fresh_count = checks.len().saturating_sub(stale_or_missing.len());
            let critical_stale_count = stale_or_missing
                .iter()
                .filter(|row| row.get("severity").and_then(Value::as_str) == Some("critical"))
                .count();
            let warning_stale_count = stale_or_missing
                .iter()
                .filter(|row| row.get("severity").and_then(Value::as_str) == Some("warning"))
                .count();
            let status_value = if stale_or_missing.is_empty() { "clean" } else { "drift" };
            let summary = json!({
                "checks_total": checks.len(),
                "fresh_count": fresh_count,
                "stale_or_missing_count": stale_or_missing.len(),
                "critical_stale_count": critical_stale_count,
                "warning_stale_count": warning_stale_count,
                "status": status_value,
                "injection_mode": std::env::var("DEV_CLI_INJECT_STALE_ARTIFACT").is_ok_and(|raw| raw == "1"),
            });
            stale_write(
                "artifacts/status/stale_artifact_artifact.json",
                &json!({
                    "scope": "stale artifact truth",
                    "generator": "bijux-dev-cli",
                    "summary": summary,
                    "checks": checks,
                }),
            )?;
            stale_write(
                "artifacts/status/stale_evidence_artifact.json",
                &json!({
                    "scope": "stale evidence truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks.iter().filter(|row| {
                        row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "bijux-dev-cli evidence audit" || cmd == "bijux-dev-cli evidence stale"
                        })
                    }).cloned().collect::<Vec<_>>(),
                    "status": if checks.iter().any(|row| {
                        row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "bijux-dev-cli evidence audit" || cmd == "bijux-dev-cli evidence stale"
                    }) && row.get("state").and_then(Value::as_str).is_some_and(|state| state == "stale" || state == "missing")
                    }) { "drift" } else { "clean" },
                }),
            )?;
            stale_write(
                "artifacts/status/stale_report_artifact.json",
                &json!({
                    "scope": "stale report truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks.iter().filter(|row| {
                        !row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "bijux-dev-cli evidence audit" || cmd == "bijux-dev-cli evidence stale"
                        })
                    }).cloned().collect::<Vec<_>>(),
                    "status": status_value,
                }),
            )?;
            stale_write(
                "artifacts/status/stale_detection_regression_suite.json",
                &json!({
                    "scope": "stale artifact regression suite",
                    "generator": "bijux-dev-cli",
                    "cases": checks.iter().map(|row| {
                        json!({
                            "scenario_id": row.get("scenario_id").cloned().unwrap_or(Value::Null),
                            "command": row.get("command").cloned().unwrap_or(Value::Null),
                            "state": row.get("state").cloned().unwrap_or(Value::Null),
                            "severity": row.get("severity").cloned().unwrap_or(Value::Null),
                        })
                    }).collect::<Vec<_>>(),
                    "status": if critical_stale_count == 0 { "clean" } else { "drift" },
                }),
            )?;
            Some(json!({"status":"ok","contract_id":contract_id,"implementation":"rust","outputs":[
                "artifacts/status/stale_artifact_artifact.json",
                "artifacts/status/stale_evidence_artifact.json",
                "artifacts/status/stale_report_artifact.json",
                "artifacts/status/stale_detection_regression_suite.json"
            ]}))
        }
        "STATUS-CONTRACT-ENFORCE-DEV-CLI-STALE-ARTIFACT-GATE" => {
            let stale_root = std::env::var("DEV_CLI_STALE_ARTIFACT_ROOT")
                .ok()
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_root.to_path_buf());
            let payload: Value = fs::read_to_string(
                stale_root.join("artifacts/status/stale_artifact_artifact.json"),
            )
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .unwrap_or_else(|| json!({}));
            let summary = payload.get("summary").cloned().unwrap_or_else(|| json!({}));
            let critical_stale =
                summary.get("critical_stale_count").and_then(Value::as_i64).unwrap_or(0);
            let warning_stale =
                summary.get("warning_stale_count").and_then(Value::as_i64).unwrap_or(0);
            let injection_mode =
                summary.get("injection_mode").and_then(Value::as_bool).unwrap_or(false);
            let allow_injection_drift =
                std::env::var("DEV_CLI_ALLOW_INJECTION_DRIFT").ok().as_deref() == Some("1");
            if critical_stale > 0 && !(injection_mode && allow_injection_drift) {
                return Some(json!({
                    "status":"failed",
                    "contract_id":contract_id,
                    "implementation":"rust",
                    "error":"critical stale artifacts detected",
                    "summary": summary
                }));
            }
            Some(json!({
                "status":"ok",
                "contract_id":contract_id,
                "implementation":"rust",
                "warnings": warning_stale,
                "summary": summary
            }))
        }
        _ => None,
    }
}
