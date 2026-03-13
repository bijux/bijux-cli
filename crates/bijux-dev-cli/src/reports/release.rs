//! Release truth control-plane reports.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{json_artifact_state, read_json_if_exists, read_text_if_exists};

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

fn with_artifact_integrity(mut payload: Value, key: &str, state: &str) -> Value {
    if !payload.is_object() {
        return json!({
            "payload": payload,
            "artifact_integrity": {
                key: state,
            },
        });
    }
    if let Some(obj) = payload.as_object_mut() {
        let mut integrity = obj.remove("artifact_integrity").unwrap_or_else(|| json!({}));
        if !integrity.is_object() {
            integrity = json!({});
        }
        if let Some(integrity_obj) = integrity.as_object_mut() {
            integrity_obj.insert(key.to_string(), Value::String(state.to_string()));
        }
        obj.insert("artifact_integrity".to_string(), integrity);
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
    let manifest_state = json_artifact_state(&manifest).to_string();
    let truth_state = json_artifact_state(&truth).to_string();
    let bundle_state = json_artifact_state(&bundle).to_string();
    json!({
        "release_status_manifest": manifest,
        "release_truth": truth,
        "release_evidence_bundle": bundle,
        "artifact_integrity": {
            "release_status_manifest": manifest_state,
            "release_truth_report": truth_state,
            "release_evidence_bundle": bundle_state,
        },
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
    let bundle_state = json_artifact_state(&bundle).to_string();
    let truth_state = json_artifact_state(&truth).to_string();
    json!({
        "bundle": bundle,
        "truth": truth,
        "truth_text": text,
        "artifact_integrity": {
            "release_evidence_bundle": bundle_state,
            "release_truth_report": truth_state,
        },
    })
}

/// `dev cli release readiness`
#[must_use]
pub fn build_readiness_report(workspace_root: &Path) -> Value {
    let manifest =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let status = manifest.get("status").and_then(Value::as_str).unwrap_or("blocked");
    let checks = manifest.get("checks").cloned().unwrap_or_else(|| json!({}));
    let manifest_state = json_artifact_state(&manifest).to_string();
    json!({
        "status": status,
        "checks": checks,
        "release_ready": status == "ready",
        "artifact_integrity": {
            "release_status_manifest": manifest_state,
        },
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
    let done_state = json_artifact_state(&done).to_string();
    let partial_state = json_artifact_state(&partial).to_string();
    let left_state = json_artifact_state(&left).to_string();
    let intentional_state = json_artifact_state(&intentional).to_string();
    json!({
        "done": done,
        "partial": partial,
        "left": left,
        "intentional_differences": intentional,
        "artifact_integrity": {
            "what_is_done": done_state,
            "what_is_partial": partial_state,
            "what_is_left": left_state,
            "what_is_intentionally_different": intentional_state,
        },
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
    let manifest_state = json_artifact_state(&manifest).to_string();
    let left_state = json_artifact_state(&left).to_string();
    json!({
        "missing_evidence": missing,
        "unresolved_gaps": unresolved,
        "status": manifest.get("status").cloned().unwrap_or_else(|| json!("blocked")),
        "artifact_integrity": {
            "release_status_manifest": manifest_state,
            "what_is_left": left_state,
        },
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
        },
        "artifact_integrity": {
            "readiness": readiness
                .get("artifact_integrity")
                .cloned()
                .unwrap_or_else(|| json!({})),
            "diff": diff
                .get("artifact_integrity")
                .cloned()
                .unwrap_or_else(|| json!({})),
        },
    })
}

/// `dev cli release manifest`
#[must_use]
pub fn build_manifest_report(workspace_root: &Path) -> Value {
    let manifest =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_status_manifest.json"));
    let state = json_artifact_state(&manifest).to_string();
    with_artifact_integrity(manifest, "release_status_manifest", &state)
}

/// `dev cli release notes`
#[must_use]
pub fn build_notes_report(workspace_root: &Path) -> Value {
    let truth =
        read_json_if_exists(&workspace_root.join("artifacts/status/release_truth_report.json"));
    let text =
        read_text_if_exists(&workspace_root.join("artifacts/status/release_truth_report.txt"));
    let truth_state = json_artifact_state(&truth).to_string();
    json!({
        "generated_notes": text,
        "source": truth,
        "artifact_integrity": {
            "release_truth_report": truth_state,
        },
    })
}

/// `dev cli release behavior-changes`
#[must_use]
pub fn build_behavior_changes_report(workspace_root: &Path) -> Value {
    let payload =
        read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json"));
    let state = json_artifact_state(&payload).to_string();
    let payload = ensure_array_report_key(payload, "commands");
    with_artifact_integrity(payload, "command_migration_matrix", &state)
}

/// `dev cli release intentional-differences`
#[must_use]
pub fn build_intentional_differences_report(workspace_root: &Path) -> Value {
    let payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/what_is_intentionally_different.json"),
    );
    let state = json_artifact_state(&payload).to_string();
    let payload = ensure_array_report_key(payload, "items");
    with_artifact_integrity(payload, "what_is_intentionally_different", &state)
}

/// `dev cli release unresolved-gaps`
#[must_use]
pub fn build_unresolved_gaps_report(workspace_root: &Path) -> Value {
    let payload = read_json_if_exists(&workspace_root.join("artifacts/status/what_is_left.json"));
    let state = json_artifact_state(&payload).to_string();
    let payload = ensure_array_report_key(payload, "items");
    with_artifact_integrity(payload, "what_is_left", &state)
}

/// `dev cli release compatibility-leftovers`
#[must_use]
pub fn build_compatibility_leftovers_report(workspace_root: &Path) -> Value {
    let payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/compatibility_debt_trend_report.json"),
    );
    let state = json_artifact_state(&payload).to_string();
    let payload = ensure_array_report_key(payload, "series");
    with_artifact_integrity(payload, "compatibility_debt_trend_report", &state)
}
