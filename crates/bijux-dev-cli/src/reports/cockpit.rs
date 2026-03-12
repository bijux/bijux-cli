//! Top-level maintainer cockpit commands for `bijux dev cli`.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{artifact_source_path, json_artifact_state, read_json_if_exists};

fn read_first_json(paths: &[&Path]) -> Value {
    for path in paths {
        let payload = read_json_if_exists(path);
        if json_artifact_state(&payload) == "valid" {
            return payload;
        }
    }
    json!({
        "_artifact_state": "missing",
        "_artifact_paths": paths
            .iter()
            .map(|path| artifact_source_path(path))
            .collect::<Vec<_>>(),
    })
}

fn ensure_evidence_first_policy(
    mut payload: Value,
    required_artifacts: &[&str],
    require_generated_roadmap: bool,
) -> Value {
    if !payload.is_object() {
        payload = json!({});
    }
    let Value::Object(payload_obj) = &mut payload else {
        return payload;
    };

    let mut policy = payload_obj.remove("evidence_first_policy").unwrap_or_else(|| json!({}));
    if !policy.is_object() {
        policy = json!({});
    }
    let Value::Object(policy_obj) = &mut policy else {
        return payload;
    };

    policy_obj
        .entry("manual_curated_priority_lists_allowed".to_string())
        .or_insert_with(|| json!(false));
    if require_generated_roadmap {
        policy_obj
            .entry("roadmap_requires_generated_artifacts".to_string())
            .or_insert_with(|| json!(true));
    }

    let has_required_artifacts = policy_obj
        .get("required_artifacts")
        .and_then(Value::as_array)
        .is_some_and(|rows| !rows.is_empty());
    if !has_required_artifacts {
        policy_obj.insert("required_artifacts".to_string(), json!(required_artifacts));
    }

    payload_obj.insert("evidence_first_policy".to_string(), policy);
    payload
}

fn truth_rows_from_status(workspace_root: &Path) -> Vec<Value> {
    let status = read_json_if_exists(&workspace_root.join("artifacts/status/status.json"));
    let rows = status.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    if !rows.is_empty() {
        return rows;
    }
    read_json_if_exists(&workspace_root.join("artifacts/parity/command_parity_matrix.json"))
        .get("commands")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn synthesize_truth_bucket(rows: &[Value], kind: &str) -> Value {
    let selected: Vec<Value> = rows
        .iter()
        .filter(|row| {
            let status = row
                .get("matrix_status")
                .and_then(Value::as_str)
                .or_else(|| row.get("status").and_then(Value::as_str))
                .unwrap_or("partial");
            match kind {
                "done" => matches!(status, "complete" | "rust-complete"),
                "missing" => matches!(status, "missing" | "python-only"),
                "partial" => matches!(status, "partial" | "rust-partial" | "shim"),
                "intentional_differences" => {
                    matches!(status, "intentionally-different" | "different-by-decision")
                }
                _ => false,
            }
        })
        .cloned()
        .collect();
    json!({
        "generated_at": "1970-01-01T00:00:00+00:00",
        "generator": "bijux-dev-cli",
        "items": selected,
        "summary": {
            "count": selected.len()
        }
    })
}

fn ensure_truth_bucket(payload: Value, rows: &[Value], kind: &str) -> Value {
    let has_count = payload
        .get("summary")
        .and_then(|summary| summary.get("count"))
        .and_then(Value::as_u64)
        .is_some();
    if has_count {
        payload
    } else {
        synthesize_truth_bucket(rows, kind)
    }
}

fn bucket_count(payload: &Value) -> u64 {
    payload
        .get("summary")
        .and_then(|summary| summary.get("count"))
        .and_then(Value::as_u64)
        .unwrap_or_default()
}

fn status_summary_counts(workspace_root: &Path, rows: &[Value]) -> (u64, u64, u64, u64) {
    let status = read_json_if_exists(&workspace_root.join("artifacts/status/status.json"));
    let summary = status.get("summary");
    let complete = summary
        .and_then(|value| value.get("complete"))
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            synthesize_truth_bucket(rows, "done")["summary"]["count"].as_u64().unwrap_or_default()
        });
    let missing = summary
        .and_then(|value| value.get("missing"))
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            synthesize_truth_bucket(rows, "missing")["summary"]["count"]
                .as_u64()
                .unwrap_or_default()
        });
    let partial = summary
        .and_then(|value| value.get("partial"))
        .and_then(Value::as_u64)
        .unwrap_or_else(|| {
            synthesize_truth_bucket(rows, "partial")["summary"]["count"]
                .as_u64()
                .unwrap_or_default()
        });
    let shim = summary.and_then(|value| value.get("shim")).and_then(Value::as_u64).unwrap_or(0);
    (complete, missing, partial, shim)
}

/// `dev cli dashboard`
#[must_use]
pub fn build_dashboard_report(workspace_root: &Path) -> Value {
    let status = read_json_if_exists(&workspace_root.join("artifacts/status/status.json"));
    let parity =
        read_json_if_exists(&workspace_root.join("artifacts/parity/parity_dashboard.json"));
    let evidence = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_evidence_audit_report.json"),
    );
    let runtime_identity = read_json_if_exists(
        &workspace_root.join("artifacts/status/install_runtime_identity_report.json"),
    );
    let package_health = read_json_if_exists(
        &workspace_root.join("artifacts/status/install_neutrality_report.json"),
    );
    let state_health =
        read_json_if_exists(&workspace_root.join("artifacts/status/state_audit_report.json"));
    let status_state = json_artifact_state(&status).to_string();
    let parity_state = json_artifact_state(&parity).to_string();
    let evidence_state = json_artifact_state(&evidence).to_string();
    let runtime_identity_state = json_artifact_state(&runtime_identity).to_string();
    let package_health_state = json_artifact_state(&package_health).to_string();
    let state_health_state = json_artifact_state(&state_health).to_string();
    json!({
        "dashboard": {
            "status": status,
            "parity": parity,
            "evidence": evidence,
            "runtime_identity": runtime_identity,
            "package_health": package_health,
            "state_health": state_health,
        },
        "artifact_integrity": {
            "status": status_state,
            "parity": parity_state,
            "evidence": evidence_state,
            "runtime_identity": runtime_identity_state,
            "package_health": package_health_state,
            "state_health": state_health_state,
        },
        "command_center": "bijux dev cli",
    })
}

/// `dev cli quickcheck`
#[must_use]
pub fn build_quickcheck_report(workspace_root: &Path) -> Value {
    let release =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let evidence = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_evidence_audit_report.json"),
    );
    let python = read_json_if_exists(
        &workspace_root.join("artifacts/status/python_sovereignty_audit_report.json"),
    );
    json!({
        "quickcheck": {
            "release_status": release.get("status").cloned().unwrap_or_else(|| json!("blocked")),
            "evidence_status": evidence.get("status").cloned().unwrap_or_else(|| json!("fail")),
            "python_sovereignty_status": python.get("status").cloned().unwrap_or_else(|| json!("needs-work")),
        },
        "artifact_integrity": {
            "release_status_manifest": json_artifact_state(&release),
            "evidence_audit": json_artifact_state(&evidence),
            "python_sovereignty_audit": json_artifact_state(&python),
        },
        "bundle": ["status", "parity", "evidence", "runtime-identity", "state-audit"],
    })
}

/// `dev cli truth`
#[must_use]
pub fn build_truth_report(workspace_root: &Path) -> Value {
    let rows = truth_rows_from_status(workspace_root);
    let mut done = ensure_truth_bucket(
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_done.json")),
        &rows,
        "done",
    );
    let mut partial = ensure_truth_bucket(
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_partial.json")),
        &rows,
        "partial",
    );
    let mut missing = ensure_truth_bucket(
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json")),
        &rows,
        "missing",
    );
    let mut intentional = ensure_truth_bucket(
        read_json_if_exists(
            &workspace_root.join("artifacts/status/what_is_intentionally_different.json"),
        ),
        &rows,
        "intentional_differences",
    );

    let (status_complete, status_missing, status_partial, status_shim) =
        status_summary_counts(workspace_root, &rows);
    let truth_done = bucket_count(&done);
    let truth_missing = bucket_count(&missing);
    let truth_partial = bucket_count(&partial);
    let truth_intentional = bucket_count(&intentional);
    let mismatch = truth_done != status_complete
        || truth_missing != status_missing
        || truth_partial + truth_intentional != status_partial + status_shim;
    if mismatch {
        done = synthesize_truth_bucket(&rows, "done");
        missing = synthesize_truth_bucket(&rows, "missing");
        partial = synthesize_truth_bucket(&rows, "partial");
        intentional = synthesize_truth_bucket(&rows, "intentional_differences");
    }

    json!({
        "truth": {
            "done": done,
            "partial": partial,
            "missing": missing,
            "intentional_differences": intentional,
        }
    })
}

/// `dev cli blockers`
#[must_use]
pub fn build_blockers_report(workspace_root: &Path) -> Value {
    let release = read_json_if_exists(
        &workspace_root.join("artifacts/status/dev_cli_release_gaps_report.json"),
    );
    let unresolved = release.get("unresolved_gaps").cloned().unwrap_or_else(|| json!([]));
    json!({
        "blockers": unresolved,
        "status": release.get("status").cloned().unwrap_or_else(|| json!("blocked")),
        "artifact_integrity": {
            "release_gaps": json_artifact_state(&release),
        },
    })
}

/// `dev cli next`
#[must_use]
pub fn build_next_report(workspace_root: &Path) -> Value {
    let priorities = ensure_evidence_first_policy(
        read_first_json(&[
            &workspace_root.join("artifacts/status/priority_plan_priorities.json"),
            &workspace_root.join("artifacts/status/priority_plan.json"),
        ]),
        &["artifacts/status/priority_plan.json", "artifacts/status/priority_plan.txt"],
        false,
    );
    let minimalism = ensure_evidence_first_policy(
        read_json_if_exists(
            &workspace_root.join("artifacts/status/simplification_priorities.json"),
        ),
        &[
            "artifacts/status/simplification_priorities.json",
            "artifacts/status/simplification_priorities.txt",
        ],
        true,
    );
    json!({
        "next": {
            "priorities": priorities,
            "minimalism": minimalism,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use serde_json::Value;

    use super::{
        build_blockers_report, build_dashboard_report, build_next_report, build_quickcheck_report,
        build_truth_report,
    };

    #[test]
    fn cockpit_reports_have_stable_top_level_keys() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(build_dashboard_report(&root).get("dashboard").is_some());
        assert!(build_quickcheck_report(&root).get("quickcheck").is_some());
        assert!(build_truth_report(&root).get("truth").is_some());
        assert!(build_blockers_report(&root).get("blockers").is_some());
        assert!(build_next_report(&root).get("next").is_some());
    }

    #[test]
    fn next_report_keeps_evidence_policy_contract_when_artifacts_are_missing() {
        let root = std::env::temp_dir().join(format!("bijux-next-missing-{}", std::process::id()));
        fs::create_dir_all(root.join("artifacts/status")).expect("create status dir");

        let report = build_next_report(&root);
        let policy = &report["next"]["minimalism"]["evidence_first_policy"];
        assert_eq!(policy["manual_curated_priority_lists_allowed"], Value::Bool(false));
        assert_eq!(policy["roadmap_requires_generated_artifacts"], Value::Bool(true));
        assert!(
            policy["required_artifacts"].as_array().is_some_and(|rows| !rows.is_empty()),
            "minimalism evidence policy must declare required artifacts"
        );
    }
}
