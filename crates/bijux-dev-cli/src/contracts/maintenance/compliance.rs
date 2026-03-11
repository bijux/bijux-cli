use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use super::generator_runner::build_generators_report;
use super::status_registry::build_status_contracts_report;
use super::shared::{
    collect_files, extract_required_test_names, generated_at_utc, is_python_file, migrated_rows,
    parse_make_targets, rel, status_generator_slug,
};

fn build_requirement_catalog(workspace_root: &Path) -> Value {
    let mut by_script = BTreeMap::<String, Vec<String>>::new();
    for path in collect_files(&workspace_root.join("scripts").join("status")) {
        let rel_path = rel(&path, workspace_root);
        let is_py = is_python_file(&path);
        if !is_py || !rel_path.contains("/generate_") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap_or_default();
        let tests = extract_required_test_names(&source);
        if tests.is_empty() {
            continue;
        }
        by_script.insert(rel_path, tests);
    }

    let mut rows = Vec::<Value>::new();
    for (script_path, tests) in by_script {
        let slug = status_generator_slug(&script_path);
        for (idx, test_name) in tests.iter().enumerate() {
            rows.push(json!({
                "requirement_id": format!("REQ-{slug}-{:03}", idx + 1),
                "owner": "bijux-dev-cli",
                "source_script": script_path,
                "test_name": test_name,
            }));
        }
    }
    json!({
        "id_policy": "REQ-<GENERATOR-SLUG>-<3DIGIT-INDEX>",
        "generated_at_utc": generated_at_utc(),
        "rows": rows,
        "count": rows.len(),
    })
}

/// Builds `dev cli scripts requirements` report payload.
#[must_use]
pub fn build_requirement_catalog_report(workspace_root: &Path) -> Value {
    build_requirement_catalog(workspace_root)
}

/// Builds `dev cli scripts flaky-tests` report payload.
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

/// Builds `dev cli scripts migrated` report payload.
#[must_use]
pub fn build_migrated_report(workspace_root: &Path) -> Value {
    let rows: Vec<Value> = migrated_rows()
        .iter()
        .map(|(from, to, rank)| {
            json!({
                "from": from,
                "to": to,
                "maintainer_value_rank": rank,
                "deleted": !workspace_root.join(from).exists(),
            })
        })
        .collect();
    json!({
        "migrated": rows,
        "summary": {
            "count": rows.len(),
            "deleted": rows.iter().filter(|r| r.get("deleted") == Some(&Value::Bool(true))).count(),
        },
    })
}

/// Builds `dev cli scripts remaining` report payload.
#[must_use]
pub fn build_remaining_report(workspace_root: &Path) -> Value {
    let migrated: BTreeSet<&str> = migrated_rows().iter().map(|(from, _, _)| *from).collect();
    let root_scripts: Vec<String> = collect_files(&workspace_root.join("scripts"))
        .into_iter()
        .filter(|p| p.parent().is_some_and(|parent| parent.ends_with("scripts")))
        .map(|p| rel(&p, workspace_root))
        .collect();
    let remaining: Vec<String> =
        root_scripts.into_iter().filter(|path| !migrated.contains(path.as_str())).collect();

    let mut make_targets = Vec::new();
    for mk in collect_files(&workspace_root.join("makes")) {
        for target in parse_make_targets(&mk) {
            make_targets.push(json!({"target": target, "file": rel(&mk, workspace_root)}));
        }
    }

    json!({
        "remaining_root_scripts": remaining,
        "make_targets": make_targets,
        "summary": {
            "remaining_root_script_count": remaining.len(),
            "make_target_count": make_targets.len(),
        }
    })
}

/// Builds `dev cli scripts diff` report payload.
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
    json!({
        "undeleted_migrated_scripts": undeleted,
        "remaining": remaining,
    })
}

/// Builds `dev cli scripts audit` report payload.
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
