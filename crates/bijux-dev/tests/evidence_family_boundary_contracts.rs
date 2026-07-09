use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn cache_compat_fault_paths_match_family_kind() {
    let root = repo_root();
    let registry: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");
    let assets = registry["assets"].as_array().expect("assets array");

    for asset in assets {
        let path = asset["canonical_path"].as_str().expect("canonical path");
        let kind = asset["kind"].as_str().expect("kind");
        if path.starts_with("evidence/cache/") {
            assert_eq!(kind, "cache", "cache asset must be classified as cache");
        }
        if path.starts_with("evidence/compat/") {
            assert_eq!(kind, "compat", "compat asset must be classified as compat");
        }
        if path.starts_with("evidence/fault/") {
            assert_eq!(kind, "fault", "fault asset must be classified as fault");
        }
    }
}

#[test]
fn cache_and_replay_assets_use_allowed_consumers() {
    let root = repo_root();
    let cache_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/metadata.json"))
            .expect("read cache metadata"),
    )
    .expect("parse cache metadata");
    let cache_allowed: BTreeSet<String> = cache_metadata["consumer_boundaries"]
        ["cache_allowed_consumers"]
        .as_array()
        .expect("cache consumer array")
        .iter()
        .map(|item| item.as_str().expect("consumer string").to_string())
        .collect();
    let replay_allowed: BTreeSet<String> = cache_metadata["consumer_boundaries"]
        ["replay_allowed_consumers"]
        .as_array()
        .expect("replay consumer array")
        .iter()
        .map(|item| item.as_str().expect("consumer string").to_string())
        .collect();

    let registry: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");
    let assets = registry["assets"].as_array().expect("assets array");
    for asset in assets {
        let path = asset["canonical_path"].as_str().expect("canonical path");
        if !path.starts_with("evidence/cache/") {
            continue;
        }
        let allowed = if path.starts_with("evidence/cache/replay/") {
            &replay_allowed
        } else {
            &cache_allowed
        };
        for consumer in asset["consumers"].as_array().expect("consumers") {
            let consumer = consumer.as_str().expect("consumer string");
            assert!(
                allowed.contains(consumer),
                "cache/replay asset `{path}` uses disallowed consumer `{consumer}`"
            );
        }
    }
}

#[test]
fn compat_and_fault_metadata_have_strict_family_semantics() {
    let root = repo_root();
    let compat: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compat/metadata.json"))
            .expect("read compat metadata"),
    )
    .expect("parse compat metadata");
    let matrix = compat["decision_matrix"].as_object().expect("compat decision matrix");
    for (path, entry) in matrix {
        assert!(
            path.starts_with("evidence/compat/"),
            "compat decision matrix path must remain in compat family: {path}"
        );
        let decision = entry["decision"].as_str().expect("decision string");
        assert!(
            ["supported", "unsupported_newer_version", "unsupported_older_version", "corrupt"]
                .contains(&decision),
            "compat decision is invalid: {decision}"
        );
    }

    let fault: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/fault/metadata.json"))
            .expect("read fault metadata"),
    )
    .expect("parse fault metadata");
    let fault_expectations = fault["fault_expectations"].as_object().expect("fault expectations");
    let fault_profiles = fault["fault_profiles"].as_object().expect("fault profiles");
    for fault_class in fault_expectations.keys() {
        assert!(
            fault_profiles.contains_key(fault_class),
            "fault profile missing fault class: {fault_class}"
        );
    }
}

#[test]
fn family_reports_exist_for_shared_usage_and_coverage() {
    let root = repo_root();
    for rel in [
        "evidence/reports/evidence_shared_asset_usage.md",
        "evidence/reports/evidence_family_coverage.md",
    ] {
        assert!(root.join(rel).exists(), "missing report: {rel}");
    }
}
