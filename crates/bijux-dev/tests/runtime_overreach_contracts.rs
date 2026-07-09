use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
struct RuntimeOverreachPolicy {
    overreach_modules: Vec<OverreachModule>,
}

#[derive(Debug, serde::Deserialize)]
struct OverreachModule {
    module: String,
    decision: String,
    owner: String,
    reason: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn overreach_policy_entries_are_complete_and_valid() {
    let root = repo_root();
    let policy: RuntimeOverreachPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/runtime_overreach_policy.json"))
            .expect("read runtime overreach policy"),
    )
    .expect("parse runtime overreach policy");

    assert!(
        !policy.overreach_modules.is_empty(),
        "runtime overreach policy must include overreach modules"
    );

    let mut seen = BTreeSet::new();
    for entry in policy.overreach_modules {
        assert!(
            seen.insert(entry.module.clone()),
            "duplicate overreach module entry: {}",
            entry.module
        );
        assert!(
            matches!(entry.decision.as_str(), "move" | "retain"),
            "overreach entry decision must be move|retain: {} -> {}",
            entry.module,
            entry.decision
        );
        assert!(
            !entry.owner.trim().is_empty(),
            "overreach entry owner must be set: {}",
            entry.module
        );
        assert!(
            !entry.reason.trim().is_empty(),
            "overreach entry reason must be set: {}",
            entry.module
        );
        let module_path = root.join("crates/bijux-dag-runtime/src").join(&entry.module);
        assert!(
            module_path.exists(),
            "runtime overreach module path missing: {}",
            module_path.display()
        );
    }
}

#[test]
fn release_evidence_set_does_not_depend_on_overreach_modules() {
    let root = repo_root();
    let policy: RuntimeOverreachPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/runtime_overreach_policy.json"))
            .expect("read runtime overreach policy"),
    )
    .expect("parse runtime overreach policy");

    let overreach_tokens: Vec<String> =
        policy.overreach_modules.into_iter().map(|entry| entry.module.replace('/', "::")).collect();

    let release = fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
        .expect("read release evidence set");
    let mut violations = Vec::new();
    for token in &overreach_tokens {
        if release.contains(token) {
            violations.push(token.clone());
        }
    }

    assert!(
        violations.is_empty(),
        "release evidence set must not depend on runtime overreach modules: {}",
        violations.join(", ")
    );
}

#[test]
fn runtime_scope_policy_marks_overreach_modules_as_move_or_retain() {
    let root = repo_root();
    let policy: RuntimeOverreachPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/runtime_overreach_policy.json"))
            .expect("read runtime overreach policy"),
    )
    .expect("parse runtime overreach policy");

    let runtime_scope: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/runtime_scope_v2.json"))
            .expect("read runtime scope policy"),
    )
    .expect("parse runtime scope policy");
    let module_entries = runtime_scope
        .get("module_entries")
        .and_then(serde_json::Value::as_array)
        .expect("runtime scope module_entries");
    let named_decisions = runtime_scope
        .get("named_decisions")
        .and_then(serde_json::Value::as_object)
        .expect("runtime scope named_decisions");

    for entry in policy.overreach_modules {
        let entry_scope_decision = module_entries.iter().find_map(|value| {
            let module = value.get("module")?.as_str()?;
            if module == entry.module {
                value.get("decision")?.as_str().map(str::to_string)
            } else {
                None
            }
        });
        assert!(
            entry_scope_decision.is_some(),
            "runtime scope missing module_entries decision for overreach module: {}",
            entry.module
        );
        let expected_scope_decision =
            if entry.decision == "retain" { "keep" } else { entry.decision.as_str() };
        assert_eq!(
            entry_scope_decision.as_deref(),
            Some(expected_scope_decision),
            "runtime scope decision mismatch for overreach module: {}",
            entry.module
        );

        let key = entry.module.rsplit('/').next().expect("module filename");
        let key = key.trim_end_matches(".rs");
        if let Some(named_decision) = named_decisions.get(key).and_then(serde_json::Value::as_str) {
            assert_eq!(
                named_decision, expected_scope_decision,
                "runtime scope named_decisions mismatch for overreach module key: {key}"
            );
        }
    }
}
