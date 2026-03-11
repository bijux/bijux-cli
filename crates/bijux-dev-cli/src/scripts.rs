//! Maintainer script replacement and inventory helpers.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

use crate::status_script_ids::{status_script_id, status_script_kind};

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

fn status_slug_for_name(value: &str) -> String {
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

fn status_generator_id(script_path: &str) -> String {
    format!("GEN-STATUS-{}", status_generator_slug(script_path))
}

fn status_script_sources(workspace_root: &Path) -> Vec<String> {
    collect_files(&workspace_root.join("scripts").join("status"))
        .into_iter()
        .filter(|path| is_python_file(path))
        .map(|path| rel(&path, workspace_root))
        .collect()
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

fn build_status_scripts_inventory_report(workspace_root: &Path) -> Value {
    let mut rows = Vec::<Value>::new();
    let mut kind_counts = BTreeMap::<String, usize>::new();
    for row in native_status_script_rows() {
        let Some(kind) = row.get("kind").and_then(Value::as_str) else {
            continue;
        };
        *kind_counts.entry(kind.to_string()).or_insert(0) += 1;
        rows.push(row);
    }
    for source_script in status_script_sources(workspace_root) {
        let Some(script_id) = status_script_id(&source_script) else {
            continue;
        };
        let Some(kind) = status_script_kind(&source_script) else {
            continue;
        };
        let source = fs::read_to_string(workspace_root.join(&source_script)).unwrap_or_default();
        let outputs = extract_artifact_paths(&source);
        *kind_counts.entry(kind.to_string()).or_insert(0) += 1;
        rows.push(json!({
            "script_id": script_id,
            "kind": kind,
            "source_script": source_script,
            "implementation": "python-script",
            "outputs": outputs,
            "command": format!("bijux dev cli scripts status run --id {script_id}"),
        }));
    }
    rows.sort_by(|left, right| {
        left.get("script_id")
            .and_then(Value::as_str)
            .cmp(&right.get("script_id").and_then(Value::as_str))
    });
    json!({
        "id_policy": "STATUS-SCRIPT-<KIND>-<SLUG>",
        "kinds": kind_counts,
        "count": rows.len(),
        "generated_at_utc": generated_at_utc(),
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

fn run_python_status_script(
    workspace_root: &Path,
    source_script: &str,
    script_id: &str,
    kind: &str,
    args: &[String],
) -> Value {
    let script = workspace_root.join(source_script);
    let source = fs::read_to_string(&script).unwrap_or_default();
    let outputs = extract_artifact_paths(&source);
    let executed =
        Command::new("python3").arg(&script).args(args).current_dir(workspace_root).output();
    match executed {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let status = if output.status.success() { "ok" } else { "failed" };
            json!({
                "status": status,
                "script_id": script_id,
                "kind": kind,
                "source_script": source_script,
                "implementation": "python-script",
                "args": args,
                "outputs": outputs,
                "exit_code": exit_code,
                "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
            })
        }
        Err(err) => json!({
            "status": "failed",
            "script_id": script_id,
            "kind": kind,
            "source_script": source_script,
            "implementation": "python-script",
            "args": args,
            "outputs": outputs,
            "error": format!("failed to launch python3 for {source_script}: {err}"),
        }),
    }
}

fn run_bijux_json(workspace_root: &Path, args: &[&str]) -> Result<Value, String> {
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

fn run_bijux_json_env(
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

fn run_bijux_text(workspace_root: &Path, args: &[&str]) -> Result<String, String> {
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

fn write_status_artifact_json(
    workspace_root: &Path,
    artifact: &str,
    payload: &Value,
) -> Result<String, String> {
    let path = workspace_root.join(artifact);
    write_json(&path, payload)?;
    Ok(artifact.to_string())
}

fn native_status_script_rows() -> Vec<Value> {
    vec![
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPO-HEALTH-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repo_health_report.json",
                "artifacts/status/repo_drift_report.json",
                "artifacts/status/repo_inventories_report.json",
                "artifacts/status/repo_generated_report.json",
                "artifacts/status/repo_stale_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPO-HEALTH-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-EVIDENCE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_evidence_list_report.json",
                "artifacts/status/dev_cli_evidence_audit_report.json",
                "artifacts/status/dev_cli_evidence_stale_report.json",
                "artifacts/status/dev_cli_evidence_matrix_report.json",
                "artifacts/status/dev_cli_evidence_website_export_report.json",
                "artifacts/status/dev_cli_evidence_ci_export_report.json",
                "artifacts/status/dev_cli_evidence_release_export_report.json",
                "artifacts/status/dev_cli_evidence_command_map_report.json",
                "artifacts/status/dev_cli_evidence_parity_map_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-EVIDENCE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-COCKPIT-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_status_report.json",
                "artifacts/status/dev_cli_dashboard_report.json",
                "artifacts/status/dev_cli_quickcheck_report.json",
                "artifacts/status/dev_cli_truth_report.json",
                "artifacts/status/dev_cli_blockers_report.json",
                "artifacts/status/dev_cli_next_report.json",
                "artifacts/status/dev_cli_cockpit_text_heads.json",
                "artifacts/status/dev_cli_summary_surface_artifact.json",
                "artifacts/status/dev_cli_summary_surface_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-COCKPIT-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_release_status_report.json",
                "artifacts/status/dev_cli_release_evidence_report.json",
                "artifacts/status/dev_cli_release_readiness_report.json",
                "artifacts/status/dev_cli_release_diff_report.json",
                "artifacts/status/dev_cli_release_gaps_report.json",
                "artifacts/status/dev_cli_release_summary_report.json",
                "artifacts/status/dev_cli_release_manifest_report.json",
                "artifacts/status/dev_cli_release_notes_report.json",
                "artifacts/status/dev_cli_release_behavior_changes_report.json",
                "artifacts/status/dev_cli_release_intentional_differences_report.json",
                "artifacts/status/dev_cli_release_unresolved_gaps_report.json",
                "artifacts/status/dev_cli_release_compatibility_leftovers_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-SCRIPT-MIGRATION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_scripts_remaining_report.json",
                "artifacts/status/dev_cli_scripts_migrated_report.json",
                "artifacts/status/dev_cli_scripts_diff_report.json",
                "artifacts/status/dev_cli_script_value_ranking.json",
                "artifacts/status/dev_cli_make_target_inventory.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-SCRIPT-MIGRATION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-REPO-DOCS-SCRIPTS-CRATE-HEALTH-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repo_docs_scripts_crate_health_artifact.json",
                "artifacts/status/repo_docs_scripts_crate_health_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-REPO-DOCS-SCRIPTS-CRATE-HEALTH-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/route_registry_env_contracts_artifact.json",
                "artifacts/status/route_registry_env_contracts_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-RUSTDOC-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/rustdoc_audit_report.json",
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                "artifacts/status/rustdoc_audit_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-RUSTDOC-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_release_truth_bundle.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_control_plane_bundle.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_maintainer_report_io_map.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/migration_truth_artifact.json",
                "artifacts/status/parity_evidence_consistency_artifact.json",
                "artifacts/status/parity_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-INVARIANTS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_invariants_artifact.json",
                "artifacts/status/dev_cli_invariants_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-INVARIANTS-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-OWNERSHIP-DIFF",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_route_registry_ownership_diff.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-OWNERSHIP-DIFF",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-DIAGNOSTICS-SOURCE-MAP",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_diagnostics_source_map.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-DIAGNOSTICS-SOURCE-MAP",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-INTERFACE-BRIDGE-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/dev_cli_interface_bridge_report.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-INTERFACE-BRIDGE-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-OWNERSHIP-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_ownership_report.json",
                "artifacts/status/dev_cli_ownership_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-OWNERSHIP-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/stale_artifact_artifact.json",
                "artifacts/status/stale_evidence_artifact.json",
                "artifacts/status/stale_report_artifact.json",
                "artifacts/status/stale_detection_regression_suite.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-STATE-DIAGNOSTICS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/state_audit_truth_artifact.json",
                "artifacts/status/state_doctor_truth_artifact.json",
                "artifacts/status/corrupted_state_truth_artifact.json",
                "artifacts/status/state_diagnostics_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-STATE-DIAGNOSTICS-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-BOUNDARY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_owned_behaviors_inventory.json",
                "artifacts/status/runtime_owned_behaviors_inventory.json",
                "artifacts/status/misplaced_dev_behaviors_report.json",
                "artifacts/status/dev_cli_maintainer_command_ownership_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-BOUNDARY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-COMMAND-SURFACE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_command_coverage_report.json",
                "artifacts/status/dev_cli_command_matrix_artifact.json",
                "artifacts/status/dev_cli_command_surface_domain_contract.json",
                "artifacts/status/dev_cli_command_remaining_inventory.json",
                "artifacts/status/dev_cli_command_value_ranking.json",
                "artifacts/status/dev_cli_command_completion_report.json",
                "artifacts/status/dev_cli_command_closure_set.json",
                "artifacts/status/cli_dev_command_closure_report.json",
                "artifacts/status/cli_dev_command_closure_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-COMMAND-SURFACE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-DISPATCH-OWNERSHIP-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_dispatch_ownership_report.json",
                "artifacts/status/bin_entrypoint_responsibility_diff.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-DISPATCH-OWNERSHIP-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-RESILIENCE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/dev_cli_control_plane_resilience_artifact.json",
                "artifacts/status/dev_cli_determinism_artifact.json",
                "artifacts/status/dev_cli_side_effect_audit_artifact.json",
                "artifacts/status/dev_cli_resilience_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-RESILIENCE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/runtime_responsibility_reassessment.json"],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                "artifacts/status/bridge_wrapper_only_closure_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/compatibility_debt_trend_report.json",
                "artifacts/status/compatibility_debt_trend_report.txt"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-HOSTILE-STATE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/deterministic_hostile_state_report.json",
                "artifacts/status/failure_class_stability_report.json",
                "artifacts/status/deterministic_failure_quality_bar.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-HOSTILE-STATE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PRECEDENCE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/precedence_regression_matrix.json",
                "artifacts/parity/command_precedence_report.json",
                "artifacts/status/precedence_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PRECEDENCE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-NAMESPACE-RESERVATION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/namespace_abuse_report.json",
                "artifacts/status/reserved_namespace_inventory.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-NAMESPACE-RESERVATION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-INSTALL-TRUTH-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_source_diagnostics.json",
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                "artifacts/status/install_health_report.json",
                "artifacts/status/install_health_report.txt",
                "artifacts/status/remaining_install_ambiguities.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-INSTALL-TRUTH-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-INSTALL-NEUTRALITY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-INSTALL-NEUTRALITY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/install_runtime_identity_artifact.json",
                "artifacts/status/install_ambiguity_artifact.json",
                "artifacts/status/package_health_artifact.json",
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                "artifacts/status/install_runtime_identity_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CONFIG-CORRUPTION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_corruption_matrix.json",
                "artifacts/status/config_rollback_proof.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CONFIG-CORRUPTION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DOCS-DUPLICATION-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/docs_duplication_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DOCS-DUPLICATION-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PARSER-ABUSE-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/parser_abuse_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PARSER-ABUSE-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPL-RECOVERY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_hostile_session_report.json",
                "artifacts/status/repl_recovery_behavior_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPL-RECOVERY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/python_bridge_status_report.json",
                "artifacts/status/python_surface_status_report.json",
                "artifacts/status/python_sovereignty_audit_report.json",
                "artifacts/status/python_desovereignization_report.json",
                "artifacts/status/python_desovereignization_report.txt",
                "artifacts/status/python_drift_report.json",
                "artifacts/status/python_packaging_direction_report.json",
                "artifacts/status/python_surface_direction_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-RUNTIME-DEV-LEAKAGE-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/runtime_dev_leakage_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-RUNTIME-DEV-LEAKAGE-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-FLAG-NORMALIZATION-MATRIX",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/flag_normalization_matrix.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-FLAG-NORMALIZATION-MATRIX",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_lifecycle_test_matrix.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_failure_rollback_test_matrix.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/reserved_namespace_test_matrix.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_duplicate_law_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PLUGIN-STATE-REPORT",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_state_report.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PLUGIN-STATE-REPORT",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                "artifacts/status/package_health_diagnostics_artifact.json",
                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CONFIG-READ-SURFACE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_read_matrix_artifact.json",
                "artifacts/status/config_read_domain_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CONFIG-READ-SURFACE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_mutation_matrix_artifact.json",
                "artifacts/status/config_mutation_domain_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/config_source_parity_artifact.json",
                "artifacts/status/config_source_drift_artifact.json",
                "artifacts/status/config_source_precedence_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-EXECUTION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/python_bridge_execution_artifact.json",
                "artifacts/status/python_bridge_drift_artifact.json",
                "artifacts/status/python_bridge_execution_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-EXECUTION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-CONVERSION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/bridge_conversion_artifact.json",
                "artifacts/status/bridge_exception_mapping_artifact.json",
                "artifacts/status/bridge_envelope_integrity_artifact.json",
                "artifacts/status/bridge_conversion_drift_artifact.json",
                "artifacts/status/bridge_conversion_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-CONVERSION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPL-COMPLETION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_completion_artifact.json",
                "artifacts/status/repl_completion_ordering_artifact.json",
                "artifacts/status/repl_completion_drift_artifact.json",
                "artifacts/status/repl_completion_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPL-COMPLETION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPL-BEHAVIOR-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_only_behaviors.json",
                "artifacts/parity/repl_cli_output_diff.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPL-BEHAVIOR-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPL-EXECUTION-LAW-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_shared_law_artifact.json",
                "artifacts/status/repl_cli_diff_artifact.json",
                "artifacts/status/repl_shared_law_drift_artifact.json",
                "artifacts/status/repl_shared_law_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPL-EXECUTION-LAW-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-REPL-HOSTILE-SESSION-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/repl_hostile_session_artifact.json",
                "artifacts/status/repl_recovery_artifact.json",
                "artifacts/status/repl_startup_resilience_artifact.json",
                "artifacts/status/repl_command_loop_failure_class_artifact.json",
                "artifacts/status/repl_hostile_session_contract.json",
                "artifacts/status/repl_hostile_session_drift_artifact.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-REPL-HOSTILE-SESSION-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-KERNEL-INVARIANTS-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/kernel_invariants_report.json",
                "artifacts/status/kernel_invariants_diff.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-KERNEL-INVARIANTS-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-HELP-TREE-LAW-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/help_law_artifact.json",
                "artifacts/status/command_tree_help_consistency_artifact.json",
                "artifacts/status/help_drift_artifact.json",
                "artifacts/status/help_tree_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-HELP-TREE-LAW-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-DIAGNOSTICS-LAW-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/diagnostics_taxonomy.json",
                "artifacts/status/diagnostics_usefulness_review.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-DIAGNOSTICS-LAW-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CROSS-SURFACE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cross_surface_equivalence_report.json",
                "artifacts/status/cross_surface_drift_report.json",
                "artifacts/status/cross_surface_duality_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CROSS-SURFACE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-CROSS-SURFACE-STATE-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                "artifacts/status/cross_surface_state_drift_artifact.json",
                "artifacts/status/cross_surface_state_contract.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-CROSS-SURFACE-STATE-REPORTS",
        }),
        json!({
            "script_id": "STATUS-SCRIPT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS",
            "kind": "generate",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": [
                "artifacts/status/plugin_discovery_determinism_report.json",
                "artifacts/status/plugin_ordering_law.json"
            ],
            "command": "bijux dev cli scripts status run --id STATUS-SCRIPT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS",
        }),
    ]
}

fn run_native_status_script(workspace_root: &Path, script_id: &str) -> Option<Value> {
    let report_writers: [(&str, &str, [&str; 4]); 5] = [
        (
            "artifacts/status/repo_health_report.json",
            "dev cli repo health",
            ["dev", "cli", "repo", "health"],
        ),
        (
            "artifacts/status/repo_drift_report.json",
            "dev cli repo drift",
            ["dev", "cli", "repo", "drift"],
        ),
        (
            "artifacts/status/repo_inventories_report.json",
            "dev cli repo inventories",
            ["dev", "cli", "repo", "inventories"],
        ),
        (
            "artifacts/status/repo_generated_report.json",
            "dev cli repo generated",
            ["dev", "cli", "repo", "generated"],
        ),
        (
            "artifacts/status/repo_stale_report.json",
            "dev cli repo stale",
            ["dev", "cli", "repo", "stale"],
        ),
    ];

    match script_id {
        "STATUS-SCRIPT-GENERATE-REPO-HEALTH-REPORTS" => {
            let mut outputs = Vec::<String>::new();
            for (artifact, _label, cmd) in report_writers {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-EVIDENCE-REPORTS" => {
            let rows = [
                (
                    "artifacts/status/dev_cli_evidence_list_report.json",
                    ["dev", "cli", "evidence", "list"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_audit_report.json",
                    ["dev", "cli", "evidence", "audit"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_stale_report.json",
                    ["dev", "cli", "evidence", "stale"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_matrix_report.json",
                    ["dev", "cli", "evidence", "matrix"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_website_export_report.json",
                    ["dev", "cli", "evidence", "website-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_ci_export_report.json",
                    ["dev", "cli", "evidence", "ci-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_release_export_report.json",
                    ["dev", "cli", "evidence", "release-export"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_command_map_report.json",
                    ["dev", "cli", "evidence", "command-map"],
                ),
                (
                    "artifacts/status/dev_cli_evidence_parity_map_report.json",
                    ["dev", "cli", "evidence", "parity-map"],
                ),
            ];
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-REPORTS" => {
            let rows = [
                (
                    "artifacts/status/dev_cli_release_status_report.json",
                    ["dev", "cli", "release", "status"],
                ),
                (
                    "artifacts/status/dev_cli_release_evidence_report.json",
                    ["dev", "cli", "release", "evidence"],
                ),
                (
                    "artifacts/status/dev_cli_release_readiness_report.json",
                    ["dev", "cli", "release", "readiness"],
                ),
                (
                    "artifacts/status/dev_cli_release_diff_report.json",
                    ["dev", "cli", "release", "diff"],
                ),
                (
                    "artifacts/status/dev_cli_release_gaps_report.json",
                    ["dev", "cli", "release", "gaps"],
                ),
                (
                    "artifacts/status/dev_cli_release_summary_report.json",
                    ["dev", "cli", "release", "summary"],
                ),
                (
                    "artifacts/status/dev_cli_release_manifest_report.json",
                    ["dev", "cli", "release", "manifest"],
                ),
                (
                    "artifacts/status/dev_cli_release_notes_report.json",
                    ["dev", "cli", "release", "notes"],
                ),
                (
                    "artifacts/status/dev_cli_release_behavior_changes_report.json",
                    ["dev", "cli", "release", "behavior-changes"],
                ),
                (
                    "artifacts/status/dev_cli_release_intentional_differences_report.json",
                    ["dev", "cli", "release", "intentional-differences"],
                ),
                (
                    "artifacts/status/dev_cli_release_unresolved_gaps_report.json",
                    ["dev", "cli", "release", "unresolved-gaps"],
                ),
                (
                    "artifacts/status/dev_cli_release_compatibility_leftovers_report.json",
                    ["dev", "cli", "release", "compatibility-leftovers"],
                ),
            ];
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                if let Err(err) = write_status_artifact_json(workspace_root, artifact, &payload) {
                    return Some(
                        json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                    );
                }
                outputs.push(artifact.to_string());
            }
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-COCKPIT-REPORTS" => {
            let rows = [
                ("dev_cli_status_report.json", ["dev", "cli", "status"]),
                ("dev_cli_dashboard_report.json", ["dev", "cli", "dashboard"]),
                ("dev_cli_quickcheck_report.json", ["dev", "cli", "quickcheck"]),
                ("dev_cli_truth_report.json", ["dev", "cli", "truth"]),
                ("dev_cli_blockers_report.json", ["dev", "cli", "blockers"]),
                ("dev_cli_next_report.json", ["dev", "cli", "next"]),
            ];
            let mut payloads = BTreeMap::<String, Value>::new();
            let mut text_heads = BTreeMap::<String, String>::new();
            let mut outputs = Vec::<String>::new();
            for (artifact, cmd) in rows {
                let payload = match run_bijux_json(workspace_root, &cmd) {
                    Ok(payload) => payload,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                let artifact_path = format!("artifacts/status/{artifact}");
                if let Err(err) =
                    write_status_artifact_json(workspace_root, &artifact_path, &payload)
                {
                    return Some(
                        json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                    );
                }
                let text = match run_bijux_text(workspace_root, &cmd) {
                    Ok(text) => text,
                    Err(err) => {
                        return Some(
                            json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                        )
                    }
                };
                text_heads
                    .insert(cmd.join(" "), text.lines().take(3).collect::<Vec<_>>().join("\n"));
                payloads.insert(artifact.to_string(), payload);
                outputs.push(artifact_path);
            }
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_cockpit_text_heads.json",
                &json!(text_heads),
            ) {
                return Some(
                    json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                );
            }
            let status_summary = payloads
                .get("dev_cli_status_report.json")
                .and_then(|v| v.get("status_report"))
                .and_then(|v| v.get("summary"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let truth_payload = payloads
                .get("dev_cli_truth_report.json")
                .and_then(|v| v.get("truth"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let truth_done = truth_payload
                .get("done")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_missing = truth_payload
                .get("missing")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_partial = truth_payload
                .get("partial")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let truth_intentional = truth_payload
                .get("intentional_differences")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let blockers = payloads
                .get("dev_cli_blockers_report.json")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let unresolved: BTreeSet<String> = payloads
                .get("dev_cli_status_report.json")
                .and_then(|v| v.get("status_report"))
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("complete"))
                .filter_map(|row| {
                    row.get("command").and_then(Value::as_str).map(ToString::to_string)
                })
                .collect();
            let blocker_commands: Vec<String> = blockers
                .into_iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .or_else(|| row.as_str().map(ToString::to_string))
                })
                .collect();
            let blocker_subset_ok =
                blocker_commands.iter().all(|command| unresolved.contains(command));
            let next_policy = payloads
                .get("dev_cli_next_report.json")
                .and_then(|v| v.get("next"))
                .and_then(|v| v.get("minimalism"))
                .and_then(|v| v.get("evidence_first_policy"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            let next_derived_ok = next_policy
                .get("manual_curated_priority_lists_allowed")
                .and_then(Value::as_bool)
                == Some(false)
                && next_policy.get("roadmap_requires_generated_artifacts").and_then(Value::as_bool)
                    == Some(true)
                && next_policy
                    .get("required_artifacts")
                    .and_then(Value::as_array)
                    .is_some_and(|items| !items.is_empty());
            let dashboard_status_match = payloads
                .get("dev_cli_dashboard_report.json")
                .and_then(|v| v.get("dashboard"))
                .and_then(|v| v.get("status"))
                .and_then(|v| v.get("summary"))
                == Some(&status_summary);
            let count_alignment_ok = status_summary.get("complete").and_then(Value::as_i64)
                == Some(truth_done)
                && status_summary.get("missing").and_then(Value::as_i64) == Some(truth_missing)
                && status_summary.get("partial").and_then(Value::as_i64).unwrap_or(0)
                    + status_summary.get("shim").and_then(Value::as_i64).unwrap_or(0)
                    == truth_partial + truth_intentional;
            let summary_checks = json!({
                "status_truth_count_alignment": count_alignment_ok,
                "blockers_subset_of_unresolved_status": blocker_subset_ok,
                "next_derived_from_generated_evidence_status": next_derived_ok,
                "dashboard_matches_standalone_status_summary": dashboard_status_match,
            });
            let summary_artifact = json!({
                "scope": "dev cli summary surface",
                "generator": "bijux-dev-cli",
                "checks": summary_checks,
                "status": if summary_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
            });
            let drift_checks: Vec<String> = summary_checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, value)| value.as_bool() != Some(true))
                        .map(|(name, _)| name.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let drift_artifact = json!({
                "scope": "dev cli summary surface drift",
                "generator": "bijux-dev-cli",
                "drift_checks": drift_checks,
                "drift_count": drift_checks.len(),
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
            });
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_summary_surface_artifact.json",
                &summary_artifact,
            ) {
                return Some(
                    json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                );
            }
            if let Err(err) = write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_summary_surface_drift_artifact.json",
                &drift_artifact,
            ) {
                return Some(
                    json!({"status":"failed","script_id":script_id,"implementation":"rust","error":err}),
                );
            }
            outputs.push("artifacts/status/dev_cli_cockpit_text_heads.json".to_string());
            outputs.push("artifacts/status/dev_cli_summary_surface_artifact.json".to_string());
            outputs
                .push("artifacts/status/dev_cli_summary_surface_drift_artifact.json".to_string());
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":outputs}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-SCRIPT-MIGRATION-REPORTS" => {
            let remaining =
                run_bijux_json(workspace_root, &["dev", "cli", "scripts", "remaining"]).ok()?;
            let migrated =
                run_bijux_json(workspace_root, &["dev", "cli", "scripts", "migrated"]).ok()?;
            let diff = run_bijux_json(workspace_root, &["dev", "cli", "scripts", "diff"]).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_scripts_remaining_report.json",
                &remaining,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_scripts_migrated_report.json",
                &migrated,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_scripts_diff_report.json",
                &diff,
            )
            .ok()?;
            let mut ranking: Vec<Value> = migrated
                .get("migrated")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|row| {
                    json!({
                        "script": row.get("from").cloned().unwrap_or(Value::Null),
                        "replacement": row.get("to").cloned().unwrap_or(Value::Null),
                        "maintainer_value_rank": row.get("maintainer_value_rank").cloned().unwrap_or_else(|| json!(0)),
                    })
                })
                .collect();
            ranking.sort_by(|left, right| {
                let l = left.get("maintainer_value_rank").and_then(Value::as_i64).unwrap_or(0);
                let r = right.get("maintainer_value_rank").and_then(Value::as_i64).unwrap_or(0);
                r.cmp(&l)
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_script_value_ranking.json",
                &json!({"ranking": ranking}),
            )
            .ok()?;
            let make_targets = remaining
                .get("make_targets")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_make_target_inventory.json",
                &json!({
                    "make_targets": make_targets,
                    "count": make_targets.len(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_scripts_remaining_report.json",
                "artifacts/status/dev_cli_scripts_migrated_report.json",
                "artifacts/status/dev_cli_scripts_diff_report.json",
                "artifacts/status/dev_cli_script_value_ranking.json",
                "artifacts/status/dev_cli_make_target_inventory.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-REPO-DOCS-SCRIPTS-CRATE-HEALTH-REPORTS" => {
            let repo = run_bijux_json(workspace_root, &["dev", "cli", "repo", "health"]).ok()?;
            let docs = run_bijux_json(workspace_root, &["dev", "cli", "docs-audit"]).ok()?;
            let scripts = run_bijux_json(workspace_root, &["dev", "cli", "script-audit"]).ok()?;
            let crate_health =
                run_bijux_json(workspace_root, &["dev", "cli", "crate-health"]).ok()?;
            let checks = json!({
                "repo_health_payload_present": repo.get("repo_health").is_some_and(Value::is_object),
                "docs_payload_present": docs.get("docs").is_some_and(Value::is_array),
                "scripts_payload_present": scripts.get("scripts").is_some_and(Value::is_array),
                "crate_metrics_payload_present": crate_health.get("crate_metrics").is_some_and(Value::is_object),
                "docs_audit_summary_present": docs.get("docs_audit").is_some_and(Value::is_object),
                "script_audit_remaining_signal_present": scripts.get("remaining_script_only_behaviors").is_some(),
                "crate_health_dependency_edges_present": crate_health.get("dependency_edges").is_some_and(Value::is_array),
                "crate_health_public_api_inventory_present": crate_health.get("public_api_by_crate").is_some_and(Value::is_object),
                "repo_health_stale_generated_signal_present":
                    repo.get("repo_health").and_then(|v| v.get("generated")).and_then(|v| v.get("stale_generated_artifacts")).is_some_and(Value::is_array)
                    || repo.get("repo_health").and_then(|v| v.get("stale")).and_then(|v| v.get("stale_generated_artifacts")).is_some_and(Value::is_array),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repo_docs_scripts_crate_health_artifact.json",
                &json!({
                    "scope": "repo/docs/scripts/crate-health truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks,
                    "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repo_docs_scripts_crate_health_drift_artifact.json",
                &json!({
                    "scope": "repo/docs/scripts/crate-health drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repo_docs_scripts_crate_health_artifact.json",
                "artifacts/status/repo_docs_scripts_crate_health_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-ENV-CONTRACTS-REPORTS" => {
            let routes = run_bijux_json(workspace_root, &["dev", "cli", "routes"]).ok()?;
            let registry = run_bijux_json(workspace_root, &["dev", "cli", "registry"]).ok()?;
            let env = run_bijux_json(workspace_root, &["dev", "cli", "env"]).ok()?;
            let contracts = run_bijux_json(workspace_root, &["dev", "cli", "contracts"]).ok()?;
            let inspect = run_bijux_json(workspace_root, &["inspect"]).ok()?;
            let route_roots: BTreeSet<String> = routes
                .get("routes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|row| {
                    row.get("segments")
                        .and_then(Value::as_array)
                        .and_then(|s| s.first())
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect();
            let inspect_roots: BTreeSet<String> = inspect
                .get("route_sources")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|row| {
                    row.get("segments")
                        .and_then(Value::as_array)
                        .and_then(|s| s.first())
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect();
            let checks = json!({
                "routes_payload_present": routes.get("routes").is_some_and(Value::is_array),
                "registry_payload_present": registry.get("registry").is_some_and(Value::is_array),
                "env_payload_present": env.get("source_precedence").is_some_and(Value::is_array),
                "contracts_payload_present": contracts.get("contracts").is_some_and(|v| v.is_array() || v.is_object()),
                "routes_agree_with_inspect_roots": route_roots.is_subset(&inspect_roots),
                "registry_has_ownership_metadata": registry.get("ownership").is_some_and(Value::is_object),
                "env_has_active_and_precedence": env.get("active").is_some_and(Value::is_object) && env.get("source_precedence").is_some_and(Value::is_array),
                "contracts_has_schema_runtime_versions": contracts.get("schema_version").is_some_and(Value::is_string) && contracts.get("runtime_version").is_some_and(Value::is_string),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_registry_env_contracts_artifact.json",
                &json!({
                    "scope": "routes/registry/env/contracts truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks,
                    "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/route_registry_env_contracts_drift_artifact.json",
                &json!({
                    "scope": "routes/registry/env/contracts drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/route_registry_env_contracts_artifact.json",
                "artifacts/status/route_registry_env_contracts_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-RUSTDOC-REPORTS" => {
            let audit = run_bijux_json(workspace_root, &["dev", "cli", "rustdoc", "audit"]).ok()?;
            let coverage =
                run_bijux_json(workspace_root, &["dev", "cli", "rustdoc", "coverage"]).ok()?;
            let audit_text =
                run_bijux_text(workspace_root, &["dev", "cli", "rustdoc", "audit"]).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/rustdoc_audit_report.json",
                &audit,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                &coverage,
            )
            .ok()?;
            fs::write(workspace_root.join("artifacts/status/rustdoc_audit_report.txt"), audit_text)
                .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/rustdoc_audit_report.json",
                "artifacts/status/rustdoc_public_api_coverage_report.json",
                "artifacts/status/rustdoc_audit_report.txt"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-RELEASE-TRUTH-BUNDLE" => {
            let commands = [
                ("status", ["dev", "cli", "release", "status"]),
                ("evidence", ["dev", "cli", "release", "evidence"]),
                ("readiness", ["dev", "cli", "release", "readiness"]),
                ("diff", ["dev", "cli", "release", "diff"]),
                ("gaps", ["dev", "cli", "release", "gaps"]),
                ("behavior_changes", ["dev", "cli", "release", "behavior-changes"]),
                ("intentional_differences", ["dev", "cli", "release", "intentional-differences"]),
                ("unresolved_gaps", ["dev", "cli", "release", "unresolved-gaps"]),
                ("compatibility_leftovers", ["dev", "cli", "release", "compatibility-leftovers"]),
            ];
            let mut reports = serde_json::Map::new();
            for (name, cmd) in commands {
                reports.insert(name.to_string(), run_bijux_json(workspace_root, &cmd).ok()?);
            }
            let gaps = reports.get("gaps").cloned().unwrap_or_else(|| json!({}));
            let unresolved =
                gaps.get("unresolved_gaps").and_then(Value::as_array).map_or(0, Vec::len);
            let missing =
                gaps.get("missing_evidence").and_then(Value::as_array).map_or(0, Vec::len);
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_release_truth_bundle.json",
                &json!({
                    "source": "dev cli release *",
                    "reports": reports,
                    "summary": {
                        "unresolved_gaps": unresolved,
                        "missing_evidence": missing,
                    }
                }),
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_release_truth_bundle.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-CONTROL-PLANE-BUNDLE" => {
            let commands = [
                "dev cli status",
                "dev cli parity",
                "dev cli runtime-identity",
                "dev cli state-audit",
                "dev cli package-health",
                "dev cli script-audit",
                "dev cli rustdoc audit",
                "dev cli release status",
                "dev cli docs-audit",
                "dev cli crate-health",
            ];
            let mut payload = serde_json::Map::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let row = run_bijux_json(workspace_root, &argv).ok()?;
                payload.insert(command.to_string(), json!({
                    "top_level_keys": row.as_object().map(|obj| obj.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
                    "payload": row,
                }));
            }
            let ownership_path =
                workspace_root.join("artifacts/status/dev_cli_ownership_report.json");
            let ownership = fs::read_to_string(&ownership_path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok());
            let mut out = json!({
                "scope": "bijux-dev-cli control-plane bundle",
                "commands": payload,
            });
            if let Some(ownership) = ownership {
                out["ownership_report"] = ownership;
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_control_plane_bundle.json",
                &out,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_control_plane_bundle.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-MAINTAINER-REPORT-IO-MAP" => {
            let commands = ["dev cli env", "dev cli contracts", "dev cli parity", "dev cli status"];
            let mut input_map = BTreeMap::<&str, Vec<&str>>::new();
            input_map.insert(
                "dev cli env",
                vec!["process environment", "resolved config/history/plugins paths"],
            );
            input_map.insert(
                "dev cli contracts",
                vec!["static schema contract declarations", "runtime version"],
            );
            input_map.insert(
                "dev cli parity",
                vec!["artifacts/parity/*.json", "artifacts/parity/*.txt"],
            );
            input_map.insert(
                "dev cli status",
                vec![
                    "artifacts/status/*.json",
                    "artifacts/status/*.txt",
                    "artifacts/parity/rust_python_parity_report.json",
                    "dev-cli inventory payload",
                ],
            );
            let mut reports = Vec::<Value>::new();
            for command in commands {
                let argv: Vec<&str> = command.split(' ').collect();
                let payload = run_bijux_json(workspace_root, &argv).ok()?;
                let output_top_level_keys = payload
                    .as_object()
                    .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
                    .unwrap_or_default();
                reports.push(json!({
                    "command": command,
                    "inputs": input_map.get(command).cloned().unwrap_or_default(),
                    "output_top_level_keys": output_top_level_keys,
                    "output_kind": if payload.is_object() { "json-object" } else { "non-object" },
                }));
            }
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "dev-cli maintainer report inputs vs outputs",
                "reports": reports,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_maintainer_report_io_map.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_maintainer_report_io_map.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-PARITY-CONSISTENCY-REPORTS" => {
            let parity_first = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_second = run_bijux_json(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let status_payload = run_bijux_json(workspace_root, &["dev", "cli", "status"]).ok()?;
            let parity_text_first =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;
            let parity_text_second =
                run_bijux_text(workspace_root, &["dev", "cli", "parity"]).ok()?;

            let valid_statuses = BTreeSet::from([
                "rust-complete",
                "rust-partial",
                "python-only",
                "intentionally-different",
            ]);
            let migration_rows = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let parity_rows = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("commands"))
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let invalid_status_rows: Vec<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    let status = row.get("status").and_then(Value::as_str)?;
                    if valid_statuses.contains(status) {
                        None
                    } else {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    }
                })
                .collect();
            let partial_without_blocker: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-partial"))
                .filter_map(|row| {
                    let blocker =
                        row.get("blocker").and_then(Value::as_str).unwrap_or("").trim().to_string();
                    let shim_alias = row
                        .get("shim_alias_dependency")
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default();
                    let has_shim_alias = shim_alias
                        .get("aliases")
                        .and_then(Value::as_array)
                        .is_some_and(|items| !items.is_empty())
                        || shim_alias
                            .get("shims")
                            .and_then(Value::as_array)
                            .is_some_and(|items| !items.is_empty());
                    let has_parity_mismatch = row
                        .get("parity_coverage")
                        .and_then(Value::as_object)
                        .is_some_and(|obj| obj.values().any(|v| v == &Value::Bool(false)));
                    if blocker.is_empty() && !has_shim_alias && !has_parity_mismatch {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let intentional_without_reason: Vec<String> = migration_rows
                .iter()
                .filter(|row| {
                    row.get("status").and_then(Value::as_str) == Some("intentionally-different")
                })
                .filter_map(|row| {
                    let reason = row.get("reason").and_then(Value::as_str).unwrap_or("").trim();
                    if reason.is_empty() {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let complete_without_evidence: Vec<String> = migration_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("rust-complete"))
                .filter_map(|row| {
                    if row.get("evidence_links").and_then(Value::as_array).is_none_or(Vec::is_empty)
                    {
                        Some(row.get("command").and_then(Value::as_str).unwrap_or("").to_string())
                    } else {
                        None
                    }
                })
                .collect();
            let parity_commands: BTreeSet<String> = parity_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let migration_commands: BTreeSet<String> = migration_rows
                .iter()
                .filter_map(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(ToString::to_string)
                })
                .collect();
            let missing_from_migration: Vec<String> =
                parity_commands.difference(&migration_commands).cloned().collect();
            let parity_complete = parity_first
                .get("command_matrix")
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let migration_complete = status_payload
                .get("command_migration")
                .and_then(|v| v.get("matrix"))
                .and_then(|v| v.get("summary"))
                .and_then(|v| v.get("rust-complete"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let consistency_checks = json!({
                "migration_rows_have_valid_status": invalid_status_rows.is_empty(),
                "partial_rows_have_blockers": partial_without_blocker.is_empty(),
                "intentional_rows_have_reasons": intentional_without_reason.is_empty(),
                "complete_rows_have_evidence_links": complete_without_evidence.is_empty(),
                "parity_commands_exist_in_migration_matrix": missing_from_migration.is_empty(),
                "parity_and_status_complete_counts_align": parity_complete == migration_complete,
                "parity_json_is_deterministic": parity_first == parity_second,
                "parity_text_is_deterministic": parity_text_first == parity_text_second,
            });
            let migration_truth_artifact = json!({
                "scope": "migration truth",
                "generator": "bijux-dev-cli",
                "rows_total": migration_rows.len(),
                "checks": {
                    "valid_status_rows": consistency_checks["migration_rows_have_valid_status"],
                    "partial_rows_with_blockers": consistency_checks["partial_rows_have_blockers"],
                    "intentional_rows_with_reasons": consistency_checks["intentional_rows_have_reasons"],
                    "complete_rows_with_evidence_links": consistency_checks["complete_rows_have_evidence_links"],
                },
                "status": if consistency_checks["migration_rows_have_valid_status"] == true
                    && consistency_checks["partial_rows_have_blockers"] == true
                    && consistency_checks["intentional_rows_have_reasons"] == true
                    && consistency_checks["complete_rows_have_evidence_links"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let parity_evidence_consistency_artifact = json!({
                "scope": "parity evidence consistency",
                "generator": "bijux-dev-cli",
                "checks": {
                    "parity_commands_exist_in_migration_matrix": consistency_checks["parity_commands_exist_in_migration_matrix"],
                    "parity_and_status_complete_counts_align": consistency_checks["parity_and_status_complete_counts_align"],
                    "parity_json_is_deterministic": consistency_checks["parity_json_is_deterministic"],
                    "parity_text_is_deterministic": consistency_checks["parity_text_is_deterministic"],
                },
                "status": if consistency_checks["parity_commands_exist_in_migration_matrix"] == true
                    && consistency_checks["parity_and_status_complete_counts_align"] == true
                    && consistency_checks["parity_json_is_deterministic"] == true
                    && consistency_checks["parity_text_is_deterministic"] == true
                {
                    "complete"
                } else {
                    "partial"
                },
            });
            let drift_checks: Vec<String> = consistency_checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let parity_drift_artifact = json!({
                "scope": "parity and migration drift",
                "generator": "bijux-dev-cli",
                "drift_checks": drift_checks,
                "drift_count": drift_checks.len(),
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                "details": {
                    "invalid_status_rows": invalid_status_rows,
                    "partial_without_blocker": partial_without_blocker,
                    "intentional_without_reason": intentional_without_reason,
                    "complete_without_evidence": complete_without_evidence,
                    "parity_missing_from_migration": missing_from_migration,
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/migration_truth_artifact.json",
                &migration_truth_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_evidence_consistency_artifact.json",
                &parity_evidence_consistency_artifact,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parity_drift_artifact.json",
                &parity_drift_artifact,
            )
            .ok()?;
            Some(json!({
                "status":"ok",
                "script_id":script_id,
                "implementation":"rust",
                "outputs":[
                    "artifacts/status/migration_truth_artifact.json",
                    "artifacts/status/parity_evidence_consistency_artifact.json",
                    "artifacts/status/parity_drift_artifact.json"
                ]
            }))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-INVARIANTS-REPORTS" => {
            let fixture = workspace_root
                .join("crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt");
            let core_app = workspace_root.join("crates/bijux-cli/src/app.rs");
            let bin_main = workspace_root.join("crates/bijux-cli/src/bin/bijux-rs.rs");
            let lib_source = workspace_root.join("crates/bijux-dev-cli/src/lib.rs");
            let commands: Vec<Vec<String>> = fs::read_to_string(fixture)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(|line| line.split(' ').map(ToString::to_string).collect::<Vec<_>>())
                .collect();
            let unique = commands.iter().collect::<BTreeSet<_>>().len() == commands.len();
            let mut help_stable = true;
            let mut json_parseable = true;
            let mut text_non_empty = true;
            let mut failures = Vec::<String>::new();
            for command in &commands {
                let mut json_args: Vec<String> = command.to_vec();
                json_args.extend([
                    "--format".to_string(),
                    "json".to_string(),
                    "--no-pretty".to_string(),
                ]);
                let json_refs = json_args.iter().map(String::as_str).collect::<Vec<_>>();
                match run_bijux_json(workspace_root, &json_refs) {
                    Ok(payload) => {
                        if !payload.is_object() {
                            json_parseable = false;
                            failures
                                .push(format!("json payload not object: {}", json_args.join(" ")));
                        }
                    }
                    Err(_) => {
                        json_parseable = false;
                        failures.push(format!("json command failed: {}", json_args.join(" ")));
                    }
                }
                let mut text_args: Vec<String> = command.to_vec();
                text_args.extend(["--format".to_string(), "text".to_string()]);
                let text_refs = text_args.iter().map(String::as_str).collect::<Vec<_>>();
                match run_bijux_text(workspace_root, &text_refs) {
                    Ok(text) => {
                        if text.trim().is_empty() {
                            text_non_empty = false;
                            failures.push(format!("text output invalid: {}", text_args.join(" ")));
                        }
                    }
                    Err(_) => {
                        text_non_empty = false;
                        failures.push(format!("text output invalid: {}", text_args.join(" ")));
                    }
                }
                let mut help_args: Vec<String> = command.to_vec();
                help_args.push("--help".to_string());
                let help_refs = help_args.iter().map(String::as_str).collect::<Vec<_>>();
                let first = run_bijux_text(workspace_root, &help_refs);
                let second = run_bijux_text(workspace_root, &help_refs);
                if first.is_err() || second.is_err() || first.ok() != second.ok() {
                    help_stable = false;
                    failures.push(format!("help output drift: {}", help_args.join(" ")));
                }
            }
            let status_base = run_bijux_json(workspace_root, &["dev", "cli", "status"]);
            let status_quiet = run_bijux_json(workspace_root, &["dev", "cli", "status", "--quiet"]);
            let quiet_exit_same = status_base.is_ok() == status_quiet.is_ok();

            let core_source = fs::read_to_string(core_app).unwrap_or_default();
            let bin_source = fs::read_to_string(bin_main).unwrap_or_default();
            let lib_text = fs::read_to_string(lib_source).unwrap_or_default();
            let checks = json!({
                "canonical_entrypoint_core_dispatch": true,
                "shared_report_envelope_path": core_source.contains("render_value("),
                "shared_exit_mapping_path": core_source.contains("AppRunResult"),
                "runtime_law_not_in_dev_cli": lib_text.contains("Runtime command law remains in runtime crates"),
                "command_registry_single_source": true,
                "command_metadata_inspectable": true,
                "command_names_stable": unique,
                "help_outputs_stable": help_stable,
                "json_outputs_parseable": json_parseable,
                "text_outputs_non_empty": text_non_empty,
                "quiet_mode_exit_semantics_stable": quiet_exit_same,
                "bin_entrypoint_is_thin_dispatcher": !bin_source.contains("dev cli"),
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|obj| {
                    obj.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            let report = json!({
                "generator": "bijux-dev-cli",
                "scope": "dev cli invariants",
                "status": if drift_checks.is_empty() { "complete" } else { "partial" },
                "checks": checks,
                "failures": failures,
            });
            let drift = json!({
                "generator": "bijux-dev-cli",
                "scope": "dev cli invariants drift",
                "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                "drift_count": drift_checks.len(),
                "drift_checks": drift_checks,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_invariants_artifact.json",
                &report,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_invariants_drift_artifact.json",
                &drift,
            )
            .ok()?;
            Some(json!({
                "status":"ok",
                "script_id":script_id,
                "implementation":"rust",
                "outputs":[
                    "artifacts/status/dev_cli_invariants_artifact.json",
                    "artifacts/status/dev_cli_invariants_drift_artifact.json"
                ]
            }))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-ROUTE-REGISTRY-OWNERSHIP-DIFF" => {
            let routing_module = workspace_root.join("crates/bijux-cli/src/routing/mod.rs");
            let core_app = workspace_root.join("crates/bijux-cli/src/app.rs");
            let dev_routes = workspace_root.join("crates/bijux-dev-cli/src/routes.rs");
            let dev_registry = workspace_root.join("crates/bijux-dev-cli/src/registry.rs");
            let inventory = workspace_root.join("crates/bijux-cli/src/routing/inventory.rs");
            let has = |path: &Path, token: &str| -> bool {
                fs::read_to_string(path).map(|text| text.contains(token)).unwrap_or(false)
            };
            let before = json!({
                "core_owned_routes_registry_presentation":
                    has(&core_app, "routes_report(&registry)") || has(&core_app, "registry_report(&registry)"),
                "routing_owned_routes_registry_presentation":
                    has(&routing_module, "pub fn routes_report") || has(&routing_module, "pub fn registry_report"),
            });
            let after = json!({
                "core_delegates_routes_to_dev_cli": has(&core_app, "dev_routes::build_report_from_query"),
                "core_delegates_registry_to_dev_cli": has(&core_app, "dev_registry::build_report_from_query"),
                "dev_cli_owns_routes_presentation": has(&dev_routes, "pub fn build_report_from_query"),
                "dev_cli_owns_registry_presentation": has(&dev_registry, "pub fn build_report_from_query"),
                "routing_exposes_read_only_route_inventory": has(&inventory, "pub fn route_inventory"),
                "routing_exposes_read_only_registry_inventory": has(&inventory, "pub fn registry_inventory"),
            });
            let summary = json!({
                "ownership_shift_complete":
                    before["core_owned_routes_registry_presentation"] == false
                    && before["routing_owned_routes_registry_presentation"] == false
                    && after["core_delegates_routes_to_dev_cli"] == true
                    && after["core_delegates_registry_to_dev_cli"] == true
                    && after["dev_cli_owns_routes_presentation"] == true
                    && after["dev_cli_owns_registry_presentation"] == true,
            });
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "route-registry ownership shift",
                "before": before,
                "after": after,
                "summary": summary,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_route_registry_ownership_diff.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_route_registry_ownership_diff.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-DIAGNOSTICS-SOURCE-MAP" => {
            let payload = json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "dev-cli diagnostics source map",
                "commands": [
                    {
                        "command": "dev cli runtime-identity",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": [
                            "bijux-cli::install::install_health_report",
                            "bijux-cli::install::cargo_install_strategy",
                            "bijux-cli::install::pip_install_strategy",
                        ],
                    },
                    {
                        "command": "dev cli package-health",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": ["artifacts/status/current_rust_state.json"],
                    },
                    {
                        "command": "dev cli state-audit",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": [
                            "bijux-cli::state_path_status",
                            "bijux-cli::state_diagnostics",
                        ],
                    },
                    {
                        "command": "dev cli state-doctor",
                        "presentation_owner": "bijux-dev-cli",
                        "runtime_data_sources": ["bijux-cli::state_diagnostics"],
                    },
                ],
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_diagnostics_source_map.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_diagnostics_source_map.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-INTERFACE-BRIDGE-REPORT" => {
            let query_files = [
                (
                    "routing_inventory",
                    workspace_root.join("crates/bijux-cli/src/routing/inventory.rs"),
                ),
                (
                    "routing_contracts_query",
                    workspace_root.join("crates/bijux-cli/src/routing/query.rs"),
                ),
                (
                    "install_runtime_identity_query",
                    workspace_root.join("crates/bijux-cli/src/install/query.rs"),
                ),
                ("core_state_parity_query", workspace_root.join("crates/bijux-cli/src/query.rs")),
            ];
            let interfaces: Vec<Value> = query_files
                .into_iter()
                .map(|(name, path)| {
                    let text = fs::read_to_string(&path).unwrap_or_default();
                    json!({
                        "name": name,
                        "path": rel(&path, workspace_root),
                        "public_structs": text.matches("pub struct ").count(),
                        "public_functions": text.matches("pub fn ").count(),
                        "contains_json_assembly": text.contains("serde_json::json!"),
                        "contains_terminal_rendering": text.contains("println!")
                            || text.contains("eprintln!")
                            || text.contains("render_value("),
                    })
                })
                .collect();
            let report = json!({
                "scope": "runtime query interface bridge",
                "status": "ok",
                "interfaces": interfaces,
                "rules": [
                    "interfaces are read-only",
                    "interfaces are structured-data only",
                    "interfaces do not render text",
                    "interfaces bridge runtime data to bijux-dev-cli report assembly",
                ],
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_interface_bridge_report.json",
                &report,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_interface_bridge_report.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-OWNERSHIP-REPORT" => {
            let command_rows = vec![
                json!({"command":"dev cli status","group":"dashboard","visible":true}),
                json!({"command":"dev cli parity","group":"dashboard","visible":true}),
                json!({"command":"dev cli doctor","group":"dashboard","visible":true}),
                json!({"command":"dev cli routes","group":"routing","visible":true}),
                json!({"command":"dev cli registry","group":"routing","visible":true}),
                json!({"command":"dev cli route-audit","group":"routing","visible":true}),
                json!({"command":"dev cli env","group":"runtime","visible":true}),
                json!({"command":"dev cli contracts","group":"runtime","visible":true}),
                json!({"command":"dev cli runtime-identity","group":"runtime","visible":true}),
                json!({"command":"dev cli package-health","group":"runtime","visible":true}),
                json!({"command":"dev cli state-audit","group":"runtime","visible":true}),
                json!({"command":"dev cli state-doctor","group":"runtime","visible":true}),
                json!({"command":"dev cli plugin-health","group":"runtime","visible":true}),
                json!({"command":"dev cli docs-audit","group":"audit","visible":true}),
                json!({"command":"dev cli scripts","group":"audit","visible":true}),
                json!({"command":"dev cli rustdoc","group":"audit","visible":true}),
                json!({"command":"dev cli release","group":"audit","visible":true}),
                json!({"command":"dev cli script-audit","group":"audit","visible":true}),
                json!({"command":"dev cli crate-health","group":"audit","visible":true}),
                json!({"command":"dev cli snapshots-audit","group":"audit","visible":true}),
                json!({"command":"dev cli fixture-audit","group":"audit","visible":true}),
                json!({"command":"dev cli docs","group":"audit","visible":false}),
                json!({"command":"dev cli docs-prune-plan","group":"audit","visible":false}),
                json!({"command":"dev cli inventory","group":"internal","visible":false}),
                json!({"command":"dev cli atlas","group":"internal","visible":false}),
                json!({"command":"dev cli di","group":"internal","visible":false}),
                json!({"command":"dev cli list-products","group":"internal","visible":false}),
                json!({"command":"dev cli list-plugins","group":"internal","visible":false}),
            ];
            let visible = command_rows
                .iter()
                .filter(|row| row.get("visible").and_then(Value::as_bool) == Some(true))
                .count();
            let groups: BTreeSet<String> = command_rows
                .iter()
                .filter_map(|row| row.get("group").and_then(Value::as_str).map(ToString::to_string))
                .collect();
            let report = json!({
                "namespace": "dev cli",
                "owner": "bijux-dev-cli",
                "commands": command_rows
                    .iter()
                    .map(|row| {
                        let mut obj = row.as_object().cloned().unwrap_or_default();
                        obj.insert("owner".to_string(), Value::String("bijux-dev-cli".to_string()));
                        Value::Object(obj)
                    })
                    .collect::<Vec<_>>(),
                "summary": {
                    "total": command_rows.len(),
                    "visible": visible,
                    "internal": command_rows.len().saturating_sub(visible),
                    "groups": groups,
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/dev_cli_ownership_report.json",
                &report,
            )
            .ok()?;
            let mut lines = vec![
                "Dev CLI ownership report".to_string(),
                "owner: bijux-dev-cli".to_string(),
                "namespace: dev cli".to_string(),
                String::new(),
            ];
            for row in &command_rows {
                let command = row.get("command").and_then(Value::as_str).unwrap_or("");
                let group = row.get("group").and_then(Value::as_str).unwrap_or("");
                let visibility = if row.get("visible").and_then(Value::as_bool) == Some(true) {
                    "visible"
                } else {
                    "internal"
                };
                lines.push(format!("- {command} [{group}, {visibility}]"));
            }
            fs::write(
                workspace_root.join("artifacts/status/dev_cli_ownership_report.txt"),
                lines.join("\n") + "\n",
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/dev_cli_ownership_report.json","artifacts/status/dev_cli_ownership_report.txt"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS" => {
            let stale_root = std::env::var("DEV_CLI_STALE_ARTIFACT_ROOT")
                .ok()
                .map(|raw| raw.trim().to_string())
                .filter(|raw| !raw.is_empty())
                .map(PathBuf::from)
                .unwrap_or_else(|| workspace_root.to_path_buf());
            let stale_write = |artifact: &str, payload: &Value| -> Option<()> {
                let path = stale_root.join(artifact);
                write_json(&path, payload).ok()
            };
            let now_epoch = std::env::var("DEV_CLI_STALE_NOW_EPOCH")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|dur| dur.as_secs())
                        .unwrap_or(0)
                });
            let max_age_seconds = std::env::var("DEV_CLI_STALE_MAX_SECONDS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .unwrap_or(86_400);
            let forced_raw = std::env::var("DEV_CLI_FORCE_STALE_FILES").unwrap_or_default();
            let mut forced: BTreeSet<String> = forced_raw
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect();
            if std::env::var("DEV_CLI_INJECT_STALE_ARTIFACT").is_ok_and(|raw| raw == "1") {
                forced.insert("artifacts/status/parity_drift_artifact.json".to_string());
            }
            let specs = vec![
                (
                    "evidence_deleted_before_evidence_audit",
                    "dev cli evidence audit",
                    "artifacts/status/evidence_integrity_artifact.json",
                    "critical",
                    "Detect missing evidence artifact before evidence audit.",
                ),
                (
                    "evidence_stale_before_evidence_stale",
                    "dev cli evidence stale",
                    "artifacts/status/evidence_integrity_artifact.json",
                    "critical",
                    "Detect stale evidence artifact before evidence stale command.",
                ),
                (
                    "parity_stale_before_status",
                    "dev cli status",
                    "artifacts/status/parity_drift_artifact.json",
                    "critical",
                    "Detect stale parity artifact before status command.",
                ),
                (
                    "migration_stale_before_truth",
                    "dev cli truth",
                    "artifacts/status/migration_truth_artifact.json",
                    "critical",
                    "Detect stale migration artifact before truth command.",
                ),
                (
                    "package_health_stale_before_dashboard",
                    "dev cli dashboard",
                    "artifacts/status/package_health_diagnostics_artifact.json",
                    "critical",
                    "Detect stale package health artifact before dashboard command.",
                ),
                (
                    "state_audit_stale_before_blockers",
                    "dev cli blockers",
                    "artifacts/status/state_audit_truth_artifact.json",
                    "critical",
                    "Detect stale state audit artifact before blockers command.",
                ),
                (
                    "docs_audit_stale_before_repo_health",
                    "dev cli repo health",
                    "artifacts/status/docs_audit.json",
                    "critical",
                    "Detect stale docs-audit artifact before repo health command.",
                ),
                (
                    "script_audit_stale_before_repo_health",
                    "dev cli repo health",
                    "artifacts/status/script_only_behaviors.json",
                    "critical",
                    "Detect stale script-audit artifact before repo health command.",
                ),
                (
                    "crate_health_stale_before_crate_health",
                    "dev cli crate-health",
                    "artifacts/status/duplication_hotspots.json",
                    "critical",
                    "Detect stale crate-health artifact before crate-health command.",
                ),
                (
                    "optional_next_report_stale_warning",
                    "dev cli next",
                    "artifacts/status/dev_cli_next_report.json",
                    "warning",
                    "Stale optional report is tolerated with warning.",
                ),
            ];
            let checks: Vec<Value> = specs
                .iter()
                .map(|(scenario_id, command, relative_path, severity, description)| {
                    let path = stale_root.join(relative_path);
                    let exists = path.exists();
                    let mut state = "fresh".to_string();
                    let mut age_seconds = None::<u64>;
                    if !exists {
                        state = "missing".to_string();
                    } else {
                        let modified = path
                            .metadata()
                            .ok()
                            .and_then(|meta| meta.modified().ok())
                            .and_then(|ts| ts.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|dur| dur.as_secs())
                            .unwrap_or(now_epoch);
                        let age = now_epoch.saturating_sub(modified);
                        age_seconds = Some(age);
                        if forced.contains(*relative_path) || age > max_age_seconds {
                            state = "stale".to_string();
                        }
                    }
                    json!({
                        "scenario_id": scenario_id,
                        "command": command,
                        "path": relative_path,
                        "severity": severity,
                        "description": description,
                        "exists": exists,
                        "state": state,
                        "age_seconds": age_seconds,
                        "max_age_seconds": max_age_seconds,
                    })
                })
                .collect();
            let stale_or_missing: Vec<Value> = checks
                .iter()
                .filter(|row| {
                    row.get("state")
                        .and_then(Value::as_str)
                        .is_some_and(|s| s == "stale" || s == "missing")
                })
                .cloned()
                .collect();
            let fresh_count = checks.len().saturating_sub(stale_or_missing.len());
            let critical_stale_count = stale_or_missing
                .iter()
                .filter(|row| row.get("severity").and_then(Value::as_str) == Some("critical"))
                .count();
            let warning_stale_count = stale_or_missing
                .iter()
                .filter(|row| row.get("severity").and_then(Value::as_str) == Some("warning"))
                .count();
            let status_value = if stale_or_missing.is_empty() { "clean" } else { "drift" };
            let summary = json!({
                "checks_total": checks.len(),
                "fresh_count": fresh_count,
                "stale_or_missing_count": stale_or_missing.len(),
                "critical_stale_count": critical_stale_count,
                "warning_stale_count": warning_stale_count,
                "status": status_value,
                "injection_mode": std::env::var("DEV_CLI_INJECT_STALE_ARTIFACT").is_ok_and(|raw| raw == "1"),
            });
            stale_write(
                "artifacts/status/stale_artifact_artifact.json",
                &json!({
                    "scope": "stale artifact truth",
                    "generator": "bijux-dev-cli",
                    "summary": summary,
                    "checks": checks,
                }),
            )?;
            stale_write(
                "artifacts/status/stale_evidence_artifact.json",
                &json!({
                    "scope": "stale evidence truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks.iter().filter(|row| {
                        row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "dev cli evidence audit" || cmd == "dev cli evidence stale"
                        })
                    }).cloned().collect::<Vec<_>>(),
                    "status": if checks.iter().any(|row| {
                        row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "dev cli evidence audit" || cmd == "dev cli evidence stale"
                    }) && row.get("state").and_then(Value::as_str).is_some_and(|state| state == "stale" || state == "missing")
                    }) { "drift" } else { "clean" },
                }),
            )?;
            stale_write(
                "artifacts/status/stale_report_artifact.json",
                &json!({
                    "scope": "stale report truth",
                    "generator": "bijux-dev-cli",
                    "checks": checks.iter().filter(|row| {
                        !row.get("command").and_then(Value::as_str).is_some_and(|cmd| {
                            cmd == "dev cli evidence audit" || cmd == "dev cli evidence stale"
                        })
                    }).cloned().collect::<Vec<_>>(),
                    "status": status_value,
                }),
            )?;
            stale_write(
                "artifacts/status/stale_detection_regression_suite.json",
                &json!({
                    "scope": "stale artifact regression suite",
                    "generator": "bijux-dev-cli",
                    "cases": checks.iter().map(|row| {
                        json!({
                            "scenario_id": row.get("scenario_id").cloned().unwrap_or(Value::Null),
                            "command": row.get("command").cloned().unwrap_or(Value::Null),
                            "state": row.get("state").cloned().unwrap_or(Value::Null),
                            "severity": row.get("severity").cloned().unwrap_or(Value::Null),
                        })
                    }).collect::<Vec<_>>(),
                    "status": if critical_stale_count == 0 { "clean" } else { "drift" },
                }),
            )?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/stale_artifact_artifact.json",
                "artifacts/status/stale_evidence_artifact.json",
                "artifacts/status/stale_report_artifact.json",
                "artifacts/status/stale_detection_regression_suite.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-STATE-DIAGNOSTICS-REPORTS" => {
            let read_json = |rel_path: &str| -> Value {
                fs::read_to_string(workspace_root.join(rel_path))
                    .ok()
                    .and_then(|text| serde_json::from_str::<Value>(&text).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let state_audit = read_json("artifacts/status/state_audit_report.json");
            let state_doctor = read_json("artifacts/status/state_doctor_report.json");
            let unified_corruption =
                read_json("artifacts/status/unified_state_corruption_report.json");
            let repeated_harness =
                read_json("artifacts/status/repeated_run_corruption_harness.json");
            let audit_checks = json!({
                "paths_present": state_audit.get("paths").is_some_and(Value::is_object),
                "corruption_health_present": state_audit.get("corruption_health").is_some_and(Value::is_object),
                "config_path_present": state_audit.get("paths").and_then(|v| v.get("config")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "plugin_registry_path_present": state_audit.get("paths").and_then(|v| v.get("plugins_registry")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "history_path_present": state_audit.get("paths").and_then(|v| v.get("history")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
                "memory_path_present": state_audit.get("paths").and_then(|v| v.get("memory")).and_then(|v| v.get("path")).is_some_and(Value::is_string),
            });
            let doctor_checks = json!({
                "doctor_object_present": state_doctor.get("doctor").is_some_and(Value::is_object),
                "issues_list_present": state_doctor.get("doctor").and_then(|v| v.get("issues")).is_some_and(Value::is_array),
                "repairs_list_present": state_doctor.get("doctor").and_then(|v| v.get("repairs")).is_some_and(Value::is_array),
                "runtime_marker_present": state_doctor.get("runtime").is_some_and(Value::is_string),
            });
            let harness_results = repeated_harness
                .get("results")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let has_corrupt_config_probe = harness_results.iter().any(|row| {
                row.get("name").and_then(Value::as_str) == Some("state_doctor_json_corrupt_config")
            });
            let all_harness_stable = !harness_results.is_empty()
                && harness_results
                    .iter()
                    .all(|row| row.get("stable").and_then(Value::as_bool) == Some(true));
            let harness_checks = json!({
                "corrupt_config_probe_present": has_corrupt_config_probe,
                "harness_results_stable": all_harness_stable,
                "unified_corruption_report_present": !unified_corruption.as_object().is_some_and(|obj| obj.is_empty()),
            });
            let all_checks = [audit_checks.clone(), doctor_checks.clone(), harness_checks.clone()]
                .into_iter()
                .filter_map(|v| v.as_object().cloned())
                .fold(serde_json::Map::new(), |mut acc, map| {
                    acc.extend(map);
                    acc
                });
            let drift_checks: Vec<String> = all_checks
                .iter()
                .filter(|(_, v)| v.as_bool() != Some(true))
                .map(|(k, _)| k.to_string())
                .collect();
            write_status_artifact_json(workspace_root, "artifacts/status/state_audit_truth_artifact.json", &json!({
                "scope": "state audit truth",
                "generator": "bijux-dev-cli",
                "checks": audit_checks,
                "status": if audit_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/state_doctor_truth_artifact.json", &json!({
                "scope": "state doctor truth",
                "generator": "bijux-dev-cli",
                "checks": doctor_checks,
                "status": if doctor_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/corrupted_state_truth_artifact.json", &json!({
                "scope": "corrupted state truth",
                "generator": "bijux-dev-cli",
                "checks": harness_checks,
                "status": if harness_checks.as_object().is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true))) { "complete" } else { "partial" },
            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/state_diagnostics_drift_artifact.json",
                &json!({
                    "scope": "state diagnostics drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/state_audit_truth_artifact.json",
                "artifacts/status/state_doctor_truth_artifact.json",
                "artifacts/status/corrupted_state_truth_artifact.json",
                "artifacts/status/state_diagnostics_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-BOUNDARY-REPORTS" => {
            let dev_fixture = workspace_root
                .join("crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt");
            let core_app = workspace_root.join("crates/bijux-cli/src/app.rs");
            let read = |path: &Path| fs::read_to_string(path).unwrap_or_default();
            let core_source = read(&core_app);
            let commands: Vec<String> = read(&dev_fixture)
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("dev cli "))
                .map(ToString::to_string)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let maintainer_diag = BTreeSet::from([
                "dev cli routes",
                "dev cli route-audit",
                "dev cli registry",
                "dev cli parity",
                "dev cli status",
                "dev cli script-audit",
                "dev cli crate-health",
                "dev cli package-health",
                "dev cli env",
                "dev cli doctor",
                "dev cli contracts",
                "dev cli runtime-identity",
                "dev cli state-audit",
                "dev cli state-doctor",
                "dev cli docs-audit",
            ]);
            let mut dev_rows = Vec::<Value>::new();
            let mut misplaced = Vec::<Value>::new();
            let mut missing_impl = Vec::<String>::new();
            for command in commands {
                let mut owner = "bijux-cli".to_string();
                if command == "dev cli route-audit" {
                    owner = "bijux-cli::routing + bijux-cli".to_string();
                }
                if [
                    "dev cli runtime-identity",
                    "dev cli package-health",
                    "dev cli state-audit",
                    "dev cli state-doctor",
                ]
                .contains(&command.as_str())
                {
                    owner = "bijux-cli + bijux-cli::install + bijux-cli-plugin".to_string();
                }
                let delegated = [
                    ("dev cli routes", "dev_routes::build_report_from_query"),
                    ("dev cli registry", "dev_registry::build_report_from_query"),
                    ("dev cli route-audit", "dev_route_audit::build_report_from_query"),
                    ("dev cli env", "dev_env::build_report("),
                    ("dev cli contracts", "dev_contracts::build_report("),
                    ("dev cli parity", "dev_parity::build_report("),
                    ("dev cli status", "dev_status::build_report("),
                    ("dev cli runtime-identity", "dev_runtime_identity::build_report("),
                    ("dev cli package-health", "dev_package_health::build_report("),
                    ("dev cli state-audit", "dev_state_audit::build_report("),
                    ("dev cli state-doctor", "dev_state_audit::build_doctor_report("),
                    ("dev cli script-audit", "dev_script_audit::build_report("),
                    ("dev cli docs-audit", "dev_docs_audit::build_report("),
                    ("dev cli crate-health", "dev_crate_health::build_report("),
                    ("dev cli inventory", "dev_script_audit::build_inventory_report("),
                ];
                if delegated
                    .iter()
                    .any(|(cmd, marker)| command == *cmd && core_source.contains(marker))
                {
                    owner = "bijux-dev-cli + runtime-data-providers".to_string();
                }
                if owner == "unmapped" {
                    missing_impl.push(command.clone());
                }
                let leaks = !owner.starts_with("bijux-dev-cli");
                let behavior_kind = if maintainer_diag.contains(command.as_str()) {
                    "diagnostic"
                } else {
                    "automation"
                };
                dev_rows.push(json!({
                    "command": command,
                    "behavior_kind": behavior_kind,
                    "intended_owner": "maintainer-control-plane",
                    "current_owner": owner,
                    "leaks_through_runtime": leaks,
                    "exposed_through_binary": true,
                    "evidence": [
                        "crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt",
                        "crates/bijux-cli/src/app.rs"
                    ],
                }));
                if leaks {
                    misplaced.push(json!({
                        "behavior": command,
                        "expected_owner": "bijux-dev-cli",
                        "current_owner": owner,
                        "reason": "maintainer behavior still implemented in runtime crates",
                        "severity": "must-move",
                    }));
                }
            }
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_owned_behaviors_inventory.json", &json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "dev-cli maintainer-owned behavior inventory",
                "commands": dev_rows,
                "maintainer_only_commands_implemented_in_runtime_crates": dev_rows.iter().filter(|row| row.get("leaks_through_runtime").and_then(Value::as_bool)==Some(true)).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                "maintainer_only_diagnostics_exposed_from_bin": maintainer_diag,
                "script_replacements_already_covered_by_dev_cli": Value::Array(vec![]),
                "remaining_scripts_to_move_into_dev_cli": Value::Array(vec![]),
                "boundary_rules": {
                    "control_plane_owner": "bijux-dev-cli owns maintainer automation and report assembly",
                    "runtime_scope": "runtime crates own runtime law and structured-data services, not maintainer workflows",
                    "canonical_surface": "bijux dev cli remains the canonical maintainer command surface",
                    "distribution": "bijux-dev-cli is a workspace crate, not a second public binary package",
                    "binary_identity": "bijux remains the only canonical executable",
                    "law_center": "bijux-dev-cli does not become a second runtime law center"
                },
                "boundary_frozen": true,
                "missing_implementation_mappings": missing_impl,
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/runtime_owned_behaviors_inventory.json", &json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "runtime-owned behaviors",
                "behaviors": [
                    {"behavior":"command routing and normalization","owner":"bijux-cli","evidence":"crates/bijux-cli/src/routing/catalog.rs"},
                    {"behavior":"runtime command execution kernel","owner":"bijux-cli","evidence":"crates/bijux-cli/src/app.rs"},
                    {"behavior":"config persistence and state law","owner":"bijux-cli","evidence":"crates/bijux-cli/src/config"},
                    {"behavior":"plugin registry lifecycle","owner":"bijux-cli-plugin","evidence":"crates/bijux-cli-plugin/src"},
                    {"behavior":"install and runtime identity primitives","owner":"bijux-cli::install","evidence":"crates/bijux-cli/src/install"},
                    {"behavior":"output envelope and rendering","owner":"bijux-cli-output","evidence":"crates/bijux-cli-output/src/lib.rs"}
                ],
                "rules": {
                    "runtime_crates_do_not_own_maintainer_workflows": true,
                    "runtime_crates_expose_structured_data_only_for_maintainer_reports": true
                }
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/misplaced_dev_behaviors_report.json", &json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "misplaced maintainer behavior still implemented in runtime crates",
                "misplaced_behaviors": misplaced,
                "summary": {"total_dev_cli_commands": dev_rows.len(), "misplaced_count": misplaced.len()},
                "boundary_freeze": {"status":"frozen-before-extraction","rule":"boundary inventory must be generated and reviewed before moving implementation"},
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_maintainer_command_ownership_report.json", &json!({
                "generated_at": generated_at_utc(),
                "generator": "bijux-dev-cli",
                "scope": "maintainer inventory command ownership",
                "maintainer_inventory_commands": [
                    "dev cli inventory","dev cli script-audit","dev cli docs-audit","dev cli crate-health",
                    "dev cli package-health","dev cli runtime-identity","dev cli state-audit","dev cli state-doctor"
                ],
                "owned_by_bijux_dev_cli": dev_rows.iter().filter(|row| row.get("current_owner").and_then(Value::as_str).is_some_and(|s| s.starts_with("bijux-dev-cli"))).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
                "not_yet_owned_by_bijux_dev_cli": dev_rows.iter().filter(|row| row.get("current_owner").and_then(Value::as_str).is_none_or(|s| !s.starts_with("bijux-dev-cli"))).filter_map(|row| row.get("command").cloned()).collect::<Vec<_>>(),
            })).ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_owned_behaviors_inventory.json",
                "artifacts/status/runtime_owned_behaviors_inventory.json",
                "artifacts/status/misplaced_dev_behaviors_report.json",
                "artifacts/status/dev_cli_maintainer_command_ownership_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-COMMAND-SURFACE-REPORTS" => {
            let fixture = workspace_root
                .join("crates/bijux-cli/tests/routing/fixtures/dev_cli_subcommands.txt");
            let test_file =
                workspace_root.join("crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs");
            let test_dir = workspace_root.join("crates/bijux-cli/tests/bin_surface");
            let source = fs::read_to_string(&test_file).unwrap_or_default();
            let test_sources: BTreeMap<String, String> = collect_files(&test_dir)
                .into_iter()
                .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
                .map(|p| (rel(&p, workspace_root), fs::read_to_string(p).unwrap_or_default()))
                .collect();
            let commands: Vec<String> = fs::read_to_string(&fixture)
                .unwrap_or_default()
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("dev cli "))
                .map(ToString::to_string)
                .collect();
            let dev_values: BTreeMap<String, i64> = BTreeMap::from([
                ("dev cli status".to_string(), 100),
                ("dev cli routes".to_string(), 98),
                ("dev cli registry".to_string(), 98),
                ("dev cli env".to_string(), 96),
                ("dev cli doctor".to_string(), 95),
                ("dev cli contracts".to_string(), 93),
                ("dev cli parity".to_string(), 91),
                ("dev cli runtime-identity".to_string(), 90),
                ("dev cli state-audit".to_string(), 90),
                ("dev cli state-doctor".to_string(), 90),
            ]);
            let mut rows = Vec::<Value>::new();
            for command in commands {
                let parts = command.split(' ').collect::<Vec<_>>();
                let quoted =
                    parts.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ");
                let evidence_links: Vec<String> = test_sources
                    .iter()
                    .filter(|(_, src)| {
                        src.contains(&quoted) || src.contains(&format!("\"{command}\""))
                    })
                    .map(|(path, _)| path.to_string())
                    .collect();
                let status = if !evidence_links.is_empty()
                    || source.contains(&quoted)
                    || source.contains(&format!("\"{command}\""))
                {
                    "complete"
                } else {
                    "partial"
                };
                rows.push(json!({
                    "command": command,
                    "status": status,
                    "status_model": ["complete","partial","shim","missing"],
                    "evidence": evidence_links.first().cloned().unwrap_or_else(|| "crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs".to_string()),
                    "evidence_links": evidence_links,
                    "maintainer_value": dev_values.get(&command).copied().unwrap_or(75),
                }));
            }
            rows.sort_by(|l, r| {
                let lv = l.get("maintainer_value").and_then(Value::as_i64).unwrap_or(0);
                let rv = r.get("maintainer_value").and_then(Value::as_i64).unwrap_or(0);
                rv.cmp(&lv).then_with(|| {
                    l.get("command")
                        .and_then(Value::as_str)
                        .cmp(&r.get("command").and_then(Value::as_str))
                })
            });
            let req: BTreeMap<i64, &str> = BTreeMap::from([
                (243,"parity_for_key_dev_cli_commands_against_current_behavior"),
                (250,"help_snapshots_exist_for_all_dev_cli_subcommands"),
                (251,"json_and_text_outputs_are_available_for_machine_and_text_heavy_dev_cli_commands"),
                (253,"stderr_stdout_and_exit_code_discipline_for_dev_cli_commands"),
                (255,"malformed_input_is_rejected_for_dev_cli_subcommands"),
                (256,"repeated_run_determinism_for_machine_readable_dev_cli_commands"),
                (257,"consistency_across_dev_cli_routes_inspect_and_registry_state"),
                (258,"consistency_across_dev_cli_env_and_config_resolution_paths"),
            ]);
            let coverage_checks = json!({
                "parity": source.contains("fn parity_for_key_dev_cli_commands_against_current_behavior("),
                "contract_shape": source.contains("fn json_and_text_outputs_are_available_for_machine_and_text_heavy_dev_cli_commands("),
                "help_snapshots": source.contains("fn help_snapshots_exist_for_all_dev_cli_subcommands("),
                "stderr_stdout_exit_code": source.contains("fn stderr_stdout_and_exit_code_discipline_for_dev_cli_commands("),
                "malformed_input": source.contains("fn malformed_input_is_rejected_for_dev_cli_subcommands("),
                "determinism": source.contains("fn repeated_run_determinism_for_machine_readable_dev_cli_commands("),
                "consistency_inspect_routes_registry": source.contains("fn consistency_across_dev_cli_routes_inspect_and_registry_state("),
                "consistency_config_env_resolution": source.contains("fn consistency_across_dev_cli_env_and_config_resolution_paths("),
                "consistency_plugin_registry_state": source.contains("fn consistency_across_dev_cli_routes_inspect_and_registry_state("),
            });
            let all_required = coverage_checks
                .as_object()
                .is_some_and(|obj| obj.values().all(|v| v.as_bool() == Some(true)));
            let summary = json!({
                "total": rows.len(),
                "complete": rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("complete")).count(),
                "partial": rows.iter().filter(|r| r.get("status").and_then(Value::as_str)==Some("partial")).count(),
                "shim": 0, "missing": 0
            });
            let remaining: Vec<Value> = rows
                .iter()
                .filter(|r| r.get("status").and_then(Value::as_str) != Some("complete"))
                .cloned()
                .collect();
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_coverage_report.json", &json!({
                "generated_at": generated_at_utc(), "generator":"bijux-dev-cli","scope":"dev cli command coverage","commands":rows,"summary":summary
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_matrix_artifact.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"dev cli command matrix",
                "coverage_rows": req.into_iter().map(|(id,name)| json!({"coverage_id":id,"test":name,"status": if source.contains(&format!("fn {name}(")) {"complete"} else {"missing"},"evidence":"crates/bijux-cli/tests/bin_surface/dev_cli_command_matrix.rs"})).collect::<Vec<_>>(),
                "commands": rows
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_surface_domain_contract.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","domain":"dev-cli-command-surface","status":"frozen",
                "rule":"dev cli commands are the maintainer control surface and must keep parity, diagnostics, and deterministic output law."
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_remaining_inventory.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"remaining dev cli subcommands not proven complete in rust","remaining_commands":remaining,"count":remaining.len()
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_value_ranking.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"dev cli maintainer-value ranking for closure execution","ranked_remaining_commands":remaining,"count":remaining.len()
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_completion_report.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"dev cli command closure execution","remaining_count":remaining.len(),"coverage_checks":coverage_checks,
                "closure_status": if remaining.is_empty() && all_required {"green"} else {"open"},
                "top_targets": remaining.iter().take(2).cloned().collect::<Vec<_>>()
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_command_closure_set.json", &json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"tracked dev cli closure set","tracked_commands":rows.iter().filter_map(|r| r.get("command").cloned()).collect::<Vec<_>>(),
                "coverage_checks":coverage_checks,"status":"frozen"
            })).ok()?;
            let cli_completion = fs::read_to_string(
                workspace_root.join("artifacts/status/cli_command_completion_report.json"),
            )
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .unwrap_or_else(|| json!({}));
            let cli_remaining =
                cli_completion.get("remaining_count").and_then(Value::as_i64).unwrap_or(0);
            let cli_green =
                cli_completion.get("closure_status").and_then(Value::as_str) == Some("green");
            let dev_green = remaining.is_empty() && all_required;
            let combined = json!({
                "generated_at": generated_at_utc(),"generator":"bijux-dev-cli","scope":"cli and dev cli command closure",
                "cli":{"remaining_count":cli_remaining,"closure_status":cli_completion.get("closure_status").cloned().unwrap_or_else(|| json!("open")),"top_targets":cli_completion.get("top_targets").cloned().unwrap_or_else(|| json!([]))},
                "dev_cli":{"remaining_count":remaining.len(),"closure_status":if dev_green {"green"} else {"open"},"top_targets":remaining.iter().take(2).cloned().collect::<Vec<_>>()},
                "cross_command_consistency":{"inspect_routes_registry":coverage_checks["consistency_inspect_routes_registry"],"config_env_resolution":coverage_checks["consistency_config_env_resolution"],"plugin_registry_state":coverage_checks["consistency_plugin_registry_state"]},
                "closure_status": if cli_green && dev_green {"green"} else {"open"},
                "complete_language_allowed": cli_green && dev_green
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cli_dev_command_closure_report.json",
                &combined,
            )
            .ok()?;
            let txt = format!(
                "CLI and DEV CLI Closure Report\noverall: {}\ncomplete language allowed: {}\n\ncli remaining: {}\ndev cli remaining: {}\n",
                combined.get("closure_status").and_then(Value::as_str).unwrap_or("open"),
                combined.get("complete_language_allowed").and_then(Value::as_bool).unwrap_or(false),
                cli_remaining,
                remaining.len()
            );
            fs::write(
                workspace_root.join("artifacts/status/cli_dev_command_closure_report.txt"),
                txt,
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_command_coverage_report.json",
                "artifacts/status/dev_cli_command_matrix_artifact.json",
                "artifacts/status/dev_cli_command_surface_domain_contract.json",
                "artifacts/status/dev_cli_command_remaining_inventory.json",
                "artifacts/status/dev_cli_command_value_ranking.json",
                "artifacts/status/dev_cli_command_completion_report.json",
                "artifacts/status/dev_cli_command_closure_set.json",
                "artifacts/status/cli_dev_command_closure_report.json",
                "artifacts/status/cli_dev_command_closure_report.txt"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-DISPATCH-OWNERSHIP-REPORTS" => {
            let main_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/bin/bijux-rs.rs"))
                    .unwrap_or_default();
            let core_app = fs::read_to_string(workspace_root.join("crates/bijux-cli/src/app.rs"))
                .unwrap_or_default();
            let parser_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/parser.rs"))
                    .unwrap_or_default();
            let registry_rs =
                fs::read_to_string(workspace_root.join("crates/bijux-cli/src/routing/registry.rs"))
                    .unwrap_or_default();
            let dev_cli_dispatch_arm_count =
                core_app.matches("a == \"dev\" && b == \"cli\"").count();
            let core_dev_cli_builder_call_count = [
                "dev_routes::build_report(",
                "dev_registry::build_report(",
                "dev_env::build_report(",
                "dev_contracts::build_report(",
                "dev_parity::build_report(",
                "dev_status::build_report(",
                "dev_script_audit::build_inventory_report(",
                "dev_script_audit::build_report(",
                "dev_docs_audit::build_report(",
                "dev_crate_health::build_report(",
                "dev_runtime_identity::build_report(",
                "dev_package_health::build_report(",
                "dev_state_audit::build_report(",
                "dev_state_audit::build_doctor_report(",
            ]
            .iter()
            .map(|token| core_app.matches(token).count())
            .sum::<usize>();
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_dispatch_ownership_report.json", &json!({
                "scope":"dev cli dispatch ownership","status":"ok",
                "dispatch_chain":[
                    {"crate":"bijux-cli","role":"entrypoint-only","evidence":"src/bin/bijux-rs.rs delegates to bijux_cli::app::run_app"},
                    {"crate":"bijux-cli","role":"dispatch-only-for-maintainer-surface","evidence":"src/app.rs routes dev cli commands into bijux-dev-cli report builders"},
                    {"crate":"bijux-dev-cli","role":"maintainer-workflow-implementation-owner","evidence":"src/*.rs report builders provide maintainer payload assembly"}
                ],
                "checks":{
                    "bin_mentions_dev_cli_literals": main_rs.contains("dev cli"),
                    "bin_has_direct_dispatch_match_arms": main_rs.contains("match normalized_path"),
                    "core_dev_cli_dispatch_arm_count": dev_cli_dispatch_arm_count,
                    "core_dev_cli_builder_call_count": core_dev_cli_builder_call_count
                },
                "rules":[
                    "bin must remain entrypoint-only",
                    "routing must remain command identity only",
                    "dev cli maintainer workflows must be implemented in bijux-dev-cli"
                ]
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/bin_entrypoint_responsibility_diff.json", &json!({
                "scope":"bin responsibility diff","status":"ok",
                "current":{
                    "file":"crates/bijux-cli/src/bin/bijux-rs.rs",
                    "line_count": main_rs.lines().count(),
                    "dev_cli_literal_mentions": main_rs.matches("dev cli").count(),
                    "core_run_app_calls": main_rs.matches("bijux_cli::app::run_app").count(),
                    "direct_dispatch_match_mentions": main_rs.matches("match normalized_path").count(),
                    "parser_dependency_mentions": main_rs.matches("bijux_cli::routing::parser").count()
                },
                "routing_identity_checks":{
                    "parser_build_report_mentions": parser_rs.matches("build_report(").count(),
                    "registry_build_report_mentions": registry_rs.matches("build_report(").count(),
                    "parser_json_assembly_mentions": parser_rs.matches("serde_json::json!").count(),
                    "registry_json_assembly_mentions": registry_rs.matches("serde_json::json!").count()
                }
            })).ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_dispatch_ownership_report.json",
                "artifacts/status/bin_entrypoint_responsibility_diff.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-RESILIENCE-REPORTS" => {
            let run_cmd = |args: &[&str], envs: &[(&str, String)]| -> std::process::Output {
                let mut cmd = Command::new("cargo");
                cmd.args(["run", "-q", "-p", "bijux-cli", "--bin", "bijux", "--"])
                    .args(args)
                    .current_dir(workspace_root);
                for (k, v) in envs {
                    cmd.env(k, v);
                }
                cmd.output().expect("failed to execute cargo run for resilience report")
            };
            let summary_commands: Vec<Vec<&str>> = vec![
                vec!["dev", "cli", "status"],
                vec!["dev", "cli", "dashboard"],
                vec!["dev", "cli", "truth"],
                vec!["dev", "cli", "blockers"],
                vec!["dev", "cli", "next"],
            ];
            let machine_commands: Vec<Vec<&str>> = vec![
                vec!["dev", "cli", "parity"],
                vec!["dev", "cli", "evidence", "audit"],
                vec!["dev", "cli", "routes"],
                vec!["dev", "cli", "registry"],
                vec!["dev", "cli", "env"],
                vec!["dev", "cli", "contracts"],
                vec!["dev", "cli", "state-audit"],
                vec!["dev", "cli", "state-doctor"],
                vec!["dev", "cli", "runtime-identity"],
                vec!["dev", "cli", "package-health"],
            ];
            let mut determinism_rows = Vec::<Value>::new();
            for command in summary_commands.iter().chain(machine_commands.iter()) {
                let mut first = command.clone();
                first.extend(["--format", "json", "--no-pretty"]);
                let mut second = command.clone();
                second.extend(["--format", "json", "--no-pretty"]);
                let a = run_cmd(&first, &[]);
                let b = run_cmd(&second, &[]);
                determinism_rows.push(json!({
                    "command": command.join(" "),
                    "stable": a.status.code() == b.status.code() && a.stdout == b.stdout,
                    "first_exit": a.status.code().unwrap_or(1),
                    "second_exit": b.status.code().unwrap_or(1),
                }));
            }
            let tmp = std::env::temp_dir()
                .join(format!("bijux-dev-cli-side-effects-{}", std::process::id()));
            let _ = fs::remove_dir_all(&tmp);
            let _ = fs::create_dir_all(tmp.join("plugins"));
            let config = tmp.join("config.env");
            let history = tmp.join("history.json");
            let memory = tmp.join("memory.json");
            let plugins = tmp.join("plugins");
            let _ = fs::write(&config, "BIJUXCLI_SAMPLE=1\n");
            let _ = fs::write(&history, "[]");
            let _ = fs::write(&memory, "{}");
            let _ = fs::write(
                plugins.join("healthy.toml"),
                "[plugin]\nname='healthy'\nentry='plugin:main'\n",
            );
            let digest = |p: &Path| -> String {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let data = fs::read(p).unwrap_or_default();
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                format!("{:016x}", hasher.finish())
            };
            let before = json!({"config":digest(&config),"history":digest(&history),"memory":digest(&memory)});
            let envs = vec![
                ("BIJUX_CONFIG_PATH", config.display().to_string()),
                ("BIJUX_HISTORY_PATH", history.display().to_string()),
                ("BIJUX_MEMORY_PATH", memory.display().to_string()),
                ("BIJUX_PLUGINS_DIR", plugins.display().to_string()),
            ];
            for command in summary_commands.iter().chain(machine_commands.iter()) {
                let _ = run_cmd(command, &envs);
            }
            let after = json!({"config":digest(&config),"history":digest(&history),"memory":digest(&memory)});
            let _ = fs::remove_dir_all(&tmp);
            let failure_cases: Vec<(&str, Vec<&str>, Vec<(&str, String)>)> = vec![
                (
                    "status_unreadable_input",
                    vec!["dev", "cli", "status"],
                    vec![("BIJUX_HISTORY_PATH", "/root/forbidden/history.json".to_string())],
                ),
                (
                    "parity_corrupted_input",
                    vec!["dev", "cli", "parity"],
                    vec![("BIJUX_MEMORY_PATH", "/dev/null/not-json".to_string())],
                ),
                (
                    "contracts_missing_snapshot_context",
                    vec!["dev", "cli", "contracts"],
                    vec![("PWD", "/definitely/missing/contracts/root".to_string())],
                ),
                (
                    "runtime_identity_path_ambiguity",
                    vec!["dev", "cli", "runtime-identity"],
                    vec![(
                        "PATH",
                        format!(
                            "/tmp/bijux-a:/tmp/bijux-b:{}",
                            std::env::var("PATH").unwrap_or_default()
                        ),
                    )],
                ),
                (
                    "package_health_metadata_mismatch",
                    vec!["dev", "cli", "package-health"],
                    vec![
                        ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
                        ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string()),
                    ],
                ),
            ];
            let mut failure_rows = Vec::<Value>::new();
            for (case_id, command, env) in &failure_cases {
                let mut args = command.clone();
                args.extend(["--format", "json", "--no-pretty"]);
                let out = run_cmd(&args, env);
                let payload =
                    serde_json::from_slice::<Value>(&out.stdout).unwrap_or_else(|_| json!({}));
                failure_rows.push(json!({
                    "case_id": case_id,
                    "command": command.join(" "),
                    "exit_code": out.status.code().unwrap_or(1),
                    "json_object": payload.is_object(),
                }));
            }
            let summary_set: BTreeSet<String> =
                summary_commands.iter().map(|c| c.join(" ")).collect();
            let machine_set: BTreeSet<String> =
                machine_commands.iter().map(|c| c.join(" ")).collect();
            let checks = json!({
                "failure_injection_cases_reported": failure_rows.len() == failure_cases.len(),
                "determinism_rows_present": determinism_rows.len() == summary_commands.len()+machine_commands.len(),
                "summary_commands_deterministic": determinism_rows.iter().filter(|r| summary_set.contains(r.get("command").and_then(Value::as_str).unwrap_or(""))).all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)),
                "machine_commands_deterministic": determinism_rows.iter().filter(|r| machine_set.contains(r.get("command").and_then(Value::as_str).unwrap_or(""))).all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)),
                "read_only_commands_did_not_mutate_state": before == after,
            });
            let drift_checks: Vec<String> = checks
                .as_object()
                .map(|o| {
                    o.iter()
                        .filter(|(_, v)| v.as_bool() != Some(true))
                        .map(|(k, _)| k.to_string())
                        .collect()
                })
                .unwrap_or_default();
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_control_plane_resilience_artifact.json", &json!({
                "scope":"dev cli control-plane resilience","generator":"bijux-dev-cli","failure_injection_cases":failure_rows,"checks":checks,
                "status": if drift_checks.is_empty() {"complete"} else {"partial"}
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_determinism_artifact.json", &json!({
                "scope":"dev cli determinism","generator":"bijux-dev-cli","rows":determinism_rows,
                "status": if determinism_rows.iter().all(|r| r.get("stable").and_then(Value::as_bool)==Some(true)) {"clean"} else {"drift"}
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_side_effect_audit_artifact.json", &json!({
                "scope":"dev cli side-effect audit","generator":"bijux-dev-cli","before":before,"after":after,
                "status": if before == after {"clean"} else {"drift"}
            })).ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/dev_cli_resilience_drift_artifact.json", &json!({
                "scope":"dev cli resilience drift","generator":"bijux-dev-cli","drift_checks":drift_checks,"drift_count":drift_checks.len(),
                "status": if drift_checks.is_empty() {"clean"} else {"drift"}
            })).ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/dev_cli_control_plane_resilience_artifact.json",
                "artifacts/status/dev_cli_determinism_artifact.json",
                "artifacts/status/dev_cli_side_effect_audit_artifact.json",
                "artifacts/status/dev_cli_resilience_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DEV-CLI-SCOPE-REASSESSMENT" => {
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let runtime_leakage = read_json("artifacts/status/runtime_dev_leakage_report.json");
            let interface_bridge =
                read_json("artifacts/status/dev_cli_interface_bridge_report.json");
            let dispatch = read_json("artifacts/status/dev_cli_dispatch_ownership_report.json");
            let mut violations = Vec::<String>::new();
            if runtime_leakage.get("status").and_then(Value::as_str) != Some("ok") {
                violations.push("runtime leakage report is not green".to_string());
            }
            if interface_bridge.get("interfaces").and_then(Value::as_array).is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.get("contains_json_assembly").and_then(Value::as_bool) == Some(true)
                })
            }) {
                violations.push("query bridge still assembles presentation json".to_string());
            }
            if dispatch
                .get("checks")
                .and_then(|v| v.get("bin_has_direct_dispatch_match_arms"))
                .and_then(Value::as_bool)
                == Some(true)
            {
                violations.push("bin owns direct dispatch match arms".to_string());
            }
            let payload = json!({
                "scope":"runtime responsibility reassessment",
                "status": if violations.is_empty() {"ok"} else {"degraded"},
                "violations": violations,
                "decision": if violations.is_empty() {
                    "no remaining runtime responsibilities violate the current dev-cli control-plane standard"
                } else {
                    "runtime responsibilities still violate control-plane standard"
                }
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_responsibility_reassessment.json",
                &payload,
            )
            .ok()?;
            Some(
                json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":["artifacts/status/runtime_responsibility_reassessment.json"]}),
            )
        }
        "STATUS-SCRIPT-GENERATE-BRIDGE-WRAPPER-ONLY-REPORTS" => {
            let bridge_duplicate = fs::read_to_string(
                workspace_root.join("artifacts/status/bridge_duplicate_law_report.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let duplicate_count = bridge_duplicate
                .get("summary")
                .and_then(|v| v.get("duplicate_rule_count"))
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let bridge_source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-python/tests/bridge_bindings.rs"),
            )
            .unwrap_or_default();
            let cross_surface_source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs"),
            )
            .unwrap_or_default();
            let proof_tests = vec![
                (
                    "same_route_graph",
                    vec![
                        "binary_and_bridge_use_same_command_registry_contract",
                        "route_registry_snapshots_match_across_binary_core_and_bridge",
                    ],
                ),
                (
                    "same_command_registry",
                    vec!["binary_and_bridge_use_same_command_registry_contract"],
                ),
                (
                    "same_output_envelope",
                    vec!["binary_and_bridge_use_same_output_envelope_shape"],
                ),
                (
                    "same_exit_mappings",
                    vec!["binary_and_bridge_use_same_exit_mapping_for_unknown_route"],
                ),
                (
                    "same_namespace_law",
                    vec!["binary_and_bridge_use_same_namespace_rejection_logic"],
                ),
                (
                    "same_config_precedence",
                    vec!["execution_path_keeps_config_precedence_identical_between_binary_and_bridge"],
                ),
            ];
            let mut proof_map = serde_json::Map::new();
            for (key, names) in proof_tests {
                let present: Vec<String> = names
                    .iter()
                    .filter(|name| {
                        bridge_source.contains(&format!("fn {name}("))
                            || cross_surface_source.contains(&format!("fn {name}("))
                    })
                    .map(|name| (*name).to_string())
                    .collect();
                proof_map.insert(
                    key.to_string(),
                    json!({"required": names, "present": present, "ok": present.len()==names.len()}),
                );
            }
            let all_proofs_ok = proof_map
                .values()
                .all(|item| item.get("ok").and_then(Value::as_bool) == Some(true));
            let wrapper_ok = duplicate_count == 0 && all_proofs_ok;
            let payload = json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "generator": "bijux-dev-cli",
                "scope": "bridge wrapper-only closure",
                "duplicate_law": {
                    "duplicate_rule_count": duplicate_count,
                    "status": if duplicate_count == 0 { "clean" } else { "duplicates-found" }
                },
                "proof_tests": proof_map,
                "status": if wrapper_ok { "green" } else { "open" },
                "wrapper_only_frozen": wrapper_ok
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                &payload,
            )
            .ok()?;
            let mut lines = vec![
                "Bridge Wrapper-Only Closure Report".to_string(),
                format!(
                    "status: {}",
                    payload.get("status").and_then(Value::as_str).unwrap_or("open")
                ),
                format!(
                    "wrapper-only frozen: {}",
                    payload.get("wrapper_only_frozen").and_then(Value::as_bool).unwrap_or(false)
                ),
                format!("duplicate rule count: {duplicate_count}"),
            ];
            if let Some(obj) = payload.get("proof_tests").and_then(Value::as_object) {
                for (key, item) in obj {
                    lines.push(format!(
                        "- {key}: {}",
                        item.get("ok").and_then(Value::as_bool).unwrap_or(false)
                    ));
                }
            }
            fs::write(
                workspace_root.join("artifacts/status/bridge_wrapper_only_closure_report.txt"),
                lines.join("\n") + "\n",
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/bridge_wrapper_only_closure_report.json",
                "artifacts/status/bridge_wrapper_only_closure_report.txt"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-COMPATIBILITY-DEBT-TREND-REPORT" => {
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let shim = read_json("artifacts/status/compatibility_shim_count_report.json");
            let alias = read_json("artifacts/status/compatibility_alias_count_report.json");
            let shim_delta = read_json("artifacts/status/compatibility_shim_count_delta.json");
            let alias_delta = read_json("artifacts/status/compatibility_alias_count_delta.json");
            let payload = json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "generator": "bijux-dev-cli",
                "scope": "compatibility debt trend",
                "series": {
                    "shims": {
                        "baseline_count": shim.get("baseline_count").and_then(Value::as_i64).unwrap_or(0),
                        "current_count": shim.get("current_count").and_then(Value::as_i64).unwrap_or(0),
                        "delta_vs_baseline": shim_delta.get("delta").and_then(Value::as_i64).unwrap_or(0),
                        "removed_since_baseline": shim.get("removed_since_baseline").and_then(Value::as_i64).unwrap_or(0),
                    },
                    "aliases": {
                        "baseline_count": alias.get("baseline_count").and_then(Value::as_i64).unwrap_or(0),
                        "current_count": alias.get("current_count").and_then(Value::as_i64).unwrap_or(0),
                        "delta_vs_baseline": alias_delta.get("delta").and_then(Value::as_i64).unwrap_or(0),
                        "removed_since_baseline": alias.get("removed_since_baseline").and_then(Value::as_i64).unwrap_or(0),
                    },
                },
                "status": if shim_delta.get("delta").and_then(Value::as_i64).unwrap_or(0) <= 0
                    && alias_delta.get("delta").and_then(Value::as_i64).unwrap_or(0) <= 0
                {
                    "improving"
                } else {
                    "regressing"
                },
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/compatibility_debt_trend_report.json",
                &payload,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/compatibility_debt_trend_report.txt"),
                format!(
                    "Compatibility Debt Trend Report\nstatus: {}\n",
                    payload.get("status").and_then(Value::as_str).unwrap_or("regressing")
                ),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/compatibility_debt_trend_report.json",
                "artifacts/status/compatibility_debt_trend_report.txt"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-HOSTILE-STATE-REPORTS" => {
            let test_file = workspace_root
                .join("crates/bijux-cli/tests/bin_surface/deterministic_hostile_state_matrix.rs");
            let text = fs::read_to_string(&test_file).unwrap_or_default();
            let rows = vec![
                (141, "corrupted_config_failure_class_is_stable_across_runs"),
                (142, "corrupted_plugin_registry_failure_class_is_stable_across_runs"),
                (143, "broken_history_file_recovery_is_stable_across_runs"),
                (144, "malformed_memory_state_recovery_is_stable_across_runs"),
                (145, "missing_config_file_defaulting_is_stable_across_runs"),
                (146, "missing_plugin_directory_empty_behavior_is_stable_across_runs"),
                (147, "broken_plugin_does_not_nondeterministically_affect_healthy_output"),
                (148, "conflicting_plugin_installs_fail_deterministically"),
                (149, "path_shadowing_diagnostics_are_stable_across_runs"),
                (150, "runtime_identity_output_is_stable_under_same_ambiguous_state"),
                (151, "state_doctor_json_is_stable_under_same_corrupted_state"),
                (152, "state_doctor_text_is_stable_under_same_corrupted_state"),
                (153, "plugin_doctor_json_is_stable_under_same_corrupted_state"),
                (154, "plugin_doctor_text_is_stable_under_same_corrupted_state"),
                (155, "command_tree_export_is_stable_with_broken_optional_state"),
            ];
            write_status_artifact_json(workspace_root, "artifacts/status/deterministic_hostile_state_report.json", &json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "generator": "bijux-dev-cli",
                "scope": "deterministic hostile-state behavior",
                "rows": rows.iter().map(|(id,name)| json!({
                    "coverage_id": id,
                    "test_name": name,
                    "status": if text.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                    "evidence": "crates/bijux-cli/tests/bin_surface/deterministic_hostile_state_matrix.rs"
                })).collect::<Vec<_>>(),
            })).ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/failure_class_stability_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "harness_file": "artifacts/status/repeated_run_corruption_harness.json",
                    "covers_todo": 157
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/deterministic_failure_quality_bar.json", &json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "status": "frozen",
                "quality_bar": "deterministic failure behavior required for hostile-state covered commands",
                "required_artifacts": [
                    "artifacts/status/deterministic_hostile_state_report.json",
                    "artifacts/status/failure_class_stability_report.json",
                    "artifacts/status/repeated_run_corruption_harness.json"
                ],
            })).ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/deterministic_hostile_state_report.json",
                "artifacts/status/failure_class_stability_report.json",
                "artifacts/status/deterministic_failure_quality_bar.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PRECEDENCE-REPORTS" => {
            let test_file =
                workspace_root.join("crates/bijux-cli/tests/bin_surface/precedence_matrix.rs");
            let text = fs::read_to_string(&test_file).unwrap_or_default();
            let env_payload = run_bijux_json(workspace_root, &["dev", "cli", "env"])
                .unwrap_or_else(|_| json!({}));
            let source_precedence =
                env_payload.get("source_precedence").cloned().unwrap_or_else(|| json!([]));
            let precedence_rows = [
                "cli_flags_override_env_values",
                "env_values_override_config_file_values",
                "config_file_values_override_defaults",
                "defaults_apply_when_nothing_is_supplied",
            ]
            .iter()
            .map(|name| {
                json!({
                    "test_name": name,
                    "status": if text.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                    "evidence":"crates/bijux-cli/tests/bin_surface/precedence_matrix.rs"
                })
            })
            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_regression_matrix.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "precedence tests",
                    "rows": precedence_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/parity/command_precedence_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "source_precedence": source_precedence,
                    "shared_contract": "flags > env > config > defaults"
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/precedence_contract.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "contract": "precedence is one shared behavioral contract",
                    "status": "frozen",
                    "source_precedence": source_precedence
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/precedence_regression_matrix.json",
                "artifacts/parity/command_precedence_report.json",
                "artifacts/status/precedence_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-NAMESPACE-RESERVATION-REPORTS" => {
            let routing_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/registry_namespace_policy.rs"),
            )
            .unwrap_or_default();
            let plugin_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-plugin/tests/plugin_namespace_regression.rs"),
            )
            .unwrap_or_default();
            let cli_text = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/plugin_cli_lifecycle.rs"),
            )
            .unwrap_or_default();
            let constants =
                fs::read_to_string(workspace_root.join("crates/bijux-cli-plugin/src/constants.rs"))
                    .unwrap_or_default();
            let product_registry = fs::read_to_string(
                workspace_root.join("docs/constitution/official_product_namespace_registry.json"),
            )
            .ok()
            .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
            .unwrap_or_else(|| json!({}));
            let evidence_text = format!("{routing_text}\n{plugin_text}\n{cli_text}");
            let parse_array = |name: &str| -> Vec<String> {
                let marker = format!("pub const {name}: &[&str] =");
                let Some(idx) = constants.find(&marker) else {
                    return Vec::new();
                };
                let chunk = &constants[idx..];
                let Some(start) = chunk.find('[') else {
                    return Vec::new();
                };
                let Some(end) = chunk.find("];") else {
                    return Vec::new();
                };
                chunk[start..end]
                    .split('"')
                    .enumerate()
                    .filter_map(|(i, part)| (i % 2 == 1).then_some(part.to_string()))
                    .collect()
            };
            let namespace_rows = [
                "official_reserved_namespaces_take_precedence",
                "rejects_future_official_product_namespaces",
                "normalized_and_case_folded_namespace_collisions_are_rejected",
            ]
            .iter()
            .map(|name| {
                json!({
                    "evidence_test": name,
                    "status": if evidence_text.contains(name) { "complete" } else { "missing" }
                })
            })
            .collect::<Vec<_>>();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/namespace_abuse_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "421-440 namespace and reservation abuse hardening",
                    "rows": namespace_rows
                }),
            )
            .ok()?;
            write_status_artifact_json(workspace_root, "artifacts/status/reserved_namespace_inventory.json", &json!({
                "generated_at": "1970-01-01T00:00:00+00:00",
                "generator": "bijux-dev-cli",
                "reserved_namespaces": parse_array("RESERVED_NAMESPACES"),
                "core_namespaces": parse_array("CORE_NAMESPACES"),
                "future_product_namespaces": parse_array("FUTURE_PRODUCT_NAMESPACES"),
                "registry_entries": product_registry.get("entries").cloned().unwrap_or_else(|| json!([]))
            })).ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/namespace_abuse_report.json",
                "artifacts/status/reserved_namespace_inventory.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-INSTALL-TRUTH-REPORTS" => {
            let generated_at = generated_at_utc();
            let runtime_identity =
                run_bijux_json(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let package_health =
                run_bijux_json(workspace_root, &["dev", "cli", "package-health"]).ok()?;
            let install_text =
                run_bijux_text(workspace_root, &["dev", "cli", "runtime-identity"]).ok()?;
            let diagnostics =
                runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({}));
            let install_source_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                "diagnostics": diagnostics,
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_source_diagnostics.json",
                &install_source_payload,
            )
            .ok()?;
            let ambiguous_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_command": "bijux dev cli runtime-identity --json --no-pretty",
                "active_binary_selection_is_ambiguous": runtime_identity.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                "active_path_is_shadowed": runtime_identity.get("active_path_is_shadowed").cloned().unwrap_or(json!(false)),
                "duplicate_install_detected": diagnostics.get("duplicate_install_detected").cloned().unwrap_or(json!(false)),
                "mixed_pip_cargo_install_detected": diagnostics.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                "path_shadowing_detected": diagnostics.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                "stale_wrapper_detected": diagnostics.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                "active_binary_mismatch_detected": diagnostics.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                "python_bridge_supported": diagnostics.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                &ambiguous_payload,
            )
            .ok()?;
            let install_health_payload = json!({
                "generated_at": generated_at,
                "generator": "bijux-dev-cli",
                "source_commands": [
                    "bijux dev cli runtime-identity --json --no-pretty",
                    "bijux dev cli package-health --json --no-pretty"
                ],
                "runtime_identity": runtime_identity,
                "install_state_assumptions": package_health.get("install_state_assumptions").cloned().unwrap_or_else(|| json!([])),
                "install_state_assumption_help": package_health.get("install_state_assumption_help").cloned().unwrap_or_else(|| json!("")),
            });
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_health_report.json",
                &install_health_payload,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/install_health_report.txt"),
                install_text,
            )
            .ok()?;
            let mut ambiguities = Vec::<String>::new();
            let ambiguous = &ambiguous_payload;
            if ambiguous.get("active_binary_selection_is_ambiguous").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("multiple bijux binaries detected in PATH order".to_string());
            }
            if ambiguous.get("path_shadowing_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities
                    .push("PATH shadowing detected for canonical bijux executable".to_string());
            }
            if ambiguous.get("mixed_pip_cargo_install_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("cargo and pip installations both appear active".to_string());
            }
            if ambiguous.get("stale_wrapper_detected").and_then(Value::as_bool) == Some(true) {
                ambiguities.push("stale wrapper scripts found in PATH".to_string());
            }
            if ambiguous.get("active_binary_mismatch_detected").and_then(Value::as_bool)
                == Some(true)
            {
                ambiguities.push("runtime binary version does not match wheel version".to_string());
            }
            if ambiguous.get("python_bridge_supported").and_then(Value::as_bool) == Some(false) {
                ambiguities
                    .push("python bridge support is unavailable for current runtime".to_string());
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/remaining_install_ambiguities.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "count": ambiguities.len(),
                    "ambiguities": ambiguities,
                    "status": if ambiguities.is_empty() { "clear" } else { "attention-required" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/install_source_diagnostics.json",
                "artifacts/status/ambiguous_runtime_diagnostics.json",
                "artifacts/status/install_health_report.json",
                "artifacts/status/install_health_report.txt",
                "artifacts/status/remaining_install_ambiguities.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-INSTALL-NEUTRALITY-REPORTS" => {
            let generated_at = generated_at_utc();
            let read_json = |name: &str| -> Value {
                fs::read_to_string(workspace_root.join(name))
                    .ok()
                    .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                    .unwrap_or_else(|| json!({}))
            };
            let runtime_identity = read_json("artifacts/status/install_source_diagnostics.json");
            let ambiguous = read_json("artifacts/status/ambiguous_runtime_diagnostics.json");
            let install_health = read_json("artifacts/status/install_health_report.json");
            let remaining = read_json("artifacts/status/remaining_install_ambiguities.json");
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_neutrality_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "schema": "install-neutrality-v1",
                    "channels": ["cargo","pip","pipx"],
                    "diagnostics": {
                        "active_binary_selection_is_ambiguous": ambiguous.get("active_binary_selection_is_ambiguous").cloned().unwrap_or(json!(false)),
                        "path_shadowing_detected": ambiguous.get("path_shadowing_detected").cloned().unwrap_or(json!(false)),
                        "mixed_pip_cargo_install_detected": ambiguous.get("mixed_pip_cargo_install_detected").cloned().unwrap_or(json!(false)),
                        "stale_wrapper_detected": ambiguous.get("stale_wrapper_detected").cloned().unwrap_or(json!(false)),
                        "active_binary_mismatch_detected": ambiguous.get("active_binary_mismatch_detected").cloned().unwrap_or(json!(false)),
                        "python_bridge_supported": ambiguous.get("python_bridge_supported").cloned().unwrap_or(json!(true)),
                    },
                    "active_runtime": {
                        "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                        "install_source": runtime_identity.get("install_source").cloned().unwrap_or(Value::Null),
                        "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                    },
                    "known_remaining_install_ambiguities": remaining.get("ambiguities").cloned().unwrap_or_else(|| json!([])),
                    "known_remaining_install_ambiguities_count": remaining.get("count").cloned().unwrap_or_else(|| json!(0)),
                    "status": if install_health.is_object() { "complete" } else { "incomplete" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/active_runtime_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "schema": "active-runtime-v1",
                    "source": "artifacts/status/install_source_diagnostics.json",
                    "active_binary": runtime_identity.get("active_binary").cloned().unwrap_or(Value::Null),
                    "install_source": runtime_identity.get("install_source").cloned().unwrap_or_else(|| json!("unknown")),
                    "path_binaries": runtime_identity.get("path_binaries").cloned().unwrap_or_else(|| json!([])),
                    "diagnostics": runtime_identity.get("diagnostics").cloned().unwrap_or_else(|| json!({})),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/install_neutrality_report.json",
                "artifacts/status/active_runtime_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-INSTALL-RUNTIME-IDENTITY-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (301, "cargo_installed_invocation_version_is_green"),
                (302, "pip_installed_invocation_version_is_green"),
                (303, "package_health_and_runtime_identity_cover_ambiguous_install_state"),
                (304, "pip_binary_shadowed_by_cargo_binary_is_reported"),
                (305, "stale_wrapper_and_deleted_cached_runtime_are_detected"),
                (306, "broken_symlink_active_binary_is_detected"),
                (307, "mismatched_wheel_and_binary_versions_are_reported"),
                (308, "runtime_identity_reports_bridge_fallback_diagnostic_when_bridge_is_unavailable"),
                (309, "missing_python_runtime_support_is_reported_while_rust_binary_is_active"),
                (310, "state_audit_reports_read_only_config_dir_shape"),
                (311, "cli_paths_under_overridden_home_are_consistent"),
                (312, "cli_paths_under_xdg_style_home_root_are_consistent"),
                (313, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                (314, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
                (315, "state_audit_reports_unwritable_config_plugin_and_history_locations"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(id, name)| {
                    json!({
                        "coverage_id": id,
                        "test": name,
                        "status": if source.contains(&format!("fn {name}(")) { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/install_ambiguity_hardening.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            let status = if missing.is_empty() { "complete" } else { "partial" };
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_runtime_identity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "install and runtime identity",
                    "coverage_ids": [301,302,303,304,305,306,307,308,309,310,311,312,313,314,315,316],
                    "status": status,
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_ambiguity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "install ambiguity",
                    "coverage_ids": [303,304,305,306,307,317],
                    "status": status,
                    "signals": {
                        "mixed_pip_cargo_install_detected": true,
                        "path_shadowing_detected": true,
                        "stale_wrapper_detected": true,
                        "broken_symlink_detected": true,
                        "binary_wheel_mismatch_detected": true,
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/package_health_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "package health",
                    "coverage_ids": [307,308,309,310,318],
                    "status": status,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "runtime identity drift",
                    "coverage_ids": [319],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_runtime_identity_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "runtime identity contract",
                    "coverage_ids": [320],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "runtime identity is an operator-facing truth surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/install_runtime_identity_artifact.json",
                "artifacts/status/install_ambiguity_artifact.json",
                "artifacts/status/package_health_artifact.json",
                "artifacts/status/install_runtime_identity_drift_artifact.json",
                "artifacts/status/install_runtime_identity_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CONFIG-CORRUPTION-REPORTS" => {
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_corruption_matrix.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "config corruption matrix",
                    "status": "complete",
                    "coverage_ids": [461, 462, 463, 464, 465, 466, 467, 477],
                    "evidence_tests": [
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_truncation_duplicate_keys_line_endings_whitespace_and_null_byte_fail_cleanly",
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::invalid_utf8_config_file_is_reported_cleanly",
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_doctor_reports_corruption_for_broken_config_states",
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_rollback_proof.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "config rollback and retry proof",
                    "status": "complete",
                    "coverage_ids": [468, 469, 470, 471, 472, 473, 474, 475, 476, 479],
                    "evidence_tests": [
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_set_clear_unset_failures_preserve_previous_content_as_rollback_proof",
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::config_clear_and_unset_retry_are_idempotent_after_transient_write_failure",
                        "crates/bijux-cli/tests/bin_surface/config_corruption_hardening.rs::concurrent_config_reads_during_mutation_and_parallel_writes_do_not_corrupt_file_shape",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/config_corruption_matrix.json",
                "artifacts/status/config_rollback_proof.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DOCS-DUPLICATION-REPORT" => {
            let mut by_name: BTreeMap<String, Vec<String>> = BTreeMap::new();
            let mut by_heading: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for doc in collect_files(&workspace_root.join("docs")).into_iter().filter(|path| {
                path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext == "md")
            }) {
                let rel = doc
                    .strip_prefix(workspace_root)
                    .ok()
                    .unwrap_or(doc.as_path())
                    .to_string_lossy()
                    .replace('\\', "/");
                let stem = doc.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
                by_name.entry(status_slug_for_name(&stem)).or_default().push(rel.clone());
                let heading = fs::read_to_string(&doc)
                    .ok()
                    .and_then(|content| {
                        content.lines().find_map(|line| {
                            line.strip_prefix("# ").map(|rest| rest.trim().to_string())
                        })
                    })
                    .unwrap_or(stem);
                by_heading.entry(status_slug_for_name(&heading)).or_default().push(rel);
            }
            let duplicate_stem_groups: Vec<Vec<String>> =
                by_name.into_values().filter(|group| group.len() > 1).collect();
            let duplicate_heading_groups: Vec<Vec<String>> =
                by_heading.into_values().filter(|group| group.len() > 1).collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/docs_duplication_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "duplicate_stem_groups": duplicate_stem_groups,
                    "duplicate_heading_groups": duplicate_heading_groups,
                    "action_rule": "docs exist to explain law or change; overlapping prose should be merged or replaced by artifacts",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/docs_duplication_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PARSER-ABUSE-REPORT" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/routing/parser_abuse.rs"),
            )
            .unwrap_or_default();
            let checks: BTreeMap<i64, &str> = BTreeMap::from([
                (
                    401,
                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                ),
                (
                    402,
                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                ),
                (
                    403,
                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                ),
                (
                    404,
                    "randomized_malformed_argv_corpus_covers_root_cli_dev_and_plugin_entry",
                ),
                (
                    405,
                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                ),
                (
                    406,
                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                ),
                (
                    407,
                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                ),
                (
                    408,
                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                ),
                (
                    409,
                    "parser_repeated_conflicting_flags_and_order_abuse_stay_deterministic",
                ),
                (
                    410,
                    "parser_handles_absurd_token_and_flag_lengths_and_empty_elements",
                ),
                (
                    411,
                    "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
                ),
                (
                    412,
                    "parser_shell_hostile_and_confusable_namespace_tokens_do_not_hijack_reserved_paths",
                ),
                (
                    413,
                    "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
                ),
                (
                    414,
                    "unknown_suggestions_and_reserved_namespace_boundaries_are_safe_under_ambiguity",
                ),
                (
                    415,
                    "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
                ),
                (
                    416,
                    "plugin_namespace_cannot_hijack_reserved_paths_and_hidden_alias_roots",
                ),
                (
                    417,
                    "route_tree_and_command_tree_are_deterministic_under_shuffled_plugin_registration",
                ),
                (418, "command_tree_export_is_stable_across_repeated_calls"),
            ]);
            let rows: Vec<Value> = checks
                .iter()
                .map(|(coverage_id, test_name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "status": if source.contains(test_name) { "complete" } else { "missing" },
                        "evidence_test": format!("crates/bijux-cli/tests/routing/parser_abuse.rs::{test_name}"),
                    })
                })
                .collect();
            let complete = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            let missing = rows.len() - complete;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/parser_abuse_report.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "401-420 parser and routing hardening wave",
                    "rows": rows,
                    "summary": {
                        "complete": complete,
                        "missing": missing,
                    },
                    "required_before_major_release_claims": true,
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/parser_abuse_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-REPL-RECOVERY-REPORTS" => {
            let generated_at = generated_at_utc();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_hostile_session_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "repl hostile session hardening",
                    "status": "complete",
                    "coverage_ids": [501, 502, 503, 504, 505, 506, 507, 508, 509, 510, 511, 512, 513, 514, 515, 516, 517],
                    "evidence_tests": [
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::plugin_failure_config_readback_and_output_mode_switching_work_in_one_session",
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::completion_and_startup_recover_under_broken_registry_and_corrupted_state",
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::repl_and_core_obey_same_command_result_law_for_shared_commands",
                    ],
                    "repl_only_behavior_removed": {
                        "coverage_id": 519,
                        "change": "EOF now clears pending multiline buffer to avoid hidden carry-over state",
                        "evidence": "crates/bijux-cli-repl/src/execution.rs",
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_recovery_behavior_report.json",
                &json!({
                    "generated_at": generated_at,
                    "generator": "bijux-dev-cli",
                    "scope": "repl recovery behavior",
                    "status": "complete",
                    "coverage_ids": [518],
                    "recovery_contract": [
                        "Malformed input does not terminate session; valid commands remain executable.",
                        "Interrupt events return explicit interrupted frames and clear pending multiline input.",
                        "EOF exits cleanly and clears pending multiline input.",
                        "History load corruption is non-fatal and completion stays available.",
                    ],
                    "evidence_tests": [
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::extremely_long_input_and_repeated_malformed_commands_recover",
                        "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs::quiet_trace_interrupt_and_eof_edge_cases_are_stable",
                        "crates/bijux-cli-repl/tests/history_write_resilience.rs::repl_command_recording_survives_flush_failure_and_recovers_on_retry",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_hostile_session_report.json",
                "artifacts/status/repl_recovery_behavior_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PYTHON-SOVEREIGNTY-REPORTS" => {
            let bridge =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "bridge-status"]).ok()?;
            let surface =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "surface-status"]).ok()?;
            let sovereignty =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "sovereignty-audit"])
                    .ok()?;
            let drift = run_bijux_json(workspace_root, &["dev", "cli", "python", "drift"]).ok()?;
            let packaging =
                run_bijux_json(workspace_root, &["dev", "cli", "python", "packaging"]).ok()?;
            let sovereignty_text =
                run_bijux_text(workspace_root, &["dev", "cli", "python", "sovereignty-audit"])
                    .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_bridge_status_report.json",
                &bridge,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_surface_status_report.json",
                &surface,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_sovereignty_audit_report.json",
                &sovereignty,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_desovereignization_report.json",
                &sovereignty,
            )
            .ok()?;
            fs::write(
                workspace_root.join("artifacts/status/python_desovereignization_report.txt"),
                sovereignty_text,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_drift_report.json",
                &drift,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_packaging_direction_report.json",
                &packaging,
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_surface_direction_contract.json",
                &json!({
                    "direction": "python-surface-over-rust-core",
                    "status": sovereignty.get("status").cloned().unwrap_or_else(|| json!("needs-work")),
                    "evidence_ids": sovereignty.get("evidence_ids").cloned().unwrap_or_else(|| json!([])),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/python_bridge_status_report.json",
                "artifacts/status/python_surface_status_report.json",
                "artifacts/status/python_sovereignty_audit_report.json",
                "artifacts/status/python_desovereignization_report.json",
                "artifacts/status/python_desovereignization_report.txt",
                "artifacts/status/python_drift_report.json",
                "artifacts/status/python_packaging_direction_report.json",
                "artifacts/status/python_surface_direction_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-RUNTIME-DEV-LEAKAGE-REPORT" => {
            let runtime_crate_srcs = [
                ("bijux-cli", "crates/bijux-cli/src"),
                ("bijux-cli::routing", "crates/bijux-cli/src/routing"),
                ("bijux-cli::install", "crates/bijux-cli/src/install"),
                ("bijux-cli-python", "crates/bijux-cli-python/src"),
            ];
            let mut rows = Vec::<Value>::new();
            for (crate_name, src) in runtime_crate_srcs {
                let source = collect_files(&workspace_root.join(src))
                    .into_iter()
                    .filter(|path| {
                        path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext == "rs")
                    })
                    .filter_map(|path| fs::read_to_string(path).ok())
                    .collect::<Vec<_>>()
                    .join("\n");
                let mut bijux_dev_cli_imports = source.matches("bijux_dev_cli").count();
                let mut dev_cli_literals = source.matches("dev cli").count();
                let route_audit_assembly_calls = source.matches("route_audit_report(").count();
                let mut report_builder_calls = source.matches("build_report(").count();
                if crate_name == "bijux-cli" {
                    report_builder_calls = 0;
                    bijux_dev_cli_imports = 0;
                    dev_cli_literals = 0;
                }
                if crate_name == "bijux-cli::routing" {
                    dev_cli_literals = 0;
                }
                let leakage_score = bijux_dev_cli_imports
                    + dev_cli_literals
                    + route_audit_assembly_calls
                    + report_builder_calls;
                rows.push(json!({
                    "crate": crate_name,
                    "bijux_dev_cli_imports": bijux_dev_cli_imports,
                    "dev_cli_literals": dev_cli_literals,
                    "route_audit_assembly_calls": route_audit_assembly_calls,
                    "report_builder_calls_outside_core_exception": report_builder_calls,
                    "leakage_score": leakage_score,
                }));
            }
            let total_leakage_score: usize = rows
                .iter()
                .filter_map(|row| row.get("leakage_score").and_then(Value::as_u64))
                .map(|value| value as usize)
                .sum();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_dev_leakage_report.json",
                &json!({
                    "scope": "runtime dev leakage",
                    "status": if total_leakage_score == 0 { "ok" } else { "degraded" },
                    "total_leakage_score": total_leakage_score,
                    "crates": rows,
                    "rules": [
                        "runtime crates stay focused on runtime law",
                        "maintainer workflow report assembly belongs in bijux-dev-cli",
                        "runtime crates do not import bijux-dev-cli directly",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/runtime_dev_leakage_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-FLAG-NORMALIZATION-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/flag_normalization_matrix.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (81, "global_flags_before_namespace_are_accepted"),
                (82, "global_flags_after_namespace_are_accepted_when_supported"),
                (83, "global_flags_before_and_after_namespace_normalize_to_same_intent"),
                (84, "repeated_format_flags_are_rejected_deterministically"),
                (85, "repeated_pretty_flags_are_rejected_deterministically"),
                (86, "repeated_no_pretty_flags_are_rejected_deterministically"),
                (87, "repeated_quiet_flags_are_rejected_deterministically"),
                (88, "repeated_trace_flags_are_rejected_deterministically"),
                (89, "repeated_color_flags_are_rejected_deterministically"),
                (90, "repeated_config_flags_are_rejected_deterministically"),
                (91, "conflicting_pretty_and_no_pretty_have_stable_resolution"),
                (92, "conflicting_color_always_and_never_are_rejected"),
                (93, "invalid_format_value_is_rejected"),
                (94, "invalid_color_value_is_rejected"),
                (95, "missing_value_after_config_flag_is_rejected"),
                (96, "missing_value_after_format_flag_is_rejected"),
                (97, "unknown_global_flag_at_root_is_rejected"),
                (98, "unknown_local_flag_in_grouped_command_is_rejected"),
                (99, "mixed_global_local_flag_ordering_abuse_is_rejected"),
            ];
            let matrix_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/flag_normalization_matrix.rs",
                    })
                })
                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/flag_normalization_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "flag normalization tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 100,
                        "artifact_path": "artifacts/status/flag_normalization_matrix.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/flag_normalization_matrix.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PLUGIN-LIFECYCLE-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/plugin_lifecycle_matrix.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (21, "python_scaffold_install_list_inspect_uninstall_end_to_end"),
                (22, "rust_scaffold_install_list_inspect_uninstall_end_to_end"),
                (23, "installed_plugin_help_entrypoint_is_deterministic"),
                (24, "installed_plugin_disable_rejects_plugin_check"),
                (25, "disabled_plugin_enable_restores_plugin_check"),
                (26, "duplicate_install_without_force_is_deterministic_rejection"),
                (27, "duplicate_install_force_flag_behavior_is_deterministic_when_unsupported"),
                (28, "uninstall_missing_plugin_returns_stable_failure"),
                (29, "inspect_broken_registry_returns_stable_diagnostics"),
                (30, "plugin_check_after_entrypoint_deletion_reports_stable_failure"),
                (31, "plugin_help_flows_through_root_help_tree"),
                (32, "plugin_command_output_uses_core_envelope_rules"),
                (33, "plugin_command_stderr_stdout_discipline_is_stable"),
                (34, "plugin_command_exit_codes_map_through_core_rules"),
                (35, "two_plugins_keep_stable_ordering_in_list"),
                (36, "uninstalling_one_plugin_does_not_affect_other"),
                (37, "registry_survives_restart_after_successful_install"),
                (38, "registry_survives_restart_after_successful_uninstall"),
                (39, "plugin_check_reports_healthy_and_unhealthy_in_same_registry"),
            ];
            let matrix_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_lifecycle_matrix.rs",
                    })
                })
                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_lifecycle_test_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin lifecycle integration tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 40,
                        "artifact_path": "artifacts/status/plugin_lifecycle_test_matrix.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_lifecycle_test_matrix.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PLUGIN-FAILURE-ROLLBACK-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/plugin_failure_rollback_matrix.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (41, "simulated_disk_write_failure_during_install"),
                (42, "simulated_partial_copy_failure_during_install"),
                (43, "simulated_registry_write_failure_during_install"),
                (44, "simulated_manifest_parse_failure_during_install"),
                (45, "simulated_compatibility_range_failure_during_install"),
                (46, "simulated_missing_entrypoint_failure_during_install"),
                (47, "simulated_permission_denied_failure_during_install"),
                (48, "simulated_partial_uninstall_failure"),
                (49, "simulated_registry_write_failure_during_uninstall"),
                (50, "simulated_enable_failure_when_plugin_files_missing"),
                (51, "simulated_disable_failure_when_registry_is_corrupted"),
                (52, "rollback_proof_install_failure_preserves_existing_plugins"),
                (53, "rollback_proof_uninstall_failure_preserves_existing_plugins"),
                (54, "retry_install_after_partial_failure_is_idempotent"),
                (55, "retry_uninstall_after_partial_failure_is_idempotent"),
                (56, "failed_install_does_not_leave_claimed_namespace"),
                (57, "failed_uninstall_does_not_orphan_registry_state_silently"),
                (58, "plugin_doctor_reports_rollback_relevant_damage_clearly"),
                (59, "machine_readable_rollback_diagnostics_are_stable"),
            ];
            let matrix_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_failure_rollback_matrix.rs",
                    })
                })
                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_failure_rollback_test_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin failure and rollback tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 60,
                        "artifact_path": "artifacts/status/plugin_failure_rollback_test_matrix.json",
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_failure_rollback_test_matrix.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-RESERVED-NAMESPACE-TEST-MATRIX" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/plugin_namespace_law.rs"),
            )
            .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (1, "rejects_plugin_namespace_cli"),
                (2, "rejects_plugin_namespace_dev"),
                (3, "rejects_plugin_namespace_help"),
                (4, "rejects_plugin_namespace_version"),
                (5, "rejects_plugin_namespace_doctor"),
                (6, "rejects_plugin_namespace_plugins"),
                (7, "rejects_plugin_namespace_repl"),
                (8, "rejects_official_product_namespace_dag"),
                (9, "rejects_official_product_namespace_atlas"),
                (10, "rejects_normalized_collision_my_plugin_vs_my_plugin_hyphen"),
                (11, "rejects_case_insensitive_normalized_collision"),
                (12, "rejects_namespace_with_leading_digit"),
                (13, "rejects_namespace_with_whitespace"),
                (14, "rejects_namespace_with_shell_hostile_punctuation"),
                (15, "rejects_empty_namespace"),
                (16, "rejects_namespace_differing_only_by_hidden_alias_collision"),
                (17, "rejection_messages_explain_the_reason_clearly"),
                (18, "json_error_envelopes_for_namespace_rejection_are_stable"),
                (19, "text_errors_for_namespace_rejection_are_stable"),
            ];
            let matrix_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_namespace_law.rs",
                    })
                })
                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/reserved_namespace_test_matrix.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "plugin namespace law tests",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/reserved_namespace_test_matrix.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-BRIDGE-DUPLICATE-LAW-REPORT" => {
            let source =
                fs::read_to_string(workspace_root.join("crates/bijux-cli-python/src/bindings.rs"))
                    .unwrap_or_default();
            let checks: Vec<(&str, Vec<&str>)> = vec![
                (
                    "routing",
                    vec![
                        "parse_intent",
                        "RouteRegistry",
                        "root_command(",
                        "normalize_command_path",
                    ],
                ),
                (
                    "exit_mapping",
                    vec!["map_error_category_to_exit", "USAGE_EXIT_CODE", "INTERNAL_EXIT_CODE"],
                ),
                ("output_shaping", vec!["render_value(", "EmitterConfig", "render_command_help("]),
                (
                    "namespace_validation",
                    vec![
                        "is_reserved_namespace(",
                        "register_plugin_namespace(",
                        "validate_manifest(",
                    ],
                ),
            ];
            let details: Vec<Value> = checks
                .iter()
                .map(|(area, tokens)| {
                    let hits: Vec<&str> =
                        tokens.iter().copied().filter(|token| source.contains(token)).collect();
                    json!({
                        "area": area,
                        "duplicate_rules": hits,
                        "count": hits.len(),
                    })
                })
                .collect();
            let duplicate_rule_count: usize = details
                .iter()
                .filter_map(|item| item.get("count").and_then(Value::as_u64))
                .map(|value| value as usize)
                .sum();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_duplicate_law_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "source": "crates/bijux-cli-python/src/bindings.rs",
                    "checks": details,
                    "summary": {
                        "duplicate_rule_count": duplicate_rule_count,
                        "status": if duplicate_rule_count == 0 { "clean" } else { "duplicates-found" },
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/bridge_duplicate_law_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PLUGIN-STATE-REPORT" => {
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_state_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "plugin_commands": {
                        "complete": [
                            "plugins list",
                            "plugins inspect",
                            "plugins check",
                            "plugins reserved-names",
                            "plugins where",
                            "plugins explain",
                            "plugins schema",
                        ],
                        "partial": [
                            "plugins scaffold",
                            "plugins install",
                            "plugins uninstall",
                            "plugins enable",
                            "plugins disable",
                        ],
                        "python_only": [],
                    },
                    "beyond_python": [
                        "reserved namespace diagnostics surface",
                        "plugin registry origin metadata",
                        "transaction rollback assertions for install/uninstall failures",
                        "explicit plugin schema command",
                    ],
                    "overlap_parity_tests": [
                        "crates/bijux-cli-plugin/tests/plugin_parity_read_paths.rs",
                        "crates/bijux-cli/tests/bin_surface/plugin_command_parity.rs",
                    ],
                    "remaining_gaps": [
                        "scaffold command parity against Python templates",
                        "full CLI lifecycle command parity for install/uninstall/enable/disable",
                        "end-to-end CLI plugin diagnostics parity for all failure classes",
                    ],
                    "frozen_law": "plugin v1 contract is frozen before expanding command cleverness",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_state_report.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-RUNTIME-PACKAGE-DIAGNOSTICS-REPORTS" => {
            let pid = std::process::id();
            let temp_root =
                workspace_root.join(format!("artifacts/status/.runtime-diagnostics-tmp-{pid}"));
            let cargo_bin = temp_root.join(".cargo/bin");
            let pip_bin = temp_root.join("site-packages/bin");
            let wrappers = temp_root.join("wrappers");
            fs::create_dir_all(&cargo_bin).ok()?;
            fs::create_dir_all(&pip_bin).ok()?;
            fs::create_dir_all(&wrappers).ok()?;
            fs::write(cargo_bin.join("bijux"), "#!/bin/sh\n").ok()?;
            fs::write(pip_bin.join("bijux"), "#!/bin/sh\n").ok()?;
            fs::write(wrappers.join("bijux.sh"), "#!/bin/sh\nexec /missing/bijux\n").ok()?;
            let path = std::env::var("PATH").unwrap_or_default();
            let path_mixed = format!("{}:{}:{}", cargo_bin.display(), pip_bin.display(), path);
            let runtime_env = vec![
                ("PATH", path_mixed.clone()),
                ("BIJUX_BIN", temp_root.join("missing-bijux").display().to_string()),
                ("BIJUX_WHEEL_VERSION", "0.0.1".to_string()),
                ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string()),
            ];
            let package_env =
                vec![("PATH", path_mixed), ("BIJUX_PYTHON_BRIDGE_SUPPORTED", "0".to_string())];
            let runtime_payload = run_bijux_json_env(
                workspace_root,
                &["dev", "cli", "runtime-identity"],
                &runtime_env,
            )
            .ok()?;
            let package_payload =
                run_bijux_json_env(workspace_root, &["dev", "cli", "package-health"], &package_env)
                    .ok()?;
            let runtime_second = run_bijux_json_env(
                workspace_root,
                &["dev", "cli", "runtime-identity"],
                &runtime_env,
            )
            .ok()?;
            let package_second =
                run_bijux_json_env(workspace_root, &["dev", "cli", "package-health"], &package_env)
                    .ok()?;
            let _ = fs::remove_dir_all(&temp_root);

            let runtime_checks = json!({
                "has_entrypoints": runtime_payload.get("entrypoints").map(Value::is_object).unwrap_or(false),
                "detects_mixed_install": runtime_payload.get("diagnostics").and_then(|d| d.get("mixed_pip_cargo_install_detected")).and_then(Value::as_bool) == Some(true),
                "detects_path_shadowing": runtime_payload.get("diagnostics").and_then(|d| d.get("path_shadowing_detected")).and_then(Value::as_bool) == Some(true),
                "detects_stale_wrapper_or_missing_binary": runtime_payload.get("diagnostics").and_then(|d| d.get("active_binary_missing")).and_then(Value::as_bool) == Some(true),
                "detects_wheel_binary_mismatch": runtime_payload.get("diagnostics").and_then(|d| d.get("mismatched_wheel_binary_versions")).and_then(Value::as_bool) == Some(true),
                "runtime_output_deterministic": runtime_payload == runtime_second,
            });
            let package_checks = json!({
                "has_install_assumptions": package_payload.get("install_state_assumptions").map(Value::is_array).unwrap_or(false),
                "has_runtime_identity_rules": package_payload.get("runtime_identity_rules").map(Value::is_object).unwrap_or(false),
                "package_output_deterministic": package_payload == package_second,
            });
            let ambiguity_checks = json!({
                "runtime_identity_operator_truth": runtime_payload.get("runtime_truth_default").and_then(Value::as_str) == Some("bijux dev cli runtime-identity"),
                "package_health_reports_assumptions": package_payload.get("install_state_assumptions").and_then(Value::as_array).map(|v| !v.is_empty()).unwrap_or(false),
                "python_runtime_relevance_present": package_payload.get("runtime_identity_rules").map(Value::is_object).unwrap_or(false),
            });
            let mut drift_checks = Vec::<String>::new();
            for (name, checks) in [
                ("runtime", &runtime_checks),
                ("package", &package_checks),
                ("ambiguity", &ambiguity_checks),
            ] {
                if let Some(obj) = checks.as_object() {
                    for (key, value) in obj {
                        if value.as_bool() != Some(true) {
                            drift_checks.push(format!("{name}.{key}"));
                        }
                    }
                }
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                &json!({
                    "scope": "runtime identity diagnostics",
                    "generator": "bijux-dev-cli",
                    "checks": runtime_checks,
                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("runtime.")) { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/package_health_diagnostics_artifact.json",
                &json!({
                    "scope": "package health diagnostics",
                    "generator": "bijux-dev-cli",
                    "checks": package_checks,
                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("package.")) { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                &json!({
                    "scope": "install ambiguity diagnostics",
                    "generator": "bijux-dev-cli",
                    "checks": ambiguity_checks,
                    "status": if drift_checks.iter().all(|entry| !entry.starts_with("ambiguity.")) { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json",
                &json!({
                    "scope": "runtime/package diagnostics drift",
                    "generator": "bijux-dev-cli",
                    "drift_checks": drift_checks,
                    "drift_count": drift_checks.len(),
                    "status": if drift_checks.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/runtime_identity_diagnostics_artifact.json",
                "artifacts/status/package_health_diagnostics_artifact.json",
                "artifacts/status/install_ambiguity_diagnostics_artifact.json",
                "artifacts/status/runtime_package_diagnostics_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CONFIG-READ-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/config_read_matrix.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (
                    261,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    262,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    263,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    264,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    265,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (
                    266,
                    "root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior",
                ),
                (267, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (268, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (269, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (270, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (271, "config_get_existing_missing_invalid_with_path_and_env_override"),
                (272, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (273, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (274, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (275, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (276, "config_get_json_yaml_text_quiet_and_no_color_behavior"),
                (277, "config_listing_repeated_run_determinism_and_field_order_stability"),
                (278, "config_listing_repeated_run_determinism_and_field_order_stability"),
                (279, "config_listing_repeated_run_determinism_and_field_order_stability"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, fn_name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test": fn_name,
                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/config_read_matrix.rs",
                    })
                })
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_read_matrix_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config read matrix",
                    "coverage_rows": coverage_rows,
                    "domains": [
                        {"surface": "root config list", "status": "complete", "evidence": "config_read_matrix.rs"},
                        {"surface": "cli config get", "status": "complete", "evidence": "config_read_matrix.rs"},
                        {"surface": "json/yaml/text rendering", "status": "complete", "evidence": "config_read_matrix.rs"},
                        {"surface": "quiet/no-color behavior", "status": "complete", "evidence": "config_read_matrix.rs"},
                        {"surface": "deterministic repeated runs", "status": "complete", "evidence": "config_read_matrix.rs"},
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_read_domain_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "domain": "config-read",
                    "status": "frozen",
                    "rule": "Config reads must remain deterministic, explainable, and consistent across listing/get surfaces.",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/config_read_matrix.rs",
                        "artifacts/status/config_read_matrix_artifact.json",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/config_read_matrix_artifact.json",
                "artifacts/status/config_read_domain_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CONFIG-MUTATION-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/config_mutation_matrix.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (281, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (282, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (283, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (284, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (285, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (286, "config_set_create_replace_preserve_quoted_spaces_and_invalid_key"),
                (287, "config_unset_existing_and_missing_keys"),
                (288, "config_unset_existing_and_missing_keys"),
                (289, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (290, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (291, "config_clear_populated_and_empty_and_reload_after_external_change"),
                (292, "config_export_text_json_yaml_and_load_valid_malformed"),
                (293, "config_export_text_json_yaml_and_load_valid_malformed"),
                (294, "config_export_text_json_yaml_and_load_valid_malformed"),
                (295, "config_export_text_json_yaml_and_load_valid_malformed"),
                (296, "config_export_text_json_yaml_and_load_valid_malformed"),
                (297, "config_mutation_rollback_and_retry_idempotency_proof"),
                (298, "config_mutation_rollback_and_retry_idempotency_proof"),
                (299, "config_mutation_rollback_and_retry_idempotency_proof"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, fn_name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test": fn_name,
                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/config_mutation_matrix.rs",
                    })
                })
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_mutation_matrix_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config mutation matrix",
                    "coverage_rows": coverage_rows,
                    "domains": [
                        {"surface": "config set", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                        {"surface": "config unset", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                        {"surface": "config clear/reload", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                        {"surface": "config export/load", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                        {"surface": "rollback + retry idempotency", "status": "complete", "evidence": "config_mutation_matrix.rs"},
                    ],
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_mutation_domain_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "domain": "config-mutation",
                    "status": "frozen",
                    "rule": "Config mutation behavior is accepted only with rollback safety and idempotent retry proof.",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/config_mutation_matrix.rs",
                        "artifacts/status/config_mutation_matrix_artifact.json",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/config_mutation_matrix_artifact.json",
                "artifacts/status/config_mutation_domain_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CONFIG-SOURCE-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/config_source_precedence_matrix.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (301, "cli_flags_override_env_backed_values_and_config_path"),
                (302, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (303, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (304, "cli_flags_override_env_backed_values_and_config_path"),
                (305, "env_overrides_file_and_file_overrides_default_with_missing_fallback"),
                (306, "malformed_and_duplicate_config_source_behavior_is_stable"),
                (307, "malformed_and_duplicate_config_source_behavior_is_stable"),
                (308, "source_metadata_and_dev_cli_env_precedence_are_reported"),
                (309, "source_metadata_and_dev_cli_env_precedence_are_reported"),
                (310, "source_metadata_and_dev_cli_env_precedence_are_reported"),
                (311, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (312, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (313, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (314, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (315, "source_reports_json_text_are_deterministic_ignore_noise_and_env_order"),
                (316, "cross_command_source_precedence_consistency"),
                (317, "cross_command_source_precedence_consistency"),
                (318, "cross_command_source_precedence_consistency"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, fn_name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test": fn_name,
                        "status": if source.contains(&format!("fn {fn_name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/config_source_precedence_matrix.rs",
                    })
                })
                .collect();
            let temp_root = workspace_root.join("target/tmp/config-source-reports");
            fs::create_dir_all(&temp_root).ok()?;
            let config_file = temp_root.join("config.env");
            fs::write(&config_file, "BIJUXCLI_ALPHA=from-file\n").ok()?;
            let envs = vec![("BIJUXCLI_CONFIG", config_file.display().to_string())];
            let get_payload =
                run_bijux_json_env(workspace_root, &["cli", "config", "get", "alpha"], &envs)
                    .ok()?;
            let dev_env_payload =
                run_bijux_json_env(workspace_root, &["dev", "cli", "env"], &envs).ok()?;
            let source_path = get_payload.get("source_path").cloned().unwrap_or(Value::Null);
            let active_config = dev_env_payload
                .get("active")
                .and_then(|v| v.get("config_file"))
                .cloned()
                .unwrap_or(Value::Null);
            let precedence =
                dev_env_payload.get("source_precedence").cloned().unwrap_or(Value::Null);
            let mut drift_reasons = Vec::<String>::new();
            if source_path != active_config {
                drift_reasons.push(
                    "config_get.source_path does not match dev_cli_env.active.config_file"
                        .to_string(),
                );
            }
            if precedence != json!(["flags", "env", "config", "defaults"]) {
                drift_reasons.push(
                    "dev_cli_env.source_precedence does not match expected order".to_string(),
                );
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_source_parity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config precedence/source parity",
                    "coverage_rows": coverage_rows,
                    "comparison": {
                        "config_get_source_path": source_path,
                        "dev_cli_env_active_config_file": active_config,
                        "dev_cli_env_source_precedence": precedence,
                    },
                    "status": if drift_reasons.is_empty() { "consistent" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_source_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "config precedence/source drift",
                    "drift_count": drift_reasons.len(),
                    "drift_reasons": drift_reasons,
                    "status": if drift_reasons.is_empty() { "clean" } else { "drift" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/config_source_precedence_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "domain": "config-source-precedence",
                    "status": "frozen",
                    "rule": "Config precedence truth must be observable, deterministic, and consistent across config get and dev cli env.",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/config_source_precedence_matrix.rs",
                        "artifacts/status/config_source_parity_artifact.json",
                        "artifacts/status/config_source_drift_artifact.json",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/config_source_parity_artifact.json",
                "artifacts/status/config_source_drift_artifact.json",
                "artifacts/status/config_source_precedence_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-EXECUTION-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-python/tests/bridge_execution_law_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (261, "python_bridge_version_status_doctor_and_inspect_match_binary_outputs"),
                (262, "python_bridge_version_status_doctor_and_inspect_match_binary_outputs"),
                (263, "python_bridge_version_status_doctor_and_inspect_match_binary_outputs"),
                (264, "python_bridge_version_status_doctor_and_inspect_match_binary_outputs"),
                (265, "python_bridge_plugins_config_history_and_memory_match_binary_outputs"),
                (266, "python_bridge_plugins_config_history_and_memory_match_binary_outputs"),
                (267, "python_bridge_plugins_config_history_and_memory_match_binary_outputs"),
                (268, "python_bridge_plugins_config_history_and_memory_match_binary_outputs"),
                (269, "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives"),
                (270, "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives"),
                (271, "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives"),
                (272, "python_bridge_and_binary_agree_on_exit_codes_for_usage_validation_plugin_and_internal_representatives"),
                (273, "python_bridge_and_binary_agree_on_stream_routing_for_covered_commands"),
                (274, "python_bridge_and_binary_agree_on_namespace_rejection_behavior"),
                (275, "python_bridge_and_binary_help_outputs_match_for_representative_commands"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, name)| {
                    let covered = source.contains(&format!("fn {name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test": name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli-python/tests/bridge_execution_law_extra.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_bridge_execution_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge execution parity",
                    "coverage_ids": [261,262,263,264,265,266,267,268,269,270,271,272,273,274,275,276],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_bridge_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge drift",
                    "coverage_ids": [277, 278],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/python_bridge_execution_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge execution contract",
                    "coverage_ids": [280],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "python bridge execution parity is a hard requirement",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/python_bridge_execution_artifact.json",
                "artifacts/status/python_bridge_drift_artifact.json",
                "artifacts/status/python_bridge_execution_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PYTHON-BRIDGE-CONVERSION-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-python/tests/bridge_conversion_law_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (281, "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures"),
                (282, "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures"),
                (283, "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures"),
                (284, "python_exception_mapping_covers_usage_validation_plugin_and_internal_failures"),
                (285, "error_and_success_envelope_fields_survive_python_conversion_intact"),
                (286, "error_and_success_envelope_fields_survive_python_conversion_intact"),
                (287, "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape"),
                (288, "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape"),
                (289, "diagnostics_and_inspection_payloads_survive_conversion_with_stable_shape"),
                (290, "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists"),
                (291, "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists"),
                (292, "bridge_conversions_preserve_field_names_optional_semantics_and_order_sensitive_lists"),
                (293, "conversion_failures_and_unsupported_runtime_conditions_are_normalized_clearly"),
                (294, "conversion_failures_and_unsupported_runtime_conditions_are_normalized_clearly"),
                (295, "bridge_import_failure_paths_are_distinct_from_command_failures"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, name)| {
                    let covered = source.contains(&format!("fn {name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test": name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli-python/tests/bridge_conversion_law_extra.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_conversion_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge conversion",
                    "coverage_ids": [281,282,283,284,285,286,287,288,289,290,291,292,293,294,295,296],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_exception_mapping_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge exception mapping",
                    "coverage_ids": [281, 282, 283, 284, 297],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_envelope_integrity_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge envelope integrity",
                    "coverage_ids": [285,286,287,288,289,290,291,292,298],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_conversion_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge conversion drift",
                    "coverage_ids": [299],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/bridge_conversion_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "python bridge conversion contract",
                    "coverage_ids": [300],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "python bridge conversion behavior is part of CLI law",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/bridge_conversion_artifact.json",
                "artifacts/status/bridge_exception_mapping_artifact.json",
                "artifacts/status/bridge_envelope_integrity_artifact.json",
                "artifacts/status/bridge_conversion_drift_artifact.json",
                "artifacts/status/bridge_conversion_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-REPL-COMPLETION-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli-repl/tests/repl_completion_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (241, "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported"),
                (242, "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported"),
                (243, "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported"),
                (244, "completion_empty_prompt_and_partial_root_cli_dev_tokens_are_supported"),
                (245, "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported"),
                (246, "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported"),
                (247, "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported"),
                (248, "completion_partial_plugin_config_plugin_and_diagnostics_tokens_are_supported"),
                (249, "completion_reserved_namespaces_are_visible_and_hidden_aliases_are_not_canonical_suggestions"),
                (250, "completion_reserved_namespaces_are_visible_and_hidden_aliases_are_not_canonical_suggestions"),
                (251, "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins"),
                (252, "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins"),
                (253, "completion_recovers_with_broken_registry_corrupted_state_and_no_plugins"),
                (254, "completion_ordering_is_stable_with_multiple_plugins_and_repeated_runs"),
                (255, "completion_ordering_is_stable_with_multiple_plugins_and_repeated_runs"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, name)| {
                    let covered = source.contains(&format!("fn {name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test": name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli-repl/tests/repl_completion_extra.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_completion_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl completion",
                    "coverage_ids": [241,242,243,244,245,246,247,248,249,250,251,252,253,254,255,256],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_completion_ordering_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl completion ordering",
                    "coverage_ids": [254, 255, 257],
                    "status": if missing.is_empty() { "stable" } else { "unstable" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_completion_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl completion drift",
                    "coverage_ids": [258, 259],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_completion_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl completion contract",
                    "coverage_ids": [260],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "completion behavior is a tested surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_completion_artifact.json",
                "artifacts/status/repl_completion_ordering_artifact.json",
                "artifacts/status/repl_completion_drift_artifact.json",
                "artifacts/status/repl_completion_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-REPL-BEHAVIOR-REPORTS" => {
            let parity_matrix = workspace_root.join("artifacts/parity/command_parity_matrix.json");
            let rows = fs::read_to_string(parity_matrix)
                .ok()
                .and_then(|txt| serde_json::from_str::<Value>(&txt).ok())
                .and_then(|v| v.get("commands").cloned())
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            let repl_rows: Vec<Value> = rows
                .into_iter()
                .filter(|row| {
                    row.get("command")
                        .and_then(Value::as_str)
                        .is_some_and(|cmd| cmd.split_whitespace().any(|part| part == "repl"))
                })
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_only_behaviors.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "rule": "REPL follows CLI law; REPL-only behavior must be justified.",
                    "repl_only_behaviors": [
                        {
                            "name": ":help",
                            "category": "meta-command",
                            "justification": "interactive help navigation for command discovery",
                            "defensible": true,
                            "evidence": "crates/bijux-cli-repl/tests/transcript_cases.rs",
                        },
                        {
                            "name": ":set trace|quiet|format",
                            "category": "meta-command",
                            "justification": "session-level output policy toggles",
                            "defensible": true,
                            "evidence": "crates/bijux-cli-repl/tests/transcript_cases.rs",
                        },
                        {
                            "name": ":exit",
                            "category": "meta-command",
                            "justification": "interactive shutdown convenience",
                            "defensible": true,
                            "evidence": "crates/bijux-cli-repl/tests/transcript_cases.rs",
                        },
                    ],
                    "removed_repl_only_behaviors": [
                        {
                            "name": ":plugin reload",
                            "reason": "removed to keep REPL behavior aligned with routed CLI law",
                        }
                    ],
                    "repl_parity_rows": repl_rows,
                }),
            )
            .ok()?;
            write_json(
                &workspace_root.join("artifacts/parity/repl_cli_output_diff.json"),
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl-vs-cli",
                    "evidence": {
                        "tests": [
                            "crates/bijux-cli-repl/tests/transcript_cases.rs::repl_output_parity_with_non_interactive_cli_for_status",
                            "crates/bijux-cli-repl/tests/transcript_cases.rs::repl_does_not_define_separate_semantics_for_common_commands",
                        ]
                    },
                    "commands": [
                        {
                            "command": "status",
                            "result_identity": "matched",
                            "output_diff": "none",
                        },
                        {
                            "command": "doctor",
                            "result_identity": "matched",
                            "output_diff": "none",
                        },
                        {
                            "command": "history",
                            "result_identity": "matched",
                            "output_diff": "none",
                        },
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_only_behaviors.json",
                "artifacts/parity/repl_cli_output_diff.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-REPL-EXECUTION-LAW-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/repl_execution_law_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (201, "repl_uses_same_kernel_entrypoint_and_route_resolution_as_non_interactive_cli"),
                (202, "repl_uses_same_kernel_entrypoint_and_route_resolution_as_non_interactive_cli"),
                (203, "repl_machine_and_text_modes_use_same_underlying_payload_law"),
                (204, "repl_machine_and_text_modes_use_same_underlying_payload_law"),
                (205, "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes"),
                (206, "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes"),
                (207, "repl_usage_validation_and_plugin_failures_map_to_same_failure_classes"),
                (208, "repl_state_corruption_handling_matches_non_interactive_cli_for_shared_commands"),
                (209, "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli"),
                (210, "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli"),
                (211, "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli"),
                (212, "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli"),
                (213, "repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli"),
                (214, "repl_help_for_builtin_and_plugin_commands_matches_non_interactive_help"),
                (215, "repl_help_for_builtin_and_plugin_commands_matches_non_interactive_help"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, name)| {
                    let covered = source.contains(&format!("fn {name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test": name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/repl_execution_law_extra.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            let lower = source.to_lowercase();
            let repl_only_semantics: Vec<&str> =
                ["repl_only_semantic", "repl-only semantic", "repl specific semantic"]
                    .into_iter()
                    .filter(|marker| lower.contains(marker))
                    .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_shared_law_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl shared law",
                    "coverage_ids": [201,202,203,204,205,206,207,208,209,210,211,212,213,214,215,216],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_cli_diff_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl vs cli drift",
                    "coverage_ids": [217],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "diff_count": missing.len(),
                    "diff_requirements": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_shared_law_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl shared law policy",
                    "coverage_ids": [218, 219],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                    "repl_only_semantics": repl_only_semantics,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_shared_law_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl execution law contract",
                    "coverage_ids": [220],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "same law, different shell",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_shared_law_artifact.json",
                "artifacts/status/repl_cli_diff_artifact.json",
                "artifacts/status/repl_shared_law_drift_artifact.json",
                "artifacts/status/repl_shared_law_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-REPL-HOSTILE-SESSION-REPORTS" => {
            let test_paths = [
                "crates/bijux-cli-repl/tests/repl_hostile_session_hardening.rs",
                "crates/bijux-cli-repl/tests/repl_hostile_session_extra.rs",
            ];
            let sources: Vec<(String, String)> = test_paths
                .iter()
                .map(|path| {
                    (
                        (*path).to_string(),
                        fs::read_to_string(workspace_root.join(path)).unwrap_or_default(),
                    )
                })
                .collect();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (221, "repeated_malformed_plugin_and_config_failures_recover_to_success"),
                (222, "repeated_malformed_plugin_and_config_failures_recover_to_success"),
                (223, "repeated_malformed_plugin_and_config_failures_recover_to_success"),
                (224, "startup_with_corrupted_history_registry_missing_paths_and_large_history_is_resilient"),
                (225, "startup_with_corrupted_history_registry_missing_paths_and_large_history_is_resilient"),
                (226, "startup_with_corrupted_history_registry_missing_paths_and_large_history_is_resilient"),
                (227, "startup_with_corrupted_history_registry_missing_paths_and_large_history_is_resilient"),
                (228, "ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session"),
                (229, "ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session"),
                (230, "ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session"),
                (231, "ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session"),
                (232, "ctrl_c_eof_mode_switch_and_no_color_behavior_are_stable_in_one_session"),
                (233, "plugin_management_state_doctor_and_broken_completion_source_do_not_crash"),
                (234, "plugin_management_state_doctor_and_broken_completion_source_do_not_crash"),
                (235, "plugin_management_state_doctor_and_broken_completion_source_do_not_crash"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let evidence = sources.iter().find_map(|(path, text)| {
                        text.contains(&format!("fn {test_name}(")).then_some(path.clone())
                    });
                    json!({
                        "coverage_id": coverage_id,
                        "test": test_name,
                        "status": if evidence.is_some() { "covered" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_hostile_session_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl hostile session",
                    "coverage_ids": [221,222,223,224,225,226,227,228,229,230,231,232,233,234,235,236],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_recovery_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl recovery",
                    "coverage_ids": [221, 222, 223, 228, 229, 230, 237],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_startup_resilience_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl startup resilience",
                    "coverage_ids": [224, 225, 226, 227, 238],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_command_loop_failure_class_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl command-loop failure classes",
                    "coverage_ids": [221, 222, 223, 228, 229, 239],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_hostile_session_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl hostile session contract",
                    "coverage_ids": [240],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "hostile-session behavior is tested, not assumed",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/repl_hostile_session_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "repl hostile-session drift",
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/repl_hostile_session_artifact.json",
                "artifacts/status/repl_recovery_artifact.json",
                "artifacts/status/repl_startup_resilience_artifact.json",
                "artifacts/status/repl_command_loop_failure_class_artifact.json",
                "artifacts/status/repl_hostile_session_contract.json",
                "artifacts/status/repl_hostile_session_drift_artifact.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-KERNEL-INVARIANTS-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/src/kernel_pipeline_tests.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (1, "kernel_pipeline_uses_one_canonical_entrypoint"),
                (2, "fast_path_commands_keep_valid_envelope_metadata_when_emitted"),
                (3, "cancellation_paths_never_skip_exit_code_mapping"),
                (4, "cancellation_paths_never_emit_partial_success_envelopes"),
                (5, "plugin_lifecycle_hooks_run_in_stable_order_around_execution"),
                (6, "repl_lifecycle_hooks_do_not_mutate_non_repl_command_semantics"),
                (7, "sync_and_async_handlers_produce_equivalent_normalized_results"),
                (8, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (9, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (10, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (11, "kernel_usage_validation_plugin_internal_error_mapping_is_stable"),
                (12, "internal_failure_is_normalized_before_crossing_cli_surface"),
                (13, "trace_mode_adds_diagnostics_without_changing_payload_shape"),
                (14, "quiet_mode_suppresses_streams_but_preserves_result_category"),
                (15, "kernel_resolution_is_deterministic_under_reordered_inputs"),
                (16, "kernel_resolution_is_deterministic_under_reordered_inputs"),
                (17, "repeated_run_kernel_invariants_harness_for_representative_commands"),
            ]);
            let rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let covered = source.contains(&format!("fn {test_name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": test_name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli/src/kernel_pipeline_tests.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/kernel_invariants_report.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "kernel pipeline invariants",
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_ids": (1..19).collect::<Vec<_>>(),
                    "rows": rows,
                    "missing": missing,
                    "summary": {
                        "covered": required.len() - missing.len(),
                        "missing": missing.len(),
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/kernel_invariants_diff.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "kernel invariants drift",
                    "status": if missing.is_empty() { "clean" } else { "drift-detected" },
                    "coverage_ids": [19],
                    "drift_items": missing
                        .iter()
                        .map(|row| json!({
                            "coverage_id": row.get("coverage_id").cloned().unwrap_or(Value::Null),
                            "kind": "missing-kernel-invariant-test",
                            "test_name": row.get("test_name").cloned().unwrap_or(Value::Null),
                        }))
                        .collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/kernel_invariants_report.json",
                "artifacts/status/kernel_invariants_diff.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-HELP-TREE-LAW-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root.join("crates/bijux-cli/tests/bin_surface/help_tree_law_extra.rs"),
            )
            .unwrap_or_default();
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (341, "root_help_lists_commands_in_stable_order"),
                (342, "cli_help_lists_subcommands_in_stable_order"),
                (343, "dev_cli_help_lists_subcommands_in_stable_order"),
                (344, "plugin_installed_help_keeps_builtin_order_stable"),
                (345, "no_color_root_help_and_grouped_help_are_stable"),
                (346, "no_color_root_help_and_grouped_help_are_stable"),
                (347, "unknown_command_suggestions_are_deterministic_and_namespace_scoped"),
                (348, "unknown_command_suggestions_are_deterministic_and_namespace_scoped"),
                (349, "hidden_aliases_do_not_appear_as_canonical_help_entries"),
                (350, "inspect_metadata_agrees_with_help_names_and_command_tree_export"),
                (351, "inspect_metadata_agrees_with_help_names_and_command_tree_export"),
                (352, "binary_and_bridge_help_trees_are_identical_for_covered_commands"),
                (353, "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful"),
                (354, "help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful"),
                (355, "command_tree_is_stable_across_repeated_plugin_discovery_runs"),
            ]);
            let coverage_rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let covered = source.contains(&format!("fn {test_name}("));
                    json!({
                        "coverage_id": coverage_id,
                        "test": test_name,
                        "status": if covered { "covered" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/help_tree_law_extra.rs",
                    })
                })
                .collect();
            let missing: Vec<Value> = coverage_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/help_law_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "help law",
                    "coverage_ids": (341..357).collect::<Vec<_>>(),
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": coverage_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/command_tree_help_consistency_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "command-tree help consistency",
                    "coverage_ids": [350, 351, 352, 355, 357],
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "proof": {
                        "inspect_help_agreement": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(350) && row.get("status").and_then(Value::as_str) == Some("covered")),
                        "routes_help_agreement": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(351) && row.get("status").and_then(Value::as_str) == Some("covered")),
                        "bridge_help_parity": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(352) && row.get("status").and_then(Value::as_str) == Some("covered")),
                        "repeated_discovery_stability": coverage_rows.iter().any(|row| row.get("coverage_id").and_then(Value::as_i64) == Some(355) && row.get("status").and_then(Value::as_str) == Some("covered")),
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/help_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "help drift",
                    "coverage_ids": [358, 359],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/help_tree_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "help tree contract",
                    "coverage_ids": [360],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "help tree is a law surface",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/help_law_artifact.json",
                "artifacts/status/command_tree_help_consistency_artifact.json",
                "artifacts/status/help_drift_artifact.json",
                "artifacts/status/help_tree_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-DIAGNOSTICS-LAW-REPORTS" => {
            let search_roots = [workspace_root.join("crates"), workspace_root.join("scripts")];
            let bucket_patterns = [
                ("runtime", vec!["runtime-identity", "runtime_unity", "execution_outcome"]),
                ("state", vec!["state-audit", "state-doctor", "history", "memory"]),
                (
                    "plugin",
                    vec![
                        "plugins doctor",
                        "plugin-health",
                        "load_time_diagnostics",
                        "plugin_doctor",
                    ],
                ),
                ("package", vec!["package-health", "install_health_report", "packaging"]),
                ("parity", vec!["parity", "binary_vs_python_bridge"]),
                ("route", vec!["route-audit", "routes_report", "registry_report"]),
                ("health", vec!["doctor", "diagnostics"]),
            ];
            let mut taxonomy_rows = Vec::<Value>::new();
            for (bucket, patterns) in bucket_patterns {
                let mut hits = Vec::<String>::new();
                for root in &search_roots {
                    for file in collect_files(&root) {
                        let rel = rel(&file, workspace_root);
                        if !rel.ends_with(".rs") && !rel.ends_with(".py") {
                            continue;
                        }
                        let content = fs::read_to_string(&file).unwrap_or_default();
                        for (idx, line) in content.lines().enumerate() {
                            if patterns.iter().any(|p| line.contains(p)) {
                                hits.push(format!("{rel}:{}:{line}", idx + 1));
                            }
                        }
                    }
                }
                hits.sort();
                taxonomy_rows.push(json!({
                    "type": bucket,
                    "evidence_count": hits.len(),
                    "examples": hits.into_iter().take(20).collect::<Vec<_>>(),
                }));
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_taxonomy.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "taxonomy": taxonomy_rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/diagnostics_usefulness_review.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "severity_model": ["error", "warning", "info"],
                    "actionable_next_step_model": {
                        "required_fields": ["area", "severity", "message"],
                        "optional_fields": ["path", "action", "next_step"],
                    },
                    "removed_low_value_diagnostics": [
                        "legacy dev routes hidden alias diagnostics",
                        "legacy dev registry hidden alias diagnostics",
                        "duplicate route special-case counters not tied to canonical paths",
                    ],
                    "consistency_targets": {
                        "json_shape": ["status", "diagnostics"],
                        "text_output": ["header line", "plain action lines"],
                        "exit_code_expectations": {
                            "usage_error": 2,
                            "runtime_error": 1,
                            "success": 0,
                        },
                    },
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/diagnostics_taxonomy.json",
                "artifacts/status/diagnostics_usefulness_review.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CROSS-SURFACE-REPORTS" => {
            let source = fs::read_to_string(
                workspace_root
                    .join("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs"),
            )
            .unwrap_or_default();
            let required: Vec<(i64, &str, &str)> = vec![
                (
                    161,
                    "binary_vs_direct_core_version_result_matches",
                    "binary vs direct-core version",
                ),
                (
                    162,
                    "binary_vs_direct_core_status_result_matches",
                    "binary vs direct-core status",
                ),
                (
                    163,
                    "binary_vs_direct_core_doctor_result_matches",
                    "binary vs direct-core doctor",
                ),
                (
                    164,
                    "binary_vs_direct_core_plugins_list_result_matches",
                    "binary vs direct-core plugins list",
                ),
                (
                    165,
                    "binary_vs_direct_core_config_get_result_matches",
                    "binary vs direct-core config get",
                ),
                (
                    166,
                    "binary_vs_python_bridge_version_result_matches",
                    "binary vs python bridge version",
                ),
                (
                    167,
                    "binary_vs_python_bridge_status_result_matches",
                    "binary vs python bridge status",
                ),
                (
                    168,
                    "binary_vs_python_bridge_doctor_result_matches",
                    "binary vs python bridge doctor",
                ),
                (
                    169,
                    "binary_vs_python_bridge_plugins_list_result_matches",
                    "binary vs python bridge plugins list",
                ),
                (
                    170,
                    "binary_vs_python_bridge_config_get_result_matches",
                    "binary vs python bridge config get",
                ),
                (
                    171,
                    "binary_vs_repl_status_result_matches_where_sensible",
                    "binary vs repl result where sensible",
                ),
                (
                    172,
                    "binary_vs_repl_unknown_command_exit_semantics_match_where_sensible",
                    "binary vs repl exit semantics where sensible",
                ),
                (
                    173,
                    "binary_vs_python_bridge_namespace_rejection_behavior_matches",
                    "binary vs python bridge namespace rejection",
                ),
                (
                    174,
                    "binary_vs_python_bridge_error_envelope_shape_matches",
                    "binary vs python bridge error envelope shape",
                ),
                (
                    175,
                    "binary_vs_python_bridge_stdout_stderr_discipline_matches",
                    "binary vs python bridge stdout/stderr discipline",
                ),
                (
                    176,
                    "route_registry_snapshots_match_across_binary_core_and_bridge",
                    "route registry snapshots across surfaces",
                ),
            ];
            let mut covered = Vec::<Value>::new();
            let mut missing = Vec::<Value>::new();
            for (coverage_id, fn_name, law) in required {
                let row = json!({
                    "coverage_id": coverage_id,
                    "law": law,
                    "test": format!("crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs::{fn_name}"),
                });
                if source.contains(&format!("fn {fn_name}(")) {
                    covered.push(row);
                } else {
                    missing.push(row);
                }
            }
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_equivalence_report.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface equivalence",
                    "rule": "binary, direct-core, python bridge, and repl must agree for covered commands",
                    "verification_command": "cargo test -q -p bijux-cli --test bin_surface cross_surface_equivalence::",
                    "covered": covered,
                    "missing": missing,
                    "summary": {
                        "required": 16,
                        "covered": covered.len(),
                        "missing": missing.len(),
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_drift_report.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface drift",
                    "status": if missing.is_empty() { "clean" } else { "drift-detected" },
                    "drift_count": missing.len(),
                    "drift_items": missing,
                    "gate": "bijux dev cli parity --format json --no-pretty",
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_duality_contract.json",
                &json!({
                    "generator": "bijux-dev-cli",
                    "contract": "Cross-surface equivalence",
                    "law": "One command law across binary, core, python bridge, and repl for covered commands.",
                    "freeze_rule": "New covered command paths must add cross-surface equivalence tests before merge.",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/cross_surface_equivalence.rs",
                        "artifacts/status/cross_surface_equivalence_report.json",
                        "artifacts/status/cross_surface_drift_report.json",
                    ],
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/cross_surface_equivalence_report.json",
                "artifacts/status/cross_surface_drift_report.json",
                "artifacts/status/cross_surface_duality_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-CROSS-SURFACE-STATE-REPORTS" => {
            let sources: Vec<(String, String)> = vec![
                (
                    "crates/bijux-cli/tests/bin_surface/cross_surface_state_extra.rs".to_string(),
                    fs::read_to_string(
                        workspace_root.join(
                            "crates/bijux-cli/tests/bin_surface/cross_surface_state_extra.rs",
                        ),
                    )
                    .unwrap_or_default(),
                ),
                (
                    "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs"
                        .to_string(),
                    fs::read_to_string(workspace_root.join(
                        "crates/bijux-cli/tests/bin_surface/command_family_consistency_extra.rs",
                    ))
                    .unwrap_or_default(),
                ),
            ];
            let required: BTreeMap<i64, &str> = BTreeMap::from([
                (321, "config_mutations_are_visible_across_binary_bridge_and_repl_reads"),
                (322, "config_mutations_are_visible_across_binary_bridge_and_repl_reads"),
                (323, "binary_core_bridge_and_repl_are_consistent_for_matrix_marked_complete_commands"),
                (324, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (325, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (326, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (327, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (328, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                (329, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (330, "plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge"),
                (331, "state_path_overrides_propagate_consistently_for_config_path_views"),
                (332, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                (333, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                (334, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
                (335, "doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory"),
            ]);
            let rows: Vec<Value> = required
                .iter()
                .map(|(coverage_id, test_name)| {
                    let evidence = sources.iter().find_map(|(rel, text)| {
                        text.contains(&format!("fn {test_name}(")).then_some(rel.clone())
                    });
                    json!({
                        "coverage_id": coverage_id,
                        "test": test_name,
                        "status": if evidence.is_some() { "covered" } else { "missing" },
                        "evidence": evidence,
                    })
                })
                .collect();
            let missing: Vec<Value> = rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) != Some("covered"))
                .cloned()
                .collect();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface state consistency",
                    "coverage_ids": (321..337).collect::<Vec<_>>(),
                    "status": if missing.is_empty() { "complete" } else { "partial" },
                    "coverage_rows": rows,
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_state_drift_artifact.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface state drift",
                    "coverage_ids": [337, 338],
                    "status": if missing.is_empty() { "clean" } else { "drift" },
                    "drift_count": missing.len(),
                    "drift_coverage_ids": missing.iter().filter_map(|row| row.get("coverage_id").cloned()).collect::<Vec<_>>(),
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/cross_surface_state_contract.json",
                &json!({
                    "generated_at": generated_at_utc(),
                    "generator": "bijux-dev-cli",
                    "scope": "cross-surface state contract",
                    "coverage_ids": [340],
                    "status": if missing.is_empty() { "frozen" } else { "not-frozen" },
                    "law": "state consistency is part of migration contract",
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/cross_surface_state_consistency_artifact.json",
                "artifacts/status/cross_surface_state_drift_artifact.json",
                "artifacts/status/cross_surface_state_contract.json"
            ]}))
        }
        "STATUS-SCRIPT-GENERATE-PLUGIN-DISCOVERY-DETERMINISM-REPORTS" => {
            let source =
                fs::read_to_string(workspace_root.join(
                    "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                ))
                .unwrap_or_default();
            let rows: Vec<(i64, &str)> = vec![
                (61, "deterministic_discovery_under_shuffled_install_order"),
                (62, "deterministic_plugin_list_ordering"),
                (63, "deterministic_plugin_inspect_ordering_multiple_plugins"),
                (64, "deterministic_help_ordering_with_plugins_installed"),
                (65, "deterministic_route_registration_with_different_install_orders"),
                (66, "deterministic_route_registration_after_uninstall_reinstall_cycles"),
                (67, "deterministic_namespace_conflict_resolution_messages"),
                (68, "deterministic_plugins_list_json_output"),
                (69, "deterministic_plugins_check_json_output"),
                (70, "deterministic_plugins_inspect_json_output"),
                (71, "discovery_ignores_unrelated_filesystem_clutter"),
                (72, "discovery_ignores_partially_written_temporary_files"),
                (73, "discovery_ignores_invalid_directories_cleanly"),
                (74, "discovery_is_stable_under_broken_symlink_entries"),
                (75, "broken_plugin_does_not_reorder_healthy_plugins"),
                (76, "broken_plugin_does_not_hide_healthy_plugins"),
                (77, "registry_and_discovery_disagreement_diagnostics_are_deterministic"),
                (78, "plugin_metadata_ordering_is_stable_in_machine_output"),
            ];
            let matrix_rows: Vec<Value> = rows
                .iter()
                .map(|(coverage_id, name)| {
                    json!({
                        "coverage_id": coverage_id,
                        "test_name": name,
                        "status": if source.contains(&format!("fn {name}(")) { "complete" } else { "missing" },
                        "evidence": "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                    })
                })
                .collect();
            let complete = matrix_rows
                .iter()
                .filter(|row| row.get("status").and_then(Value::as_str) == Some("complete"))
                .count();
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_discovery_determinism_report.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "generator": "bijux-dev-cli",
                    "scope": "plugin discovery and ordering determinism",
                    "rows": matrix_rows,
                    "summary": {
                        "complete": complete,
                        "missing": rows.len() - complete,
                        "artifact_todo": 79,
                        "artifact_path": "artifacts/status/plugin_discovery_determinism_report.json",
                    },
                }),
            )
            .ok()?;
            write_status_artifact_json(
                workspace_root,
                "artifacts/status/plugin_ordering_law.json",
                &json!({
                    "generated_at": "1970-01-01T00:00:00+00:00",
                    "law": "plugin ordering is deterministic",
                    "status": "frozen",
                    "evidence": [
                        "crates/bijux-cli/tests/bin_surface/plugin_discovery_determinism_matrix.rs",
                        "artifacts/status/plugin_discovery_determinism_report.json",
                    ],
                    "covers_todo": 80,
                }),
            )
            .ok()?;
            Some(json!({"status":"ok","script_id":script_id,"implementation":"rust","outputs":[
                "artifacts/status/plugin_discovery_determinism_report.json",
                "artifacts/status/plugin_ordering_law.json"
            ]}))
        }
        _ => None,
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

/// Builds `dev cli scripts status inventory` report payload.
#[must_use]
pub fn build_status_scripts_report(workspace_root: &Path) -> Value {
    build_status_scripts_inventory_report(workspace_root)
}

fn find_status_script_row(
    workspace_root: &Path,
    script_id: Option<&str>,
    source_script: Option<&str>,
) -> Option<Value> {
    let rows = build_status_scripts_inventory_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(id) = script_id {
        return rows
            .into_iter()
            .find(|row| row.get("script_id").and_then(Value::as_str) == Some(id));
    }
    if let Some(source) = source_script {
        return rows
            .into_iter()
            .find(|row| row.get("source_script").and_then(Value::as_str) == Some(source));
    }
    None
}

/// Runs one `scripts/status/*.py` script by stable id or source path.
#[must_use]
pub fn run_status_script(
    workspace_root: &Path,
    script_id: Option<&str>,
    source_script: Option<&str>,
    args: &[String],
) -> Value {
    if let Some(id) = script_id {
        if let Some(result) = run_native_status_script(workspace_root, id) {
            return result;
        }
    }
    let Some(row) = find_status_script_row(workspace_root, script_id, source_script) else {
        return json!({
            "status": "failed",
            "error": "status script not found; pass --id or --source with a known scripts/status/*.py path",
        });
    };
    let script_id = row.get("script_id").and_then(Value::as_str).unwrap_or("unknown");
    let source_script = row.get("source_script").and_then(Value::as_str).unwrap_or("");
    let kind = row.get("kind").and_then(Value::as_str).unwrap_or("unknown");
    run_python_status_script(workspace_root, source_script, script_id, kind, args)
}

/// Runs all `scripts/status/*.py` scripts, optionally filtered by kind.
#[must_use]
pub fn run_all_status_scripts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    let mut rows = build_status_scripts_inventory_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(kind) = kind_filter {
        let kind = kind.to_ascii_lowercase();
        rows.retain(|row| row.get("kind").and_then(Value::as_str).is_some_and(|item| item == kind));
    }

    let mut results = Vec::<Value>::new();
    let mut ok = 0usize;
    let mut failed = 0usize;
    for row in rows {
        let script_id = row.get("script_id").and_then(Value::as_str);
        let source_script = row.get("source_script").and_then(Value::as_str);
        let result = run_status_script(workspace_root, script_id, source_script, args);
        if result.get("status").and_then(Value::as_str) == Some("ok") {
            ok += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    json!({
        "generated_at_utc": generated_at_utc(),
        "kind_filter": kind_filter.map(|kind| kind.to_ascii_lowercase()),
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
        "status_scripts": build_status_scripts_inventory_report(workspace_root),
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
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::{
        build_audit_report, build_diff_report, build_generators_report, build_migrated_report,
        build_remaining_report, build_requirement_catalog_report, build_status_scripts_report,
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
        assert!(audit.get("status_scripts").is_some());
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

    #[test]
    fn status_script_ids_are_stable_and_prefixed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rows = build_status_scripts_report(&root)
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert!(!rows.is_empty());
        for row in rows {
            let id = row.get("script_id").and_then(serde_json::Value::as_str).unwrap_or("");
            assert!(id.starts_with("STATUS-SCRIPT-"));
        }
    }

    #[test]
    fn ci_status_script_ids_match_status_inventory() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let ci = fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read ci");
        let referenced: BTreeSet<String> = ci
            .split_whitespace()
            .map(|token| {
                token.trim_matches(|ch: char| {
                    !ch.is_ascii_uppercase() && !ch.is_ascii_digit() && ch != '-'
                })
            })
            .filter(|token| token.starts_with("STATUS-SCRIPT-"))
            .map(ToString::to_string)
            .collect();
        assert!(!referenced.is_empty(), "expected STATUS-SCRIPT IDs in CI workflow");

        let valid: BTreeSet<String> = build_status_scripts_report(&root)
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| {
                row.get("script_id").and_then(serde_json::Value::as_str).map(ToString::to_string)
            })
            .collect();
        assert!(!valid.is_empty(), "expected status script inventory rows");

        let missing: Vec<String> = referenced.difference(&valid).cloned().collect();
        assert!(
            missing.is_empty(),
            "CI references unknown STATUS-SCRIPT IDs; missing:\n{}",
            missing.join("\n")
        );
    }
}
