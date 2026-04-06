use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde::Deserialize;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct SourceLayoutPolicy {
    global: SourceLayoutGlobal,
    transitional_ceiling: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Deserialize)]
struct SourceLayoutGlobal {
    max_rust_source_lines: usize,
    warning_rust_source_lines: Option<usize>,
    hard_max_rust_source_lines: Option<usize>,
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path).expect("read file").lines().count()
}

#[test]
fn source_files_stay_under_size_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy_path = root.join("configs/dag/policy/source_layout.json");
    let policy_text = fs::read_to_string(&policy_path).expect("read source layout policy");
    let policy: SourceLayoutPolicy =
        serde_json::from_str(&policy_text).expect("parse source layout policy");
    let warning_budget = policy.global.warning_rust_source_lines.unwrap_or(9000);
    let hard_budget = policy.global.hard_max_rust_source_lines.unwrap_or(10000);

    let mut violations = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("");
                if matches!(name, "target" | "artifacts") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .expect("strip prefix")
                .to_string_lossy()
                .replace('\\', "/");
            let count = line_count(&path);
            if count > warning_budget {
                eprintln!(
                    "warning: rust source exceeds warning LOC budget: {rel} has {count} lines (warning {warning_budget})"
                );
            }
            assert!(count <= hard_budget, "{rel} has {count} lines (hard max {hard_budget})");
            let max_lines = policy
                .transitional_ceiling
                .get(&rel)
                .copied()
                .unwrap_or(policy.global.max_rust_source_lines);
            if count > max_lines {
                violations.push(format!("{rel} has {count} lines (max {max_lines})"));
            }
        }
    }

    if !violations.is_empty() {
        eprintln!(
            "warning: transitional rust source size ceilings exceeded: {}",
            violations.join(", ")
        );
    }
}
