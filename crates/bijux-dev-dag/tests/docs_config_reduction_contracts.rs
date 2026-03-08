use bijux_dag_testkit as _;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use sha2 as _;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct DocsConfigGovernance {
    docs_root_markdown_budget: usize,
    banned_docs: Vec<String>,
    banned_phrases: Vec<String>,
    title_normalization_stopwords: Vec<String>,
    allowed_duplicate_titles: Vec<String>,
    roadmap_growth_freeze: FreezePolicy,
}

#[derive(Debug, Deserialize)]
struct FreezePolicy {
    enabled: bool,
    authority: String,
}

#[derive(Debug, Deserialize)]
struct ConfigConsumers {
    consumer_rules: Vec<ConsumerRule>,
}

#[derive(Debug, Deserialize)]
struct ConsumerRule {
    glob: String,
    consumer: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn glob_match(pattern: &str, value: &str) -> bool {
    if pattern == value {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return value.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return value.ends_with(suffix);
    }
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return value.starts_with(prefix) && value.ends_with(suffix);
    }
    false
}

#[test]
fn docs_root_stays_within_budget_and_banned_docs_are_removed() {
    let root = repo_root();
    let governance: DocsConfigGovernance = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/docs_config_governance.json"))
            .expect("docs config governance policy should exist"),
    )
    .expect("docs config governance policy should parse");

    let docs_root = root.join("docs");
    let count = fs::read_dir(&docs_root)
        .expect("docs root should be readable")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("md")).then_some(path)
        })
        .count();

    assert!(
        count <= governance.docs_root_markdown_budget,
        "docs root markdown budget exceeded: {count} > {}",
        governance.docs_root_markdown_budget
    );

    for doc in governance.banned_docs {
        assert!(
            !root.join(&doc).exists(),
            "banned docs theater surface must not exist: {doc}"
        );
    }
}

#[test]
fn docs_truthfulness_phrases_are_not_present_in_root_docs() {
    let root = repo_root();
    let governance: DocsConfigGovernance = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/docs_config_governance.json"))
            .expect("docs config governance policy should exist"),
    )
    .expect("docs config governance policy should parse");

    let docs_root = root.join("docs");
    for entry in fs::read_dir(&docs_root).expect("docs root should be readable") {
        let path = entry.expect("docs entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let body = fs::read_to_string(&path).expect("doc should be readable");
        let lowered = body.to_lowercase();
        for phrase in &governance.banned_phrases {
            assert!(
                !lowered.contains(phrase),
                "banned docs phrase `{phrase}` found in {}",
                path.display()
            );
        }
    }
}

#[test]
fn docs_title_overlap_is_detected_for_root_docs() {
    let root = repo_root();
    let governance: DocsConfigGovernance = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/docs_config_governance.json"))
            .expect("docs config governance policy should exist"),
    )
    .expect("docs config governance policy should parse");

    let stopwords: BTreeSet<String> = governance
        .title_normalization_stopwords
        .into_iter()
        .collect();
    let allowed: BTreeSet<String> = governance.allowed_duplicate_titles.into_iter().collect();

    let mut seen = BTreeMap::<String, String>::new();
    for entry in fs::read_dir(root.join("docs")).expect("docs root readable") {
        let path = entry.expect("docs entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let body = fs::read_to_string(&path).expect("doc readable");
        let Some(title) = body
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").trim().to_string())
        else {
            continue;
        };

        let normalized = title
            .split_whitespace()
            .map(|token| {
                token
                    .trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
                    .to_ascii_lowercase()
            })
            .filter(|token| !token.is_empty() && !stopwords.contains(token))
            .collect::<Vec<_>>()
            .join(" ");

        if allowed.contains(&title) {
            continue;
        }

        if let Some(existing) = seen.get(&normalized) {
            panic!(
                "duplicate normalized docs title `{normalized}` in {} and {}",
                existing,
                path.display()
            );
        }
        seen.insert(normalized, path.display().to_string());
    }
}

#[test]
fn config_inventory_rules_cover_all_config_files() {
    let root = repo_root();
    let consumers: ConfigConsumers = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/config_consumers.json"))
            .expect("config consumers policy should exist"),
    )
    .expect("config consumers policy should parse");

    for rule in &consumers.consumer_rules {
        assert!(
            !rule.consumer.trim().is_empty(),
            "consumer rule has empty consumer for glob {}",
            rule.glob
        );
    }

    let mut config_files = Vec::new();
    collect_files(&root.join("configs"), &mut config_files);
    for file in config_files {
        let rel = file
            .strip_prefix(&root)
            .expect("config under root")
            .to_string_lossy()
            .replace('\\', "/");
        let matched = consumers
            .consumer_rules
            .iter()
            .any(|rule| glob_match(&rule.glob, &rel));
        assert!(matched, "config file missing consumer mapping: {rel}");
    }
}

#[test]
fn docs_reduction_reports_and_authority_docs_exist() {
    let root = repo_root();
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
        assert!(
            root.join(required).exists(),
            "missing required docs surface: {required}"
        );
    }

    let governance: DocsConfigGovernance = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/docs_config_governance.json"))
            .expect("docs config governance policy should exist"),
    )
    .expect("docs config governance policy should parse");
    assert!(governance.roadmap_growth_freeze.enabled);
    assert!(
        root.join(governance.roadmap_growth_freeze.authority)
            .exists(),
        "roadmap growth freeze authority must exist"
    );

    let repo_suites = fs::read_to_string(root.join("crates/bijux-dev-dag/src/suites/repo.rs"))
        .expect("repo suites should exist");
    assert!(
        repo_suites.contains("\"docs-config-reduction\""),
        "repo suite must keep docs-config-reduction guard"
    );

    let command_surface = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("commands module should exist");
    assert!(
        command_surface.contains("\"docs-config-reduction\""),
        "foundation verification must keep docs-config-reduction guard"
    );
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("dir should be readable") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect_files(&path, out);
            continue;
        }
        out.push(path);
    }
}
