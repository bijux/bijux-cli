#![forbid(unsafe_code)]
//! Guard that command-tree diff report stays in sync with fixture inventories.

use std::collections::BTreeSet;
use std::fs;

use clap as _;
use proptest as _;
use schemars as _;
use semver as _;
use serde as _;
use serde_json as _;
use thiserror as _;

fn read_set(path: &str) -> BTreeSet<String> {
    fs::read_to_string(path)
        .expect("fixture should exist")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[test]
fn command_tree_diff_report_counts_match_fixtures() {
    let py = read_set("tests/data/fixtures/routing/python_documented_commands.txt");
    let rs = read_set("tests/data/fixtures/routing/rust_routed_root_commands.txt");

    let overlap = py.intersection(&rs).count();
    let py_only = py.difference(&rs).count();
    let rs_only = rs.difference(&py).count();

    let report = fs::read_to_string("../../docs/architecture/routing/command-tree-diff.md")
        .expect("diff report should exist");

    assert!(report.contains(&format!("Python documented root commands: {}", py.len())));
    assert!(report.contains(&format!("Rust routed root commands: {}", rs.len())));
    assert!(report.contains(&format!("Overlap: {overlap}")));
    assert!(report.contains(&format!("Python-only: {py_only}")));
    assert!(report.contains(&format!("Rust-only: {rs_only}")));
}
