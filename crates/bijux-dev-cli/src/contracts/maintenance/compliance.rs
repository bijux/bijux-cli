use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use super::build_status_contracts_report;
use super::generators::build_generators_report;
use super::inventory::{collect_files, generated_at_utc, parse_make_targets, rel};

const MIGRATION_CONTROLS: [(&str, &str, &str, usize); 4] = [
    (
        "CTRL-ROOT-MAINTENANCE-DIRECTORY-REMOVED",
        "maintenance",
        "bijux dev cli maintenance audit",
        100,
    ),
    (
        "CTRL-ROOT-TARGET-DIRECTORY-REMOVED",
        "target",
        "artifacts/rust/target via CARGO_TARGET_DIR",
        95,
    ),
    (
        "CTRL-GITHUB-LEGACY-MAINTENANCE-FILE-REMOVED",
        ".github/maintenance_additions_allowlist.txt",
        "bijux dev cli maintenance compliance hard-rules",
        90,
    ),
    (
        "CTRL-CONFIG-SOURCE-TMP-RELOCATED",
        "target/tmp/config-source-reports",
        "artifacts/tmp/config-source-reports",
        85,
    ),
];

fn report_rows(report: &Value, source: &str, issues: &mut Vec<Value>) -> Vec<Value> {
    match report.get("rows") {
        Some(Value::Array(rows)) => rows.clone(),
        Some(_) => {
            issues.push(json!({
                "source": source,
                "error": "invalid-report-shape",
                "message": "expected `rows` to be an array",
            }));
            Vec::new()
        }
        None => {
            issues.push(json!({
                "source": source,
                "error": "missing-report-key",
                "message": "required key `rows` is missing",
            }));
            Vec::new()
        }
    }
}

fn required_non_empty_string(
    row: &Value,
    key: &str,
    source: &str,
    row_index: usize,
    issues: &mut Vec<Value>,
) -> Option<String> {
    match row.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => Some(value.to_string()),
        None => {
            issues.push(json!({
                "source": source,
                "row_index": row_index + 1,
                "field": key,
                "error": "missing-or-invalid-field",
                "message": format!("row is missing required non-empty string field `{key}`"),
            }));
            None
        }
    }
}

fn required_string_array(
    row: &Value,
    key: &str,
    source: &str,
    row_index: usize,
    issues: &mut Vec<Value>,
) -> Option<Vec<String>> {
    let Some(values) = row.get(key).and_then(Value::as_array) else {
        issues.push(json!({
            "source": source,
            "row_index": row_index + 1,
            "field": key,
            "error": "missing-or-invalid-field",
            "message": format!("row is missing required array field `{key}`"),
        }));
        return None;
    };

    let mut out = Vec::new();
    for (index, value) in values.iter().enumerate() {
        let Some(as_str) = value.as_str().map(str::trim).filter(|item| !item.is_empty()) else {
            issues.push(json!({
                "source": source,
                "row_index": row_index + 1,
                "field": key,
                "array_index": index,
                "error": "invalid-array-item",
                "message": format!("array `{key}` must contain only non-empty strings"),
            }));
            return None;
        };
        out.push(as_str.to_string());
    }
    Some(out)
}

fn migration_controls(workspace_root: &Path) -> Vec<Value> {
    MIGRATION_CONTROLS
        .iter()
        .map(|(control_id, from, replacement, rank)| {
            let path = workspace_root.join(from);
            let exists = path.exists();
            let status = if exists && *control_id == "CTRL-ROOT-TARGET-DIRECTORY-REMOVED" {
                "migrated"
            } else if exists {
                "remaining"
            } else {
                "removed"
            };
            json!({
                "control_id": control_id,
                "from": from,
                "replacement": replacement,
                "maintainer_value_rank": rank,
                "exists": exists,
                "status": status,
            })
        })
        .collect()
}

fn build_requirement_catalog(workspace_root: &Path) -> Value {
    let mut integrity_issues = Vec::<Value>::new();
    let status_report = build_status_contracts_report(workspace_root);
    let generator_report = build_generators_report(workspace_root);
    let status_rows = report_rows(&status_report, "status-contracts", &mut integrity_issues);
    let generator_rows = report_rows(&generator_report, "status-generators", &mut integrity_issues);

    let mut rows = Vec::<Value>::new();
    for (index, row) in status_rows.iter().enumerate() {
        let Some(contract_id) = required_non_empty_string(
            row,
            "contract_id",
            "status-contracts",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(kind) = required_non_empty_string(
            row,
            "kind",
            "status-contracts",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(implementation) = required_non_empty_string(
            row,
            "implementation",
            "status-contracts",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(command) = required_non_empty_string(
            row,
            "command",
            "status-contracts",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(outputs) =
            required_string_array(row, "outputs", "status-contracts", index, &mut integrity_issues)
        else {
            continue;
        };
        rows.push(json!({
            "requirement_id": format!("REQ-STATUS-{index:03}", index = index + 1),
            "domain": "STATUS",
            "source_kind": "status-contract",
            "contract_id": contract_id,
            "kind": kind,
            "implementation": implementation,
            "command": command,
            "outputs": outputs,
            "rule": "every status contract in inventory must remain executable through the maintenance control plane",
        }));
    }

    for (index, row) in generator_rows.iter().enumerate() {
        let Some(generator_id) = required_non_empty_string(
            row,
            "generator_id",
            "status-generators",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(implementation) = required_non_empty_string(
            row,
            "implementation",
            "status-generators",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(command) = required_non_empty_string(
            row,
            "command",
            "status-generators",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        let Some(outputs) = required_string_array(
            row,
            "outputs",
            "status-generators",
            index,
            &mut integrity_issues,
        ) else {
            continue;
        };
        rows.push(json!({
            "requirement_id": format!("REQ-GENERATOR-{index:03}", index = index + 1),
            "domain": "GENERATOR",
            "source_kind": "status-generator",
            "generator_id": generator_id,
            "implementation": implementation,
            "command": command,
            "outputs": outputs,
            "rule": "every status generator in inventory must remain runnable and produce declared artifacts",
        }));
    }

    let mut domains = BTreeMap::<String, usize>::new();
    for row in &rows {
        if let Some(domain) = row.get("domain").and_then(Value::as_str) {
            *domains.entry(domain.to_string()).or_insert(0) += 1;
        }
    }

    json!({
        "id_policy": "REQ-<DOMAIN>-<3DIGIT-INDEX>",
        "generated_at_utc": generated_at_utc(),
        "rows": rows,
        "count": domains.values().sum::<usize>(),
        "domains": domains,
        "rule": "requirements are derived from live maintenance inventories, not static placeholders",
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
    })
}

/// Builds `dev cli maintenance requirements` report payload.
#[must_use]
pub fn build_requirement_catalog_report(workspace_root: &Path) -> Value {
    build_requirement_catalog(workspace_root)
}

/// Builds `dev cli maintenance flaky-tests` report payload.
#[must_use]
pub fn build_flaky_tests_report(workspace_root: &Path) -> Value {
    let mut tests = Vec::<Value>::new();
    let mut scan_errors = Vec::<Value>::new();
    for path in collect_files(&workspace_root.join("crates")) {
        if path.extension().is_none_or(|ext| ext != "rs")
            || !path.components().any(|segment| segment.as_os_str() == "tests")
            || path.components().any(|segment| segment.as_os_str() == "target")
        {
            continue;
        }
        let source = match fs::read_to_string(&path) {
            Ok(source) => source,
            Err(error) => {
                scan_errors.push(json!({
                    "path": rel(&path, workspace_root),
                    "error": "read-failed",
                    "message": error.to_string(),
                }));
                continue;
            }
        };
        for line in source.lines().filter(|line| line.contains("#[ignore")) {
            let Some(first_quote) = line.find('"') else {
                continue;
            };
            let tail = &line[first_quote + 1..];
            let Some(second_quote) = tail.find('"') else {
                continue;
            };
            let reason = tail[..second_quote].trim().to_ascii_lowercase();
            if reason.contains("flaky") {
                tests.push(json!({
                    "path": rel(&path, workspace_root),
                    "label": "flaky",
                    "reason": if reason.is_empty() { "flaky" } else { &reason },
                }));
            }
        }
    }
    json!({
        "generated_at_utc": generated_at_utc(),
        "label": "flaky",
        "count": tests.len(),
        "tests": tests,
        "policy": "no flaky test may be silently ignored; each flaky marker requires remediation tracking",
        "generator": "crates/bijux-dev-cli/src/contracts/maintenance/compliance.rs::build_flaky_tests_report",
        "integrity_status": if scan_errors.is_empty() { "ok" } else { "degraded" },
        "scan_errors": scan_errors,
    })
}

/// Builds `dev cli maintenance migrated` report payload.
#[must_use]
pub fn build_migrated_report(workspace_root: &Path) -> Value {
    let controls = migration_controls(workspace_root);
    let migrated: Vec<Value> = controls
        .iter()
        .map(|control| {
            let from =
                control.get("from").and_then(Value::as_str).expect("migration control `from`");
            let to = control
                .get("replacement")
                .and_then(Value::as_str)
                .expect("migration control `replacement`");
            let rank = control
                .get("maintainer_value_rank")
                .and_then(Value::as_u64)
                .expect("migration control `maintainer_value_rank`");
            let control_id = control
                .get("control_id")
                .and_then(Value::as_str)
                .expect("migration control `control_id`");
            json!({
                "from": from,
                "to": to,
                "maintainer_value_rank": rank,
                "deleted": control.get("status") == Some(&json!("removed")),
                "control_id": control_id,
            })
        })
        .collect();

    let removed =
        controls.iter().filter(|row| row.get("status") == Some(&json!("removed"))).count();

    json!({
        "generated_at_utc": generated_at_utc(),
        "migrated": migrated,
        "migration_controls": controls,
        "summary": {
            "count": MIGRATION_CONTROLS.len(),
            "removed": removed,
            "remaining": MIGRATION_CONTROLS.len().saturating_sub(removed),
        },
    })
}

/// Builds `dev cli maintenance remaining` report payload.
#[must_use]
pub fn build_remaining_report(workspace_root: &Path) -> Value {
    let controls = migration_controls(workspace_root);
    let failing_controls: Vec<Value> = controls
        .iter()
        .filter(|row| row.get("status") == Some(&json!("remaining")))
        .cloned()
        .collect();

    let root_maintenance: Vec<String> = collect_files(&workspace_root.join("maintenance"))
        .into_iter()
        .filter(|p| p.parent().is_some_and(|parent| parent.ends_with("maintenance")))
        .map(|p| rel(&p, workspace_root))
        .collect();

    let mut make_targets = Vec::new();
    for mk in collect_files(&workspace_root.join("makes")) {
        for target in parse_make_targets(&mk) {
            make_targets.push(json!({"target": target, "file": rel(&mk, workspace_root)}));
        }
    }

    json!({
        "generated_at_utc": generated_at_utc(),
        "remaining_root_maintenance": root_maintenance,
        "migration_controls_remaining": failing_controls,
        "make_targets": make_targets,
        "summary": {
            "remaining_root_maintenance_count": root_maintenance.len(),
            "migration_controls_remaining_count": controls
                .iter()
                .filter(|row| row.get("status") == Some(&json!("remaining")))
                .count(),
            "make_target_count": make_targets.len(),
        }
    })
}

/// Builds `dev cli maintenance diff` report payload.
#[must_use]
pub fn build_diff_report(workspace_root: &Path) -> Value {
    let migrated = build_migrated_report(workspace_root);
    let remaining = build_remaining_report(workspace_root);

    let undeleted: Vec<Value> = match migrated.get("migrated").and_then(Value::as_array) {
        Some(rows) => rows
            .iter()
            .filter(|row| row.get("deleted") == Some(&Value::Bool(false)))
            .cloned()
            .collect(),
        None => Vec::new(),
    };

    let remaining_controls: BTreeSet<String> =
        match remaining.get("migration_controls_remaining").and_then(Value::as_array) {
            Some(rows) => rows
                .iter()
                .filter_map(|row| {
                    row.get("control_id").and_then(Value::as_str).map(ToString::to_string)
                })
                .collect(),
            None => BTreeSet::new(),
        };

    json!({
        "generated_at_utc": generated_at_utc(),
        "undeleted_migrated_maintenance": undeleted,
        "remaining_control_ids": remaining_controls,
        "remaining": remaining,
    })
}

/// Builds `dev cli maintenance audit` report payload.
#[must_use]
pub fn build_audit_report(workspace_root: &Path) -> Value {
    json!({
        "migrated": build_migrated_report(workspace_root),
        "remaining": build_remaining_report(workspace_root),
        "diff": build_diff_report(workspace_root),
        "status_generators": build_generators_report(workspace_root),
        "status_contracts": build_status_contracts_report(workspace_root),
        "requirement_catalog": build_requirement_catalog(workspace_root),
        "flaky_tests": build_flaky_tests_report(workspace_root),
    })
}
