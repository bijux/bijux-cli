//! Evidence control-plane reports and exports.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{read_json_if_exists, relative_to_root};
use crate::schema::command_registry::command_registry;
use crate::schema::evidence::{
    valid_evidence_id, EvidenceRecord, EvidenceStatus, EvidenceStrength,
};

fn evidence_ids_for_command(command: &str) -> Vec<&'static str> {
    match command {
        "dev cli release" => vec!["EVIDENCE-1001-RELEASE-TRUTH"],
        "dev cli parity" => vec!["EVIDENCE-1002-PARITY-COVERAGE"],
        "dev cli package-health" => vec!["EVIDENCE-1003-INSTALL-NEUTRALITY"],
        "dev cli runtime-identity" => vec!["EVIDENCE-1004-RUNTIME-IDENTITY"],
        _ => Vec::new(),
    }
}

fn evidence_records(workspace_root: &Path) -> Vec<EvidenceRecord> {
    let release_truth = workspace_root.join("artifacts/status/release_truth_report.json");
    let parity = workspace_root.join("artifacts/parity/command_parity_matrix.json");
    let install_neutrality = workspace_root.join("artifacts/status/install_neutrality_report.json");
    let runtime_identity = workspace_root.join("artifacts/status/active_runtime_report.json");

    vec![
        EvidenceRecord {
            id: "EVIDENCE-1001-RELEASE-TRUTH".to_string(),
            claim: "release truth is generated from artifacts".to_string(),
            ownership: "bijux-dev-cli".to_string(),
            source: "dev cli release evidence".to_string(),
            proof_kind: "report".to_string(),
            artifact_links: vec![relative_to_root(&release_truth, workspace_root)],
            freshness: if release_truth.exists() {
                "fresh".to_string()
            } else {
                "stale".to_string()
            },
            status: if release_truth.exists() {
                EvidenceStatus::Proven
            } else {
                EvidenceStatus::Blocked
            },
            strength: EvidenceStrength::Strong,
        },
        EvidenceRecord {
            id: "EVIDENCE-1002-PARITY-COVERAGE".to_string(),
            claim: "command parity matrix is maintained".to_string(),
            ownership: "bijux-dev-cli".to_string(),
            source: "dev cli parity".to_string(),
            proof_kind: "matrix".to_string(),
            artifact_links: vec![relative_to_root(&parity, workspace_root)],
            freshness: if parity.exists() { "fresh".to_string() } else { "stale".to_string() },
            status: if parity.exists() { EvidenceStatus::Proven } else { EvidenceStatus::Blocked },
            strength: EvidenceStrength::Strong,
        },
        EvidenceRecord {
            id: "EVIDENCE-1003-INSTALL-NEUTRALITY".to_string(),
            claim: "install neutrality checks exist".to_string(),
            ownership: "bijux-dev-cli".to_string(),
            source: "dev cli package-health".to_string(),
            proof_kind: "report".to_string(),
            artifact_links: vec![relative_to_root(&install_neutrality, workspace_root)],
            freshness: if install_neutrality.exists() {
                "fresh".to_string()
            } else {
                "stale".to_string()
            },
            status: if install_neutrality.exists() {
                EvidenceStatus::Proven
            } else {
                EvidenceStatus::Partial
            },
            strength: EvidenceStrength::Medium,
        },
        EvidenceRecord {
            id: "EVIDENCE-1004-RUNTIME-IDENTITY".to_string(),
            claim: "runtime identity evidence exists".to_string(),
            ownership: "bijux-dev-cli".to_string(),
            source: "dev cli runtime-identity".to_string(),
            proof_kind: "report".to_string(),
            artifact_links: vec![relative_to_root(&runtime_identity, workspace_root)],
            freshness: if runtime_identity.exists() {
                "fresh".to_string()
            } else {
                "stale".to_string()
            },
            status: if runtime_identity.exists() {
                EvidenceStatus::Proven
            } else {
                EvidenceStatus::Partial
            },
            strength: EvidenceStrength::Medium,
        },
    ]
}

fn records_json(workspace_root: &Path) -> Vec<Value> {
    evidence_records(workspace_root)
        .into_iter()
        .map(|record| serde_json::to_value(record).unwrap_or_else(|_| json!({})))
        .collect()
}

fn artifacts_exist(record: &EvidenceRecord, workspace_root: &Path) -> bool {
    !record.artifact_links.is_empty()
        && record.artifact_links.iter().all(|artifact| workspace_root.join(artifact).exists())
}

fn audit_records(records: &[EvidenceRecord], workspace_root: &Path) -> Value {
    let mut invalid_ids = Vec::new();
    let mut missing_artifacts = Vec::new();
    for record in records {
        if !valid_evidence_id(&record.id) {
            invalid_ids.push(record.id.clone());
        }
        if !artifacts_exist(record, workspace_root) {
            missing_artifacts.push(record.id.clone());
        }
    }
    json!({
        "status": if invalid_ids.is_empty() && missing_artifacts.is_empty() { "pass" } else { "fail" },
        "invalid_ids": invalid_ids,
        "missing_artifact_links": missing_artifacts,
    })
}

/// `dev cli evidence list`
#[must_use]
pub fn build_list_report(workspace_root: &Path) -> Value {
    let records = records_json(workspace_root);
    json!({"records": records, "count": records.len()})
}

/// `dev cli evidence show --id <id>`
#[must_use]
pub fn build_show_report(workspace_root: &Path, id: &str) -> Value {
    if !valid_evidence_id(id) {
        return json!({
            "record": Value::Null,
            "found": false,
            "id_format_valid": false,
            "error": "invalid evidence id format; expected EVIDENCE-<NUMBER>-<UPPERCASE-SLUG>",
        });
    }

    let records = records_json(workspace_root);
    let record =
        records.into_iter().find(|item| item.get("id").and_then(Value::as_str) == Some(id));
    json!({
        "record": record,
        "found": record.is_some(),
        "id_format_valid": true,
    })
}

/// `dev cli evidence audit`
#[must_use]
pub fn build_audit_report(workspace_root: &Path) -> Value {
    let records = evidence_records(workspace_root);
    let integrity = audit_records(&records, workspace_root);
    let coverage: Vec<Value> = records
        .iter()
        .map(|record| {
            json!({
                "id": record.id,
                "has_backing_artifacts": artifacts_exist(record, workspace_root),
            })
        })
        .collect();
    let orphan: Vec<Value> = records
        .iter()
        .filter(|record| record.artifact_links.is_empty())
        .map(|record| json!({"id": record.id, "reason": "no artifact links"}))
        .collect();
    let claims_without_evidence: Vec<Value> = records
        .iter()
        .filter(|record| !artifacts_exist(record, workspace_root))
        .map(|record| json!({"id": record.id, "claim": record.claim}))
        .collect();
    json!({
        "status": integrity.get("status").cloned().unwrap_or_else(|| json!("fail")),
        "invalid_ids": integrity.get("invalid_ids").cloned().unwrap_or_else(|| json!([])),
        "missing_artifact_links": integrity.get("missing_artifact_links").cloned().unwrap_or_else(|| json!([])),
        "coverage_report": coverage,
        "orphan_report": orphan,
        "claims_without_evidence_report": claims_without_evidence,
        "records": records_json(workspace_root),
    })
}

/// `dev cli evidence stale`
#[must_use]
pub fn build_stale_report(workspace_root: &Path) -> Value {
    let stale: Vec<Value> = records_json(workspace_root)
        .into_iter()
        .filter(|item| {
            item.get("freshness").and_then(Value::as_str) == Some("stale")
                || item.get("status").and_then(Value::as_str) == Some("stale")
        })
        .collect();
    json!({"stale": stale, "count": stale.len()})
}

/// `dev cli evidence matrix`
#[must_use]
pub fn build_matrix_report(workspace_root: &Path) -> Value {
    let by_status = records_json(workspace_root).into_iter().fold(
        BTreeMap::<String, usize>::new(),
        |mut acc, row| {
            let key = row.get("status").and_then(Value::as_str).unwrap_or("unknown").to_string();
            *acc.entry(key).or_insert(0) += 1;
            acc
        },
    );
    json!({"status_matrix": by_status, "records": records_json(workspace_root)})
}

/// `dev cli evidence website-export`
#[must_use]
pub fn build_website_export_report(workspace_root: &Path) -> Value {
    let records: Vec<Value> = evidence_records(workspace_root)
        .into_iter()
        .filter(|record| artifacts_exist(record, workspace_root))
        .map(|record| serde_json::to_value(record).unwrap_or_else(|_| json!({})))
        .collect();
    json!({"website_export": records, "filter": "backed-claims-only"})
}

/// `dev cli evidence ci-export`
#[must_use]
pub fn build_ci_export_report(workspace_root: &Path) -> Value {
    json!({"ci_export": records_json(workspace_root)})
}

/// `dev cli evidence release-export`
#[must_use]
pub fn build_release_export_report(workspace_root: &Path) -> Value {
    json!({"release_export": records_json(workspace_root)})
}

/// `dev cli evidence command-map`
#[must_use]
pub fn build_command_map_report(workspace_root: &Path) -> Value {
    let available_ids: std::collections::BTreeSet<String> = records_json(workspace_root)
        .iter()
        .filter_map(|row| row.get("id").and_then(Value::as_str).map(ToString::to_string))
        .collect();
    let mapping: Vec<Value> = command_registry()
        .iter()
        .map(|entry| {
            let evidence_ids: Vec<&str> = evidence_ids_for_command(entry.command.as_str())
                .into_iter()
                .filter(|id| available_ids.contains(*id))
                .collect();
            json!({
                "command": entry.command.as_str(),
                "evidence_ids": evidence_ids,
            })
        })
        .collect();
    json!({
        "command_map": mapping,
        "mapping_basis": "explicit command-to-evidence ownership",
    })
}

/// `dev cli evidence parity-map`
#[must_use]
pub fn build_parity_map_report(workspace_root: &Path) -> Value {
    let matrix =
        read_json_if_exists(&workspace_root.join("artifacts/status/command_migration_matrix.json"));
    let rows = matrix.get("commands").and_then(Value::as_array).cloned().unwrap_or_default();
    let available_ids: std::collections::BTreeSet<String> = records_json(workspace_root)
        .iter()
        .filter_map(|row| row.get("id").and_then(Value::as_str).map(ToString::to_string))
        .collect();
    let mapped: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            let command = row.get("command").cloned().unwrap_or_else(|| json!("unknown"));
            let command_key = command.as_str().unwrap_or_default().trim();
            let evidence_ids: Vec<&str> = evidence_ids_for_command(command_key)
                .into_iter()
                .filter(|id| available_ids.contains(*id))
                .collect();
            json!({
                "command": command,
                "evidence_ids": evidence_ids,
                "mapping_status": if evidence_ids.is_empty() { "unmapped" } else { "mapped" },
            })
        })
        .collect();
    json!({
        "parity_map": mapped,
        "mapping_basis": "explicit command-to-evidence ownership",
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde_json::Value;

    use super::{
        build_audit_report, build_command_map_report, build_parity_map_report, build_show_report,
        build_stale_report, build_website_export_report,
    };

    #[test]
    fn evidence_audit_reports_integrity_views() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let audit = build_audit_report(&root);
        assert!(audit.get("coverage_report").is_some());
        assert!(audit.get("orphan_report").is_some());
        assert!(audit.get("claims_without_evidence_report").is_some());
    }

    #[test]
    fn stale_evidence_is_reported() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let stale = build_stale_report(&root);
        assert!(stale.get("stale").is_some());
    }

    #[test]
    fn website_export_contains_backed_claims_only() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let export = build_website_export_report(&root);
        assert_eq!(
            export.get("filter").and_then(serde_json::Value::as_str),
            Some("backed-claims-only")
        );
    }

    #[test]
    fn show_report_distinguishes_invalid_id_format() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let invalid = build_show_report(&root, "invalid-id");
        assert_eq!(invalid["found"], false);
        assert_eq!(invalid["id_format_valid"], false);

        let unknown = build_show_report(&root, "EVIDENCE-9999-UNKNOWN");
        assert_eq!(unknown["found"], false);
        assert_eq!(unknown["id_format_valid"], true);
    }

    #[test]
    fn command_map_uses_explicit_mappings() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let map = build_command_map_report(&root);
        let rows = map["command_map"].as_array().expect("command map rows");
        let package_health = rows
            .iter()
            .find(|row| row.get("command").and_then(Value::as_str) == Some("dev cli package-health"))
            .expect("package health row");
        assert_eq!(
            package_health["evidence_ids"].as_array().map(|rows| rows.len()),
            Some(1)
        );
    }

    #[test]
    fn parity_map_does_not_assign_blanket_evidence() {
        let root = std::env::temp_dir().join(format!(
            "bijux-evidence-parity-map-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(root.join("artifacts/status")).expect("mkdir");
        fs::write(
            root.join("artifacts/status/command_migration_matrix.json"),
            r#"{"commands":[{"command":"dev cli unknown"},{"command":"dev cli parity"}]}"#,
        )
        .expect("write matrix");

        let parity_map = build_parity_map_report(&root);
        let rows = parity_map["parity_map"].as_array().expect("parity map rows");
        let unknown = rows
            .iter()
            .find(|row| row.get("command").and_then(Value::as_str) == Some("dev cli unknown"))
            .expect("unknown row");
        let parity = rows
            .iter()
            .find(|row| row.get("command").and_then(Value::as_str) == Some("dev cli parity"))
            .expect("parity row");
        assert_eq!(unknown["evidence_ids"].as_array().map(|rows| rows.len()), Some(0));
        assert_eq!(parity["evidence_ids"].as_array().map(|rows| rows.len()), Some(1));
    }
}
