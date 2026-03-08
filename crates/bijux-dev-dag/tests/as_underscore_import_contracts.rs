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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize)]
struct AsUnderscorePolicy {
    explicit_exceptions: Vec<AsUnderscoreException>,
}

#[derive(Debug, serde::Deserialize)]
struct AsUnderscoreException {
    path: String,
    reason: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|v| v.to_str())
                    .unwrap_or_default();
                if matches!(name, "target" | "artifacts" | ".git") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

fn is_allowed_path(rel: &str, explicit: &BTreeSet<String>) -> bool {
    if explicit.contains(rel) {
        return true;
    }
    if rel.contains("/tests/") || rel.contains("/benches/") {
        return true;
    }
    if !rel.starts_with("crates/") {
        return false;
    }
    rel.ends_with("/src/lib.rs") || rel.ends_with("/src/main.rs") || rel.contains("/src/bin/")
}

#[test]
fn as_underscore_imports_stay_within_allowed_contexts() {
    let root = repo_root();
    let policy: AsUnderscorePolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/as_underscore_import_policy.json"))
            .expect("read as underscore policy"),
    )
    .expect("parse as underscore policy");
    let explicit_paths: BTreeSet<String> = policy
        .explicit_exceptions
        .iter()
        .map(|entry| entry.path.clone())
        .collect();

    let mut files = Vec::new();
    collect_rs_files(&root.join("crates"), &mut files);

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&file).expect("read file");
        for (line_no, line) in content.lines().enumerate() {
            if !line.trim_start().starts_with("use ") || !line.contains(" as _;") {
                continue;
            }
            if !is_allowed_path(&rel, &explicit_paths) {
                violations.push(format!("{}:{} -> {}", rel, line_no + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`use ... as _;` found outside allowed contexts: {}",
        violations.join(" | ")
    );
}

#[test]
fn as_underscore_policy_exceptions_have_reasons() {
    let root = repo_root();
    let policy: AsUnderscorePolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/as_underscore_import_policy.json"))
            .expect("read as underscore policy"),
    )
    .expect("parse as underscore policy");

    for exception in policy.explicit_exceptions {
        assert!(
            !exception.reason.trim().is_empty(),
            "as-underscore exception must include a reason: {}",
            exception.path
        );
    }
}

#[test]
fn as_underscore_audit_report_stays_in_sync_with_crate_counts() {
    let root = repo_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/as_underscore_import_audit.md"))
            .expect("read as underscore audit report");

    let mut files = Vec::new();
    collect_rs_files(&root.join("crates"), &mut files);

    let mut by_crate: BTreeMap<String, usize> = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");
        let crate_name = rel.split('/').nth(1).unwrap_or_default().to_string();
        let content = fs::read_to_string(&file).expect("read file");
        let count = content
            .lines()
            .filter(|line| line.trim_start().starts_with("use ") && line.contains(" as _;"))
            .count();
        if count > 0 {
            *by_crate.entry(crate_name).or_insert(0) += count;
        }
    }

    for (crate_name, count) in by_crate {
        let needle = format!("| {} | {} |", crate_name, count);
        assert!(
            report.contains(&needle),
            "audit report missing or stale crate count row: {}",
            needle
        );
    }
}
