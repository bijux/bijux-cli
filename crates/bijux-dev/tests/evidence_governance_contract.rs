use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use std::process::Command;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn evidence_ledger_path(root: &Path) -> PathBuf {
    let canonical = root.join("evidence/dag/ownership/evidence_ledger.json");
    if canonical.exists() {
        return canonical;
    }
    root.join("evidence/ownership/evidence_ledger.json")
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
                if is_transient_path(root, &path) {
                    continue;
                }
                stack.push(path);
            } else if path.is_file() {
                if is_transient_path(root, &path) {
                    continue;
                }
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

fn is_transient_component(component: &str) -> bool {
    matches!(
        component,
        ".git" | "target" | "artifacts" | "node_modules" | ".venv" | "venv" | "build" | "dist"
    )
}

fn is_transient_path(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    relative
        .components()
        .any(|component| component.as_os_str().to_str().is_some_and(is_transient_component))
}

fn tracked_json_files(root: &Path) -> Option<BTreeSet<String>> {
    let output =
        Command::new("git").args(["ls-files", "--", "*.json"]).current_dir(root).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    Some(files)
}

fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut index = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !pattern.starts_with('*') {
            if !text[index..].starts_with(part) {
                return false;
            }
            index += part.len();
            continue;
        }
        if i == parts.len() - 1 && !pattern.ends_with('*') {
            return text.ends_with(part);
        }
        if let Some(found) = text[index..].find(part) {
            index += found + part.len();
        } else {
            return false;
        }
    }
    true
}

#[test]
fn evidence_governance_contract_enforces_ownership_and_freeze() {
    let root = repo_root();
    let release_subset: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/battle_release_blocking_subset.json"))
            .expect("read battle release subset policy"),
    )
    .expect("parse battle release subset policy");
    let advisory_battle_paths: BTreeSet<String> = release_subset["advisory_scenarios"]
        .as_array()
        .expect("advisory_scenarios array")
        .iter()
        .map(|value| {
            let scenario = value.as_str().expect("advisory scenario id");
            format!(
                "evidence/battle/workflows/adversarial/{}.json",
                scenario.trim_start_matches("adversarial-").replace('-', "_")
            )
        })
        .collect();

    let policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_governance.json"))
            .expect("read evidence governance policy");
    let policy: Value = serde_json::from_str(&policy_payload).expect("parse evidence governance");

    let ledger_payload =
        fs::read_to_string(evidence_ledger_path(&root)).expect("read evidence ledger");
    let ledger: Value = serde_json::from_str(&ledger_payload).expect("parse evidence ledger");

    let managed_roots = policy["managed_roots"].as_array().expect("managed_roots array");
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
    let allowed_impl_status: BTreeSet<String> = policy["allowed_implementation_statuses"]
        .as_array()
        .expect("allowed_implementation_statuses array")
        .iter()
        .map(|value| value.as_str().expect("implementation status string").to_string())
        .collect();
    let forbidden_globs: Vec<String> = policy["forbidden_globs"]
        .as_array()
        .expect("forbidden_globs array")
        .iter()
        .map(|value| value.as_str().expect("forbidden glob string").to_string())
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
            assert!(map.contains_key(field), "entry missing required field `{field}`");
        }
        let path = entry["path"].as_str().expect("entry path").to_string();
        let owner = entry["owner"].as_str().expect("entry owner");
        let class = entry["evidence_class"].as_str().expect("entry class");
        let trust = entry["trust_property"].as_str().expect("entry trust");
        let canonical_location =
            entry["canonical_location"].as_str().expect("entry canonical_location");
        let consumer_surfaces =
            entry["consumer_surfaces"].as_array().expect("entry consumer_surfaces array");
        let trust_properties_protected = entry["trust_properties_protected"]
            .as_array()
            .expect("entry trust_properties_protected array");
        let implementation_status =
            entry["implementation_status"].as_str().expect("entry implementation_status");
        let release_blocking =
            entry["release_blocking"].as_bool().expect("entry release_blocking bool");
        let duplicate_of = &entry["duplicate_of"];
        let retirement_date = &entry["retirement_date"];
        let why_exists = entry["why_exists"].as_str().expect("entry why_exists");
        let deletion_review = entry["deletion_review"].as_str().expect("entry deletion_review");
        let decision = entry["decision"].as_str().expect("entry decision");

        assert!(!owner.trim().is_empty(), "owner is empty for {path}");
        assert!(!trust.trim().is_empty(), "trust_property is empty for {path}");
        assert!(!why_exists.trim().is_empty(), "why_exists is empty for {path}");
        assert!(!deletion_review.trim().is_empty(), "deletion_review is empty for {path}");
        assert!(!canonical_location.trim().is_empty(), "canonical_location is empty for {path}");
        assert!(!consumer_surfaces.is_empty(), "consumer_surfaces is empty for {path}");
        assert!(
            !trust_properties_protected.is_empty(),
            "trust_properties_protected is empty for {path}"
        );
        assert!(
            allowed_impl_status.contains(implementation_status),
            "invalid implementation_status `{implementation_status}` for {path}"
        );
        if class == "battle" {
            if advisory_battle_paths.contains(&path) {
                assert!(
                    !release_blocking,
                    "advisory battle evidence must not be release_blocking for {path}"
                );
            } else {
                assert!(release_blocking, "battle evidence must be release_blocking for {path}");
            }
        }
        match duplicate_of {
            Value::Null => {}
            Value::String(value) => {
                assert!(!value.trim().is_empty(), "duplicate_of cannot be empty string for {path}")
            }
            _ => panic!("duplicate_of must be string or null for {path}"),
        }
        match retirement_date {
            Value::Null => {}
            Value::String(value) => assert!(
                !value.trim().is_empty(),
                "retirement_date cannot be empty string for {path}"
            ),
            _ => panic!("retirement_date must be string or null for {path}"),
        }
        assert!(allowed_classes.contains(class), "invalid evidence_class `{class}` for {path}");
        assert!(allowed_decisions.contains(decision), "invalid decision `{decision}` for {path}");
        assert!(root.join(&path).exists(), "ledger path does not exist: {path}");
        ledger_paths.insert(path);
    }

    let asset_families = ledger["asset_families"].as_array().expect("asset_families array");
    for family in asset_families {
        let family_id = family["family_id"].as_str().expect("asset family id");
        let version = family["version"].as_str().expect("asset family version");
        let owner = family["owner"].as_str().expect("asset family owner");
        let trust_property =
            family["trust_property_protected"].as_str().expect("asset family trust property");
        let canonical_location =
            family["canonical_location"].as_str().expect("asset family canonical location");
        let consumer_surfaces =
            family["consumer_surfaces"].as_array().expect("asset family consumer surfaces");
        let implementation_status =
            family["implementation_status"].as_str().expect("asset family implementation status");
        let release_blocking =
            family["release_blocking"].as_bool().expect("asset family release_blocking");
        assert!(!family_id.trim().is_empty(), "asset family id is empty");
        assert!(!version.trim().is_empty(), "asset family version is empty");
        assert!(!owner.trim().is_empty(), "asset family owner is empty");
        assert!(
            !trust_property.trim().is_empty(),
            "asset family trust_property_protected is empty for {family_id}"
        );
        assert!(
            !consumer_surfaces.is_empty(),
            "asset family consumer_surfaces is empty for {family_id}"
        );
        assert!(
            root.join(canonical_location).is_dir(),
            "asset family canonical location does not exist: {canonical_location}"
        );
        assert!(
            allowed_impl_status.contains(implementation_status),
            "invalid asset family implementation_status `{implementation_status}` for {family_id}"
        );
        if family_id == "battle" {
            assert!(release_blocking, "battle asset family must be release_blocking");
        }
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

    let fixture_families = ledger["fixture_families"].as_array().expect("fixture_families array");
    for family in fixture_families {
        let path = family["path"].as_str().expect("fixture family path");
        let status = family["status"].as_str().expect("fixture family status");
        let version = family["version"].as_str().expect("fixture family version");
        let owner = family["owner"].as_str().expect("fixture family owner");
        let canonical_location =
            family["canonical_location"].as_str().expect("fixture family canonical location");
        let consumer_surfaces =
            family["consumer_surfaces"].as_array().expect("fixture family consumer_surfaces");
        let trust_property_protected = family["trust_property_protected"]
            .as_str()
            .expect("fixture family trust_property_protected");
        let implementation_status =
            family["implementation_status"].as_str().expect("fixture family implementation_status");
        let _release_blocking =
            family["release_blocking"].as_bool().expect("fixture family release_blocking bool");
        let duplicate_of = &family["duplicate_of"];
        let retirement_date = &family["retirement_date"];
        assert!(root.join(path).exists(), "fixture family path does not exist: {path}");
        assert!(!version.trim().is_empty(), "fixture family version is empty for {path}");
        assert!(
            status == "canonical" || status == "duplicate",
            "invalid fixture family status `{status}` for {path}"
        );
        assert!(!owner.trim().is_empty(), "fixture family owner is empty for {path}");
        assert!(
            !canonical_location.trim().is_empty(),
            "fixture family canonical location is empty for {path}"
        );
        assert!(
            !consumer_surfaces.is_empty(),
            "fixture family consumer_surfaces is empty for {path}"
        );
        assert!(
            !trust_property_protected.trim().is_empty(),
            "fixture family trust_property_protected is empty for {path}"
        );
        assert!(
            allowed_impl_status.contains(implementation_status),
            "invalid fixture family implementation_status `{implementation_status}` for {path}"
        );
        match duplicate_of {
            Value::Null => {}
            Value::String(value) => assert!(
                !value.trim().is_empty(),
                "fixture family duplicate_of cannot be empty for {path}"
            ),
            _ => panic!("fixture family duplicate_of must be string or null for {path}"),
        }
        match retirement_date {
            Value::Null => {}
            Value::String(value) => assert!(
                !value.trim().is_empty(),
                "fixture family retirement_date cannot be empty for {path}"
            ),
            _ => panic!("fixture family retirement_date must be string or null for {path}"),
        }
    }

    let path_policy_payload =
        fs::read_to_string(root.join("configs/dag/policy/evidence_path_policy.json"))
            .expect("read evidence path policy");
    let path_policy: Value =
        serde_json::from_str(&path_policy_payload).expect("parse evidence path policy");
    let governed_roots: Vec<String> = path_policy["governed_roots"]
        .as_array()
        .expect("governed_roots array")
        .iter()
        .map(|value| value.as_str().expect("governed root string").to_string())
        .collect();
    let schema_fixture_roots: Vec<String> = path_policy["schema_fixture_roots"]
        .as_array()
        .expect("schema_fixture_roots array")
        .iter()
        .map(|value| value.as_str().expect("schema fixture root string").to_string())
        .collect();
    let legacy_scenario_roots: Vec<String> = path_policy["legacy_scenario_roots"]
        .as_array()
        .expect("legacy_scenario_roots array")
        .iter()
        .map(|value| value.as_str().expect("legacy scenario root string").to_string())
        .collect();
    let helper_allowlist: Vec<String> = path_policy["helper_allowlist"]
        .as_array()
        .expect("helper_allowlist array")
        .iter()
        .map(|value| value.as_str().expect("helper allowlist pattern").to_string())
        .collect();

    let all_files = tracked_json_files(&root).unwrap_or_else(|| {
        let mut files = BTreeSet::new();
        collect_files(&root, ".", &mut files);
        files
    });
    for rel in all_files.into_iter().filter(|path| path.ends_with(".json")) {
        let in_governed_root = governed_roots.iter().any(|governed_root| {
            rel == *governed_root || rel.starts_with(&format!("{governed_root}/"))
        });
        let in_schema_fixture_root = schema_fixture_roots
            .iter()
            .any(|schema_root| rel == *schema_root || rel.starts_with(&format!("{schema_root}/")));
        let in_helper_allowlist = helper_allowlist.iter().any(|pattern| glob_match(pattern, &rel));
        let in_legacy_scenario_root = legacy_scenario_roots
            .iter()
            .any(|legacy_root| rel == *legacy_root || rel.starts_with(&format!("{legacy_root}/")));
        if in_governed_root
            || in_schema_fixture_root
            || in_helper_allowlist
            || in_legacy_scenario_root
        {
            continue;
        }
        let is_scenario_like = rel.ends_with(".dag.json")
            || rel.contains("/scenarios/")
            || rel.contains("/fixtures/")
            || rel.starts_with("examples/");
        if is_scenario_like {
            panic!("scenario-like json path outside evidence-governed roots is forbidden: {rel}");
        }
        if forbidden_globs.iter().any(|pattern| glob_match(pattern, &rel)) {
            panic!("path is forbidden by evidence governance freeze policy: {rel}");
        }
        if rel.starts_with("tests/authoring/examples/") || rel.starts_with("tests/authoring/bad/") {
            panic!("authoring evidence outside evidence/authoring is forbidden: {rel}");
        }
    }
}
