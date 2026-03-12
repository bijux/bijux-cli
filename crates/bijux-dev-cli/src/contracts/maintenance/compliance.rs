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
    let status_rows = build_status_contracts_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let generator_rows = build_generators_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut rows = Vec::<Value>::new();
    for (index, row) in status_rows.iter().enumerate() {
        let contract_id = row.get("contract_id").and_then(Value::as_str).unwrap_or("UNKNOWN");
        rows.push(json!({
            "requirement_id": format!("REQ-STATUS-{index:03}", index = index + 1),
            "domain": "STATUS",
            "source_kind": "status-contract",
            "contract_id": contract_id,
            "kind": row.get("kind").cloned().unwrap_or_else(|| json!("unknown")),
            "implementation": row.get("implementation").cloned().unwrap_or_else(|| json!("unknown")),
            "command": row.get("command").cloned().unwrap_or_else(|| json!("")),
            "outputs": row.get("outputs").cloned().unwrap_or_else(|| json!([])),
            "rule": "every status contract in inventory must remain executable through the maintenance control plane",
        }));
    }

    for (index, row) in generator_rows.iter().enumerate() {
        let generator_id = row.get("generator_id").and_then(Value::as_str).unwrap_or("UNKNOWN");
        rows.push(json!({
            "requirement_id": format!("REQ-GENERATOR-{index:03}", index = index + 1),
            "domain": "GENERATOR",
            "source_kind": "status-generator",
            "generator_id": generator_id,
            "implementation": row.get("implementation").cloned().unwrap_or_else(|| json!("unknown")),
            "command": row.get("command").cloned().unwrap_or_else(|| json!("")),
            "outputs": row.get("outputs").cloned().unwrap_or_else(|| json!([])),
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
    for path in collect_files(&workspace_root.join("crates")) {
        if path.extension().is_none_or(|ext| ext != "rs")
            || !path.components().any(|segment| segment.as_os_str() == "tests")
            || path.components().any(|segment| segment.as_os_str() == "target")
        {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap_or_default();
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
    })
}

/// Builds `dev cli maintenance migrated` report payload.
#[must_use]
pub fn build_migrated_report(workspace_root: &Path) -> Value {
    let controls = migration_controls(workspace_root);
    let migrated: Vec<Value> = controls
        .iter()
        .map(|control| {
            json!({
                "from": control.get("from").cloned().unwrap_or_else(|| json!("")),
                "to": control.get("replacement").cloned().unwrap_or_else(|| json!("")),
                "maintainer_value_rank": control.get("maintainer_value_rank").cloned().unwrap_or_else(|| json!(0)),
                "deleted": control.get("status") == Some(&json!("removed")),
                "control_id": control.get("control_id").cloned().unwrap_or_else(|| json!("")),
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

    let undeleted: Vec<Value> = migrated
        .get("migrated")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.get("deleted") == Some(&Value::Bool(false)))
        .collect();

    let remaining_controls: BTreeSet<String> = remaining
        .get("migration_controls_remaining")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| row.get("control_id").and_then(Value::as_str).map(ToString::to_string))
        .collect();

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
