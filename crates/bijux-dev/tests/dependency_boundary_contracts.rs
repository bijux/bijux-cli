use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn direct_dependency_names(crate_name: &str) -> BTreeSet<String> {
    let root = repo_root();
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(&root)
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    let payload: Value = serde_json::from_slice(&output.stdout).expect("parse metadata json");
    let packages = payload["packages"].as_array().expect("packages array");
    let package = packages
        .iter()
        .find(|pkg| pkg["name"].as_str() == Some(crate_name))
        .expect("package in metadata");
    package["dependencies"]
        .as_array()
        .expect("dependencies array")
        .iter()
        .filter(|dep| dep["kind"].is_null())
        .filter_map(|dep| dep["name"].as_str().map(|v| v.to_string()))
        .collect()
}

#[test]
fn kernel_scope_does_not_reference_app_or_governance_crates() {
    let root = repo_root();
    let mut stack = vec![root.join("crates/bijux-dag-runtime/src/runtime_core")];
    let forbidden = ["bijux_dag_app", "bijux_dev_dag", "bijux_dag_cli"];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let body = fs::read_to_string(&path).expect("read source");
            for token in forbidden {
                assert!(
                    !body.contains(token),
                    "kernel source references forbidden crate token `{token}`: {}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn app_source_does_not_import_runtime_internals() {
    let root = repo_root();
    let mut stack = vec![root.join("crates/bijux-dag-app/src")];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let body = fs::read_to_string(&path).expect("read source");
            assert!(
                !body.contains("runtime_core::")
                    && !body.contains("bijux_dag_runtime::runtime_core"),
                "app source imports runtime internals: {}",
                path.display()
            );
        }
    }
}

#[test]
fn core_dependencies_match_kernel_allowed_list() {
    let allowed: BTreeSet<String> = [
        "criterion",
        "hex",
        "serde",
        "serde_json",
        "serde_yaml",
        "sha2",
        "tempfile",
        "thiserror",
        "unicode-normalization",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let deps = direct_dependency_names("bijux-dag-core");
    for dep in &deps {
        assert!(
            allowed.contains(dep),
            "bijux-dag-core direct dependency is outside kernel allowlist: {dep}"
        );
    }
}

#[test]
fn runtime_dependencies_match_runtime_allowed_list() {
    let allowed: BTreeSet<String> = [
        "base64",
        "bijux-dag-artifacts",
        "bijux-dag-core",
        "chrono",
        "chrono-tz",
        "croner",
        "ctrlc",
        "hex",
        "flate2",
        "reqwest",
        "serde",
        "serde_json",
        "sha2",
        "thiserror",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let deps = direct_dependency_names("bijux-dag-runtime");
    for dep in &deps {
        assert!(
            allowed.contains(dep),
            "bijux-dag-runtime direct dependency is outside runtime allowlist: {dep}"
        );
    }
}

#[test]
fn dev_governance_dependencies_match_allowed_list() {
    let allowed: BTreeSet<String> = [
        "anyhow",
        "bijux-cli",
        "bijux-dag-app",
        "bijux-dag-artifacts",
        "bijux-dag-core",
        "bijux-dag-runtime",
        "bijux-dag-testkit",
        "clap",
        "hex",
        "serde",
        "serde_json",
        "sha2",
        "tempfile",
        "toml",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let deps = direct_dependency_names("bijux-dev");
    for dep in &deps {
        assert!(
            allowed.contains(dep),
            "bijux-dev direct dependency is outside governance allowlist: {dep}"
        );
    }
}

#[test]
fn kernel_public_symbols_do_not_use_modeled_or_future_naming() {
    let root = repo_root();
    let mut dirs = vec![
        root.join("crates/bijux-dag-core/src"),
        root.join("crates/bijux-dag-runtime/src/runtime_core"),
    ];
    let forbidden = [
        "simulated",
        "modeled",
        "aspirational",
        "future",
        "speculative",
        "control_plane",
        "ai_operator",
    ];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            for line in fs::read_to_string(&path).expect("read source").lines() {
                let trimmed = line.trim_start();
                if !(trimmed.starts_with("pub ") || trimmed.starts_with("pub(")) {
                    continue;
                }
                let lower = trimmed.to_ascii_lowercase();
                for token in forbidden {
                    assert!(
                        !lower.contains(token),
                        "kernel-adjacent public symbol uses modeled/future naming `{token}` in {}: {}",
                        path.display(),
                        trimmed
                    );
                }
            }
        }
    }
}
