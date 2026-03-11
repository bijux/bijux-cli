use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

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

pub(crate) fn migrated_rows() -> &'static [(&'static str, &'static str, usize)] {
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

pub(crate) fn is_python_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("py"))
}

pub(crate) fn status_generator_sources(workspace_root: &Path) -> Vec<String> {
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

pub(crate) fn status_generator_slug(script_path: &str) -> String {
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

pub(crate) fn status_generator_id(script_path: &str) -> String {
    format!("GEN-STATUS-{}", status_generator_slug(script_path))
}

pub(crate) fn extract_artifact_paths(source: &str) -> Vec<String> {
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

pub(crate) fn extract_required_test_names(source: &str) -> Vec<String> {
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
