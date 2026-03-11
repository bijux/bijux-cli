use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

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
    let e2e_roots = [
        workspace_root.join("crates/bijux-cli/tests/integration"),
        workspace_root.join("crates/bijux-dev-cli/tests/e2e"),
    ];
    let mut files = Vec::new();
    for root in &e2e_roots {
        files.extend(collect_files(root));
    }

    let mut errors = Vec::new();
    let mut test_count = 0usize;
    for file in files {
        if file.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
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

/// Builds runtime-behavior capture report from lock artifact.
#[must_use]
pub fn build_python_capture_report(workspace_root: &Path) -> Value {
    let lock_path = workspace_root.join("artifacts/current-runtime-behavior-lock.json");
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

pub(crate) fn run_bijux_json(workspace_root: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-cli", "--bin", "bijux", "--"])
        .args(args)
        .args(["--format", "json", "--no-pretty"])
        .current_dir(workspace_root)
        .output()
        .map_err(|err| format!("failed to run bijux command: {err}"))?;
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
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-q", "-p", "bijux-cli", "--bin", "bijux", "--"])
        .args(args)
        .args(["--format", "json", "--no-pretty"])
        .current_dir(workspace_root);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let output = cmd.output().map_err(|err| format!("failed to run bijux command: {err}"))?;
    if !output.status.success() {
        return Err(format!("command failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    serde_json::from_slice::<Value>(&output.stdout)
        .map_err(|err| format!("failed to parse command JSON output: {err}"))
}

pub(crate) fn run_bijux_text(workspace_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["run", "-q", "-p", "bijux-cli", "--bin", "bijux", "--"])
        .args(args)
        .args(["--format", "text"])
        .current_dir(workspace_root)
        .output()
        .map_err(|err| format!("failed to run bijux command: {err}"))?;
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
