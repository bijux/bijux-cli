//! Maintainer script replacement and inventory helpers.

use std::collections::BTreeSet;
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

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn migrated_rows() -> &'static [(&'static str, &'static str, usize)] {
    &[
        ("scripts/check-package-metadata.py", "bijux dev cli scripts package-metadata", 100),
        ("scripts/check_e2e_contract.py", "bijux dev cli scripts e2e-contract", 95),
        ("scripts/helper_pip_audit.py", "bijux dev cli scripts pip-audit", 90),
        ("scripts/capture_python_behavior.py", "bijux dev cli scripts capture-python-behavior", 85),
        (
            "scripts/generate-provenance-statement.sh",
            "bijux dev cli scripts provenance-statement",
            80,
        ),
    ]
}

fn parse_make_targets(path: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('\t') || line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((left, _)) = line.split_once(':') else {
            continue;
        };
        let target = left.trim();
        if !target.is_empty()
            && !target.contains(' ')
            && !target.contains('=')
            && !target.starts_with('.')
        {
            out.push(target.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
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
    })
}

/// Builds replacement for `scripts/check-package-metadata.py`.
#[must_use]
pub fn build_package_metadata_report(workspace_root: &Path) -> Value {
    let workspace_toml = workspace_root.join("Cargo.toml");
    let pyproject_toml = workspace_root.join("pyproject.toml");
    let workspace = fs::read_to_string(workspace_toml).unwrap_or_default();
    let pyproject = fs::read_to_string(pyproject_toml).unwrap_or_default();

    let mut failures = Vec::new();
    if !pyproject.contains("name = \"bijux-cli\"") {
        failures.push("project.name must be bijux-cli".to_string());
    }
    if !workspace.contains("repository") || !pyproject.contains("Homepage") {
        failures
            .push("repository metadata must exist in Cargo.toml and pyproject.toml".to_string());
    }
    if !workspace.contains("license") || !pyproject.contains("license") {
        failures.push("license metadata must exist in Cargo.toml and pyproject.toml".to_string());
    }

    json!({
        "status": if failures.is_empty() { "pass" } else { "fail" },
        "failures": failures,
    })
}

/// Builds replacement for `scripts/check_e2e_contract.py`.
#[must_use]
pub fn build_e2e_contract_report(workspace_root: &Path) -> Value {
    let e2e_dir = workspace_root.join("tests/e2e");
    let inventory = e2e_dir.join("INVENTORY.md");
    let files = collect_files(&e2e_dir);

    let mut errors = Vec::new();
    if !inventory.exists() {
        errors.push("tests/e2e/INVENTORY.md is missing".to_string());
    }

    let mut test_count = 0usize;
    for file in files {
        if !file.file_name().is_some_and(|name| name.to_string_lossy().starts_with("test_")) {
            continue;
        }
        if file.extension().is_none_or(|ext| ext != "py") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        test_count += text.match_indices("def test_").count();
        for required in ["@pytest.mark.e2e", "@pytest.mark.slow"] {
            if !text.contains(required) {
                errors.push(format!("{} missing {required}", rel(&file, workspace_root)));
            }
        }
        if !(text.contains("assert_no_state_corruption")
            || text.contains("assert_exit_code_stable")
            || text.contains("assert_config_consistent")
            || text.contains("assert_plugins_consistent")
            || text.contains("assert_no_traceback"))
        {
            errors.push(format!("{} missing invariant assertion", rel(&file, workspace_root)));
        }
    }

    if test_count < 100 {
        errors.push(format!("tests/e2e below minimum: {test_count} < 100"));
    }
    if test_count > 150 {
        errors.push(format!("tests/e2e exceeds hard cap: {test_count} > 150"));
    }

    json!({
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "test_count": test_count,
        "errors": errors,
    })
}

/// Builds replacement for `scripts/helper_pip_audit.py`.
#[must_use]
pub fn build_pip_audit_report(workspace_root: &Path, report_path: Option<&str>) -> Value {
    let path = report_path
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("artifacts_pages/security/pip-audit.json"));
    let parsed: Value = fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!([]));

    let dependencies = parsed
        .as_array()
        .cloned()
        .or_else(|| parsed.get("dependencies").and_then(Value::as_array).cloned())
        .unwrap_or_default();

    let mut remaining = Vec::new();
    for dep in dependencies {
        let name = dep.get("name").and_then(Value::as_str).unwrap_or("?");
        let version = dep.get("version").and_then(Value::as_str).unwrap_or("?");
        for vuln in dep.get("vulns").and_then(Value::as_array).cloned().unwrap_or_default() {
            let id = vuln.get("id").and_then(Value::as_str).unwrap_or("?");
            let fix =
                vuln.get("fix_versions").and_then(Value::as_array).cloned().unwrap_or_default();
            remaining.push(json!({
                "package": name,
                "version": version,
                "id": id,
                "fix_versions": fix,
            }));
        }
    }

    json!({
        "status": if remaining.is_empty() { "pass" } else { "fail" },
        "report_path": path,
        "remaining_vulnerabilities": remaining,
    })
}

/// Builds replacement for `scripts/capture_python_behavior.py`.
#[must_use]
pub fn build_python_capture_report(workspace_root: &Path) -> Value {
    let lock_path = workspace_root.join("artifacts/current-python-behavior-lock.json");
    let lock: Value = fs::read_to_string(&lock_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}));
    let capture_count =
        lock.get("captures").and_then(Value::as_object).map_or(0, |captures| captures.len());
    json!({
        "status": if capture_count > 0 { "pass" } else { "fail" },
        "lock_path": lock_path,
        "capture_count": capture_count,
    })
}

/// Builds replacement for `scripts/generate-provenance-statement.sh`.
#[must_use]
pub fn build_provenance_statement_report(tag: &str, output_dir: &Path) -> Value {
    let generated_at = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z\n".to_string())
        .trim()
        .to_string();

    let _ = fs::create_dir_all(output_dir);
    let file = output_dir.join(format!("provenance-{tag}.json"));
    let payload = json!({
      "tag": tag,
      "generated_at_utc": generated_at,
      "generator": "bijux dev cli scripts provenance-statement",
      "note": "Provenance hook scaffold. Replace with signed attestation workflow when enabled."
    });
    let _ = fs::write(
        &file,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string()) + "\n",
    );
    json!({"status": "ok", "file": file, "payload": payload})
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        build_audit_report, build_diff_report, build_migrated_report, build_remaining_report,
    };

    #[test]
    fn scripts_reports_are_shaped() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(build_migrated_report(&root).get("migrated").is_some());
        assert!(build_remaining_report(&root).get("remaining_root_scripts").is_some());
        assert!(build_diff_report(&root).get("remaining").is_some());
        assert!(build_audit_report(&root).get("diff").is_some());
    }
}
