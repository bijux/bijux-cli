//! Top-level maintainer cockpit commands for `bijux dev cli`.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
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
    let unresolved = release.get("unresolved_gaps").cloned().unwrap_or_else(|| json!([]));
    json!({
        "blockers": unresolved,
        "status": release.get("status").cloned().unwrap_or_else(|| json!("blocked")),
    })
}

/// `dev cli next`
#[must_use]
pub fn build_next_report(workspace_root: &Path) -> Value {
    let priorities =
        read_json_if_exists(&workspace_root.join("artifacts/status/next_phase_priorities.json"));
    let minimalism =
        read_json_if_exists(&workspace_root.join("artifacts/status/next_phase_minimalism.json"));
    json!({
        "next": {
            "priorities": priorities,
            "minimalism": minimalism,
        }
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

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
}
