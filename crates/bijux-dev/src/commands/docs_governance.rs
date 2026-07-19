use crate::commands::repo_root;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const KNOWN_LIMITATIONS_REL_PATH: &str = "docs/bijux-dag/quality/known-limitations.md";
const LIMITATION_REQUIRED_FIELDS: [&str; 7] = [
    "- stability class:",
    "- affected command or API:",
    "- limitation:",
    "- impact:",
    "- workaround:",
    "- planned fix:",
    "- release target:",
];
const REQUIRED_LIMITATION_SECTION_HEADINGS: [&str; 7] = [
    "## Stable Local Execution Limitations",
    "## Shell Isolation Limitations",
    "## Container Limitations",
    "## Scheduling Limitations",
    "## Remote/Distributed Limitations",
    "## API Stability Limitations",
    "## Cache/Replay Limitations",
];
const RISK_REGISTER_REL_PATH: &str = "docs/bijux-dag/quality/risk-register.md";
const RISK_REQUIRED_FIELDS: [&str; 6] = [
    "- severity:",
    "- affected component:",
    "- current status:",
    "- risk:",
    "- mitigation:",
    "- release decision:",
];
const REQUIRED_RISK_IDS: [&str; 10] = [
    "RISK-001", "RISK-002", "RISK-003", "RISK-004", "RISK-005", "RISK-006", "RISK-007", "RISK-008",
    "RISK-009", "RISK-010",
];
const ROADMAP_REFERENCE_ALLOWLIST: [&str; 11] = [
    "docs/index.md",
    "docs/bijux-dag/index.md",
    "docs/bijux-dag/foundation/release-boundary.md",
    "docs/bijux-dag/foundation/scope-and-boundaries.md",
    "docs/bijux-dag/interfaces/support-matrix.md",
    "docs/bijux-dag/quality/known-limitations.md",
    "docs/bijux-dag/operations/v0-4-0-release-notes.md",
    "docs/bijux-core/governance/documentation-governance-alignment.md",
    "docs/bijux-core/foundation/documentation-system.md",
    "docs/bijux-core/foundation/module-surface-lanes.md",
    "docs/reports/governance/documentation-authority-report.md",
];

#[derive(Debug, Deserialize, Default)]
struct DocsLintPolicy {
    #[serde(default)]
    exclude_prefixes: Vec<String>,
    #[serde(default)]
    orphan_exempt_prefixes: Vec<String>,
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
        "assets",
        "automation",
        "bijux-cli",
        "bijux-core",
        "bijux-dag",
        "bijux-dev",
        "overrides",
        "reports",
        "spec",
    ];

    for entry in fs::read_dir(&docs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !allowed_dirs.contains(&name.as_str()) {
            return Err(format!("docs taxonomy violation: docs/{name} is not allowed"));
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
        "docs/index.md",
        "docs/bijux-core/governance/documentation-standards.md",
        "docs/bijux-core/governance/index.md",
        "docs/bijux-dev/governance/documentation-standard.md",
        "docs/bijux-dev/operations/docs-operations.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing docs governance artifact: {rel}"));
        }
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
    let mut unauthorized_roadmap_references = Vec::new();
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
        if lower.contains("roadmap")
            && rel != "docs/bijux-dag/roadmap.md"
            && !roadmap_reference_allowed(&rel)
        {
            unauthorized_roadmap_references.push(rel.clone());
        }
        if content.contains("AUTO-GENERATED") && !rel.starts_with("docs/generated/") {
            return Err(format!(
                "generated-doc marker must only appear under docs/generated: {rel}"
            ));
        }
    }
    if !unauthorized_roadmap_references.is_empty() {
        return Err(format!(
            "speculative roadmap content must live in the owned product roadmap: {}",
            unauthorized_roadmap_references.join(", ")
        ));
    }

    run_known_limitations_guard()?;
    run_risk_register_guard()?;

    Ok(())
}

fn roadmap_reference_allowed(rel: &str) -> bool {
    ROADMAP_REFERENCE_ALLOWLIST.contains(&rel)
}

pub(super) fn run_known_limitations_guard() -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(KNOWN_LIMITATIONS_REL_PATH);
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    validate_known_limitations_content(&content)
        .map_err(|err| format!("{KNOWN_LIMITATIONS_REL_PATH}: {err}"))
}

pub(super) fn run_risk_register_guard() -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(RISK_REGISTER_REL_PATH);
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    validate_risk_register_content(&content)
        .map_err(|err| format!("{RISK_REGISTER_REL_PATH}: {err}"))
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
                if should_skip_markdown_link(link) {
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

        violations.extend(broken_inline_code_anchors(&root, &file, &content)?);
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn should_skip_markdown_link(link: &str) -> bool {
    link.starts_with("http://")
        || link.starts_with("https://")
        || link.starts_with("mailto:")
        || link.starts_with('#')
        || link.contains("{{")
        || link.contains("}}")
}

fn extract_inline_code_spans(content: &str) -> Vec<(usize, String)> {
    let mut spans = Vec::new();
    let mut fence_open = false;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            fence_open = !fence_open;
            continue;
        }
        if fence_open {
            continue;
        }

        let mut cursor = 0usize;
        while let Some(start_rel) = line[cursor..].find('`') {
            let start = cursor + start_rel + 1;
            let Some(end_rel) = line[start..].find('`') else {
                break;
            };
            let end = start + end_rel;
            let span = line[start..end].trim();
            if !span.is_empty() {
                spans.push((line_index + 1, span.to_string()));
            }
            cursor = end + 1;
        }
    }

    spans
}

fn repo_code_anchor_candidate(span: &str) -> Option<&str> {
    let anchor = span.trim();
    if anchor.is_empty() || anchor.contains("://") || anchor.contains(' ') {
        return None;
    }

    let starts_with_repo_root =
        ["crates/", "docs/", "configs/", ".github/", "makes/", "templates/", "evidence/"]
            .iter()
            .any(|prefix| anchor.starts_with(prefix));
    if !starts_with_repo_root {
        return None;
    }

    let looks_like_path = anchor.ends_with('/')
        || Path::new(anchor).extension().and_then(|ext| ext.to_str()).is_some();
    looks_like_path.then_some(anchor)
}

fn broken_inline_code_anchors(
    root: &Path,
    file: &Path,
    content: &str,
) -> Result<Vec<String>, String> {
    let rel = file
        .strip_prefix(root)
        .map_err(|err| err.to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    let mut violations = Vec::new();

    for (line_number, span) in extract_inline_code_spans(content) {
        let Some(anchor) = repo_code_anchor_candidate(&span) else {
            continue;
        };
        if !root.join(anchor).exists() {
            violations.push(format!("{rel}:{line_number}: broken code anchor {anchor}"));
        }
    }

    Ok(violations)
}

#[derive(Debug, Clone)]
struct LimitationRecord {
    id: String,
    heading_line: usize,
    body: Vec<String>,
}

fn parse_limitation_records(content: &str) -> Vec<LimitationRecord> {
    let mut records = Vec::new();
    let mut current: Option<LimitationRecord> = None;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(heading) = trimmed.strip_prefix("### ") {
            if heading.starts_with("LIM-") {
                if let Some(record) = current.take() {
                    records.push(record);
                }
                let id = heading.split_whitespace().next().unwrap_or_default().to_string();
                current =
                    Some(LimitationRecord { id, heading_line: line_index + 1, body: Vec::new() });
                continue;
            }
            if let Some(record) = current.take() {
                records.push(record);
            }
        }

        if let Some(record) = current.as_mut() {
            record.body.push(line.to_string());
        }
    }

    if let Some(record) = current {
        records.push(record);
    }

    records
}

fn limitation_field_value<'a>(record: &'a LimitationRecord, field: &str) -> Option<&'a str> {
    record.body.iter().find_map(|line| {
        line.trim_start().strip_prefix(field).map(str::trim).filter(|value| !value.is_empty())
    })
}

fn validate_known_limitations_content(content: &str) -> Result<(), String> {
    let mut violations = Vec::new();
    for heading in REQUIRED_LIMITATION_SECTION_HEADINGS {
        if !content.contains(heading) {
            violations.push(format!("missing limitations section heading `{heading}`"));
        }
    }

    let records = parse_limitation_records(content);
    if records.is_empty() {
        return Err("missing `### LIM-...` limitation records".to_string());
    }

    let mut seen_ids = BTreeSet::new();
    let mut has_experimental = false;
    let mut has_simulation = false;

    for record in &records {
        if !seen_ids.insert(record.id.clone()) {
            violations.push(format!(
                "{}:{}: duplicate limitation identifier",
                record.id, record.heading_line
            ));
        }

        for field in LIMITATION_REQUIRED_FIELDS {
            if limitation_field_value(record, field).is_none() {
                violations.push(format!(
                    "{}:{}: missing limitation field `{field}`",
                    record.id, record.heading_line
                ));
            }
        }

        if let Some(stability_class) = limitation_field_value(record, "- stability class:") {
            match stability_class.trim_matches('`') {
                "experimental-surface" => has_experimental = true,
                "simulation-surface" => has_simulation = true,
                _ => {}
            }
        }
    }

    if !has_experimental {
        violations.push("missing `experimental-surface` limitation record".to_string());
    }
    if !has_simulation {
        violations.push("missing `simulation-surface` limitation record".to_string());
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

#[derive(Debug, Clone)]
struct RiskRecord {
    id: String,
    heading_line: usize,
    body: Vec<String>,
}

fn parse_risk_records(content: &str) -> Vec<RiskRecord> {
    let mut records = Vec::new();
    let mut current: Option<RiskRecord> = None;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(heading) = trimmed.strip_prefix("### ") {
            if heading.starts_with("RISK-") {
                if let Some(record) = current.take() {
                    records.push(record);
                }
                let id = heading.split_whitespace().next().unwrap_or_default().to_string();
                current = Some(RiskRecord { id, heading_line: line_index + 1, body: Vec::new() });
                continue;
            }
            if let Some(record) = current.take() {
                records.push(record);
            }
        }

        if let Some(record) = current.as_mut() {
            record.body.push(line.to_string());
        }
    }

    if let Some(record) = current {
        records.push(record);
    }

    records
}

fn risk_field_value<'a>(record: &'a RiskRecord, field: &str) -> Option<&'a str> {
    record.body.iter().find_map(|line| {
        line.trim_start().strip_prefix(field).map(str::trim).filter(|value| !value.is_empty())
    })
}

fn validate_risk_register_content(content: &str) -> Result<(), String> {
    let records = parse_risk_records(content);
    if records.is_empty() {
        return Err("missing `### RISK-...` risk records".to_string());
    }

    let mut violations = Vec::new();
    let mut seen_ids = BTreeSet::new();

    for record in &records {
        if !seen_ids.insert(record.id.clone()) {
            violations
                .push(format!("{}:{}: duplicate risk identifier", record.id, record.heading_line));
        }

        for field in RISK_REQUIRED_FIELDS {
            if risk_field_value(record, field).is_none() {
                violations.push(format!(
                    "{}:{}: missing risk field `{field}`",
                    record.id, record.heading_line
                ));
            }
        }
    }

    for required_id in REQUIRED_RISK_IDS {
        if !seen_ids.contains(required_id) {
            violations.push(format!("missing required risk record `{required_id}`"));
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
    let policy_path = root.join("configs/dag/policy/naming_rules.json");
    if !policy_path.exists() {
        return Err(
            "missing naming governance artifact: configs/dag/policy/naming_rules.json".to_string()
        );
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let required_docs = policy
        .get("required_docs")
        .and_then(Value::as_array)
        .ok_or_else(|| "naming_rules.json missing required_docs".to_string())?;
    if required_docs.is_empty() {
        return Err("naming_rules.json required_docs must not be empty".to_string());
    }
    for rel in required_docs {
        let Some(rel) = rel.as_str() else {
            return Err("naming_rules.json required_docs must contain only strings".to_string());
        };
        if !root.join(rel).exists() {
            return Err(format!("missing naming governance artifact: {rel}"));
        }
    }

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
        let stem =
            file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_ascii_lowercase();
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
        "docs/bijux-core/foundation/current-implemented-capabilities.md",
        "docs/bijux-dag/foundation/release-boundary.md",
        "docs/bijux-core/governance/spec-to-code-and-test-ownership.md",
        "docs/reports/foundation/docs-root-inventory-report.md",
        "docs/reports/foundation/foundation-governance-posture.md",
        "docs/reports/foundation/repository-proof-statement.md",
        "docs/reports/governance/documentation-authority-report.md",
        "docs/bijux-core/governance/documentation-governance-alignment.md",
    ] {
        if !root.join(required).exists() {
            return Err(format!("missing docs config reduction authority: {required}"));
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
            let clean = token
                .trim_matches(|c: char| matches!(c, ')' | '(' | '[' | ']' | ',' | ';' | '"' | '`'));
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
    let mut violations = Vec::new();
    let crate_docs = [
        ("bijux-dag-core", "docs/bijux-dag/packages/bijux-dag-core.md"),
        ("bijux-dag-artifacts", "docs/bijux-dag/packages/bijux-dag-artifacts.md"),
        ("bijux-dag-runtime", "docs/bijux-dag/packages/bijux-dag-runtime.md"),
        ("bijux-dag-app", "docs/bijux-dag/packages/bijux-dag-app.md"),
        ("bijux-dag-cli", "docs/bijux-dag/packages/bijux-dag-cli.md"),
        ("bijux-dag-testkit", "docs/bijux-dag/packages/bijux-dag-testkit.md"),
        ("bijux-dev", "docs/bijux-dev/packages/bijux-dev.md"),
    ];

    for (crate_name, doc_rel) in crate_docs {
        let crate_dir = root.join("crates").join(crate_name);
        if !crate_dir.join("README.md").exists() {
            violations.push(format!("{crate_name} missing README.md"));
        }
        if !crate_dir.join("CONTRACT.md").exists() {
            violations.push(format!("{crate_name} missing CONTRACT.md"));
        }
        let doc_path = root.join(doc_rel);
        if !doc_path.exists() {
            violations.push(format!("missing package handbook page: {doc_rel}"));
            continue;
        }
        let doc_text = fs::read_to_string(&doc_path).map_err(|err| err.to_string())?;
        if !doc_text.contains(crate_name) {
            violations.push(format!("{doc_rel} missing crate mention: {crate_name}"));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

pub(super) fn run_docs_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let crate_docs = [
        ("bijux-dag-core", "docs/bijux-dag/packages/bijux-dag-core.md"),
        ("bijux-dag-artifacts", "docs/bijux-dag/packages/bijux-dag-artifacts.md"),
        ("bijux-dag-runtime", "docs/bijux-dag/packages/bijux-dag-runtime.md"),
        ("bijux-dag-app", "docs/bijux-dag/packages/bijux-dag-app.md"),
        ("bijux-dag-cli", "docs/bijux-dag/packages/bijux-dag-cli.md"),
        ("bijux-dag-testkit", "docs/bijux-dag/packages/bijux-dag-testkit.md"),
        ("bijux-dev", "docs/bijux-dev/packages/bijux-dev.md"),
    ];

    let mut missing = Vec::new();
    for (crate_name, doc_rel) in crate_docs {
        if !root.join("crates").join(crate_name).join("CONTRACT.md").exists() {
            missing.push(format!("missing contract doc for {crate_name}"));
        }
        if !root.join(doc_rel).exists() {
            missing.push(format!("missing package handbook page for {crate_name}: {doc_rel}"));
        }
    }

    for rel in
        ["docs/bijux-cli/interfaces/cli-surface.md", "docs/bijux-dag/interfaces/cli-surface.md"]
    {
        if !root.join(rel).exists() {
            missing.push(format!("missing command surface doc: {rel}"));
        }
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
    let nav_entries = collect_mkdocs_nav_entries(&root)?;

    let mut metadata_errors = Vec::new();
    let mut bad_status = Vec::new();
    let mut title_map: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut topic_map: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    let mut orphan_docs = Vec::new();

    for rel_path in &markdown_files {
        let path = root.join(rel_path);
        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let lines = content.lines().collect::<Vec<_>>();
        let head = lines.iter().take(60).map(|line| line.to_ascii_lowercase()).collect::<Vec<_>>();

        let metadata_required = required_exact.contains(rel_path)
            || policy.metadata_required_prefixes.iter().any(|prefix| rel_path.starts_with(prefix));
        if metadata_required {
            let has_audience = head.iter().any(|line| line.starts_with("audience:"));
            let has_owner = head.iter().any(|line| line.starts_with("owner:"));
            let status_line = head.iter().find(|line| line.starts_with("status:")).cloned();
            if !has_audience {
                metadata_errors.push(format!("{rel_path}: missing `audience`"));
            }
            if !has_owner {
                metadata_errors.push(format!("{rel_path}: missing `owner`"));
            }
            match status_line {
                None => metadata_errors.push(format!("{rel_path}: missing `status`")),
                Some(line) => {
                    let value = line.trim_start_matches("status:").trim().to_ascii_lowercase();
                    if !valid_documentation_status(&value) {
                        bad_status.push(format!("{rel_path}: invalid `status` value `{value}`"));
                    }
                }
            }
        }

        if let Some(title) = lines.iter().find_map(|line| line.strip_prefix("# ").map(str::trim)) {
            if !title.is_empty() {
                let scope = documentation_scope(rel_path);
                title_map
                    .entry((scope.clone(), title.to_string()))
                    .or_default()
                    .push(rel_path.clone());
                let topic = normalize_topic(title);
                if !topic.is_empty() {
                    topic_map.entry((scope, topic)).or_default().push(rel_path.clone());
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
        let in_nav = nav_entries.contains(rel_path);
        let orphan_exempt =
            policy.orphan_exempt_prefixes.iter().any(|prefix| rel_path.starts_with(prefix));
        let inbound_count = inbound.get(rel_path).copied().unwrap_or(0);
        if !is_index
            && !standalone_marker
            && !in_allowlist
            && !in_nav
            && !orphan_exempt
            && inbound_count == 0
        {
            orphan_docs.push(rel_path.clone());
        }
    }

    let duplicate_titles = title_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|((scope, title), paths)| {
            format!("duplicate title `{title}` in `{scope}`: {}", paths.join(", "))
        })
        .collect::<Vec<_>>();
    let duplicate_topics = topic_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|((scope, topic), paths)| {
            format!("duplicate topic `{topic}` in `{scope}`: {}", paths.join(", "))
        })
        .collect::<Vec<_>>();

    let mut violations = Vec::new();
    violations.extend(metadata_errors);
    violations.extend(bad_status);
    violations.extend(duplicate_titles);
    violations.extend(duplicate_topics);
    violations.extend(orphan_docs.into_iter().map(|path| format!("orphan doc: {path}")));
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
    let nav_entries = collect_mkdocs_nav_entries(&root)?;
    let mut section_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut metadata_gaps = Vec::new();
    let mut orphan_candidates = Vec::new();

    for rel_path in &markdown_files {
        let parts = rel_path.split('/').collect::<Vec<_>>();
        let section = if parts.len() > 1 { parts[1].to_string() } else { "root".to_string() };
        *section_counts.entry(section).or_insert(0) += 1;

        let content = fs::read_to_string(root.join(rel_path)).map_err(|err| err.to_string())?;
        let lines = content.lines().collect::<Vec<_>>();
        let head = lines.iter().take(60).map(|line| line.to_ascii_lowercase()).collect::<Vec<_>>();
        let status = head
            .iter()
            .find(|line| line.starts_with("status:"))
            .map(|line| line.trim_start_matches("status:").trim().to_ascii_lowercase())
            .filter(|status| valid_documentation_status(status))
            .unwrap_or_else(|| "missing_or_invalid".to_string());
        *status_counts.entry(status).or_insert(0) += 1;

        let metadata_required = required_exact.contains(rel_path)
            || policy.metadata_required_prefixes.iter().any(|prefix| rel_path.starts_with(prefix));
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
        let in_nav = nav_entries.contains(rel_path);
        let orphan_exempt =
            policy.orphan_exempt_prefixes.iter().any(|prefix| rel_path.starts_with(prefix));
        let inbound_count = inbound.get(rel_path).copied().unwrap_or(0);
        if !is_index
            && !standalone_marker
            && !in_allowlist
            && !in_nav
            && !orphan_exempt
            && inbound_count == 0
        {
            orphan_candidates.push(rel_path.clone());
        }
    }

    let inventory_path = root.join("docs/reports/governance/documentation-inventory.md");
    let mut inventory_lines = vec![
        "# Documentation inventory".to_string(),
        "".to_string(),
        "Generated by `bijux-dev-dag docs-inventory`.".to_string(),
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

    let consolidation_path =
        root.join("docs/reports/governance/documentation-consolidation-candidates.md");
    let mut candidate_lines = vec![
        "# Documentation consolidation candidates".to_string(),
        "".to_string(),
        "Generated by `bijux-dev-dag docs-inventory`.".to_string(),
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
    fs::write(consolidation_path, format!("{}\n", candidate_lines.join("\n")))
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
    policy.exclude_prefixes.iter().any(|prefix| rel_path.starts_with(prefix))
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
    let canonical_root = fs::canonicalize(root).map_err(|err| err.to_string())?;
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
                if should_skip_markdown_link(link) {
                    continue;
                }
                let link_no_anchor = link.split('#').next().unwrap_or(link).trim();
                if link_no_anchor.is_empty() {
                    continue;
                }
                let resolved = source_path.parent().unwrap_or(Path::new(".")).join(link_no_anchor);
                if resolved.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }
                let Ok(canonical_target) = fs::canonicalize(&resolved) else {
                    continue;
                };
                let rel_target = canonical_target
                    .strip_prefix(&canonical_root)
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

fn collect_mkdocs_nav_entries(root: &Path) -> Result<BTreeSet<String>, String> {
    let content = fs::read_to_string(root.join("mkdocs.yml")).map_err(|err| err.to_string())?;
    let mut entries = BTreeSet::new();
    for line in content.lines() {
        let Some((_, value)) = line.trim().split_once(':') else {
            continue;
        };
        let path = value.trim();
        if path.ends_with(".md") {
            entries.insert(format!("docs/{path}"));
        }
    }
    Ok(entries)
}

fn documentation_scope(rel_path: &str) -> String {
    rel_path.split('/').nth(1).unwrap_or("repository").to_string()
}

fn valid_documentation_status(status: &str) -> bool {
    matches!(status, "canonical" | "stable" | "generated" | "historical" | "internal")
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

#[cfg(test)]
mod tests {
    use super::{
        broken_inline_code_anchors, collect_inbound_counts, collect_mkdocs_nav_entries,
        documentation_scope, extract_inline_code_spans, repo_code_anchor_candidate,
        roadmap_reference_allowed, should_skip_markdown_link, validate_known_limitations_content,
        validate_risk_register_content, DocsLintPolicy, KNOWN_LIMITATIONS_REL_PATH,
        REQUIRED_RISK_IDS, RISK_REGISTER_REL_PATH,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn inline_code_span_extraction_skips_fenced_blocks_and_keeps_line_numbers() {
        let content = "\
before `crates/demo/src/lib.rs`\n\
```text\n\
`crates/ignored/src/lib.rs`\n\
```\n\
after `docs/index.md`\n";

        let spans = extract_inline_code_spans(content);
        assert_eq!(
            spans,
            vec![(1, "crates/demo/src/lib.rs".to_string()), (5, "docs/index.md".to_string())]
        );
    }

    #[test]
    fn repo_code_anchor_candidate_filters_to_repository_paths() {
        assert_eq!(
            repo_code_anchor_candidate("crates/demo/src/lib.rs"),
            Some("crates/demo/src/lib.rs")
        );
        assert_eq!(repo_code_anchor_candidate("docs/guide/"), Some("docs/guide/"));
        assert_eq!(repo_code_anchor_candidate("https://crates.io/crates/demo"), None);
        assert_eq!(repo_code_anchor_candidate("cargo run demo"), None);
        assert_eq!(repo_code_anchor_candidate("demo"), None);
    }

    #[test]
    fn broken_inline_code_anchors_report_missing_repo_paths_with_line_numbers() {
        let root = tempdir().expect("tempdir");
        let docs_dir = root.path().join("docs");
        let crates_dir = root.path().join("crates/demo/src");
        fs::create_dir_all(&docs_dir).expect("docs dir");
        fs::create_dir_all(&crates_dir).expect("crate dir");
        fs::write(crates_dir.join("lib.rs"), "// ok").expect("write crate");
        fs::write(docs_dir.join("index.md"), "# ok").expect("write docs");

        let source_file = docs_dir.join("guide.md");
        let content = "\
good `crates/demo/src/lib.rs`\n\
bad `crates/demo/src/missing.rs`\n\
also good `docs/index.md`\n";

        let violations =
            broken_inline_code_anchors(root.path(), &source_file, content).expect("violations");
        assert_eq!(
            violations,
            vec!["docs/guide.md:2: broken code anchor crates/demo/src/missing.rs".to_string()]
        );
    }

    #[test]
    fn markdown_link_skip_rules_allow_template_placeholders() {
        assert!(should_skip_markdown_link("https://bijux.io"));
        assert!(should_skip_markdown_link("{{ docs_url }}"));
        assert!(should_skip_markdown_link("#section"));
        assert!(!should_skip_markdown_link("../guide.md"));
    }

    #[test]
    fn documentation_scope_separates_product_handbooks() {
        assert_eq!(documentation_scope("docs/bijux-cli/interfaces/api-surface.md"), "bijux-cli");
        assert_eq!(documentation_scope("docs/bijux-dag/interfaces/api-surface.md"), "bijux-dag");
        assert_eq!(documentation_scope("docs/spec/REPLAY_CONTRACT.md"), "spec");
    }

    #[test]
    fn inbound_counts_normalize_parent_directory_links() {
        let root = tempdir().expect("tempdir");
        let source_dir = root.path().join("docs/product/quality");
        let target_dir = root.path().join("docs/product/interfaces");
        fs::create_dir_all(&source_dir).expect("source dir");
        fs::create_dir_all(&target_dir).expect("target dir");
        fs::write(source_dir.join("comparison.md"), "[Report](../interfaces/report.md)\n")
            .expect("source");
        fs::write(target_dir.join("report.md"), "# Report\n").expect("target");
        let files = vec![
            "docs/product/interfaces/report.md".to_string(),
            "docs/product/quality/comparison.md".to_string(),
        ];

        let inbound = collect_inbound_counts(root.path(), &files, &DocsLintPolicy::default())
            .expect("inbound counts");

        assert_eq!(inbound.get("docs/product/interfaces/report.md"), Some(&1));
    }

    #[test]
    fn mkdocs_navigation_entries_are_reader_entrypoints() {
        let root = tempdir().expect("tempdir");
        fs::write(
            root.path().join("mkdocs.yml"),
            "nav:\n  - Trust Evidence: bijux-core/governance/trust-evidence.md\n",
        )
        .expect("mkdocs");

        let entries = collect_mkdocs_nav_entries(root.path()).expect("nav entries");

        assert!(entries.contains("docs/bijux-core/governance/trust-evidence.md"));
    }

    #[test]
    fn roadmap_reference_allowlist_covers_boundary_and_entrypoint_docs() {
        for rel in [
            "docs/index.md",
            "docs/bijux-dag/index.md",
            "docs/bijux-dag/foundation/release-boundary.md",
            "docs/bijux-dag/foundation/scope-and-boundaries.md",
            "docs/bijux-dag/interfaces/support-matrix.md",
            "docs/bijux-dag/quality/known-limitations.md",
            "docs/bijux-core/foundation/documentation-system.md",
            "docs/reports/governance/documentation-authority-report.md",
        ] {
            assert!(roadmap_reference_allowed(rel), "{rel} should allow roadmap routing");
        }
    }

    #[test]
    fn roadmap_reference_allowlist_keeps_general_docs_outside_tracking_blocked() {
        assert!(!roadmap_reference_allowed("docs/bijux-dag/operations/common-workflows.md"));
        assert!(!roadmap_reference_allowed(
            "docs/bijux-dag/architecture/execution-mode-responsibilities.md"
        ));
    }

    #[test]
    fn known_limitations_validation_accepts_complete_records() {
        let content = "\
## Stable Local Execution Limitations\n\
\n\
## Shell Isolation Limitations\n\
\n\
### LIM-100 Experimental route example\n\
\n\
- stability class: `experimental-surface`\n\
- affected command or API: `bijux-dag hidden`\n\
- limitation: hidden commands do not carry a public compatibility guarantee.\n\
- impact: downstream automation may break.\n\
- workaround: use the visible operator contract only.\n\
- planned fix: promote only fully documented commands.\n\
- release target: no guarantee in `v0.4.x`.\n\
\n\
## Container Limitations\n\
\n\
## Scheduling Limitations\n\
\n\
## Remote/Distributed Limitations\n\
\n\
## API Stability Limitations\n\
\n\
### LIM-101 Simulation namespace example\n\
\n\
- stability class: `simulation-surface`\n\
- affected command or API: `bijux-dag simulated`\n\
- limitation: simulation namespaces model behavior rather than shipping it.\n\
- impact: operators cannot treat them as production capabilities.\n\
- workaround: use stable commands for real workflows.\n\
- planned fix: add real backend semantics before promotion.\n\
- release target: remain non-public in `v0.4.x`.\n\
\n\
## Cache/Replay Limitations\n";

        assert!(validate_known_limitations_content(content).is_ok());
    }

    #[test]
    fn known_limitations_validation_reports_missing_fields_and_surface_classes() {
        let content = "\
### LIM-100 Experimental route example\n\
\n\
- stability class: `experimental-surface`\n\
- affected command or API: `bijux-dag hidden`\n\
- limitation: hidden commands do not carry a public compatibility guarantee.\n\
- impact: downstream automation may break.\n\
- workaround: use the visible operator contract only.\n\
- release target: no guarantee in `v0.4.x`.\n";

        let error =
            validate_known_limitations_content(content).expect_err("validation should fail");
        assert!(error.contains("LIM-100:1: missing limitation field `- planned fix:`"));
        assert!(error.contains("missing `simulation-surface` limitation record"));
    }

    #[test]
    fn known_limitations_validation_requires_backlog_section_headings() {
        let content = "\
### LIM-100 Experimental route example\n\
\n\
- stability class: `experimental-surface`\n\
- affected command or API: `bijux-dag hidden`\n\
- limitation: hidden commands do not carry a public compatibility guarantee.\n\
- impact: downstream automation may break.\n\
- workaround: use the visible operator contract only.\n\
- planned fix: promote only fully documented commands.\n\
- release target: no guarantee in `v0.4.x`.\n\
\n\
### LIM-101 Simulation namespace example\n\
\n\
- stability class: `simulation-surface`\n\
- affected command or API: `bijux-dag simulated`\n\
- limitation: simulation namespaces model behavior rather than shipping it.\n\
- impact: operators cannot treat them as production capabilities.\n\
- workaround: use stable commands for real workflows.\n\
- planned fix: add real backend semantics before promotion.\n\
- release target: remain non-public in `v0.4.x`.\n";

        let error =
            validate_known_limitations_content(content).expect_err("validation should fail");
        assert!(error.contains(
            "missing limitations section heading `## Stable Local Execution Limitations`"
        ));
        assert!(error.contains("missing limitations section heading `## Cache/Replay Limitations`"));
    }

    #[test]
    fn known_limitations_validation_rejects_duplicate_identifiers() {
        let content = "\
### LIM-100 Experimental route example\n\
\n\
- stability class: `experimental-surface`\n\
- affected command or API: `bijux-dag hidden`\n\
- limitation: hidden commands do not carry a public compatibility guarantee.\n\
- impact: downstream automation may break.\n\
- workaround: use the visible operator contract only.\n\
- planned fix: promote only fully documented commands.\n\
- release target: no guarantee in `v0.4.x`.\n\
\n\
### LIM-100 Simulation namespace example\n\
\n\
- stability class: `simulation-surface`\n\
- affected command or API: `bijux-dag simulated`\n\
- limitation: simulation namespaces model behavior rather than shipping it.\n\
- impact: operators cannot treat them as production capabilities.\n\
- workaround: use stable commands for real workflows.\n\
- planned fix: add real backend semantics before promotion.\n\
- release target: remain non-public in `v0.4.x`.\n";

        let error =
            validate_known_limitations_content(content).expect_err("validation should fail");
        assert!(error.contains("LIM-100:"));
        assert!(error.contains("duplicate limitation identifier"));
    }

    #[test]
    fn known_limitations_handbook_matches_record_contract() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let content = fs::read_to_string(repo_root.join(KNOWN_LIMITATIONS_REL_PATH))
            .expect("read known limitations handbook");

        assert!(validate_known_limitations_content(&content).is_ok());
    }

    fn complete_risk_record(id: &str, component: &str) -> String {
        format!(
            "### {id} Example risk\n\n\
- severity: `high`\n\
- affected component: {component}\n\
- current status: `mitigating`\n\
- risk: this risk remains active until release review closes it.\n\
- mitigation: keep docs, tests, and release checks aligned.\n\
- release decision: keep the affected surface gated until the evidence stays green.\n"
        )
    }

    fn complete_risk_register_fixture() -> String {
        REQUIRED_RISK_IDS
            .iter()
            .enumerate()
            .map(|(index, id)| complete_risk_record(id, &format!("component-{index}")))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn risk_register_validation_accepts_complete_records() {
        let content = complete_risk_register_fixture();
        assert!(validate_risk_register_content(&content).is_ok());
    }

    #[test]
    fn risk_register_validation_reports_missing_fields_and_required_ids() {
        let content = "\
### RISK-001 Example risk\n\n\
- severity: `high`\n\
- affected component: local shell execution\n\
- current status: `mitigating`\n\
- risk: this risk remains active until release review closes it.\n\
- release decision: keep the affected surface gated until the evidence stays green.\n";

        let error = validate_risk_register_content(content).expect_err("validation should fail");
        assert!(error.contains("RISK-001:1: missing risk field `- mitigation:`"));
        assert!(error.contains("missing required risk record `RISK-002`"));
    }

    #[test]
    fn risk_register_validation_rejects_duplicate_identifiers() {
        let content = complete_risk_register_fixture().replace("### RISK-010", "### RISK-001");
        let error = validate_risk_register_content(&content).expect_err("validation should fail");
        assert!(error.contains("RISK-001:"));
        assert!(error.contains("duplicate risk identifier"));
        assert!(error.contains("missing required risk record `RISK-010`"));
    }

    #[test]
    fn risk_register_handbook_matches_record_contract() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let content = fs::read_to_string(repo_root.join(RISK_REGISTER_REL_PATH))
            .expect("read risk register handbook");

        assert!(validate_risk_register_content(&content).is_ok());
    }
}
