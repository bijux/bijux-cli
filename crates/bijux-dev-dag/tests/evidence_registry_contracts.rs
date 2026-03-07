use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
}

fn is_registry_asset_path(rel: &str) -> bool {
    let allowed_roots = [
        "evidence/authoring/examples/",
        "evidence/authoring/patterns/",
        "evidence/authoring/negative/",
        "evidence/battle/workflows/",
        "evidence/compat/graph_schema/",
        "evidence/compat/export_bundle/",
        "evidence/compat/run_dir/",
        "evidence/compat/scenarios/",
        "evidence/cache/corrupt/",
        "evidence/cache/scenarios/",
        "evidence/cache/replay/",
        "evidence/fault/classes/",
        "evidence/fault/corrupt_runs/",
        "evidence/perf/scenarios/",
        "evidence/compare/scenarios/",
        "evidence/operator/scenarios/",
    ];
    if !allowed_roots.iter().any(|prefix| rel.starts_with(prefix)) {
        return false;
    }
    rel.ends_with(".json") || rel.ends_with(".dag.json")
}

fn load_registry(root: &Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry")
}

#[test]
fn every_evidence_asset_appears_exactly_once_in_registry() {
    let root = repo_root();
    let registry = load_registry(&root);
    let assets = registry["assets"].as_array().expect("assets array");

    let mut registry_paths = BTreeSet::new();
    for asset in assets {
        let path = asset["canonical_path"].as_str().expect("canonical path");
        assert!(
            registry_paths.insert(path.to_string()),
            "duplicate path in registry: {path}"
        );
    }

    let mut files = Vec::new();
    collect_files(&root.join("evidence"), &mut files);
    let fs_paths: BTreeSet<String> = files
        .into_iter()
        .filter_map(|file| {
            let rel = file
                .strip_prefix(&root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            if is_registry_asset_path(&rel) {
                Some(rel)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        registry_paths, fs_paths,
        "registry and evidence filesystem asset set diverged"
    );
}

#[test]
fn no_two_assets_claim_same_canonical_identity() {
    let root = repo_root();
    let registry = load_registry(&root);
    let assets = registry["assets"].as_array().expect("assets array");
    let mut canonical = BTreeSet::new();
    for asset in assets {
        let path = asset["canonical_path"].as_str().expect("canonical path");
        assert!(
            canonical.insert(path.to_string()),
            "duplicate canonical identity: {path}"
        );
    }
}

#[test]
fn consumer_and_reference_links_resolve_to_existing_assets() {
    let root = repo_root();
    let registry = load_registry(&root);
    let assets = registry["assets"].as_array().expect("assets array");

    let ids: BTreeSet<String> = assets
        .iter()
        .map(|asset| asset["id"].as_str().expect("id").to_string())
        .collect();
    let keys: BTreeSet<String> = assets
        .iter()
        .map(|asset| {
            asset["registry_key"]
                .as_str()
                .expect("registry key")
                .to_string()
        })
        .collect();
    let paths: BTreeSet<String> = assets
        .iter()
        .map(|asset| {
            asset["canonical_path"]
                .as_str()
                .expect("canonical path")
                .to_string()
        })
        .collect();

    for asset in assets {
        let id = asset["id"].as_str().expect("id");
        for field in ["duplicate_of", "derived_from"] {
            let reference = &asset[field];
            if let Some(value) = reference.as_str() {
                if value.trim().is_empty() {
                    continue;
                }
                let resolves = ids.contains(value) || keys.contains(value) || paths.contains(value);
                assert!(resolves, "{field} for {id} does not resolve: {value}");
            }
        }

        for consumer in asset["consumers"].as_array().expect("consumers") {
            let text = consumer.as_str().expect("consumer string");
            if let Some(target) = text.strip_prefix("asset:") {
                let resolves =
                    ids.contains(target) || keys.contains(target) || paths.contains(target);
                assert!(
                    resolves,
                    "consumer reference for {id} does not resolve: {text}"
                );
            }
        }
    }
}

#[test]
fn registry_generation_is_deterministic() {
    let root = repo_root();
    let registry = load_registry(&root);
    let assets = registry["assets"].as_array().expect("assets array");

    let mut seen = BTreeMap::new();
    for asset in assets {
        let key = asset["registry_key"]
            .as_str()
            .expect("registry key")
            .to_string();
        let canonical = asset["canonical_path"]
            .as_str()
            .expect("canonical path")
            .to_string();
        seen.insert(key, canonical);
    }

    let sorted_keys: Vec<String> = seen.keys().cloned().collect();
    let registry_keys: Vec<String> = assets
        .iter()
        .map(|asset| {
            asset["registry_key"]
                .as_str()
                .expect("registry key")
                .to_string()
        })
        .collect();
    assert_eq!(
        registry_keys, sorted_keys,
        "registry keys must be stably sorted"
    );
}

#[test]
fn registry_drift_is_blocking_in_foundation_verify() {
    let root = repo_root();
    let source =
        fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs")).expect("read");
    assert!(
        source.contains("run_evidence_registry_verify()?"),
        "foundation verify must include evidence registry verification"
    );
}
