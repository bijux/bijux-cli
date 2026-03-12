//! Python bridge surface and sovereignty audits for maintainer control-plane workflows.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{json_artifact_state, read_json_if_exists};

fn fallback_report(payload: Value, source: &str) -> Value {
    let state = json_artifact_state(&payload);
    if state != "valid" {
        json!({
            "status": "unavailable",
            "source": source,
            "artifact_state": state,
        })
    } else {
        payload
    }
}

fn duplication_list(payload: &Value, key: &str) -> Vec<Value> {
    payload.get(key).and_then(Value::as_array).cloned().unwrap_or_default()
}

fn bridge_duplicate_area(payload: &Value, area: &str) -> Vec<Value> {
    payload
        .get("checks")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter(|row| row.get("area").and_then(Value::as_str) == Some(area))
                .flat_map(|row| {
                    row.get("duplicate_rules")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// `dev cli python bridge-status`
#[must_use]
pub fn build_bridge_status_report(workspace_root: &Path) -> Value {
    let execution_payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/python_bridge_execution_artifact.json"),
    );
    let conversion_payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/bridge_conversion_artifact.json"),
    );
    let drift_payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/python_bridge_drift_artifact.json"),
    );
    let execution_state = json_artifact_state(&execution_payload).to_string();
    let conversion_state = json_artifact_state(&conversion_payload).to_string();
    let drift_state = json_artifact_state(&drift_payload).to_string();
    let execution = fallback_report(
        execution_payload,
        "artifacts/status/python_bridge_execution_artifact.json",
    );
    let conversion =
        fallback_report(conversion_payload, "artifacts/status/bridge_conversion_artifact.json");
    let drift =
        fallback_report(drift_payload, "artifacts/status/python_bridge_drift_artifact.json");
    json!({
        "bridge_status": {
            "execution": execution,
            "conversion": conversion,
            "drift": drift,
        },
        "artifact_integrity": {
            "execution": execution_state,
            "conversion": conversion_state,
            "drift": drift_state,
        },
        "runtime_direction": "python-surface-over-rust-core",
    })
}

/// `dev cli python surface-status`
#[must_use]
pub fn build_surface_status_report(workspace_root: &Path) -> Value {
    let command_surface_payload = read_json_if_exists(
        &workspace_root.join("artifacts/status/python_path_command_inventory.json"),
    );
    let command_surface_state = json_artifact_state(&command_surface_payload).to_string();
    let command_surface = fallback_report(
        command_surface_payload,
        "artifacts/status/python_path_command_inventory.json",
    );
    json!({
        "surface_status": command_surface,
        "artifact_integrity": {
            "surface_status": command_surface_state,
        },
        "python_role": "surface-and-bridge",
    })
}

/// `dev cli python sovereignty-audit`
#[must_use]
pub fn build_sovereignty_audit_report(workspace_root: &Path) -> Value {
    let duplication = read_json_if_exists(
        &workspace_root.join("artifacts/status/python_duplicate_law_report.json"),
    );
    let bridge_duplication = read_json_if_exists(
        &workspace_root.join("artifacts/status/bridge_duplicate_law_report.json"),
    );

    let command_law_duplication = {
        let direct = duplication_list(&duplication, "command_law_duplication");
        if direct.is_empty() {
            bridge_duplicate_area(&bridge_duplication, "routing")
        } else {
            direct
        }
    };
    let output_law_duplication = {
        let direct = duplication_list(&duplication, "output_law_duplication");
        if direct.is_empty() {
            bridge_duplicate_area(&bridge_duplication, "output_shaping")
        } else {
            direct
        }
    };
    let exit_law_duplication = {
        let direct = duplication_list(&duplication, "exit_law_duplication");
        if direct.is_empty() {
            bridge_duplicate_area(&bridge_duplication, "exit_mapping")
        } else {
            direct
        }
    };
    let route_law_duplication = {
        let direct = duplication_list(&duplication, "route_law_duplication");
        if direct.is_empty() {
            bridge_duplicate_area(&bridge_duplication, "namespace_validation")
        } else {
            direct
        }
    };
    let state_law_duplication = duplication_list(&duplication, "state_law_duplication");

    let total_duplication = command_law_duplication.len()
        + output_law_duplication.len()
        + exit_law_duplication.len()
        + route_law_duplication.len()
        + state_law_duplication.len();

    json!({
        "status": if total_duplication == 0 { "green" } else { "needs-work" },
        "python_sovereignty_audit": {
            "python_behaviors_still_sovereign": duplication.get("python_behaviors_still_sovereign").cloned().unwrap_or_else(|| json!([])),
            "python_behaviors_delegated_to_rust": duplication.get("python_behaviors_delegated_to_rust").cloned().unwrap_or_else(|| json!([])),
            "command_law_duplication": command_law_duplication,
            "output_law_duplication": output_law_duplication,
            "exit_law_duplication": exit_law_duplication,
            "route_law_duplication": route_law_duplication,
            "state_law_duplication": state_law_duplication,
            "duplication_total": total_duplication,
        },
        "evidence_ids": ["EVIDENCE-1501-PYTHON-SURFACE-ONLY"],
        "artifact_integrity": {
            "python_duplicate_law_report": json_artifact_state(&duplication),
            "bridge_duplicate_law_report": json_artifact_state(&bridge_duplication),
        },
        "direction_contract": "python-surface-over-rust-core",
    })
}

/// `dev cli python drift`
#[must_use]
pub fn build_drift_report(workspace_root: &Path) -> Value {
    let sovereignty = build_sovereignty_audit_report(workspace_root);
    let duplication_total =
        sovereignty["python_sovereignty_audit"]["duplication_total"].as_u64().unwrap_or(0);
    json!({
        "drift": {
            "duplication_total": duplication_total,
            "status": if duplication_total == 0 { "clean" } else { "drift" },
        }
    })
}

/// `dev cli python packaging`
#[must_use]
pub fn build_packaging_report(workspace_root: &Path) -> Value {
    let runtime_identity = read_json_if_exists(
        &workspace_root.join("artifacts/status/install_runtime_identity_report.json"),
    );
    let runtime_identity_state = json_artifact_state(&runtime_identity).to_string();
    json!({
        "packaging": {
            "linkage_model": "polars-like-thin-python-surface",
            "python_package_role": "bridge-only",
            "rust_core_role": "source-of-truth",
            "runtime_identity": runtime_identity,
        },
        "artifact_integrity": {
            "runtime_identity": runtime_identity_state,
        },
        "packaging_evidence_ids": ["EVIDENCE-1502-PYTHON-PACKAGING-DIRECTION"],
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::build_sovereignty_audit_report;

    #[test]
    fn sovereignty_audit_reaches_zero_when_duplication_is_zero() {
        let root = std::env::temp_dir().join(format!(
            "bijux-python-sovereignty-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(root.join("artifacts/status")).expect("mkdir");
        fs::write(
            root.join("artifacts/status/python_duplicate_law_report.json"),
            r#"{
              "command_law_duplication": [],
              "output_law_duplication": [],
              "exit_law_duplication": [],
              "route_law_duplication": [],
              "state_law_duplication": [],
              "python_behaviors_still_sovereign": [],
              "python_behaviors_delegated_to_rust": ["status", "doctor"]
            }"#,
        )
        .expect("write");
        let report = build_sovereignty_audit_report(&root);
        assert_eq!(report["status"], "green");
        assert_eq!(report["python_sovereignty_audit"]["duplication_total"], 0);
    }
}
