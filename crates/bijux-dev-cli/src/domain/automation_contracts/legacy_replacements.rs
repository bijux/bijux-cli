use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::support::collect_files;
use super::support::rel;

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
