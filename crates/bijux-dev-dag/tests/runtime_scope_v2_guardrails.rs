use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn runtime_scope_v2_tracks_all_runtime_modules_and_freezes_top_level_layout() {
    let root = repo_root();
    let runtime_src = root.join("crates/bijux-dag-runtime/src");

    let policy_payload = fs::read_to_string(root.join("configs/policy/runtime_scope_v2.json"))
        .expect("read runtime scope policy");
    let policy: Value = serde_json::from_str(&policy_payload).expect("parse runtime scope policy");

    let doc_payload = fs::read_to_string(root.join("docs/architecture/runtime_scope_v2.md"))
        .expect("read runtime scope v2 doc");
    assert!(
        doc_payload.contains("# Runtime Scope v2"),
        "runtime scope document title is missing"
    );

    let allowed_dirs: BTreeSet<String> = policy["allowed_top_level_dirs"]
        .as_array()
        .expect("allowed_top_level_dirs array")
        .iter()
        .map(|value| value.as_str().expect("allowed dir string").to_string())
        .collect();
    let allowed_files: BTreeSet<String> = policy["allowed_top_level_files"]
        .as_array()
        .expect("allowed_top_level_files array")
        .iter()
        .map(|value| value.as_str().expect("allowed file string").to_string())
        .collect();

    for entry in fs::read_dir(&runtime_src).expect("read runtime src") {
        let path = entry.expect("runtime src entry").path();
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .expect("runtime src file name")
            .to_string();
        if path.is_dir() {
            assert!(
                allowed_dirs.contains(&name),
                "runtime top-level directory is not allowed by scope freeze: {name}"
            );
        } else if path.is_file() {
            assert!(
                allowed_files.contains(&name),
                "runtime top-level file is not allowed by scope freeze: {name}"
            );
        }
    }

    let entries = policy["module_entries"]
        .as_array()
        .expect("module_entries array");
    let mut tracked = BTreeSet::new();
    let mut hard_keep = BTreeSet::new();
    for entry in entries {
        let module = entry["module"].as_str().expect("module path");
        let decision = entry["decision"].as_str().expect("module decision");
        let class = entry["classification"].as_str().expect("module class");
        let rationale = entry["rationale"].as_str().expect("module rationale");
        assert!(
            !rationale.trim().is_empty(),
            "rationale is empty for {module}"
        );
        assert!(
            matches!(decision, "keep" | "move" | "delete"),
            "invalid decision `{decision}` for {module}"
        );
        assert!(
            matches!(
                class,
                "core-runtime"
                    | "backend"
                    | "policy"
                    | "diagnostics"
                    | "replay"
                    | "security"
                    | "support"
                    | "speculative"
                    | "wrong-crate"
            ),
            "invalid classification `{class}` for {module}"
        );
        assert!(
            runtime_src.join(module).exists(),
            "policy references missing runtime module file: {module}"
        );
        tracked.insert(module.to_string());
        if entry["hard_keep"].as_bool().unwrap_or(false) {
            assert_eq!(
                decision, "keep",
                "hard_keep module must have keep decision: {module}"
            );
            hard_keep.insert(module.to_string());
        }
    }

    for module in runtime_src
        .read_dir()
        .expect("list runtime src")
        .flat_map(|entry| {
            let path = entry.expect("runtime src entry").path();
            if path.is_file() {
                if path.extension().and_then(|v| v.to_str()) == Some("rs") {
                    return vec![path];
                }
                return Vec::new();
            }
            let mut out = Vec::new();
            if path.is_dir() {
                let mut stack = vec![path];
                while let Some(dir) = stack.pop() {
                    for nested in fs::read_dir(dir).expect("walk runtime src dir") {
                        let nested = nested.expect("nested entry").path();
                        if nested.is_dir() {
                            stack.push(nested);
                        } else if nested.extension().and_then(|v| v.to_str()) == Some("rs") {
                            out.push(nested);
                        }
                    }
                }
            }
            out
        })
    {
        let rel = module
            .strip_prefix(&runtime_src)
            .expect("strip runtime prefix")
            .to_string_lossy()
            .replace('\\', "/");
        if rel == "lib.rs" {
            continue;
        }
        assert!(
            tracked.contains(&rel),
            "runtime module missing from runtime_scope_v2 policy inventory: {rel}"
        );
    }

    let hard_keep_modules: BTreeSet<String> = policy["hard_keep_modules"]
        .as_array()
        .expect("hard_keep_modules array")
        .iter()
        .map(|value| value.as_str().expect("hard keep module string").to_string())
        .collect();
    assert_eq!(
        hard_keep, hard_keep_modules,
        "hard_keep_modules list must match module_entries hard_keep flags"
    );

    let named = policy["named_decisions"]
        .as_object()
        .expect("named_decisions object");
    let named_expected: BTreeMap<&str, &str> = BTreeMap::from([
        ("geo_federation", "move"),
        ("ha_scheduler", "move"),
        ("federated_scheduling", "move"),
        ("control_plane_api", "move"),
        ("operations_governance", "move"),
        ("adaptive_scheduler", "move"),
        ("cost_optimization", "move"),
        ("dataset_semantics", "move"),
        ("formal_verification", "keep"),
        ("ai_operator_assist", "move"),
        ("workflow_product", "move"),
        ("tenancy", "move"),
        ("provenance_compliance", "move"),
        ("supply_chain_trust", "move"),
    ]);
    for (key, expected) in named_expected {
        let actual = named
            .get(key)
            .and_then(Value::as_str)
            .expect("named decision value");
        assert_eq!(actual, expected, "unexpected decision for {key}");
    }
}
