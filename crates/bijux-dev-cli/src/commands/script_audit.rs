//! Maintainer script audit report assembly.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn collect_files(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn rel_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn classify_script(path: &str) -> &'static str {
    if path == "scripts/__init__.py" {
        return "delete";
    }
    "replace"
}

fn replacement_command(path: &str) -> Option<String> {
    match path {
        "scripts/check-package-metadata.py" => {
            Some("bijux dev cli scripts package-metadata".to_string())
        }
        "scripts/check_e2e_contract.py" => Some("bijux dev cli scripts e2e-contract".to_string()),
        "scripts/helper_pip_audit.py" => Some("bijux dev cli scripts pip-audit".to_string()),
        "scripts/capture_python_behavior.py" => {
            Some("bijux dev cli scripts capture-python-behavior".to_string())
        }
        "scripts/generate-provenance-statement.sh" => {
            Some("bijux dev cli scripts provenance-statement".to_string())
        }
        _ => None,
    }
}

fn parse_make_targets(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    for raw in text.lines() {
        if raw.starts_with('\t') || raw.starts_with('#') || raw.trim().is_empty() {
            continue;
        }
        let Some((left, _)) = raw.split_once(':') else {
            continue;
        };
        let target = left.trim();
        if target.is_empty()
            || target.contains(' ')
            || target.contains('=')
            || target.starts_with('.')
        {
            continue;
        }
        out.push(target.to_string());
    }
    out.sort();
    out.dedup();
    out
}

fn classify_make_target(target: &str) -> &'static str {
    if target.starts_with("docs") || target.starts_with("api") || target.starts_with("test") {
        "replace"
    } else if target.starts_with("publish")
        || target.starts_with("sbom")
        || target.starts_with("security")
    {
        "keep"
    } else {
        "replace"
    }
}

/// Builds the dev-cli inventory payload consumed by maintainer audits.
#[must_use]
pub fn build_inventory_report(workspace_root: &Path) -> Value {
    let script_files = collect_files(&workspace_root.join("scripts"));
    let scripts: Vec<Value> = script_files
        .iter()
        .map(|p| {
            let rel = rel_to_root(p, workspace_root);
            json!({
                "path": rel,
                "classification": classify_script(&rel),
                "replacement_command": replacement_command(&rel),
            })
        })
        .collect();

    let mut makes = Vec::new();
    for mk in collect_files(&workspace_root.join("makes")) {
        let rel = rel_to_root(&mk, workspace_root);
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

    let script_summary = scripts.iter().fold(BTreeMap::<String, usize>::new(), |mut acc, item| {
        let key =
            item.get("classification").and_then(Value::as_str).unwrap_or("unknown").to_string();
        *acc.entry(key).or_insert(0) += 1;
        acc
    });
    let remaining_script_only_behaviors: Vec<String> = scripts
        .iter()
        .filter_map(|item| {
            let classification = item.get("classification").and_then(Value::as_str).unwrap_or("");
            if classification != "keep" {
                return None;
            }
            item.get("path").and_then(Value::as_str).map(ToString::to_string)
        })
        .collect();
    let remaining_task_runner_only_behaviors: Vec<String> = makes
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

    let maintainer_script_replacements: Vec<Value> = scripts
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
        "scripts": scripts,
        "makes": makes,
        "summary": {
            "script_classification_counts": script_summary,
        },
        "maintainer_script_replacements": maintainer_script_replacements,
        "remaining_script_only_behaviors": remaining_script_only_behaviors,
        "remaining_task_runner_only_behaviors": remaining_task_runner_only_behaviors,
        "rule": "new maintainer automation defaults to bijux dev cli commands",
    })
}

/// Builds the maintainer script audit report payload.
#[must_use]
pub fn build_report(inventory: Value) -> Value {
    json!({
        "scripts": inventory.get("scripts").cloned().unwrap_or_else(|| json!([])),
        "summary": inventory.get("summary").cloned().unwrap_or_else(|| json!({})),
        "remaining_script_only_behaviors": inventory.get("remaining_script_only_behaviors").cloned().unwrap_or_else(|| json!([])),
        "remaining_task_runner_only_behaviors": inventory.get("remaining_task_runner_only_behaviors").cloned().unwrap_or_else(|| json!([])),
        "replacement_rule": inventory.get("rule").cloned().unwrap_or_else(|| json!("")),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{build_inventory_report, build_report};

    #[test]
    fn script_audit_report_shape_is_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inventory = build_inventory_report(&root);
        let report = build_report(inventory);
        assert!(report.get("scripts").is_some());
        assert!(report.get("summary").is_some());
    }
}
