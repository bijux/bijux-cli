#![forbid(unsafe_code)]
//! Guardrails for Python wrapper intent coverage by Rust integration tests.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct EquivalenceInventory {
    entries: Vec<EquivalenceEntry>,
}

#[derive(Debug, Deserialize)]
struct EquivalenceEntry {
    python_test_file: String,
    coverage_level: String,
    rust_test_files: Vec<String>,
}

#[test]
fn every_python_e2e_test_file_is_mapped() {
    let root = repo_root();
    let inventory = load_inventory(&root);

    let mapped: BTreeSet<String> =
        inventory.entries.iter().map(|entry| entry.python_test_file.clone()).collect();
    let discovered = discover_python_e2e_tests(&root);

    assert_eq!(
        mapped, discovered,
        "python wrapper to rust equivalence mapping drifted; update inventory to cover the exact set of crates/bijux-cli-python/tests/python/test_*.py files"
    );
}

#[test]
fn mapped_rust_test_files_exist_and_match_coverage_policy() {
    let root = repo_root();
    let inventory = load_inventory(&root);
    let mut seen_python = BTreeSet::<String>::new();

    for entry in inventory.entries {
        assert!(
            seen_python.insert(entry.python_test_file.clone()),
            "duplicate python test entry in equivalence inventory: {}",
            entry.python_test_file
        );

        let python_path = root.join(&entry.python_test_file);
        assert!(
            python_path.is_file(),
            "mapped python e2e file does not exist: {}",
            entry.python_test_file
        );

        assert!(
            !entry.rust_test_files.is_empty(),
            "rust coverage list must not be empty for {}",
            entry.python_test_file
        );

        for rust_file in &entry.rust_test_files {
            assert!(
                rust_file.starts_with("crates/bijux-cli/tests/")
                    || rust_file.starts_with("crates/bijux-cli-python/tests/"),
                "rust coverage file must live under crates/bijux-cli/tests or crates/bijux-cli-python/tests: {rust_file}"
            );
            let rust_path = root.join(rust_file);
            assert!(rust_path.is_file(), "mapped rust test file does not exist: {rust_file}");
            assert!(
                rust_path.extension().is_some_and(|ext| ext == "rs"),
                "mapped rust test path must be a Rust test file: {rust_file}"
            );
        }

        let minimum = match entry.coverage_level.as_str() {
            "same" => 1,
            "better" => 2,
            other => panic!("unsupported coverage level `{other}` for {}", entry.python_test_file),
        };
        assert!(
            entry.rust_test_files.len() >= minimum,
            "coverage level `{}` requires at least {minimum} rust test files for {}",
            entry.coverage_level,
            entry.python_test_file
        );
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn load_inventory(root: &Path) -> EquivalenceInventory {
    let path = root.join(
        "crates/bijux-cli/tests/data/fixtures/coverage/python_e2e_equivalence_inventory.json",
    );
    let raw = std::fs::read_to_string(&path).expect("read python e2e equivalence inventory");
    serde_json::from_str(&raw).expect("parse python e2e equivalence inventory")
}

fn discover_python_e2e_tests(root: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::<String>::new();
    let start = root.join("crates/bijux-cli-python/tests/python");
    let mut stack = vec![start];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let is_test_py = path.extension().is_some_and(|ext| ext == "py")
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("test_"));
            if !is_test_py {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .expect("path should be under repo root")
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(rel);
        }
    }
    files
}
