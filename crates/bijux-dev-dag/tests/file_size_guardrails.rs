use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use clap as _;
use hex as _;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use sha2 as _;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct SourceLayoutPolicy {
    global: SourceLayoutGlobal,
    transitional_ceiling: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Deserialize)]
struct SourceLayoutGlobal {
    max_rust_source_lines: usize,
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .expect("read file")
        .lines()
        .count()
}

#[test]
fn source_files_stay_under_size_budget() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let policy_path = root.join("configs/policy/source_layout.json");
    let policy_text = fs::read_to_string(&policy_path).expect("read source layout policy");
    let policy: SourceLayoutPolicy =
        serde_json::from_str(&policy_text).expect("parse source layout policy");

    let mut violations = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
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

    assert!(violations.is_empty(), "{}", violations.join(", "));
}
