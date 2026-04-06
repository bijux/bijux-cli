//! Repository health and drift reports for maintainer control-plane workflows.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{collect_files_recursive, relative_to_root};
fn stale_generated_artifacts(root: &Path) -> Vec<String> {
    let status = root.join("artifacts/status");
    if !status.exists() {
        return vec![];
    }
    collect_files_recursive(&status)
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
        .map(|path| relative_to_root(&path, root))
        .collect()
}

fn stale_snapshots(root: &Path) -> Vec<String> {
    collect_files_recursive(root)
        .into_iter()
        .filter(|path| {
            let rel_path = relative_to_root(path, root);
            rel_path.contains("/tests/data/golden/cli_surface/")
                && path.file_name().and_then(|name| name.to_str()).is_some_and(|name| {
                    name.contains(".old.")
                        || Path::new(name)
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("bak"))
                })
        })
        .map(|path| relative_to_root(&path, root))
        .collect()
}

fn stale_inventories(root: &Path) -> Vec<String> {
    let status = root.join("artifacts/status");
    if !status.exists() {
        return vec![];
    }
    collect_files_recursive(&status)
        .into_iter()
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("inventory") && name.contains("stale"))
        })
        .map(|path| relative_to_root(&path, root))
        .collect()
}

fn dead_maintenance_references(root: &Path) -> Vec<String> {
    [
        "configs/allowlists",
        "configs/allowlists/automation.toml",
        "configs/allowlists/public_api.toml",
        ".github/maintenance_additions_allowlist.txt",
        ".github/root_maintenance_additions_allowlist.txt",
        ".github/public_api_allowlist.txt",
    ]
    .into_iter()
    .filter(|entry| root.join(entry).exists())
    .map(ToString::to_string)
    .collect()
}

fn dead_docs_references(root: &Path) -> Vec<String> {
    let docs = root.join("docs");
    if !docs.exists() {
        return vec![];
    }
    let mut dead = Vec::new();
    for path in collect_files_recursive(&docs)
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
    {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for token in text.split_whitespace() {
            for reference in docs_refs_in_token(token) {
                if !root.join(&reference).exists() {
                    dead.push(format!("{} -> {}", relative_to_root(&path, root), reference));
                }
            }
        }
    }
    dead.sort();
    dead.dedup();
    dead
}

fn dead_evidence_references(root: &Path) -> Vec<String> {
    let evidence = root.join("artifacts/status/maintainer_evidence_audit_report.json");
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
    let _ = root;
    Vec::new()
}

fn docs_refs_in_token(token: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (idx, _) in token.match_indices("docs/") {
        let slice = &token[idx..];
        let stop = slice.find([')', ']', '>', '"', '\'', ',', ';', '`']).unwrap_or(slice.len());
        let candidate = slice[..stop].trim_matches(['(', '[', '<']);
        let canonical =
            candidate.split(['#', '?']).next().map(str::trim).unwrap_or_default().to_string();
        if canonical.starts_with("docs/") && !canonical.is_empty() {
            out.push(canonical);
        }
    }
    out
}

/// `bijux-dev-cli repo generated`
#[must_use]
pub fn build_generated_report(workspace_root: &Path) -> Value {
    let stale_generated = stale_generated_artifacts(workspace_root);
    let orphan_generated_outputs: Vec<String> =
        collect_files_recursive(&workspace_root.join("artifacts/status"))
            .into_iter()
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .filter(|path| {
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
                name.starts_with("orphan_")
            })
            .map(|path| relative_to_root(&path, workspace_root))
            .collect();
    json!({
        "stale_generated_artifacts": stale_generated,
        "orphan_generated_outputs": orphan_generated_outputs,
    })
}

/// `bijux-dev-cli repo inventories`
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

/// `bijux-dev-cli repo stale`
#[must_use]
pub fn build_stale_report(workspace_root: &Path) -> Value {
    json!({
        "stale_generated_artifacts": stale_generated_artifacts(workspace_root),
        "stale_snapshots": stale_snapshots(workspace_root),
        "stale_inventories": stale_inventories(workspace_root),
    })
}

/// `bijux-dev-cli repo drift`
#[must_use]
pub fn build_drift_report(workspace_root: &Path) -> Value {
    let dead_maintenance = dead_maintenance_references(workspace_root);
    let dead_docs = dead_docs_references(workspace_root);
    let dead_evidence = dead_evidence_references(workspace_root);
    let dead_commands = dead_command_references(workspace_root);
    json!({
        "status": if dead_maintenance.is_empty() && dead_docs.is_empty() && dead_evidence.is_empty() && dead_commands.is_empty() { "clean" } else { "drift" },
        "dead_maintenance_references": dead_maintenance,
        "dead_docs_references": dead_docs,
        "dead_evidence_references": dead_evidence,
        "dead_command_references": dead_commands,
    })
}

/// `bijux-dev-cli repo health`
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

    use super::{build_drift_report, build_generated_report, build_health_report};

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

    #[test]
    fn repo_drift_flags_forbidden_legacy_exception_paths() {
        let root = temp_root("forbidden-legacy-files");
        fs::create_dir_all(root.join("configs/allowlists")).expect("mkdir allowlists");
        fs::write(root.join("configs/allowlists/automation.toml"), "version = 1\n")
            .expect("write allowlist");

        let drift = build_drift_report(&root);
        let dead = drift["dead_maintenance_references"].as_array().expect("dead refs");
        assert_eq!(dead.len(), 2);
        assert_eq!(dead[0], "configs/allowlists");
        assert_eq!(dead[1], "configs/allowlists/automation.toml");
    }

    #[test]
    fn repo_drift_reports_all_broken_docs_references_per_file() {
        let root = temp_root("docs-refs");
        fs::create_dir_all(root.join("docs/02-getting-started")).expect("mkdir docs");
        fs::write(
            root.join("docs/02-getting-started/index.md"),
            "[one](docs/missing/one.md) and [two](docs/missing/two.md)\n",
        )
        .expect("write docs");

        let drift = build_drift_report(&root);
        let dead = drift["dead_docs_references"].as_array().expect("dead docs");
        assert_eq!(dead.len(), 2, "expected both broken references to be reported");
    }

    #[test]
    fn repo_drift_ignores_malformed_disposable_command_inventory_artifacts() {
        let root = temp_root("command-inventory");
        fs::write(
            root.join("artifacts/status/maintainer_control_plane_commands.json"),
            r#"{
                "commands": [
                    {"command":"bijux-dev-cli status","owner":"wrong-owner"},
                    {"command":"bijux-dev-cli unknown"},
                    {"broken": true}
                ]
            }"#,
        )
        .expect("write command inventory");

        let drift = build_drift_report(&root);
        let dead = drift["dead_command_references"].as_array().expect("dead commands");
        assert!(dead.is_empty(), "disposable command inventory artifacts must be ignored");
    }

    #[test]
    fn repo_drift_ignores_missing_disposable_command_inventory_artifacts() {
        let root = temp_root("missing-command-inventory");
        let drift = build_drift_report(&root);
        let dead = drift["dead_command_references"].as_array().expect("dead commands");
        assert!(dead.is_empty(), "missing disposable artifacts must not count as drift");
    }
}
