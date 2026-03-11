use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use super::shared::collect_files;
use super::shared::rel;

/// Builds the package metadata integrity report.
#[must_use]
pub fn build_package_metadata_report(workspace_root: &Path) -> Value {
    let workspace_toml = workspace_root.join("Cargo.toml");
    let workspace = fs::read_to_string(workspace_toml).unwrap_or_default();

    let mut failures = Vec::new();
    if !workspace.contains("name") {
        failures.push("Cargo.toml must contain package or workspace name metadata".to_string());
    }
    if !workspace.contains("repository") {
        failures.push("Cargo.toml must include repository metadata".to_string());
    }
    if !workspace.contains("license") {
        failures.push("Cargo.toml must include license metadata".to_string());
    }

    json!({
        "status": if failures.is_empty() { "pass" } else { "fail" },
        "failures": failures,
    })
}

/// Builds the end-to-end contract report from Rust test surfaces.
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
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        let file_test_count = text.matches("#[test]").count();
        test_count += file_test_count;

        if file_test_count == 0 {
            errors.push(format!(
                "{} contains no #[test] entries",
                rel(&file, workspace_root)
            ));
        }

        if !(text.contains("assert!") || text.contains("assert_eq!") || text.contains("assert_ne!")) {
            errors.push(format!(
                "{} contains no assertion macros",
                rel(&file, workspace_root)
            ));
        }
    }

    if test_count == 0 {
        errors.push("tests/e2e does not define any Rust tests".to_string());
    }

    json!({
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "test_count": test_count,
        "errors": errors,
    })
}

/// Builds dependency vulnerability report from a JSON audit artifact.
#[must_use]
pub fn build_pip_audit_report(workspace_root: &Path, report_path: Option<&str>) -> Value {
    let path = report_path
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("artifacts_pages/security/dependency-audit.json"));
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
        for vuln in dep
            .get("vulnerabilities")
            .and_then(Value::as_array)
            .cloned()
            .or_else(|| dep.get("vulns").and_then(Value::as_array).cloned())
            .unwrap_or_default()
        {
            let id = vuln.get("id").and_then(Value::as_str).unwrap_or("?");
            let fix = vuln
                .get("fix_versions")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
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

/// Builds runtime-behavior capture report from lock artifact.
#[must_use]
pub fn build_python_capture_report(workspace_root: &Path) -> Value {
    let lock_path = workspace_root.join("artifacts/current-runtime-behavior-lock.json");
    let lock: Value = fs::read_to_string(&lock_path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}));
    let capture_count = lock
        .get("captures")
        .and_then(Value::as_object)
        .map_or(0, |captures| captures.len());
    json!({
        "status": if capture_count > 0 { "pass" } else { "fail" },
        "lock_path": lock_path,
        "capture_count": capture_count,
    })
}

/// Builds provenance statement report.
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
      "generator": "bijux dev cli maintenance provenance-statement",
      "note": "Provenance hook scaffold. Replace with signed attestation workflow when enabled."
    });
    let _ = fs::write(
        &file,
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string()) + "\n",
    );
    json!({"status": "ok", "file": file, "payload": payload})
}
