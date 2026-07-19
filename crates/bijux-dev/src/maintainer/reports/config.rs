//! Config ownership and drift reports for maintainer control-plane workflows.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{json_artifact_state, read_json_if_exists};

fn ownership_report(workspace_root: &Path) -> Value {
    let path = workspace_root.join("artifacts/status/config_ownership_truth.json");
    let payload = read_json_if_exists(&path);
    let state = json_artifact_state(&payload);
    if state != "valid" {
        json!({
            "artifact_state": state,
            "owners": {
                "rust": ["crates/bijux-cli"],
                "python": ["crates/bijux-cli-python"]
            },
            "schemas": [],
            "compatibility_shims": [],
            "sources": [],
            "precedence_proofs": [],
            "rollback_proofs": [],
            "corruption_evidence": [],
        })
    } else {
        payload
    }
}

/// `bijux-dev-cli config rust-owner`
#[must_use]
pub fn build_rust_owner_report(workspace_root: &Path) -> Value {
    let report = ownership_report(workspace_root);
    json!({
        "rust_owner": report.get("owners").and_then(|v| v.get("rust")).cloned().unwrap_or_else(|| json!([])),
        "source": "config ownership truth"
    })
}

/// `bijux-dev-cli config python-owner`
#[must_use]
pub fn build_python_owner_report(workspace_root: &Path) -> Value {
    let report = ownership_report(workspace_root);
    json!({
        "python_owner": report.get("owners").and_then(|v| v.get("python")).cloned().unwrap_or_else(|| json!([])),
        "source": "config ownership truth"
    })
}

/// `bijux-dev-cli config ownership`
#[must_use]
pub fn build_ownership_report(workspace_root: &Path) -> Value {
    ownership_report(workspace_root)
}

/// `bijux-dev-cli config drift`
#[must_use]
pub fn build_drift_report(workspace_root: &Path) -> Value {
    let report = ownership_report(workspace_root);
    let rust = report
        .get("owners")
        .and_then(|v| v.get("rust"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let python = report
        .get("owners")
        .and_then(|v| v.get("python"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    json!({
        "status": if rust.is_empty() { "blocked" } else { "pass" },
        "drift": {
            "missing_rust_owner": rust.is_empty(),
            "python_shim_count": report.get("compatibility_shims").and_then(Value::as_array).map_or(0, |rows| rows.len()),
            "python_owner_count": python.len(),
        }
    })
}

/// `bijux-dev-cli config shape`
#[must_use]
pub fn build_shape_report(workspace_root: &Path) -> Value {
    let report = ownership_report(workspace_root);
    json!({
        "owners": report.get("owners").cloned().unwrap_or_else(|| json!({})),
        "schemas": report.get("schemas").cloned().unwrap_or_else(|| json!([])),
        "sources": report.get("sources").cloned().unwrap_or_else(|| json!([])),
        "precedence_proofs": report.get("precedence_proofs").cloned().unwrap_or_else(|| json!([])),
        "rollback_proofs": report.get("rollback_proofs").cloned().unwrap_or_else(|| json!([])),
        "corruption_evidence": report.get("corruption_evidence").cloned().unwrap_or_else(|| json!([])),
    })
}

/// `bijux-dev-cli config evidence-map`
#[must_use]
pub fn build_evidence_map_report(workspace_root: &Path) -> Value {
    let report = ownership_report(workspace_root);
    let evidence_ids = vec![
        "EVIDENCE-1201-CONFIG-OWNERSHIP".to_string(),
        "EVIDENCE-1202-CONFIG-PRECEDENCE".to_string(),
        "EVIDENCE-1203-CONFIG-CORRUPTION".to_string(),
    ];
    json!({
        "config_behaviors": {
            "ownership": report.get("owners").cloned().unwrap_or_else(|| json!({})),
            "precedence_proofs": report.get("precedence_proofs").cloned().unwrap_or_else(|| json!([])),
            "corruption_evidence": report.get("corruption_evidence").cloned().unwrap_or_else(|| json!([])),
        },
        "evidence_ids": evidence_ids,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        build_drift_report, build_evidence_map_report, build_ownership_report,
        build_python_owner_report, build_rust_owner_report, build_shape_report,
    };

    #[test]
    fn config_reports_expose_stable_top_level_keys() {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(build_rust_owner_report(&workspace_root).get("rust_owner").is_some());
        assert!(build_python_owner_report(&workspace_root).get("python_owner").is_some());
        assert!(build_ownership_report(&workspace_root).get("owners").is_some());
        assert!(build_drift_report(&workspace_root).get("drift").is_some());
        assert!(build_shape_report(&workspace_root).get("schemas").is_some());
        assert!(build_evidence_map_report(&workspace_root).get("evidence_ids").is_some());
    }

    #[test]
    fn drift_report_detects_missing_rust_owner() {
        let temp_root = std::env::temp_dir().join(format!(
            "bijux-config-drift-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(temp_root.join("artifacts/status")).expect("mkdir");
        fs::write(
            temp_root.join("artifacts/status/config_ownership_truth.json"),
            r#"{"owners":{"rust":[],"python":["crates/bijux-cli-python"]}}"#,
        )
        .expect("write truth");
        let report = build_drift_report(&temp_root);
        assert_eq!(report["status"], "blocked");
        assert_eq!(report["drift"]["missing_rust_owner"], true);
    }

    #[test]
    fn ownership_and_shape_share_same_source_of_truth() {
        let temp_root = std::env::temp_dir().join(format!(
            "bijux-config-source-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(temp_root.join("artifacts/status")).expect("mkdir");
        fs::write(
            temp_root.join("artifacts/status/config_ownership_truth.json"),
            r#"{"owners":{"rust":["crates/bijux-cli"],"python":["crates/bijux-cli-python"]},"schemas":["config-v1"],"sources":["core::config"],"precedence_proofs":["artifacts/status/config_source_precedence_contract.json"],"rollback_proofs":["artifacts/status/config_mutation_coverage_artifact.json"],"corruption_evidence":["artifacts/status/config_corruption_campaign_artifact.json"],"compatibility_shims":[]}"#,
        )
        .expect("write truth");

        let ownership = build_ownership_report(&temp_root);
        let shape = build_shape_report(&temp_root);
        assert_eq!(ownership["schemas"], shape["schemas"]);
        assert_eq!(ownership["sources"], shape["sources"]);
    }
}
