//! Repository health and drift reports for maintainer control-plane workflows.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == ".git" || name == "target")
            {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files
}

fn rel(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn stale_generated_artifacts(root: &Path) -> Vec<String> {
    let status = root.join("artifacts/status");
    if !status.exists() {
        return vec![];
    }
    collect_files(&status)
        .into_iter()
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext == "tmp")
                || path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
                    name.contains("stale")
                        || Path::new(name)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("bak"))
                })
        })
        .map(|path| rel(&path, root))
        .collect()
}

fn stale_snapshots(root: &Path) -> Vec<String> {
    collect_files(root)
        .into_iter()
        .filter(|path| {
            let rel_path = rel(path, root);
            rel_path.contains("/tests/data/golden/cli_surface/")
                && path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
                    name.contains(".old.")
                        || Path::new(name)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("bak"))
                })
        })
        .map(|path| rel(&path, root))
        .collect()
}

fn stale_inventories(root: &Path) -> Vec<String> {
    let status = root.join("artifacts/status");
    if !status.exists() {
        return vec![];
    }
    collect_files(&status)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("inventory") && name.contains("stale"))
        })
        .map(|path| rel(&path, root))
        .collect()
}

fn dead_script_references(root: &Path) -> Vec<String> {
    let allowlist = root.join(".github/script_additions_allowlist.txt");
    let content = fs::read_to_string(&allowlist).unwrap_or_default();
    content
        .lines()
        .filter_map(|line| line.split('#').next())
        .map(str::trim)
        .filter(|line| line.starts_with("scripts/") && !line.is_empty())
        .filter(|script| !root.join(script).exists())
        .map(ToString::to_string)
        .collect()
}

fn dead_docs_references(root: &Path) -> Vec<String> {
    let docs = root.join("docs");
    if !docs.exists() {
        return vec![];
    }
    collect_files(&docs)
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            for token in text.split_whitespace() {
                if token.starts_with("docs/")
                    && !root.join(token.trim_matches(|c| c == ')' || c == '(')).exists()
                {
                    return Some(format!("{} -> {}", rel(&path, root), token));
                }
            }
            None
        })
        .collect()
}

fn dead_evidence_references(root: &Path) -> Vec<String> {
    let evidence = root.join("artifacts/status/dev_cli_evidence_audit_report.json");
    let payload = fs::read_to_string(evidence)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    payload
        .get("missing_artifact_links")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(ToString::to_string))
        .collect()
}

fn dead_command_references(root: &Path) -> Vec<String> {
    let commands = root.join("artifacts/status/dev_cli_inventory.json");
    let payload = fs::read_to_string(commands)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .unwrap_or_else(|| json!({}));
    let mut dead = Vec::new();
    if payload.get("commands").is_none() {
        dead.push("dev_cli_inventory.json missing commands key".to_string());
    }
    dead
}

/// `dev cli repo generated`
#[must_use]
pub fn build_generated_report(workspace_root: &Path) -> Value {
    let stale_generated = stale_generated_artifacts(workspace_root);
    let orphan_generated_outputs: Vec<String> =
        collect_files(&workspace_root.join("artifacts/status"))
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .filter(|path| {
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
                name.starts_with("orphan_")
            })
            .map(|path| rel(&path, workspace_root))
            .collect();
    json!({
        "stale_generated_artifacts": stale_generated,
        "orphan_generated_outputs": orphan_generated_outputs,
    })
}

/// `dev cli repo inventories`
#[must_use]
pub fn build_inventories_report(workspace_root: &Path) -> Value {
    json!({
        "stale_inventories": stale_inventories(workspace_root),
        "stale_package_metadata": if workspace_root.join("artifacts/status/package_metadata_stale.json").exists() {
            json!(["artifacts/status/package_metadata_stale.json"])
        } else {
            json!([])
        },
    })
}

/// `dev cli repo stale`
#[must_use]
pub fn build_stale_report(workspace_root: &Path) -> Value {
    json!({
        "stale_generated_artifacts": stale_generated_artifacts(workspace_root),
        "stale_snapshots": stale_snapshots(workspace_root),
        "stale_inventories": stale_inventories(workspace_root),
    })
}

/// `dev cli repo drift`
#[must_use]
pub fn build_drift_report(workspace_root: &Path) -> Value {
    let dead_scripts = dead_script_references(workspace_root);
    let dead_docs = dead_docs_references(workspace_root);
    let dead_evidence = dead_evidence_references(workspace_root);
    let dead_commands = dead_command_references(workspace_root);
    json!({
        "status": if dead_scripts.is_empty() && dead_docs.is_empty() && dead_evidence.is_empty() && dead_commands.is_empty() { "clean" } else { "drift" },
        "dead_scripts_references": dead_scripts,
        "dead_docs_references": dead_docs,
        "dead_evidence_references": dead_evidence,
        "dead_command_references": dead_commands,
    })
}

/// `dev cli repo health`
#[must_use]
pub fn build_health_report(workspace_root: &Path) -> Value {
    let generated = build_generated_report(workspace_root);
    let inventories = build_inventories_report(workspace_root);
    let stale = build_stale_report(workspace_root);
    let drift = build_drift_report(workspace_root);
    let stale_crate_api_docs =
        if workspace_root.join("artifacts/status/stale_crate_api_docs.json").exists() {
            json!(["artifacts/status/stale_crate_api_docs.json"])
        } else {
            json!([])
        };
    json!({
        "repo_health": {
            "generated": generated,
            "inventories": inventories,
            "stale": stale,
            "drift": drift,
            "stale_crate_api_docs": stale_crate_api_docs,
        },
        "status": if drift.get("status").and_then(Value::as_str) == Some("clean") { "healthy" } else { "degraded" },
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{build_generated_report, build_health_report};

    fn temp_root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bijux-repo-health-{name}-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
        ));
        fs::create_dir_all(root.join("artifacts/status")).expect("mkdir");
        root
    }

    #[test]
    fn repo_health_detects_injected_stale_artifacts() {
        let root = temp_root("stale");
        fs::write(root.join("artifacts/status/sample.stale.tmp"), "x").expect("write");
        let report = build_health_report(&root);
        let stale = report["repo_health"]["stale"]["stale_generated_artifacts"]
            .as_array()
            .expect("stale list");
        assert!(!stale.is_empty());
    }

    #[test]
    fn repo_health_detects_orphan_generated_outputs() {
        let root = temp_root("orphan");
        fs::write(root.join("artifacts/status/orphan_generated_output.json"), "{}").expect("write");
        let generated = build_generated_report(&root);
        let orphan = generated["orphan_generated_outputs"].as_array().expect("orphan");
        assert!(!orphan.is_empty());
    }
}
