use crate::commands::repo_root;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn run_contract_test_links_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
    let mut violations = Vec::new();

    for file in contracts {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !content.contains("## Related tests") {
            let rel = file.strip_prefix(&root).map_err(|err| err.to_string())?;
            violations.push(format!("{} missing '## Related tests' section", rel.display()));
            continue;
        }
        let mut test_link_count = 0usize;
        for line in content.lines() {
            if line.contains("tests/") && line.contains('`') {
                test_link_count += 1;
            }
        }
        if test_link_count == 0 {
            let rel = file.strip_prefix(&root).map_err(|err| err.to_string())?;
            violations.push(format!("{} has no linked test paths", rel.display()));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_contract_schema_owner_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    for contract_root in [root.join("docs/spec"), root.join("crates"), root.join("evidence")] {
        collect_contract_files(&contract_root, &mut contracts)?;
    }
    let mut contract_blob = String::new();
    for file in contracts {
        contract_blob.push_str(&fs::read_to_string(file).map_err(|err| err.to_string())?);
        contract_blob.push('\n');
    }

    let mut missing = Vec::new();
    for entry in fs::read_dir(root.join("configs/dag/schema")).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !contract_blob.contains(&rel) {
            missing.push(rel);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("schemas missing owning contract links: {}", missing.join(", ")))
    }
}

pub(super) fn run_contract_command_ownership_guard() -> Result<(), String> {
    let root = repo_root()?;
    let dag_surface = load_json_contract::<DagReleaseTruthTableContract>(
        &root.join("contracts/foundation/dag_release_truth_table.v1.json"),
    )?;
    let dag_surface_doc = fs::read_to_string(root.join("docs/bijux-dag/interfaces/cli-surface.md"))
        .map_err(|err| err.to_string())?;
    let maintainer_surface = load_json_contract::<MaintainerCommandSurfaceContract>(
        &root.join("contracts/foundation/maintainer_command_surface.v1.json"),
    )?;
    let maintainer_surface_doc =
        fs::read_to_string(root.join("docs/bijux-dev/operations/command-surface.md"))
            .map_err(|err| err.to_string())?;

    let mut violations = Vec::new();
    validate_command_surface_section(
        "docs/bijux-dag/interfaces/cli-surface.md",
        "## Visible Root Surface",
        &dag_surface_doc,
        &dag_surface.stable_operator_surface.root_commands,
        &mut violations,
    )?;
    validate_command_surface_section(
        "docs/bijux-dag/interfaces/cli-surface.md",
        "## Hidden Experimental Routes",
        &dag_surface_doc,
        &dag_surface.experimental_operator_surface.root_commands,
        &mut violations,
    )?;
    let dag_hidden_commands = dag_surface
        .simulated_surface
        .root_commands
        .iter()
        .chain(dag_surface.internal_surface.root_commands.iter())
        .cloned()
        .collect::<Vec<_>>();
    validate_command_surface_section(
        "docs/bijux-dag/interfaces/cli-surface.md",
        "## Hidden Simulation And Maintainer Namespaces",
        &dag_surface_doc,
        &dag_hidden_commands,
        &mut violations,
    )?;
    validate_command_surface_section(
        "docs/bijux-dev/operations/command-surface.md",
        "## `bijux-dev-dag` Root Surface",
        &maintainer_surface_doc,
        &maintainer_surface.visible_root_commands,
        &mut violations,
    )?;

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_contract_versioning_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
    let mut violations = Vec::new();
    for file in contracts {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !content.contains("## Versioning and change policy") {
            let rel = file
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            violations.push(format!("{rel} missing versioning policy section"));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_contract_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let mut missing = Vec::new();
    let mut stale = Vec::new();

    for entry in fs::read_dir(root.join("crates")).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if !path.is_dir() || !path.join("Cargo.toml").exists() {
            continue;
        }
        let crate_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
        if !path.join("docs/CONTRACTS.md").exists() {
            missing.push(format!("crate contract missing: {crate_name}"));
        }
    }

    let mut spec_contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut spec_contracts)?;
    if spec_contracts.is_empty() {
        missing.push("spec contract missing: docs/spec/*CONTRACT*.md".to_string());
    }
    for path in spec_contracts {
        let file_name = path.file_name().and_then(|x| x.to_str()).unwrap_or_default();
        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        if !content.contains("## Scope") {
            stale.push(format!("{file_name} missing scope section"));
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "missing": missing,
            "orphaned": [],
            "stale": stale
        }))
        .map_err(|err| err.to_string())?
    );

    if missing.is_empty() && stale.is_empty() {
        Ok(())
    } else {
        Err("contract coverage report found gaps".to_string())
    }
}

pub(super) fn run_error_code_registry_report() -> Result<(), String> {
    let root = repo_root()?;
    let registry = load_error_code_registry(&root)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": registry.version,
            "categories": registry.categories,
            "codes": registry.codes,
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

pub(super) fn run_error_code_docs_tests_guard() -> Result<(), String> {
    let root = repo_root()?;
    let registry = load_error_code_registry(&root)?;
    let docs_error_ref = fs::read_to_string(root.join("docs/bijux-dag/interfaces/error-codes.md"))
        .map_err(|err| err.to_string())?;
    let docs_error_contract = fs::read_to_string(root.join("docs/spec/ERROR_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let tests = [
        root.join("crates/bijux-dag-app/tests/error_output_contract.rs"),
        root.join("crates/bijux-dag-app/tests/error_exit_contract.rs"),
    ];

    let mut violations = Vec::new();
    if !docs_error_contract.contains("Public error code additions require docs plus test coverage")
    {
        violations
            .push("docs/spec/ERROR_CONTRACT.md missing public code governance rule".to_string());
    }
    for code in &registry.codes {
        if !docs_error_ref.contains(&code.category) {
            violations.push(format!(
                "docs/bijux-dag/interfaces/error-codes.md missing category {} for {}",
                code.category, code.code
            ));
        }
        if !docs_error_ref.contains(&code.code) {
            violations.push(format!(
                "docs/bijux-dag/interfaces/error-codes.md missing public code {}",
                code.code
            ));
        }
    }

    for test in tests {
        if !test.exists() {
            violations.push(format!(
                "missing required error contract test file: {}",
                test.strip_prefix(&root).map_err(|err| err.to_string())?.display()
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn collect_contract_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_contract_files(&path, out)?;
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("CONTRACT.md"))
        {
            out.push(path);
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ErrorCodeRegistry {
    version: u64,
    categories: Vec<String>,
    codes: Vec<ErrorCodeEntry>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct ErrorCodeEntry {
    code: String,
    category: String,
    owner: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct DagReleaseTruthTableContract {
    stable_operator_surface: CommandSurfaceContract,
    experimental_operator_surface: CommandSurfaceContract,
    simulated_surface: CommandSurfaceContract,
    internal_surface: CommandSurfaceContract,
}

#[derive(Debug, Deserialize)]
struct CommandSurfaceContract {
    root_commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MaintainerCommandSurfaceContract {
    visible_root_commands: Vec<String>,
}

fn load_error_code_registry(root: &Path) -> Result<ErrorCodeRegistry, String> {
    let payload = fs::read_to_string(root.join("configs/dag/policy/error_codes.json"))
        .map_err(|err| err.to_string())?;
    let registry: ErrorCodeRegistry =
        serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut seen_codes = BTreeSet::new();
    let mut seen_categories = BTreeSet::new();
    for category in &registry.categories {
        seen_categories.insert(category.clone());
    }
    for entry in &registry.codes {
        if !seen_categories.contains(&entry.category) {
            return Err(format!(
                "error code {} references unknown category {}",
                entry.code, entry.category
            ));
        }
        if entry.owner.trim().is_empty() || entry.description.trim().is_empty() {
            return Err(format!("error code {} has empty owner or description", entry.code));
        }
        if !seen_codes.insert(entry.code.clone()) {
            return Err(format!("duplicate error code {}", entry.code));
        }
    }
    Ok(registry)
}

fn load_json_contract<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let payload = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str(&payload)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))
}

fn validate_command_surface_section(
    document_path: &str,
    heading: &str,
    content: &str,
    expected_commands: &[String],
    violations: &mut Vec<String>,
) -> Result<(), String> {
    let section = markdown_section(content, heading)
        .ok_or_else(|| format!("{document_path} missing section {heading}"))?;
    let expected = expected_commands.iter().cloned().collect::<BTreeSet<_>>();
    let counts = count_documented_commands(section, &expected);

    for command in expected_commands {
        let count = counts.get(command).copied().unwrap_or(0);
        if count != 1 {
            violations.push(format!(
                "{document_path} section {heading} documents `{command}` {count} times"
            ));
        }
    }

    Ok(())
}

fn markdown_section<'a>(content: &'a str, heading: &str) -> Option<&'a str> {
    let start = content.find(heading)?;
    let remainder = &content[start + heading.len()..];
    let end = remainder.find("\n## ").unwrap_or(remainder.len());
    Some(remainder[..end].trim())
}

fn count_documented_commands(
    section: &str,
    expected: &BTreeSet<String>,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();

    for token in extract_backtick_tokens(section) {
        if let Some(command) = normalize_documented_command(&token) {
            if expected.contains(&command) {
                *counts.entry(command).or_default() += 1;
            }
        }
    }

    counts
}

fn extract_backtick_tokens(section: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_token = false;

    for ch in section.chars() {
        if ch == '`' {
            if in_token {
                tokens.push(current.trim().to_string());
                current.clear();
            }
            in_token = !in_token;
            continue;
        }
        if in_token {
            current.push(ch);
        }
    }

    tokens
}

fn normalize_documented_command(token: &str) -> Option<String> {
    let trimmed = token.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_binary = trimmed
        .strip_prefix("bijux-dag ")
        .or_else(|| trimmed.strip_prefix("bijux-dev-dag "))
        .or_else(|| trimmed.strip_prefix("dag "))
        .unwrap_or(trimmed);
    let root = without_binary.split_whitespace().next()?.trim_end_matches("...");

    if root.is_empty() {
        None
    } else {
        Some(root.to_string())
    }
}
