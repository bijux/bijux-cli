#![forbid(unsafe_code)]
//! Module layout contracts for durable dev-cli crate architecture.

use std::path::Path;

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
