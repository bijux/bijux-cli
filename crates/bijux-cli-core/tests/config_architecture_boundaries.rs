#![forbid(unsafe_code)]
//! Architecture boundaries specific to config layering.

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_routing as _;
use clap as _;
use futures as _;
use serde_json as _;

use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

#[test]
fn bin_entrypoint_stays_free_of_config_business_logic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let main_rs = root.join("crates/bijux-cli-bin/src/main.rs");
    let text = read(main_rs.to_str().expect("utf-8 path"));

    assert!(!text.contains("cli config get"));
    assert!(!text.contains("cli config set"));
    assert!(!text.contains("run_config_migrations"));
    assert!(!text.contains("BIJUXCLI_CONFIG"));
}

#[test]
fn config_storage_stays_free_of_output_formatting_logic() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let storage_rs = root.join("bijux-cli-core/src/config/storage.rs");
    let text = read(storage_rs.to_str().expect("utf-8 path"));

    assert!(!text.contains("render_value"));
    assert!(!text.contains("EmitterConfig"));
    assert!(!text.contains("OutputFormat"));
    assert!(!text.contains("serde_json::json"));
}
