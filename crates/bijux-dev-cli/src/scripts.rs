//! Maintainer script replacement and inventory helpers.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn is_python_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
}

fn status_generator_sources(workspace_root: &Path) -> Vec<String> {
    collect_files(&workspace_root.join("scripts").join("status"))
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("generate_"))
                && is_python_file(path)
        })
        .map(|path| rel(&path, workspace_root))
        .collect()
}

fn status_generator_slug(script_path: &str) -> String {
    let file = script_path.rsplit('/').next().unwrap_or(script_path);
    let stem = file.strip_suffix(".py").unwrap_or(file);
    let stem = stem.strip_prefix("generate_").unwrap_or(stem);
    let stem = stem.strip_suffix("_reports").unwrap_or(stem);
    stem.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_uppercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn status_generator_id(script_path: &str) -> String {
    format!("GEN-STATUS-{}", status_generator_slug(script_path))
}

fn extract_artifact_paths(source: &str) -> Vec<String> {
    let mut out = BTreeSet::<String>::new();
    for line in source.lines() {
        let mut search = line;
        while let Some(idx) = search.find("artifacts/") {
            let tail = &search[idx..];
            let token = tail
                .chars()
                .take_while(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '.'))
                .collect::<String>()
                .trim_end_matches('.')
                .trim_end_matches(',')
                .trim_end_matches(')')
                .trim_end_matches('"')
                .trim_end_matches('\'')
                .to_string();
            if token.starts_with("artifacts/") {
                out.insert(token);
            }
            search = &tail["artifacts/".len()..];
        }
    }
    out.into_iter().collect()
}

fn extract_required_test_names(source: &str) -> Vec<String> {
    let Some(start) = source.find("REQUIRED_TESTS = {") else {
        return Vec::new();
    };
    let block = &source[start..];
    let Some(end) = block.find("\n}") else {
        return Vec::new();
    };
    let mut out = Vec::<String>::new();
    for line in block[..end].lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("REQUIRED_TESTS = {") || trimmed.is_empty() {
            continue;
        }
        let Some(first_quote) = trimmed.find('"') else {
            continue;
        };
        let tail = &trimmed[first_quote + 1..];
        let Some(second_quote) = tail.find('"') else {
            continue;
        };
        let test_name = &tail[..second_quote];
        if !test_name.is_empty() {
            out.push(test_name.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

fn generated_at_utc() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z\n".to_string())
        .trim()
        .to_string()
}

fn build_status_generators_report(workspace_root: &Path) -> Value {
    let mut rows: Vec<Value> = status_generator_sources(workspace_root)
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(workspace_root.join(&path)).unwrap_or_default();
            let outputs = extract_artifact_paths(&source);
            let id = status_generator_id(&path);
            json!({
                "generator_id": id,
                "source_script": path,
                "implementation": "python-script",
                "outputs": outputs,
                "command": format!("bijux dev cli scripts generate --id {id}"),
            })
        })
        .collect();
    rows.push(json!({
        "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
        "source_script": Value::Null,
        "implementation": "rust",
        "outputs": ["artifacts/status/flaky_tests.json"],
        "command": "bijux dev cli scripts generate --id GEN-STATUS-FLAKY-TEST-LABELS",
    }));
    rows.sort_by(|left, right| {
        left.get("generator_id")
            .and_then(Value::as_str)
            .cmp(&right.get("generator_id").and_then(Value::as_str))
    });
    json!({
        "id_policy": "GEN-STATUS-<GENERATOR-SLUG>",
        "generated_at_utc": generated_at_utc(),
        "count": rows.len(),
        "rows": rows,
    })
}

fn write_json(path: &Path, payload: &Value) -> Result<(), String> {
    fs::create_dir_all(path.parent().unwrap_or_else(|| Path::new(".")))
        .map_err(|err| format!("failed to create parent dir for {}: {err}", path.display()))?;
    let serialized = serde_json::to_string_pretty(payload)
        .map_err(|err| format!("failed to serialize json for {}: {err}", path.display()))?;
    fs::write(path, serialized + "\n")
        .map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn run_python_generator(workspace_root: &Path, source_script: &str, outputs: &[String]) -> Value {
    let script = workspace_root.join(source_script);
    let executed = Command::new("python3").arg(&script).current_dir(workspace_root).output();
    match executed {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let status = if output.status.success() { "ok" } else { "failed" };
            json!({
                "status": status,
                "generator_id": status_generator_id(source_script),
                "source_script": source_script,
                "implementation": "python-script",
                "exit_code": exit_code,
                "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
                "outputs": outputs,
            })
        }
        Err(err) => json!({
            "status": "failed",
            "generator_id": status_generator_id(source_script),
            "source_script": source_script,
            "implementation": "python-script",
            "error": format!("failed to launch python3 for {source_script}: {err}"),
            "outputs": outputs,
        }),
    }
}

fn run_flaky_tests_generator(workspace_root: &Path) -> Value {
    let report = build_flaky_tests_report(workspace_root);
    let output_path = workspace_root.join("artifacts/status/flaky_tests.json");
    match write_json(&output_path, &report) {
        Ok(()) => json!({
            "status": "ok",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
        }),
        Err(err) => json!({
            "status": "failed",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
            "error": err,
        }),
    }
}

fn run_status_generator_entry(workspace_root: &Path, row: &Value) -> Value {
    let Some(generator_id) = row.get("generator_id").and_then(Value::as_str) else {
        return json!({"status": "failed", "error": "missing generator_id"});
    };
    let outputs: Vec<String> = row
        .get("outputs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(ToString::to_string))
        .collect();
    if generator_id == "GEN-STATUS-FLAKY-TEST-LABELS" {
        return run_flaky_tests_generator(workspace_root);
    }
    let Some(source_script) = row.get("source_script").and_then(Value::as_str) else {
        return json!({
            "status": "failed",
            "generator_id": generator_id,
            "error": "missing source_script for python generator",
        });
    };
    run_python_generator(workspace_root, source_script, &outputs)
}

/// Builds `dev cli scripts generators` report payload.
#[must_use]
pub fn build_generators_report(workspace_root: &Path) -> Value {
    build_status_generators_report(workspace_root)
}

/// Runs one status generator by stable id or source path.
#[must_use]
pub fn run_generator(
    workspace_root: &Path,
    generator_id: Option<&str>,
    source_script: Option<&str>,
) -> Value {
    let rows = build_status_generators_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let selection = if let Some(id) = generator_id {
        rows.into_iter().find(|row| row.get("generator_id").and_then(Value::as_str) == Some(id))
    } else if let Some(source) = source_script {
        rows.into_iter()
            .find(|row| row.get("source_script").and_then(Value::as_str) == Some(source))
    } else {
        None
    };

    if let Some(row) = selection {
        return run_status_generator_entry(workspace_root, &row);
    }
    json!({
        "status": "failed",
        "error": "generator not found; pass --id or --source with a known status generator",
    })
}

/// Runs all status generators.
#[must_use]
pub fn run_all_generators(workspace_root: &Path) -> Value {
    let rows = build_status_generators_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut results = Vec::<Value>::new();
    let mut ok = 0usize;
    let mut failed = 0usize;
    for row in rows {
        let result = run_status_generator_entry(workspace_root, &row);
        if result.get("status").and_then(Value::as_str) == Some("ok") {
            ok += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }
    json!({
        "generated_at_utc": generated_at_utc(),
        "count": results.len(),
        "ok": ok,
        "failed": failed,
        "results": results,
    })
}

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
        "generator": "bijux dev cli scripts flaky-tests",
        "label": "flaky",
        "count": tests.len(),
        "tests": tests,
        "policy": "no flaky test may be silently ignored; each flaky marker requires remediation tracking",
        "generator": "crates/bijux-dev-cli/src/scripts.rs::build_flaky_tests_report",
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
        "status_generators": build_status_generators_report(workspace_root),
        "requirement_catalog": build_requirement_catalog(workspace_root),
        "flaky_tests": build_flaky_tests_report(workspace_root),
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
        build_audit_report, build_diff_report, build_generators_report, build_migrated_report,
        build_remaining_report, build_requirement_catalog_report,
    };

    #[test]
    fn scripts_reports_are_shaped() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(build_migrated_report(&root).get("migrated").is_some());
        assert!(build_remaining_report(&root).get("remaining_root_scripts").is_some());
        assert!(build_diff_report(&root).get("remaining").is_some());
        let audit = build_audit_report(&root);
        assert!(audit.get("diff").is_some());
        assert!(audit.get("status_generators").is_some());
        assert!(audit.get("requirement_catalog").is_some());
        assert!(audit.get("flaky_tests").is_some());
    }

    #[test]
    fn status_generator_ids_are_stable_and_prefixed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rows = build_generators_report(&root)
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(!rows.is_empty());
        for row in rows {
            let id = row.get("generator_id").and_then(serde_json::Value::as_str).unwrap_or("");
            assert!(id.starts_with("GEN-STATUS-"));
        }
    }

    #[test]
    fn requirement_ids_use_req_prefix() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rows = build_requirement_catalog_report(&root)
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        for row in rows {
            let id = row.get("requirement_id").and_then(serde_json::Value::as_str).unwrap_or("");
            assert!(id.starts_with("REQ-"));
        }
    }
}
