//! Repository health and drift reports for maintainer control-plane workflows.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{collect_files_recursive, relative_to_root};
use crate::schema::command_registry::command_registry;

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
    let candidates = [
        "artifacts/status/dev_cli_ownership_report.json",
        "artifacts/status/maintainer_control_plane_commands.json",
        "artifacts/status/dev_cli_inventory.json",
    ];

    let mut selection_errors = Vec::new();
    let mut parsed_inventory: Option<(String, CommandInventory)> = None;
    for rel in candidates {
        let path = root.join(rel);
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) => {
                selection_errors.push(format!("{rel}: unreadable ({error})"));
                continue;
            }
        };
        let payload = match serde_json::from_str::<Value>(&text) {
            Ok(payload) => payload,
            Err(error) => {
                selection_errors.push(format!("{rel}: malformed json ({error})"));
                continue;
            }
        };
        match parse_command_inventory(&payload) {
            Ok(inventory) => {
                parsed_inventory = Some((rel.to_string(), inventory));
                break;
            }
            Err(error) => selection_errors.push(format!("{rel}: {error}")),
        }
    }

    let Some((source_path, inventory)) = parsed_inventory else {
        selection_errors.push(
            "no command inventory artifact with a parseable commands list was found".to_string(),
        );
        return selection_errors;
    };

    let expected: std::collections::BTreeSet<String> =
        command_registry().iter().map(|row| row.command.as_str().to_string()).collect();
    let actual: std::collections::BTreeSet<String> = inventory.commands.into_iter().collect();

    let missing_expected: Vec<String> =
        expected.difference(&actual).map(ToString::to_string).collect();
    let unknown: Vec<String> = actual.difference(&expected).map(ToString::to_string).collect();

    let mut dead = Vec::new();
    if !inventory.parse_warnings.is_empty() {
        dead.extend(inventory.parse_warnings);
    }
    if !inventory.duplicate_commands.is_empty() {
        dead.push(format!(
            "{source_path}: duplicate commands ({})",
            inventory.duplicate_commands.join(", ")
        ));
    }
    if !inventory.owner_mismatches.is_empty() {
        dead.push(format!(
            "{source_path}: owner mismatches ({})",
            inventory.owner_mismatches.join(", ")
        ));
    }
    if !missing_expected.is_empty() {
        dead.push(format!(
            "{source_path}: missing expected commands ({})",
            missing_expected.join(", ")
        ));
    }
    if !unknown.is_empty() {
        dead.push(format!("{source_path}: unknown commands ({})", unknown.join(", ")));
    }
    dead
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

#[derive(Debug)]
struct CommandInventory {
    commands: Vec<String>,
    duplicate_commands: Vec<String>,
    owner_mismatches: Vec<String>,
    parse_warnings: Vec<String>,
}

fn parse_command_inventory(payload: &Value) -> Result<CommandInventory, String> {
    let rows = payload.get("commands").and_then(Value::as_array).ok_or("missing commands key")?;
    let mut commands = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = std::collections::BTreeSet::new();
    let mut owner_mismatches = Vec::new();
    let mut parse_warnings = Vec::new();

    for (idx, row) in rows.iter().enumerate() {
        let command = if let Some(command) = row.as_str() {
            command.trim().to_string()
        } else if let Some(command) = row.get("command").and_then(Value::as_str) {
            if let Some(owner) = row.get("owner").and_then(Value::as_str) {
                if owner != "bijux-dev-cli" {
                    owner_mismatches.push(format!("{command}:{owner}"));
                }
            }
            command.trim().to_string()
        } else {
            parse_warnings.push(format!("commands[{idx}] is neither string nor object.command"));
            continue;
        };

        if command.is_empty() {
            parse_warnings.push(format!("commands[{idx}] command is empty"));
            continue;
        }
        if !seen.insert(command.clone()) {
            duplicates.insert(command.clone());
        }
        commands.push(command);
    }

    Ok(CommandInventory {
        commands,
        duplicate_commands: duplicates.into_iter().collect(),
        owner_mismatches,
        parse_warnings,
    })
}

/// `dev cli repo generated`
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
        fs::create_dir_all(root.join("docs/guides")).expect("mkdir docs");
        fs::write(
            root.join("docs/guides/index.md"),
            "[one](docs/missing/one.md) and [two](docs/missing/two.md)\n",
        )
        .expect("write docs");

        let drift = build_drift_report(&root);
        let dead = drift["dead_docs_references"].as_array().expect("dead docs");
        assert_eq!(dead.len(), 2, "expected both broken references to be reported");
    }

    #[test]
    fn repo_drift_validates_command_inventory_against_registry() {
        let root = temp_root("command-inventory");
        fs::write(
            root.join("artifacts/status/maintainer_control_plane_commands.json"),
            r#"{
                "commands": [
                    {"command":"dev cli status","owner":"wrong-owner"},
                    {"command":"dev cli unknown"},
                    {"broken": true}
                ]
            }"#,
        )
        .expect("write command inventory");

        let drift = build_drift_report(&root);
        let dead = drift["dead_command_references"].as_array().expect("dead commands");
        assert!(
            dead.iter().any(|item| item.as_str().is_some_and(|s| s.contains("owner mismatches"))),
            "owner mismatch must be reported"
        );
        assert!(
            dead.iter().any(|item| item.as_str().is_some_and(|s| s.contains("unknown commands"))),
            "unknown command must be reported"
        );
        assert!(
            dead.iter()
                .any(|item| item.as_str().is_some_and(|s| s.contains("missing expected commands"))),
            "missing expected commands must be reported"
        );
    }
}
