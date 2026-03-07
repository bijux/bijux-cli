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
    let required_fields: BTreeSet<String> = policy["required_metadata_fields"]
        .as_array()
        .expect("required_metadata_fields array")
        .iter()
        .map(|value| value.as_str().expect("required field string").to_string())
        .collect();

    let mut governed_files = BTreeSet::new();
    for root_entry in managed_roots {
        let rel = root_entry.as_str().expect("managed root string");
        collect_files(&root, rel, &mut governed_files);
    }

    let entries = ledger["entries"].as_array().expect("ledger entries array");
    let mut ledger_paths = BTreeSet::new();
    for entry in entries {
        let map = entry.as_object().expect("entry object");
        for field in &required_fields {
            assert!(
                map.contains_key(field),
                "entry missing required field `{field}`"
            );
        }
        let path = entry["path"].as_str().expect("entry path").to_string();
        let owner = entry["owner"].as_str().expect("entry owner");
        let class = entry["evidence_class"].as_str().expect("entry class");
        let trust = entry["trust_property"].as_str().expect("entry trust");
        let why_exists = entry["why_exists"].as_str().expect("entry why_exists");
        let deletion_review = entry["deletion_review"]
            .as_str()
            .expect("entry deletion_review");
        let decision = entry["decision"].as_str().expect("entry decision");

        assert!(!owner.trim().is_empty(), "owner is empty for {path}");
        assert!(
            !trust.trim().is_empty(),
            "trust_property is empty for {path}"
        );
        assert!(
            !why_exists.trim().is_empty(),
            "why_exists is empty for {path}"
        );
        assert!(
            !deletion_review.trim().is_empty(),
            "deletion_review is empty for {path}"
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

    let comparison_scenarios = root.join("comparisons/scenarios");
    let mut scenario_ids = BTreeSet::new();
    for entry in fs::read_dir(&comparison_scenarios).expect("read comparisons scenarios") {
        let path = entry.expect("scenario entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(path).expect("read comparison scenario");
        let json: Value = serde_json::from_str(&payload).expect("parse comparison scenario");
        let id = json["id"].as_str().expect("comparison scenario id");
        scenario_ids.insert(id.to_string());
    }

    let coverage_payload = fs::read_to_string(root.join("comparisons/external/coverage_map.json"))
        .expect("read external coverage map");
    let coverage: Value = serde_json::from_str(&coverage_payload).expect("parse coverage map");
    for comparator in coverage["comparators"]
        .as_array()
        .expect("coverage comparators array")
    {
        let note = comparator["note"].as_str().expect("coverage note path");
        assert!(root.join(note).exists(), "external note missing: {note}");
        let linked = comparator["linked_scenarios"]
            .as_array()
            .expect("linked_scenarios array");
        assert!(!linked.is_empty(), "linked_scenarios is empty for {note}");
        for id in linked {
            let id = id.as_str().expect("linked scenario id");
            assert!(
                scenario_ids.contains(id),
                "external note `{note}` references missing scenario id `{id}`"
            );
        }
        let baseline = comparator["bijux_baseline"]
            .as_str()
            .expect("coverage baseline path");
        assert!(
            root.join(baseline).exists(),
            "coverage baseline missing: {baseline}"
        );
    }
}
