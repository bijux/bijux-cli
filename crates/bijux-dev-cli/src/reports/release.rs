//! Release truth control-plane reports.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{read_json_if_exists, read_text_if_exists};

fn ensure_array_report_key(mut payload: Value, key: &str) -> Value {
    if payload.get(key).is_some() {
        return payload;
    }
    if let Some(obj) = payload.as_object_mut() {
        obj.insert(key.to_string(), json!([]));
    } else {
        payload = json!({ key: [] });
    }
    payload
}

/// `dev cli release status`
#[must_use]
pub fn build_status_report(workspace_root: &Path) -> Value {
    let manifest =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let truth =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_truth_report.json"));
    let bundle =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_evidence_bundle.json"));
    json!({
        "release_status_manifest": manifest,
        "release_truth": truth,
        "release_evidence_bundle": bundle,
        "source_of_truth": "dev cli release *",
    })
}

/// `dev cli release evidence`
#[must_use]
pub fn build_evidence_report(workspace_root: &Path) -> Value {
    let bundle =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_evidence_bundle.json"));
    let truth =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_truth_report.json"));
    let text =
        read_text_if_exists(&workspace_root.join("artifacts/status/release_truth_report.txt"));
    json!({
        "bundle": bundle,
        "truth": truth,
        "truth_text": text,
    })
}

/// `dev cli release readiness`
#[must_use]
pub fn build_readiness_report(workspace_root: &Path) -> Value {
    let manifest =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let status = manifest.get("status").and_then(Value::as_str).unwrap_or("blocked");
    let checks = manifest.get("checks").cloned().unwrap_or_else(|| json!({}));
    json!({
        "status": status,
        "checks": checks,
        "release_ready": status == "ready",
    })
}

/// `dev cli release diff`
#[must_use]
pub fn build_diff_report(workspace_root: &Path) -> Value {
    let done = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_done.json"));
    let partial =
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_partial.json"));
    let left = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json"));
    let intentional = read_json_if_exists(
        &workspace_root.join("artifacts/status/what_is_intentionally_different.json"),
    );
    json!({
        "done": done,
        "partial": partial,
        "left": left,
        "intentional_differences": intentional,
    })
}

/// `dev cli release gaps`
#[must_use]
pub fn build_gaps_report(workspace_root: &Path) -> Value {
    let manifest =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let left = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json"));
    let missing = manifest
        .get("checks")
        .and_then(|checks| checks.get("missing_evidence"))
        .cloned()
        .unwrap_or_else(|| json!([]));
    let unresolved = left.get("items").cloned().unwrap_or_else(|| json!([]));
    json!({
        "missing_evidence": missing,
        "unresolved_gaps": unresolved,
        "status": manifest.get("status").cloned().unwrap_or_else(|| json!("blocked")),
    })
}

/// `dev cli release summary`
#[must_use]
pub fn build_summary_report(workspace_root: &Path) -> Value {
    let readiness = build_readiness_report(workspace_root);
    let diff = build_diff_report(workspace_root);
    json!({
        "readiness": readiness,
        "highlights": {
            "done": diff.get("done").cloned().unwrap_or_else(|| json!({})),
            "partial": diff.get("partial").cloned().unwrap_or_else(|| json!({})),
            "left": diff.get("left").cloned().unwrap_or_else(|| json!({})),
        }
    })
}

/// `dev cli release manifest`
#[must_use]
pub fn build_manifest_report(workspace_root: &Path) -> Value {
    read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"))
}

/// `dev cli release notes`
#[must_use]
pub fn build_notes_report(workspace_root: &Path) -> Value {
    let truth =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_truth_report.json"));
    let text =
        read_text_if_exists(&workspace_root.join("artifacts/status/release_truth_report.txt"));
    json!({
        "generated_notes": text,
        "source": truth,
    })
}

/// `dev cli release behavior-changes`
#[must_use]
pub fn build_behavior_changes_report(workspace_root: &Path) -> Value {
    ensure_array_report_key(
        read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json")),
        "commands",
    )
}

/// `dev cli release intentional-differences`
#[must_use]
pub fn build_intentional_differences_report(workspace_root: &Path) -> Value {
    ensure_array_report_key(
        read_json_if_exists(
            &workspace_root.join("artifacts/status/what_is_intentionally_different.json"),
        ),
        "items",
    )
}

/// `dev cli release unresolved-gaps`
#[must_use]
pub fn build_unresolved_gaps_report(workspace_root: &Path) -> Value {
    ensure_array_report_key(
        read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json")),
        "items",
    )
}

/// `dev cli release compatibility-leftovers`
#[must_use]
pub fn build_compatibility_leftovers_report(workspace_root: &Path) -> Value {
    ensure_array_report_key(
        read_json_if_exists(
            &workspace_root.join("artifacts/status/compatibility_debt_trend_report.json"),
        ),
        "series",
    )
}
