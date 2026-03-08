use crate::commands::repo_root;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn run_docs_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let allowed_dirs = [
        "spec",
        "architecture",
        "user",
        "dev",
        "reference",
        "tracking",
        "generated",
        "_tracking",
        "adr",
        "operations",
    ];

    for entry in fs::read_dir(&docs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !allowed_dirs.contains(&name.as_str()) {
            return Err(format!(
                "docs taxonomy violation: docs/{name} is not allowed"
            ));
        }
    }

    let root_markdown_count = fs::read_dir(&docs_root)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some("md"))
        .count();
    let max_root_docs = 110usize;
    if root_markdown_count > max_root_docs {
        return Err(format!(
            "docs root budget exceeded: {} > {}",
            root_markdown_count, max_root_docs
        ));
    }

    for rel in [
        "docs/spec/DOCS_GOVERNANCE.md",
        "docs/tracking/DOC_OWNERSHIP.json",
        "docs/tracking/DOCS_PRUNING_CHECKLIST.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing docs governance artifact: {rel}"));
        }
    }

    let owners: Value = serde_json::from_str(
        &fs::read_to_string(root.join("docs/tracking/DOC_OWNERSHIP.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    if owners
        .get("owners")
        .and_then(Value::as_array)
        .is_none_or(|items| items.is_empty())
    {
        return Err("docs ownership metadata has no owners entries".to_string());
    }

    for forbidden in ["production-grade", "world-class"] {
        let mut files = Vec::new();
        collect_markdown_files(&docs_root, &mut files)?;
        for file in files {
            let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
            for line in content.lines() {
                let lower = line.to_ascii_lowercase();
                if lower.contains(forbidden) && !line.contains('"') {
                    return Err(format!(
                        "marketing maturity phrase not allowed without quote: {}",
                        forbidden
                    ));
                }
            }
        }
    }

    let mut files = Vec::new();
    collect_markdown_files(&docs_root, &mut files)?;
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let lower = content.to_ascii_lowercase();
        for stale in ["bijux-dag-compat", "legacy-cli", "old_runtime_path"] {
            if lower.contains(stale) {
                return Err(format!("stale crate/path reference in {rel}: {stale}"));
            }
        }
        if lower.contains("roadmap") && !rel.starts_with("docs/tracking/") {
            return Err(format!(
                "speculative roadmap content must live under docs/tracking: {rel}"
            ));
        }
        if content.contains("AUTO-GENERATED") && !rel.starts_with("docs/generated/") {
            return Err(format!(
                "generated-doc marker must only appear under docs/generated: {rel}"
            ));
        }
    }

    Ok(())
}

pub(super) fn run_docs_link_check() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files)?;
    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for cap in content.match_indices("](") {
            let start = cap.0 + 2;
            if let Some(end_rel) = content[start..].find(')') {
                let link = &content[start..start + end_rel];
                if link.starts_with("http://")
                    || link.starts_with("https://")
                    || link.starts_with("mailto:")
                    || link.starts_with('#')
                {
                    continue;
                }
                let resolved = file.parent().unwrap_or(Path::new(".")).join(link);
                if !resolved.exists() {
                    let rel = file
                        .strip_prefix(&root)
                        .map_err(|err| err.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/");
                    violations.push(format!("{rel}: broken link target {link}"));
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_naming_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs = [
        "docs/spec/NAMING_GUIDELINES.md",
        "docs/spec/TERMINOLOGY_GLOSSARY.md",
        "docs/spec/NAMING_PHILOSOPHY.md",
        "docs/spec/NAMING_REVIEW_POLICY.md",
        "docs/architecture/naming_audit.md",
        "configs/policy/naming_rules.json",
    ];
    for rel in required_docs {
        if !root.join(rel).exists() {
            return Err(format!("missing naming governance artifact: {rel}"));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/naming_rules.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let banned_terms = policy
        .get("runtime_module_banned_terms")
        .and_then(Value::as_array)
        .ok_or_else(|| "naming_rules.json missing runtime_module_banned_terms".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if banned_terms.is_empty() {
        return Err("runtime_module_banned_terms must not be empty".to_string());
    }

    let mut runtime_files = Vec::new();
    collect_source_files_with_extension(
        &root.join("crates/bijux-dag-runtime/src"),
        "rs",
        &mut runtime_files,
    )?;
    let mut violations = Vec::new();
    for file in runtime_files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        for term in &banned_terms {
            if stem.contains(term) {
                violations.push(format!("{rel}: banned runtime module term `{term}`"));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_docs_config_reduction_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs_policy = root.join("configs/policy/docs_config_governance.json");
    let config_policy = root.join("configs/policy/config_consumers.json");
    if !docs_policy.exists() {
        return Err("missing docs config governance policy".to_string());
    }
    if !config_policy.exists() {
        return Err("missing config consumers policy".to_string());
    }

    for required in [
        "docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md",
        "docs/spec/MODELED_AND_FUTURE_SURFACES.md",
        "docs/spec/SPEC_TO_CODE_AND_TEST_OWNERSHIP.md",
        "docs/reports/foundation/docs_root_inventory_report.md",
        "docs/reports/foundation/config_inventory_report.md",
        "docs/reports/foundation/evidence_claim_links.md",
        "docs/reports/foundation/renovation_burndown_report.md",
        "docs/architecture/ADR_RENOVATION_ALIGNMENT.md",
    ] {
        if !root.join(required).exists() {
            return Err(format!(
                "missing docs config reduction authority: {required}"
            ));
        }
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&docs_policy).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let freeze_enabled = policy
        .get("roadmap_growth_freeze")
        .and_then(|node| node.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !freeze_enabled {
        return Err("roadmap growth freeze must stay enabled".to_string());
    }

    Ok(())
}

pub(super) fn run_docs_schema_reference_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files)?;
    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for token in content.split_whitespace() {
            if !token.contains("configs/schema/") {
                continue;
            }
            let clean =
                token.trim_matches(|c: char| matches!(c, ')' | '(' | '[' | ']' | ',' | ';' | '"'));
            let path = if clean.contains("configs/schema/") {
                let idx = clean.find("configs/schema/").unwrap_or(0);
                &clean[idx..]
            } else {
                clean
            };
            if !root.join(path).exists() {
                let rel = file
                    .strip_prefix(&root)
                    .map_err(|err| err.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                violations.push(format!("{rel}: missing schema reference {path}"));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_docs_contract_reference_guard() -> Result<(), String> {
    let root = repo_root()?;
    let crate_names = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];
    let mut violations = Vec::new();

    let docs_index = fs::read_to_string(root.join("docs/reference/DOCS_INDEX.md"))
        .map_err(|err| err.to_string())?;

    for crate_name in crate_names {
        let crate_dir = root.join("crates").join(crate_name);
        if !crate_dir.join("README.md").exists() {
            violations.push(format!("{crate_name} missing README.md"));
        }
        if !crate_dir.join("CONTRACT.md").exists() {
            violations.push(format!("{crate_name} missing CONTRACT.md"));
        }
        if !docs_index.contains(crate_name) {
            violations.push(format!(
                "docs/reference/DOCS_INDEX.md missing crate mention: {crate_name}"
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_docs_index_generate() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let sections = [
        "spec",
        "architecture",
        "user",
        "dev",
        "reference",
        "tracking",
        "generated",
    ];

    let mut lines = vec![
        "# Documentation index".to_string(),
        "".to_string(),
        "Generated from docs taxonomy.".to_string(),
        "".to_string(),
    ];

    for section in sections {
        let dir = docs_root.join(section);
        if !dir.exists() {
            continue;
        }
        lines.push(format!("## {}", section));
        let mut entries: Vec<String> = fs::read_dir(&dir)
            .map_err(|err| err.to_string())?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        for entry in entries {
            lines.push(format!("- `{}`", entry));
        }
        lines.push(String::new());
    }

    lines.push("## crate-doc-contracts".to_string());
    for crate_name in [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ] {
        lines.push(format!("- `{}`", crate_name));
    }
    lines.push(String::new());

    fs::write(
        docs_root.join("reference").join("DOCS_INDEX.md"),
        lines.join("\n"),
    )
    .map_err(|err| err.to_string())
}

pub(super) fn run_docs_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let crate_names = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];

    let mut missing = Vec::new();
    for crate_name in crate_names {
        if !root
            .join("crates")
            .join(crate_name)
            .join("CONTRACT.md")
            .exists()
        {
            missing.push(format!("missing contract doc for {crate_name}"));
        }
    }

    let command_taxonomy = root.join("docs/CLI_COMMAND_TAXONOMY.md");
    if !command_taxonomy.exists() {
        missing.push("missing CLI command taxonomy doc".to_string());
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({ "missing": missing }))
            .map_err(|err| err.to_string())?
    );

    if missing.is_empty() {
        Ok(())
    } else {
        Err("docs coverage has missing entries".to_string())
    }
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn collect_source_files_with_extension(
    dir: &Path,
    extension: &str,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_source_files_with_extension(&path, extension, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            out.push(path);
        }
    }
    Ok(())
}
