#![forbid(unsafe_code)]
//! Enforces allowed dependency edges between internal Rust crates.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

use anyhow as _;
use bijux_cli as _;
use bijux_dev_cli as _;
use clap as _;
use futures as _;
use serde_json::Value;

fn is_internal_workspace_crate(name: &str) -> bool {
    name == "bijux-cli" || name == "bijux-dev-cli" || name.starts_with("bijux-cli-")
}

fn internal_workspace_deps(pkg: &Value) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    let Some(dep_items) = pkg.get("dependencies").and_then(Value::as_array) else {
        return deps;
    };

    for dep in dep_items {
        if !dep.get("kind").is_none_or(Value::is_null) {
            continue;
        }
        let Some(name) = dep.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !is_internal_workspace_crate(name) {
            continue;
        }
        if dep.get("path").is_some() {
            deps.insert(name.to_string());
        }
    }

    deps
}

#[test]
fn enforces_internal_crate_boundaries() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            manifest.to_str().expect("workspace manifest path must be valid UTF-8"),
        ])
        .output()
        .expect("cargo metadata command must execute");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let root: Value = serde_json::from_slice(&output.stdout).expect("valid metadata JSON");
    let packages =
        root.get("packages").and_then(Value::as_array).expect("metadata contains packages");

    let expected: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::from([
        ("bijux-dev-cli", BTreeSet::from(["bijux-cli-evidence"])),
        ("bijux-cli", BTreeSet::from(["bijux-dev-cli"])),
        ("bijux-cli-evidence", BTreeSet::new()),
        ("bijux-cli-python", BTreeSet::from(["bijux-cli"])),
    ]);
    let mut observed_internal_packages = BTreeSet::new();

    for pkg in packages {
        let Some(name) = pkg.get("name").and_then(Value::as_str) else {
            continue;
        };
        if !is_internal_workspace_crate(name) {
            continue;
        }
        observed_internal_packages.insert(name.to_string());

        let Some(expected_deps) = expected.get(name) else {
            panic!("unexpected internal package present in workspace: {name}");
        };

        let observed = internal_workspace_deps(pkg);
        let expected_owned: BTreeSet<String> =
            expected_deps.iter().map(|item| (*item).to_string()).collect();

        assert_eq!(
            observed, expected_owned,
            "boundary mismatch for {name}: observed {observed:?}, expected {expected_owned:?}"
        );
    }

    let expected_internal_packages: BTreeSet<String> =
        expected.keys().map(|name| (*name).to_string()).collect();
    assert_eq!(
        observed_internal_packages, expected_internal_packages,
        "workspace package set drifted; boundary map must be updated intentionally"
    );
}
