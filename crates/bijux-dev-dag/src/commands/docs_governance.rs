use crate::commands::repo_root;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default)]
struct DocsLintPolicy {
    #[serde(default)]
    exclude_prefixes: Vec<String>,
    #[serde(default)]
    metadata_required_prefixes: Vec<String>,
    #[serde(default)]
    metadata_required_exact: Vec<String>,
    #[serde(default)]
    standalone_allowlist: Vec<String>,
}

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
        "docs/reference/NAMING_AUDIT.md",
        "configs/dag/policy/naming_rules.json",
    ];
    for rel in required_docs {
        if !root.join(rel).exists() {
            return Err(format!("missing naming governance artifact: {rel}"));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/naming_rules.json"))
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
    let docs_policy = root.join("configs/dag/policy/docs_config_governance.json");
    let config_policy = root.join("configs/dag/policy/config_consumers.json");
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
        "docs/reports/foundation/DOCS_ROOT_INVENTORY_REPORT.md",
        "docs/reports/foundation/FOUNDATION_FINAL_REPORT.md",
        "docs/reports/foundation/REPOSITORY_PROOF_STATEMENT.md",
        "docs/reports/foundation/archive/RENOVATION_BURNDOWN_REPORT.md",
        "docs/adr/20260309-DOCUMENTATION-GOVERNANCE-ALIGNMENT.md",
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
            if !token.contains("configs/dag/schema/") {
                continue;
            }
            let clean =
                token.trim_matches(|c: char| matches!(c, ')' | '(' | '[' | ']' | ',' | ';' | '"'));
            let path = if clean.contains("configs/dag/schema/") {
                let idx = clean.find("configs/dag/schema/").unwrap_or(0);
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
    .map_err(|err| err.to_string())?;

    run_docs_inventory_generate()
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

    let command_taxonomy = root.join("docs/reference/COMMAND_TAXONOMY.md");
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

pub(super) fn run_docs_governance_lint() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let policy = load_docs_lint_policy(&root)?;
    let markdown_files = collect_markdown_files_filtered(&root, &docs_root, &policy)?;
    let inbound = collect_inbound_counts(&root, &markdown_files, &policy)?;

    let required_exact: BTreeSet<String> = policy.metadata_required_exact.iter().cloned().collect();
    let standalone_allowlist: BTreeSet<String> =
        policy.standalone_allowlist.iter().cloned().collect();

    let mut metadata_errors = Vec::new();
    let mut bad_status = Vec::new();
    let mut title_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut topic_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut orphan_docs = Vec::new();

    for rel_path in &markdown_files {
        let path = root.join(rel_path);
        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let lines = content.lines().collect::<Vec<_>>();
        let head = lines
            .iter()
            .take(60)
            .map(|line| line.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let metadata_required = required_exact.contains(rel_path)
            || policy
                .metadata_required_prefixes
                .iter()
                .any(|prefix| rel_path.starts_with(prefix));
        if metadata_required {
            let has_audience = head.iter().any(|line| line.starts_with("audience:"));
            let has_owner = head.iter().any(|line| line.starts_with("owner:"));
            let status_line = head
                .iter()
                .find(|line| line.starts_with("status:"))
                .cloned();
            if !has_audience {
                metadata_errors.push(format!("{rel_path}: missing `audience`"));
            }
            if !has_owner {
                metadata_errors.push(format!("{rel_path}: missing `owner`"));
            }
            match status_line {
                None => metadata_errors.push(format!("{rel_path}: missing `status`")),
                Some(line) => {
                    let value = line
                        .trim_start_matches("status:")
                        .trim()
                        .to_ascii_lowercase();
                    if !matches!(
                        value.as_str(),
                        "stable" | "generated" | "historical" | "internal"
                    ) {
                        bad_status.push(format!("{rel_path}: invalid `status` value `{value}`"));
                    }
                }
            }
        }

        if let Some(title) = lines
            .iter()
            .find_map(|line| line.strip_prefix("# ").map(str::trim))
        {
            if !title.is_empty() {
                title_map
                    .entry(title.to_string())
                    .or_default()
                    .push(rel_path.clone());
                let topic = normalize_topic(title);
                if !topic.is_empty() {
                    topic_map.entry(topic).or_default().push(rel_path.clone());
                }
            }
        }

        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_index = file_name.starts_with("readme") || file_name.starts_with("index");
        let standalone_marker = head.iter().any(|line| line.trim() == "standalone: yes");
        let in_allowlist = standalone_allowlist.contains(rel_path);
        let inbound_count = inbound.get(rel_path).copied().unwrap_or(0);
        if !is_index && !standalone_marker && !in_allowlist && inbound_count == 0 {
            orphan_docs.push(rel_path.clone());
        }
    }

    let duplicate_titles = title_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(title, paths)| format!("duplicate title `{title}`: {}", paths.join(", ")))
        .collect::<Vec<_>>();
    let duplicate_topics = topic_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(topic, paths)| format!("duplicate topic `{topic}`: {}", paths.join(", ")))
        .collect::<Vec<_>>();

    let mut violations = Vec::new();
    violations.extend(metadata_errors);
    violations.extend(bad_status);
    violations.extend(duplicate_titles);
    violations.extend(duplicate_topics);
    violations.extend(
        orphan_docs
            .into_iter()
            .map(|path| format!("orphan doc: {path}")),
    );
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_docs_inventory_generate() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let policy = load_docs_lint_policy(&root)?;
    let markdown_files = collect_markdown_files_filtered(&root, &docs_root, &policy)?;
    let inbound = collect_inbound_counts(&root, &markdown_files, &policy)?;

    let required_exact: BTreeSet<String> = policy.metadata_required_exact.iter().cloned().collect();
    let standalone_allowlist: BTreeSet<String> =
        policy.standalone_allowlist.iter().cloned().collect();
    let mut section_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut metadata_gaps = Vec::new();
    let mut orphan_candidates = Vec::new();

    for rel_path in &markdown_files {
        let parts = rel_path.split('/').collect::<Vec<_>>();
        let section = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "root".to_string()
        };
        *section_counts.entry(section).or_insert(0) += 1;

        let content = fs::read_to_string(root.join(rel_path)).map_err(|err| err.to_string())?;
        let lines = content.lines().collect::<Vec<_>>();
        let head = lines
            .iter()
            .take(60)
            .map(|line| line.to_ascii_lowercase())
            .collect::<Vec<_>>();
        let status = head
            .iter()
            .find(|line| line.starts_with("status:"))
            .map(|line| {
                line.trim_start_matches("status:")
                    .trim()
                    .to_ascii_lowercase()
            })
            .filter(|status| {
                matches!(
                    status.as_str(),
                    "stable" | "generated" | "historical" | "internal"
                )
            })
            .unwrap_or_else(|| "missing_or_invalid".to_string());
        *status_counts.entry(status).or_insert(0) += 1;

        let metadata_required = required_exact.contains(rel_path)
            || policy
                .metadata_required_prefixes
                .iter()
                .any(|prefix| rel_path.starts_with(prefix));
        if metadata_required {
            if !head.iter().any(|line| line.starts_with("audience:")) {
                metadata_gaps.push(format!("{rel_path}: missing `audience`"));
            }
            if !head.iter().any(|line| line.starts_with("owner:")) {
                metadata_gaps.push(format!("{rel_path}: missing `owner`"));
            }
            if !head.iter().any(|line| line.starts_with("status:")) {
                metadata_gaps.push(format!("{rel_path}: missing `status`"));
            }
        }

        let file_name = Path::new(rel_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let is_index = file_name.starts_with("readme") || file_name.starts_with("index");
        let standalone_marker = head.iter().any(|line| line.trim() == "standalone: yes");
        let in_allowlist = standalone_allowlist.contains(rel_path);
        let inbound_count = inbound.get(rel_path).copied().unwrap_or(0);
        if !is_index && !standalone_marker && !in_allowlist && inbound_count == 0 {
            orphan_candidates.push(rel_path.clone());
        }
    }

    let inventory_path = root.join("docs/generated/DOCS_INVENTORY.md");
    let mut inventory_lines = vec![
        "# Documentation inventory".to_string(),
        "".to_string(),
        "Generated by `bijux-dev-dag docs-index`.".to_string(),
        "".to_string(),
        "## Counts by section".to_string(),
        "".to_string(),
    ];
    for (section, count) in section_counts {
        inventory_lines.push(format!("- `{section}`: {count}"));
    }
    inventory_lines.push(String::new());
    inventory_lines.push("## Counts by status".to_string());
    inventory_lines.push(String::new());
    for (status, count) in status_counts {
        inventory_lines.push(format!("- `{status}`: {count}"));
    }
    inventory_lines.push(String::new());
    inventory_lines.push("## Metadata gaps".to_string());
    inventory_lines.push(String::new());
    if metadata_gaps.is_empty() {
        inventory_lines.push("- none".to_string());
    } else {
        for gap in metadata_gaps.into_iter().take(200) {
            inventory_lines.push(format!("- {gap}"));
        }
    }
    fs::write(inventory_path, format!("{}\n", inventory_lines.join("\n")))
        .map_err(|err| err.to_string())?;

    let consolidation_path = root.join("docs/generated/DOCS_CONSOLIDATION_CANDIDATES.md");
    let mut candidate_lines = vec![
        "# Documentation consolidation candidates".to_string(),
        "".to_string(),
        "Generated by `bijux-dev-dag docs-index`.".to_string(),
        "".to_string(),
        "These files currently have no inbound links from non-archived docs and are candidates for merge, move, or deletion.".to_string(),
        "".to_string(),
    ];
    if orphan_candidates.is_empty() {
        candidate_lines.push("- none".to_string());
    } else {
        orphan_candidates.sort();
        for rel_path in orphan_candidates.into_iter().take(300) {
            candidate_lines.push(format!("- `{rel_path}`"));
        }
    }
    fs::write(
        consolidation_path,
        format!("{}\n", candidate_lines.join("\n")),
    )
    .map_err(|err| err.to_string())?;

    Ok(())
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

fn collect_markdown_files_filtered(
    root: &Path,
    docs_root: &Path,
    policy: &DocsLintPolicy,
) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    collect_markdown_files(docs_root, &mut paths)?;
    let mut rel_paths = paths
        .into_iter()
        .map(|path| {
            path.strip_prefix(root)
                .map(|rel| rel.to_string_lossy().replace('\\', "/"))
                .map_err(|err| err.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|path| !is_excluded(path, policy))
        .collect::<Vec<_>>();
    rel_paths.sort();
    Ok(rel_paths)
}

fn is_excluded(rel_path: &str, policy: &DocsLintPolicy) -> bool {
    policy
        .exclude_prefixes
        .iter()
        .any(|prefix| rel_path.starts_with(prefix))
}

fn load_docs_lint_policy(root: &Path) -> Result<DocsLintPolicy, String> {
    let path = root.join("configs/dag/policy/docs_lint_policy.json");
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str::<DocsLintPolicy>(&content).map_err(|err| err.to_string())
}

fn collect_inbound_counts(
    root: &Path,
    markdown_files: &[String],
    policy: &DocsLintPolicy,
) -> Result<BTreeMap<String, usize>, String> {
    let tracked: BTreeSet<String> = markdown_files.iter().cloned().collect();
    let mut inbound: BTreeMap<String, usize> = BTreeMap::new();
    for rel_path in markdown_files {
        let content = fs::read_to_string(root.join(rel_path)).map_err(|err| err.to_string())?;
        let source_path = root.join(rel_path);
        let mut cursor = 0usize;
        while let Some(start) = content[cursor..].find("](") {
            let open = cursor + start + 2;
            if let Some(close_rel) = content[open..].find(')') {
                let close = open + close_rel;
                let link = content[open..close].trim();
                cursor = close + 1;
                if link.starts_with("http://")
                    || link.starts_with("https://")
                    || link.starts_with("mailto:")
                    || link.starts_with('#')
                {
                    continue;
                }
                let link_no_anchor = link.split('#').next().unwrap_or(link).trim();
                if link_no_anchor.is_empty() {
                    continue;
                }
                let resolved = source_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(link_no_anchor);
                if !resolved.exists()
                    || resolved.extension().and_then(|ext| ext.to_str()) != Some("md")
                {
                    continue;
                }
                let rel_target = resolved
                    .strip_prefix(root)
                    .map_err(|err| err.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                if is_excluded(&rel_target, policy) || !tracked.contains(&rel_target) {
                    continue;
                }
                *inbound.entry(rel_target).or_insert(0) += 1;
            } else {
                break;
            }
        }
    }
    Ok(inbound)
}

fn normalize_topic(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
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
