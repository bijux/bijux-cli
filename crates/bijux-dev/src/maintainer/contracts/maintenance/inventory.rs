use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

/// Builds the package metadata integrity report.
#[must_use]
pub fn build_package_metadata_report(workspace_root: &Path) -> Value {
    let workspace_toml = workspace_root.join("Cargo.toml");
    let workspace = match fs::read_to_string(&workspace_toml) {
        Ok(contents) => contents,
        Err(error) => {
            return json!({
                "status": "fail",
                "failures": [format!("failed to read {}: {error}", workspace_toml.display())],
            });
        }
    };

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
    let e2e_roots = [
        workspace_root.join("crates/bijux-cli/tests/integration"),
        workspace_root.join("crates/bijux-dev/tests/maintainer/e2e"),
    ];
    let mut files = Vec::new();
    for root in &e2e_roots {
        files.extend(collect_files(root));
    }

    let mut errors = Vec::new();
    let mut scan_errors = Vec::new();
    let mut test_count = 0usize;
    for file in files {
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(error) => {
                scan_errors.push(json!({
                    "path": rel(&file, workspace_root),
                    "error": "read-failed",
                    "message": error.to_string(),
                }));
                continue;
            }
        };
        let file_test_count = text.matches("#[test]").count();
        test_count += file_test_count;
    }

    if test_count == 0 {
        errors.push(
            "crate-scoped end-to-end surfaces do not define Rust #[test] entries".to_string(),
        );
    }

    json!({
        "status": if errors.is_empty() { "pass" } else { "fail" },
        "test_count": test_count,
        "errors": errors,
        "integrity_status": if scan_errors.is_empty() { "ok" } else { "degraded" },
        "scan_errors": scan_errors,
    })
}

/// Builds dependency vulnerability report from a JSON audit artifact.
#[must_use]
pub fn build_pip_audit_report(workspace_root: &Path, report_path: Option<&str>) -> Value {
    let path = report_path
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("artifacts_pages/security/dependency-audit.json"));
    let (parsed, parse_error) = match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(parsed) => (Some(parsed), None),
            Err(error) => {
                (None, Some(format!("audit report is invalid JSON at {}: {error}", path.display())))
            }
        },
        Err(error) => {
            (None, Some(format!("failed to read audit report at {}: {error}", path.display())))
        }
    };
    let Some(parsed) = parsed else {
        return json!({
            "status": "fail",
            "report_path": path,
            "remaining_vulnerabilities": [],
            "integrity_status": "degraded",
            "integrity_error": parse_error,
        });
    };

    let dependencies = match parsed {
        Value::Array(rows) => rows,
        Value::Object(_) => match parsed.get("dependencies").and_then(Value::as_array) {
            Some(rows) => rows.clone(),
            None => {
                return json!({
                    "status": "fail",
                    "report_path": path,
                    "remaining_vulnerabilities": [],
                    "integrity_status": "degraded",
                    "integrity_error": "audit report must be an array or include an array at key `dependencies`",
                });
            }
        },
        _ => {
            return json!({
                "status": "fail",
                "report_path": path,
                "remaining_vulnerabilities": [],
                "integrity_status": "degraded",
                "integrity_error": "audit report JSON root must be an object or array",
            });
        }
    };

    let mut remaining = Vec::new();
    let mut integrity_issues = Vec::new();
    for dep in dependencies {
        let Some(name) = dep.get("name").and_then(Value::as_str).map(str::trim) else {
            integrity_issues.push(json!({
                "error": "missing-field",
                "field": "name",
                "row": dep,
            }));
            continue;
        };
        let Some(version) = dep.get("version").and_then(Value::as_str).map(str::trim) else {
            integrity_issues.push(json!({
                "error": "missing-field",
                "field": "version",
                "row": dep,
            }));
            continue;
        };
        let vulnerabilities = dep
            .get("vulnerabilities")
            .and_then(Value::as_array)
            .or_else(|| dep.get("vulns").and_then(Value::as_array));
        let Some(vulnerabilities) = vulnerabilities else {
            integrity_issues.push(json!({
                "error": "missing-field",
                "field": "vulnerabilities",
                "package": name,
                "version": version,
            }));
            continue;
        };
        for vuln in vulnerabilities {
            let Some(id) = vuln.get("id").and_then(Value::as_str).map(str::trim) else {
                integrity_issues.push(json!({
                    "error": "missing-field",
                    "field": "id",
                    "package": name,
                    "version": version,
                    "vulnerability": vuln,
                }));
                continue;
            };
            let fix_versions = vuln.get("fix_versions").and_then(Value::as_array).cloned();
            let Some(fix_versions) = fix_versions else {
                integrity_issues.push(json!({
                    "error": "missing-field",
                    "field": "fix_versions",
                    "package": name,
                    "version": version,
                    "id": id,
                }));
                continue;
            };
            remaining.push(json!({
                "package": name,
                "version": version,
                "id": id,
                "fix_versions": fix_versions,
            }));
        }
    }

    json!({
        "status": if remaining.is_empty() { "pass" } else { "fail" },
        "report_path": path,
        "remaining_vulnerabilities": remaining,
        "integrity_status": if integrity_issues.is_empty() { "ok" } else { "degraded" },
        "integrity_issues": integrity_issues,
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

    if let Err(error) = fs::create_dir_all(output_dir) {
        return json!({
            "status": "failed",
            "error": format!("failed to create output directory {}: {error}", output_dir.display()),
        });
    }
    let file = output_dir.join(format!("provenance-{tag}.json"));
    let payload = json!({
      "tag": tag,
      "generated_at_utc": generated_at,
      "generator": "bijux-dev-cli maintenance provenance-statement",
      "note": "Provenance hook scaffold. Replace with signed attestation workflow when enabled."
    });
    let serialized = match serde_json::to_string_pretty(&payload) {
        Ok(serialized) => serialized,
        Err(error) => {
            return json!({
                "status": "failed",
                "file": file,
                "error": format!("failed to serialize provenance payload: {error}"),
            });
        }
    };
    if let Err(error) = fs::write(&file, serialized + "\n") {
        return json!({
            "status": "failed",
            "file": file,
            "error": format!("failed to write provenance payload: {error}"),
        });
    }
    json!({"status": "ok", "file": file, "payload": payload})
}

pub(crate) fn collect_files(base: &Path) -> Vec<PathBuf> {
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

pub(crate) fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

pub(crate) fn parse_make_targets(path: &Path) -> Vec<String> {
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

pub(crate) fn status_slug_for_name(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in value.chars().flat_map(|c| c.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    let mut cleaned = slug.trim_matches('-').to_string();
    for suffix in ["-report", "-audit", "-baseline", "-guide", "-rules", "-law", "-status"] {
        if cleaned.ends_with(suffix) {
            cleaned.truncate(cleaned.len().saturating_sub(suffix.len()));
        }
    }
    cleaned.trim_matches('-').to_string()
}

pub(crate) fn generated_at_utc() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z\n".to_string())
        .trim()
        .to_string()
}

pub(crate) fn write_json(path: &Path, payload: &Value) -> Result<(), String> {
    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|err| format!("failed to create parent dir for {}: {err}", path.display()))?;
    let serialized = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to serialize json for {}: {err}", path.display()))?;
    fs::write(path, serialized + "\n")
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn normalize_maintainer_args(args: &[&str]) -> Vec<String> {
    match args {
        ["dev", "cli", rest @ ..] => rest.iter().map(|value| (*value).to_string()).collect(),
        _ => args.iter().map(|value| (*value).to_string()).collect(),
    }
}

pub(crate) fn run_bijux_json(workspace_root: &Path, args: &[&str]) -> Result<Value, String> {
    let normalized_args = normalize_maintainer_args(args);
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-dev-cli", "--bin", "bijux-dev-cli", "--"])
        .args(&normalized_args)
        .args(["--format", "json", "--no-pretty"])
        .current_dir(workspace_root)
        .output()
        .map_err(|err| format!("failed to run bijux-dev-cli command: {err}"))?;
    if !output.status.success() {
        return Err(format!("command failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|err| format!("failed to parse command JSON output: {err}"))
}

pub(crate) fn run_bijux_json_env(
    workspace_root: &Path,
    args: &[&str],
    envs: &[(&str, String)],
) -> Result<Value, String> {
    let normalized_args = normalize_maintainer_args(args);
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-q", "-p", "bijux-dev-cli", "--bin", "bijux-dev-cli", "--"])
        .args(&normalized_args)
        .args(["--format", "json", "--no-pretty"])
        .current_dir(workspace_root);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let output =
        cmd.output().map_err(|err| format!("failed to run bijux-dev-cli command: {err}"))?;
    if !output.status.success() {
        return Err(format!("command failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|err| format!("failed to parse command JSON output: {err}"))
}

pub(crate) fn run_bijux_text(workspace_root: &Path, args: &[&str]) -> Result<String, String> {
    let normalized_args = normalize_maintainer_args(args);
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-dev-cli", "--bin", "bijux-dev-cli", "--"])
        .args(&normalized_args)
        .args(["--format", "text"])
        .current_dir(workspace_root)
        .output()
        .map_err(|err| format!("failed to run bijux-dev-cli command: {err}"))?;
    if !output.status.success() {
        return Err(format!("command failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn write_status_artifact_json(
    workspace_root: &Path,
    artifact: &str,
    payload: &Value,
) -> Result<String, String> {
    let path = workspace_root.join(artifact);
    write_json(&path, payload)?;
    Ok(artifact.to_string())
}
