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
use serde_json as _;
use sha2 as _;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct TestTrustLedger {
    classification_rules: Vec<ClassificationRule>,
    must_never_break: Vec<String>,
    required_semantic_surfaces: BTreeMap<String, String>,
    snapshot_surface_policy: SnapshotSurfacePolicy,
    trust_coverage_families: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ClassificationRule {
    class: String,
    #[serde(default)]
    match_: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SnapshotSurfacePolicy {
    allowed_assertion_files: Vec<String>,
    forbidden_macros: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn load_ledger(root: &Path) -> TestTrustLedger {
    let payload = fs::read_to_string(root.join("configs/dag/policy/test_trust_ledger.json"))
        .expect("test trust ledger policy should exist");
    let mut value: serde_json::Value =
        serde_json::from_str(&payload).expect("test trust ledger should parse as json");
    // rename reserved key for serde field mapping
    for rule in
        value["classification_rules"].as_array_mut().expect("classification_rules should be array")
    {
        if let Some(map) = rule.as_object_mut() {
            if let Some(matched) = map.remove("match") {
                map.insert("match_".to_string(), matched);
            }
        }
    }
    serde_json::from_value(value).expect("test trust ledger schema should parse")
}

fn runtime_test_files(root: &Path) -> Vec<String> {
    let dir = root.join("crates/bijux-dag-runtime/tests");
    let mut files = Vec::new();
    for entry in fs::read_dir(&dir).expect("runtime tests dir") {
        let path = entry.expect("runtime test entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        files.push(
            path.file_name().and_then(|name| name.to_str()).expect("utf8 filename").to_string(),
        );
    }
    files.sort();
    files
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
fn runtime_tests_are_classified_and_must_never_break_is_strict() {
    let root = repo_root();
    let ledger = load_ledger(&root);
    let tests = runtime_test_files(&root);

    let classes: BTreeSet<String> =
        ledger.classification_rules.iter().map(|rule| rule.class.clone()).collect();
    for required in ["critical", "useful", "shallow", "cosmetic", "duplicate"] {
        assert!(classes.contains(required), "missing class `{required}`");
    }

    let mut classified = BTreeMap::<String, String>::new();
    for file in &tests {
        let mut matched_class = None::<String>;
        for rule in &ledger.classification_rules {
            if rule.match_.iter().any(|pattern| glob_match(pattern, file)) {
                matched_class = Some(rule.class.clone());
                break;
            }
        }
        let class =
            matched_class.unwrap_or_else(|| panic!("unclassified runtime test file: {file}"));
        classified.insert(file.clone(), class);
    }

    for file in &ledger.must_never_break {
        assert!(tests.contains(file), "must-never-break file missing: {file}");
        let class = classified
            .get(file)
            .unwrap_or_else(|| panic!("must-never-break file not classified: {file}"));
        assert_ne!(class, "cosmetic");
        assert_ne!(class, "duplicate");
    }
}

#[test]
fn semantic_surfaces_and_trust_families_are_complete() {
    let root = repo_root();
    let ledger = load_ledger(&root);
    let tests = runtime_test_files(&root);

    assert!(
        !ledger.required_semantic_surfaces.is_empty(),
        "required_semantic_surfaces must not be empty"
    );
    for (surface, file) in &ledger.required_semantic_surfaces {
        assert!(
            tests.contains(file),
            "required semantic surface `{surface}` references missing file `{file}`"
        );
    }

    for (family, files) in &ledger.trust_coverage_families {
        assert!(!files.is_empty(), "trust family `{family}` must not be empty");
        for file in files {
            assert!(
                tests.contains(file),
                "trust family `{family}` references missing file `{file}`"
            );
        }
    }
}

#[test]
fn snapshot_assertions_are_restricted_to_allowlist() {
    let root = repo_root();
    let ledger = load_ledger(&root);

    let allowed: BTreeSet<String> = ledger
        .snapshot_surface_policy
        .allowed_assertion_files
        .iter()
        .map(|path| root.join(path).to_string_lossy().replace('\\', "/"))
        .collect();

    let mut files = Vec::new();
    collect_test_files(&root.join("crates"), &mut files);

    for file in files {
        let content = fs::read_to_string(&file).expect("test file read");
        let normalized = file.to_string_lossy().replace('\\', "/");
        for forbidden in &ledger.snapshot_surface_policy.forbidden_macros {
            if content.contains(forbidden) {
                assert!(
                    allowed.contains(&normalized),
                    "forbidden snapshot macro `{forbidden}` used outside allowlist: {normalized}"
                );
            }
        }
    }
}

#[test]
fn foundation_repo_suite_keeps_test_trust_maintenance_guard() {
    let root = repo_root();
    let repo_suites =
        fs::read_to_string(root.join("crates/bijux-dev/src/suites/repo.rs")).expect("repo suites");
    assert!(
        repo_suites.contains("\"test-trust-maintenance\""),
        "repo suite must keep test-trust-maintenance guard"
    );
}

fn collect_test_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).expect("dir read") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_test_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.components().any(|component| component.as_os_str() == "tests") {
            out.push(path);
        }
    }
}
