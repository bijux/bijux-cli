//! Maintainer maintenance inventory report assembly.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{collect_files_recursive, parse_make_targets, relative_to_root};

fn classify_maintenance_path(path: &str) -> &'static str {
    if path.starts_with("maintenance/status/") {
        "status-contract-input"
    } else {
        "maintenance-asset"
    }
}

fn replacement_command(path: &str) -> Option<String> {
    if path.starts_with("maintenance/status/") {
        Some("bijux dev cli maintenance status inventory".to_string())
    } else if path.starts_with("maintenance/") {
        Some("bijux dev cli maintenance audit".to_string())
    } else {
        None
    }
}

fn classify_make_target(target: &str) -> &'static str {
    if target.starts_with("publish") || target.starts_with("sbom") || target.starts_with("security")
    {
        "keep"
    } else {
        "replace"
    }
}

/// Builds the dev-cli inventory payload consumed by maintainer audits.
#[must_use]
pub fn build_inventory_report(workspace_root: &Path) -> Value {
    let maintenance_files = collect_files_recursive(&workspace_root.join("maintenance"));
    let maintenance: Vec<Value> = maintenance_files
        .iter()
        .map(|path| {
            let rel = relative_to_root(path, workspace_root);
            json!({
                "path": rel,
                "classification": classify_maintenance_path(&rel),
                "replacement_command": replacement_command(&rel),
            })
        })
        .collect();

    let mut makes = Vec::new();
    for mk in collect_files_recursive(&workspace_root.join("makes")) {
        let rel = relative_to_root(&mk, workspace_root);
        let targets: Vec<Value> = parse_make_targets(&mk)
            .into_iter()
            .map(|target| {
                json!({
                    "target": target,
                    "classification": classify_make_target(&target),
                })
            })
            .collect();
        makes.push(json!({
            "file": rel,
            "targets": targets,
        }));
    }

    let maintenance_summary =
        maintenance.iter().fold(BTreeMap::<String, usize>::new(), |mut acc, item| {
            let key =
                item.get("classification").and_then(Value::as_str).unwrap_or("unknown").to_string();
            *acc.entry(key).or_insert(0) += 1;
            acc
        });

    let remaining_legacy_only_behaviors: Vec<String> = maintenance
        .iter()
        .filter_map(|item| {
            let classification = item.get("classification").and_then(Value::as_str).unwrap_or("");
            if classification != "legacy" {
                return None;
            }
            item.get("path").and_then(Value::as_str).map(ToString::to_string)
        })
        .collect();

    let remaining_make_only_behaviors: Vec<String> = makes
        .iter()
        .flat_map(|mk| mk.get("targets").and_then(Value::as_array).cloned().unwrap_or_default())
        .filter_map(|target| {
            let classification = target.get("classification").and_then(Value::as_str).unwrap_or("");
            if classification != "keep" {
                return None;
            }
            target.get("target").and_then(Value::as_str).map(ToString::to_string)
        })
        .collect();

    let maintainer_maintenance_replacements: Vec<Value> = maintenance
        .iter()
        .filter_map(|row| {
            let path = row.get("path").and_then(Value::as_str)?;
            let replacement = row.get("replacement_command").and_then(Value::as_str)?;
            if replacement.is_empty() {
                return None;
            }
            Some(json!({"from": path, "to": replacement}))
        })
        .collect();

    json!({
        "maintenance": maintenance,
        "makes": makes,
        "summary": {
            "maintenance_classification_counts": maintenance_summary,
        },
        "maintainer_maintenance_replacements": maintainer_maintenance_replacements,
        "remaining_legacy_only_behaviors": remaining_legacy_only_behaviors,
        "remaining_make_only_behaviors": remaining_make_only_behaviors,
        "rule": "new maintainer automation defaults to bijux dev cli commands",
    })
}

/// Builds the maintainer maintenance audit report payload.
#[must_use]
pub fn build_report(inventory: Value) -> Value {
    json!({
        "maintenance": inventory.get("maintenance").cloned().unwrap_or_else(|| json!([])),
        "summary": inventory.get("summary").cloned().unwrap_or_else(|| json!({})),
        "remaining_legacy_only_behaviors": inventory
            .get("remaining_legacy_only_behaviors")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "remaining_make_only_behaviors": inventory
            .get("remaining_make_only_behaviors")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "replacement_rule": inventory.get("rule").cloned().unwrap_or_else(|| json!("")),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{build_inventory_report, build_report};

    #[test]
    fn maintenance_audit_report_shape_is_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inventory = build_inventory_report(&root);
        let report = build_report(inventory);
        assert!(report.get("maintenance").is_some());
        assert!(report.get("summary").is_some());
    }
}
