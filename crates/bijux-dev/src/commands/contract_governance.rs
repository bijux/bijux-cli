use crate::commands::repo_root;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeSet;
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
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
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
    let taxonomy = fs::read_to_string(root.join("docs/CLI_COMMAND_TAXONOMY.md"))
        .map_err(|err| err.to_string())?;
    let contract = fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md"))
        .map_err(|err| err.to_string())?;

    let mut commands = Vec::new();
    for line in taxonomy.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- `") || !trimmed.ends_with('`') {
            continue;
        }
        let value = trimmed.trim_start_matches("- `").trim_end_matches('`').to_string();
        if value.starts_with("migrate ") {
            if !commands.contains(&"migrate".to_string()) {
                commands.push("migrate".to_string());
            }
        } else {
            commands.push(value);
        }
    }

    let mut violations = Vec::new();
    for command in commands {
        let token = format!("`dag {command}`");
        let count = contract.matches(&token).count();
        if count != 1 {
            violations.push(format!(
                "command ownership token {} appears {} times in docs/spec/CLI_CONTRACT.md",
                token, count
            ));
        }
    }

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
    let mut orphaned = Vec::new();
    let mut stale = Vec::new();

    let crate_names = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];
    for crate_name in crate_names {
        if !root.join("crates").join(crate_name).join("CONTRACT.md").exists() {
            missing.push(format!("crate contract missing: {crate_name}"));
        }
    }

    let specs = [
        "CLI_CONTRACT.md",
        "RUN_DIR_CONTRACT.md",
        "CACHE_CONTRACT.md",
        "REPLAY_CONTRACT.md",
        "ERROR_CONTRACT.md",
        "TRACE_CONTRACT.md",
        "IMPORT_EXPORT_CONTRACT.md",
        "CONFIG_CONTRACT.md",
        "POLICY_CONTRACT.md",
        "SELECTOR_CONTRACT.md",
    ];
    for file in specs {
        let path = root.join("docs/spec").join(file);
        if !path.exists() {
            missing.push(format!("spec contract missing: docs/spec/{file}"));
        }
    }

    for entry in fs::read_dir(root.join("docs/spec")).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path
            .file_name()
            .and_then(|x| x.to_str())
            .is_some_and(|name| name.ends_with("CONTRACT.md"))
        {
            let file_name = path.file_name().and_then(|x| x.to_str()).unwrap_or_default();
            if !specs.contains(&file_name)
                && file_name != "WORKSPACE_CONTRACT.md"
                && file_name != "PROJECT_CONTRACT.md"
                && file_name != "ADAPTER_CONTRACT.md"
                && file_name != "EXECUTION_SEMANTICS_CONTRACT.md"
                && file_name != "SCHEDULER_STATESPACE_CONTRACT.md"
                && file_name != "DETERMINISTIC_SCHEDULING_CONTRACT.md"
                && file_name != "CONFIG_PRECEDENCE_CONTRACT.md"
                && file_name != "OPERATOR_INSPECTION_CONTRACT.md"
            {
                orphaned.push(format!("unknown contract doc: docs/spec/{file_name}"));
            }
            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if !content.contains("## Scope") {
                stale.push(format!("{} missing scope section", file_name));
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "missing": missing,
            "orphaned": orphaned,
            "stale": stale
        }))
        .map_err(|err| err.to_string())?
    );

    if missing.is_empty() && orphaned.is_empty() && stale.is_empty() {
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
    let docs_error_ref =
        fs::read_to_string(root.join("docs/reference/ERRORS.md")).map_err(|err| err.to_string())?;
    let docs_error_contract = fs::read_to_string(root.join("docs/spec/ERROR_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let tests = [
        root.join("crates/bijux-dag-app/tests/error_output_contract.rs"),
        root.join("crates/bijux-dag-app/tests/error_exit_contract.rs"),
    ];

    let mut violations = Vec::new();
    for code in &registry.codes {
        if !docs_error_ref.contains(&code.category) {
            violations.push(format!(
                "docs/reference/ERRORS.md missing category {} for {}",
                code.category, code.code
            ));
        }
        if !docs_error_contract
            .contains("Public error code additions require docs plus test coverage")
        {
            violations.push(
                "docs/spec/ERROR_CONTRACT.md missing public code governance rule".to_string(),
            );
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
