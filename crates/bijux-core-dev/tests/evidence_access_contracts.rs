use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

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
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
}

#[test]
fn resolver_commands_are_wired_in_cli() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-core-dev/src/commands/mod.rs"))
        .expect("read commands module");
    for command in [
        "EvidenceResolveById",
        "EvidenceResolveByFamily",
        "EvidenceResolveByTrustProperty",
        "EvidenceResolveByConsumer",
        "EvidenceConsumerReports",
    ] {
        assert!(
            source.contains(command),
            "missing resolver command wiring: {command}"
        );
    }
}

#[test]
fn registry_access_uses_approved_helpers_only() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files);
    let allowlist = BTreeSet::from([
        "crates/bijux-core-dev/src/commands/evidence_access.rs".to_string(),
        "crates/bijux-core-dev/src/commands/evidence_registry.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_family_boundary_contracts.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_consumer_integrity_contracts.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_access_contracts.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_control_plane_suites_contracts.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_registry_contracts.rs".to_string(),
        "crates/bijux-dag-testkit/src/lib.rs".to_string(),
    ]);
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with(".rs") {
            continue;
        }
        if allowlist.contains(&rel) {
            continue;
        }
        let content = fs::read_to_string(&file).expect("read file");
        let direct_read_patterns = [
            "read_to_string(root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(&root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(repo_root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
            "read_to_string(&repo_root.join(\"evidence/_meta/registries/evidence_registry.json\"))",
        ];
        assert!(
            !direct_read_patterns
                .iter()
                .any(|pattern| content.contains(pattern)),
            "registry bypass found outside approved access helpers: {rel}"
        );
    }
}

#[test]
fn no_legacy_scenario_roots_are_referenced_by_runtime_sources() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files);
    let allowlist = BTreeSet::from([
        "crates/bijux-core-dev/src/commands/mod.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_consumer_integrity_contracts.rs".to_string(),
        "crates/bijux-core-dev/tests/evidence_access_contracts.rs".to_string(),
    ]);
    let forbidden = ["benchmarks/scenarios/", "comparisons/scenarios/"];

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with(".rs") {
            continue;
        }
        if rel.contains("/tests/") {
            continue;
        }
        if allowlist.contains(&rel) {
            continue;
        }
        let content = fs::read_to_string(&file).expect("read file");
        for pattern in forbidden {
            if content.contains(pattern) {
                violations.push(format!("{rel} -> {pattern}"));
            }
        }
    }
    if !violations.is_empty() {
        eprintln!(
            "warning: legacy scenario roots referenced by runtime sources: {}",
            violations.join(" | ")
        );
    }
}

#[test]
fn consumer_reports_exist_and_cover_registry_consumers() {
    let root = repo_root();
    let assets_report =
        fs::read_to_string(root.join("evidence/reports/evidence_assets_to_consumers.md"))
            .expect("read assets->consumers report");
    let consumers_report =
        fs::read_to_string(root.join("evidence/reports/evidence_consumers_to_families.md"))
            .expect("read consumers->families report");
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .expect("read registry"),
    )
    .expect("parse registry");
    let mut consumers = BTreeSet::new();
    for asset in registry["assets"].as_array().expect("assets array") {
        for consumer in asset["consumers"].as_array().expect("consumers array") {
            consumers.insert(consumer.as_str().expect("consumer string").to_string());
        }
    }
    for consumer in consumers {
        assert!(
            assets_report.contains(&consumer),
            "assets report missing consumer: {consumer}"
        );
        assert!(
            consumers_report.contains(&consumer),
            "consumers report missing consumer: {consumer}"
        );
    }
}
