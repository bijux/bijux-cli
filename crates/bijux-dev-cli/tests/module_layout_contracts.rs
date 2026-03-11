#![forbid(unsafe_code)]
//! Module layout contracts for durable dev-cli crate architecture.

use std::path::Path;
use std::{fs, path::PathBuf};

#[test]
fn command_and_status_contract_namespaces_exist() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(crate_root.join("src/commands").is_dir());
    assert!(crate_root.join("src/status_contracts").is_dir());
}

#[test]
fn legacy_native_directory_names_are_removed() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(!crate_root.join("src/contracts/maintenance/native/catalog").exists());
    assert!(!crate_root.join("src/contracts/maintenance/native/handlers").exists());
}

#[test]
fn feature_module_is_alias_only() {
    let features_mod = include_str!("../src/features/mod.rs");
    assert!(features_mod.contains("Compatibility aliases"));
    assert!(features_mod.contains("pub use crate::commands::status;"));
}

#[test]
fn native_contract_modules_use_spec_and_executor_naming() {
    let native_mod = include_str!("../src/contracts/maintenance/native/mod.rs");
    assert!(native_mod.contains("mod executors;"));
    assert!(native_mod.contains("mod specs;"));
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn workspace_root_scripts_directory_is_removed_and_name_is_blocked() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = crate_root.join("..").join("..");
    assert!(
        !workspace_root.join("scripts").exists(),
        "workspace root must not define scripts/ directory"
    );

    let mut rs_files = Vec::new();
    collect_rs_files(&crate_root.join("src"), &mut rs_files);
    collect_rs_files(&crate_root.join("tests"), &mut rs_files);

    let this_file = crate_root.join("tests/module_layout_contracts.rs");
    for file in rs_files {
        if file == this_file {
            continue;
        }
        let text = fs::read_to_string(&file).unwrap_or_default();
        assert!(
            !text.contains("scripts"),
            "forbidden legacy token `scripts` found in {}",
            file.display()
        );
    }
}
