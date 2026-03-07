use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_files(root: &Path, rel: &str, out: &mut BTreeSet<String>) {
    let base = root.join(rel);
    if !base.exists() {
        return;
    }
    let mut stack = vec![base];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                let rel_path = path
                    .strip_prefix(root)
                    .expect("strip prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(rel_path);
            }
        }
    }
}

#[test]
fn evidence_governance_contract_enforces_ownership_and_freeze() {
    let root = repo_root();

    let policy_payload = fs::read_to_string(root.join("configs/policy/evidence_governance.json"))
        .expect("read evidence governance policy");
    let policy: Value = serde_json::from_str(&policy_payload).expect("parse evidence governance");

    let ledger_payload = fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
        .expect("read evidence ledger");
    let ledger: Value = serde_json::from_str(&ledger_payload).expect("parse evidence ledger");

    let managed_roots = policy["managed_roots"]
        .as_array()
        .expect("managed_roots array");
    let exempt_paths: BTreeSet<String> = policy["exempt_paths"]
        .as_array()
        .expect("exempt_paths array")
        .iter()
        .map(|value| value.as_str().expect("exempt path string").to_string())
        .collect();
    let allowed_decisions: BTreeSet<String> = policy["allowed_decisions"]
        .as_array()
        .expect("allowed_decisions array")
        .iter()
        .map(|value| value.as_str().expect("decision string").to_string())
        .collect();
    let allowed_classes: BTreeSet<String> = policy["allowed_evidence_classes"]
        .as_array()
        .expect("allowed classes array")
        .iter()
        .map(|value| value.as_str().expect("class string").to_string())
        .collect();

    let mut governed_files = BTreeSet::new();
    for root_entry in managed_roots {
        let rel = root_entry.as_str().expect("managed root string");
        collect_files(&root, rel, &mut governed_files);
    }

    let entries = ledger["entries"].as_array().expect("ledger entries array");
    let mut ledger_paths = BTreeSet::new();
    for entry in entries {
        let path = entry["path"].as_str().expect("entry path").to_string();
        let owner = entry["owner"].as_str().expect("entry owner");
        let class = entry["evidence_class"].as_str().expect("entry class");
        let trust = entry["trust_property"].as_str().expect("entry trust");
        let decision = entry["decision"].as_str().expect("entry decision");

        assert!(!owner.trim().is_empty(), "owner is empty for {path}");
        assert!(
            !trust.trim().is_empty(),
            "trust_property is empty for {path}"
        );
        assert!(
            allowed_classes.contains(class),
            "invalid evidence_class `{class}` for {path}"
        );
        assert!(
            allowed_decisions.contains(decision),
            "invalid decision `{decision}` for {path}"
        );
        assert!(
            root.join(&path).exists(),
            "ledger path does not exist: {path}"
        );
        ledger_paths.insert(path);
    }

    for rel in &governed_files {
        if exempt_paths.contains(rel) {
            continue;
        }
        assert!(
            ledger_paths.contains(rel),
            "governed evidence file missing ledger ownership entry: {rel}"
        );
    }

    for rel in &ledger_paths {
        assert!(
            governed_files.contains(rel) || exempt_paths.contains(rel),
            "ledger contains out-of-scope path not in managed roots or exemptions: {rel}"
        );
    }

    let fixture_families = ledger["fixture_families"]
        .as_array()
        .expect("fixture_families array");
    for family in fixture_families {
        let path = family["path"].as_str().expect("fixture family path");
        let status = family["status"].as_str().expect("fixture family status");
        let owner = family["owner"].as_str().expect("fixture family owner");
        assert!(
            root.join(path).exists(),
            "fixture family path does not exist: {path}"
        );
        assert!(
            status == "canonical" || status == "duplicate",
            "invalid fixture family status `{status}` for {path}"
        );
        assert!(
            !owner.trim().is_empty(),
            "fixture family owner is empty for {path}"
        );
    }
}
