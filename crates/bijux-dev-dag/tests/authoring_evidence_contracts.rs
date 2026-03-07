use bijux_dag_artifacts as _;
use bijux_dag_core::{parse_graph_strict, Severity};
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde::Deserialize;
use sha2 as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct AuthoringMetadata {
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

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn load_metadata() -> AuthoringMetadata {
    let root = repo_root();
    let payload =
        fs::read_to_string(root.join("evidence/authoring/metadata.json")).expect("read metadata");
    serde_json::from_str(&payload).expect("parse metadata")
}

fn authoring_doc_refs() -> BTreeSet<String> {
    let root = repo_root();
    let mut refs = BTreeSet::new();
    for rel in [
        "docs/spec/AUTHORING_UX_CONTRACT.md",
        "docs/user/AUTHORING_GUIDE.md",
    ] {
        let text = fs::read_to_string(root.join(rel)).expect("read authoring doc");
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
    refs
}

#[test]
fn all_positive_authoring_assets_parse_and_validate() {
    let root = repo_root();
    let metadata = load_metadata();
    for (path, _asset) in metadata
        .assets
        .iter()
        .filter(|(_, asset)| asset.expected_validation == "pass")
    {
        let payload = fs::read_to_string(root.join(path)).expect("read authoring asset");
        let graph = parse_graph_strict(&payload).expect("parse positive authoring");
        let has_error = graph
            .validate_with_warnings()
            .iter()
            .any(|d| d.severity == Severity::Error);
        assert!(
            !has_error,
            "positive authoring asset has validation error: {path}"
        );
    }
}

#[test]
fn all_negative_authoring_assets_fail_as_intended() {
    let root = repo_root();
    let metadata = load_metadata();
    for (path, asset) in metadata
        .assets
        .iter()
        .filter(|(_, asset)| asset.expected_validation == "fail")
    {
        let payload = fs::read_to_string(root.join(path)).expect("read negative authoring asset");
        let parsed = parse_graph_strict(&payload);
        if let Ok(graph) = parsed {
            let has_error = graph
                .validate_with_warnings()
                .iter()
                .any(|d| d.severity == Severity::Error);
            assert!(
                has_error,
                "negative authoring asset unexpectedly validated: {path}"
            );
        }
        assert!(
            !asset.expected_rule_ids.is_empty(),
            "negative authoring asset must map to stable rule IDs: {path}"
        );
        for rule_id in &asset.expected_rule_ids {
            assert!(
                rule_id.starts_with("DAG-VAL-"),
                "negative authoring rule ID must use stable DAG-VAL namespace: {path} => {rule_id}"
            );
        }
    }
}

#[test]
fn authoring_docs_only_reference_existing_authoring_assets() {
    let root = repo_root();
    let metadata = load_metadata();
    let refs = authoring_doc_refs();
    for path in refs {
        assert!(
            metadata.assets.contains_key(&path),
            "authoring docs reference unknown asset: {path}"
        );
        assert!(
            root.join(&path).exists(),
            "authoring docs reference missing file: {path}"
        );
    }
}

#[test]
fn authoring_examples_remain_instructional_and_not_battle_proofs() {
    let root = repo_root();
    let metadata = load_metadata();
    for (path, asset) in &metadata.assets {
        assert!(
            !path.starts_with("evidence/battle/"),
            "battle workflow cannot masquerade as authoring asset: {path}"
        );
        assert!(
            matches!(
                asset.group.as_str(),
                "minimal" | "patterns" | "negative" | "examples"
            ),
            "unexpected authoring group `{}` for {path}",
            asset.group
        );
        assert!(
            matches!(asset.authoring_mode.as_str(), "normative" | "illustrative"),
            "unexpected authoring mode `{}` for {path}",
            asset.authoring_mode
        );
        assert!(
            !asset.command_surfaces.is_empty() && !asset.consumers.is_empty(),
            "authoring asset must declare command surfaces and consumers: {path}"
        );
        assert!(
            matches!(
                asset.expected_lowering.as_str(),
                "required" | "optional" | "none"
            ),
            "unexpected expected_lowering `{}` for {path}",
            asset.expected_lowering
        );

        let payload = fs::read_to_string(root.join(path)).expect("read authoring asset");
        let line_count = payload.lines().count();
        assert!(
            line_count >= 4,
            "authoring asset is too compact to be instructional: {path}"
        );
        assert!(
            line_count <= 250,
            "authoring asset is too large to stay instructional: {path}"
        );
        assert!(
            payload.lines().all(|line| line.len() <= 160),
            "authoring asset contains non-human-readable long lines: {path}"
        );
        let lowered = payload.to_ascii_lowercase();
        for banned in [
            "\"distributed_controller\"",
            "\"federation\"",
            "\"enterprise_scheduler\"",
            "\"ha_scheduler\"",
            "\"future_only\"",
            "\"not_implemented\"",
        ] {
            assert!(
                !lowered.contains(banned),
                "authoring asset contains speculative unsupported feature marker {banned}: {path}"
            );
        }
    }
}

#[test]
fn authoring_reports_exist() {
    let root = repo_root();
    for rel in [
        "evidence/reports/authoring_coverage_by_docs_and_commands.md",
        "evidence/reports/authoring_unused_assets.md",
    ] {
        assert!(root.join(rel).exists(), "missing authoring report: {rel}");
    }
}

#[test]
fn authoring_repo_commands_are_wired() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read commands source");
    for token in [
        "ValidateAllAuthoring",
        "ShowEffectiveAllAuthoring",
        "AuthoringCoverageReport",
        "repo.validate-all-authoring",
        "repo.show-effective-all-authoring",
        "repo.authoring-coverage-report",
    ] {
        assert!(
            source.contains(token),
            "authoring repo command token missing from command surface: {token}"
        );
    }
}
