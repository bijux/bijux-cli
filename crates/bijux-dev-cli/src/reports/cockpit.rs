//! Top-level maintainer cockpit commands for `bijux dev cli`.

use std::path::Path;

use serde_json::{json, Value};

use crate::infrastructure::artifacts::read_json_if_exists;

fn read_first_json(paths: &[&Path]) -> Value {
    for path in paths {
        let payload = read_json_if_exists(path);
        if payload != json!({}) {
            return payload;
        }
    }
    json!({})
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

    let mut policy = payload_obj
        .remove("evidence_first_policy")
        .unwrap_or_else(|| json!({}));
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

/// `dev cli dashboard`
#[must_use]
pub fn build_dashboard_report(workspace_root: &Path) -> Value {
    json!({
        "dashboard": {
            "status": read_json_if_exists(&workspace_root.join("artifacts/status/status.json")),
            "parity": read_json_if_exists(&workspace_root.join("artifacts/parity/parity_dashboard.json")),
            "evidence": read_json_if_exists(&workspace_root.join("artifacts/status/dev_cli_evidence_audit_report.json")),
            "runtime_identity": read_json_if_exists(&workspace_root.join("artifacts/status/install_runtime_identity_report.json")),
            "package_health": read_json_if_exists(&workspace_root.join("artifacts/status/install_neutrality_report.json")),
            "state_health": read_json_if_exists(&workspace_root.join("artifacts/status/state_audit_report.json")),
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
        "bundle": ["status", "parity", "evidence", "runtime-identity", "state-audit"],
    })
}

/// `dev cli truth`
#[must_use]
pub fn build_truth_report(workspace_root: &Path) -> Value {
    let done = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_done.json"));
    let partial =
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_partial.json"));
    let missing = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json"));
    let intentional = read_json_if_exists(
        &workspace_root.join("artifacts/status/what_is_intentionally_different.json"),
    );
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
    let unresolved = release
        .get("unresolved_gaps")
        .cloned()
        .unwrap_or_else(|| json!([]));
    json!({
        "blockers": unresolved,
        "status": release.get("status").cloned().unwrap_or_else(|| json!("blocked")),
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
        &[
            "artifacts/status/priority_plan.json",
            "artifacts/status/priority_plan.txt",
        ],
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
        assert_eq!(
            policy["manual_curated_priority_lists_allowed"],
            Value::Bool(false)
        );
        assert_eq!(
            policy["roadmap_requires_generated_artifacts"],
            Value::Bool(true)
        );
        assert!(
            policy["required_artifacts"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty()),
            "minimalism evidence policy must declare required artifacts"
        );
    }
}
