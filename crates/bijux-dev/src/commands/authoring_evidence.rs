use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::repo_root;

#[derive(Debug, Deserialize)]
struct AuthoringMetadata {
    version: String,
    owner: String,
    assets: BTreeMap<String, AuthoringAsset>,
}

#[derive(Debug, Deserialize)]
struct AuthoringAsset {
    group: String,
    authoring_mode: String,
    expected_validation: String,
    expected_lowering: String,
    command_surfaces: Vec<String>,
    consumers: Vec<String>,
    #[serde(default)]
    expected_rule_ids: Vec<String>,
}

fn load_authoring_metadata(root: &Path) -> Result<AuthoringMetadata, String> {
    let payload = fs::read_to_string(root.join("evidence/authoring/metadata.json"))
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&payload).map_err(|err| err.to_string())
}

fn is_human_readable_json(payload: &str) -> bool {
    let lines: Vec<&str> = payload.lines().collect();
    if lines.len() < 4 {
        return false;
    }
    if lines.iter().any(|line| line.len() > 160) {
        return false;
    }
    lines.iter().any(|line| line.starts_with("  \""))
}

fn has_speculative_keywords(payload: &str) -> bool {
    let lowered = payload.to_ascii_lowercase();
    [
        "\"distributed_controller\"",
        "\"federation\"",
        "\"enterprise_scheduler\"",
        "\"ha_scheduler\"",
        "\"future_only\"",
        "\"not_implemented\"",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

fn collect_doc_references(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut refs = BTreeSet::new();
    let docs =
        ["docs/spec/AUTHORING_UX_CONTRACT.md", "docs/bijux-dag/interfaces/authoring-guide.md"];
    for rel in docs {
        let text = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
        for token in text.split_whitespace() {
            let cleaned = token
                .trim_matches(|c: char| {
                    c == '`' || c == ',' || c == '.' || c == ')' || c == '(' || c == ';'
                })
                .to_string();
            if cleaned.starts_with("evidence/authoring/") && cleaned.ends_with(".json") {
                refs.insert(cleaned);
            }
        }
    }
    Ok(refs)
}

pub(super) fn run_validate_all_authoring() -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_authoring_metadata(&root)?;
    if metadata.version.trim().is_empty() {
        return Err("authoring metadata version must be non-empty".to_string());
    }
    if metadata.owner.trim().is_empty() {
        return Err("authoring metadata owner must be non-empty".to_string());
    }

    let mut groups_seen = BTreeSet::new();
    let docs_refs = collect_doc_references(&root)?;
    let mut missing_doc_refs = Vec::new();

    for (rel, asset) in &metadata.assets {
        groups_seen.insert(asset.group.clone());
        if !root.join(rel).exists() {
            return Err(format!("authoring asset missing on disk: {rel}"));
        }
        if !rel.starts_with("evidence/authoring/") {
            return Err(format!("authoring metadata contains non-authoring path: {rel}"));
        }
        if rel.starts_with("evidence/battle/") {
            return Err(format!("battle workflow cannot masquerade as authoring evidence: {rel}"));
        }
        if !matches!(asset.group.as_str(), "minimal" | "patterns" | "negative" | "examples") {
            return Err(format!("authoring asset has unsupported group `{}`: {rel}", asset.group));
        }
        if !matches!(asset.authoring_mode.as_str(), "normative" | "illustrative") {
            return Err(format!(
                "authoring asset has unsupported authoring_mode `{}`: {rel}",
                asset.authoring_mode
            ));
        }
        if !matches!(asset.expected_validation.as_str(), "pass" | "fail") {
            return Err(format!(
                "authoring asset has unsupported expected_validation `{}`: {rel}",
                asset.expected_validation
            ));
        }
        if !matches!(asset.expected_lowering.as_str(), "required" | "optional" | "none") {
            return Err(format!(
                "authoring asset has unsupported expected_lowering `{}`: {rel}",
                asset.expected_lowering
            ));
        }
        if asset.command_surfaces.is_empty() {
            return Err(format!("authoring asset has no command_surfaces: {rel}"));
        }
        if asset.consumers.is_empty() {
            return Err(format!("authoring asset has no consumers: {rel}"));
        }

        let payload = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
        if !is_human_readable_json(&payload) {
            return Err(format!("authoring asset is not human-readable JSON: {rel}"));
        }
        if has_speculative_keywords(&payload) {
            return Err(format!(
                "authoring asset contains speculative unsupported features: {rel}"
            ));
        }

        if asset.group == "negative" {
            if asset.expected_rule_ids.is_empty() {
                return Err(format!(
                    "negative authoring asset must declare expected_rule_ids: {rel}"
                ));
            }
            for rule in &asset.expected_rule_ids {
                if !rule.starts_with("DAG-VAL-") || rule.len() < "DAG-VAL-000".len() {
                    return Err(format!(
                        "negative authoring asset has invalid rule ID `{rule}`: {rel}"
                    ));
                }
            }
        }

        let parsed = bijux_dag_core::parse_graph_strict(&payload);
        match (asset.expected_validation.as_str(), parsed) {
            ("pass", Ok(graph)) => {
                let has_errors = graph
                    .validate_with_warnings()
                    .iter()
                    .any(|d| d.severity == bijux_dag_core::Severity::Error);
                if has_errors {
                    return Err(format!(
                        "authoring asset expected pass but validation failed: {rel}"
                    ));
                }
            }
            ("fail", Ok(graph)) => {
                let has_errors = graph
                    .validate_with_warnings()
                    .iter()
                    .any(|d| d.severity == bijux_dag_core::Severity::Error);
                if !has_errors {
                    return Err(format!(
                        "negative authoring asset expected fail but passed: {rel}"
                    ));
                }
            }
            ("pass", Err(err)) => {
                return Err(format!(
                    "authoring asset expected pass but failed parse: {rel}: {err}"
                ));
            }
            ("fail", Err(_)) => {}
            _ => unreachable!(),
        }

        if asset.group != "examples" && !docs_refs.contains(rel) {
            missing_doc_refs.push(rel.clone());
        }
    }

    for required in ["minimal", "patterns", "negative", "examples"] {
        if !groups_seen.contains(required) {
            return Err(format!("authoring metadata does not cover required group `{required}`"));
        }
    }
    if !missing_doc_refs.is_empty() {
        return Err(format!(
            "authoring docs do not reference required assets: {}",
            missing_doc_refs.join(", ")
        ));
    }
    Ok(())
}

pub(super) fn run_show_effective_all_authoring() -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_authoring_metadata(&root)?;
    let mut assets: Vec<_> = metadata
        .assets
        .iter()
        .map(|(path, asset)| {
            json!({
                "path": path,
                "group": asset.group,
                "authoring_mode": asset.authoring_mode,
                "expected_validation": asset.expected_validation,
                "expected_lowering": asset.expected_lowering,
                "command_surfaces": asset.command_surfaces,
                "consumers": asset.consumers,
                "expected_rule_ids": asset.expected_rule_ids
            })
        })
        .collect();
    assets.sort_by(|a, b| a["path"].as_str().unwrap_or("").cmp(b["path"].as_str().unwrap_or("")));
    let payload = json!({
        "version": metadata.version,
        "owner": metadata.owner,
        "asset_count": assets.len(),
        "assets": assets
    });
    println!("{}", serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?);
    Ok(())
}

pub(super) fn run_authoring_coverage_report(out: &Path, unused_out: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let metadata = load_authoring_metadata(&root)?;
    let refs = collect_doc_references(&root)?;
    let mut all_assets: Vec<String> = metadata.assets.keys().cloned().collect();
    all_assets.sort();

    let mut coverage_rows = Vec::new();
    let mut unused = Vec::new();
    for path in &all_assets {
        let referenced = refs.contains(path);
        let commands =
            metadata.assets.get(path).map(|a| a.command_surfaces.join(", ")).unwrap_or_default();
        coverage_rows.push((path.clone(), referenced, commands));
        if !referenced {
            unused.push(path.clone());
        }
    }

    let mut coverage = String::new();
    coverage.push_str("# Authoring Coverage by Docs and Commands\n\n");
    coverage.push_str("| Asset | Referenced in docs | Command surfaces |\n");
    coverage.push_str("| --- | --- | --- |\n");
    for (path, referenced, commands) in &coverage_rows {
        coverage.push_str(&format!(
            "| `{}` | {} | `{}` |\n",
            path,
            if *referenced { "yes" } else { "no" },
            commands
        ));
    }
    let mut unused_report = String::new();
    unused_report.push_str("# Unused Authoring Assets\n\n");
    if unused.is_empty() {
        unused_report.push_str("All authoring assets are referenced by authoring docs.\n");
    } else {
        for path in &unused {
            unused_report.push_str(&format!("- `{path}`\n"));
        }
    }

    let coverage_path = if out.is_absolute() { PathBuf::from(out) } else { root.join(out) };
    if let Some(parent) = coverage_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&coverage_path, coverage).map_err(|err| err.to_string())?;

    let unused_path =
        if unused_out.is_absolute() { PathBuf::from(unused_out) } else { root.join(unused_out) };
    if let Some(parent) = unused_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(&unused_path, unused_report).map_err(|err| err.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{has_speculative_keywords, is_human_readable_json};

    #[test]
    fn readability_requires_multiline_indented_json() {
        assert!(is_human_readable_json(
            "{\n  \"name\": \"demo\",\n  \"nodes\": [],\n  \"edges\": []\n}"
        ));
        assert!(!is_human_readable_json("{\"name\":\"demo\"}"));
    }

    #[test]
    fn speculative_keyword_detection_is_case_insensitive() {
        assert!(has_speculative_keywords("{\"mode\":\"HA_SCHEDULER\"}"));
        assert!(!has_speculative_keywords("{\"mode\":\"local\"}"));
    }
}
