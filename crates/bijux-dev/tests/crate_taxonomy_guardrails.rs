use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cargo_toml(path: &str) -> String {
    fs::read_to_string(root().join(path)).expect("read Cargo.toml")
}

#[test]
#[ignore = "legacy crate taxonomy contract assumes single bin ownership from pre-merge layout"]
fn only_cli_crate_declares_bin_target() {
    let crates_dir = root().join("crates");
    let mut offenders = Vec::new();

    for entry in fs::read_dir(&crates_dir).expect("read crates") {
        let entry = entry.expect("crate dir entry");
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let text = fs::read_to_string(&manifest).expect("read manifest");
        let has_bin = text.contains("[[bin]]");
        let is_cli = entry.file_name().to_string_lossy() == "bijux-dag-cli";
        if has_bin && !is_cli {
            offenders.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    assert!(offenders.is_empty(), "non-cli crates declare [[bin]]: {offenders:?}");
}

#[test]
fn core_and_artifacts_do_not_depend_on_clap_or_process_execution_crates() {
    let forbidden = ["clap", "assert_cmd", "duct", "xshell"];
    let manifests = ["crates/bijux-dag-core/Cargo.toml", "crates/bijux-dag-artifacts/Cargo.toml"];

    for manifest in manifests {
        let text = cargo_toml(manifest);
        for dep in forbidden {
            assert!(!text.contains(&format!("{dep} =")), "{manifest} must not depend on {dep}");
        }
    }
}

#[test]
fn dev_crate_does_not_depend_on_runtime_crates() {
    let text = cargo_toml("crates/bijux-dev/Cargo.toml");
    for forbidden in ["bijux-dag-app"] {
        assert!(!text.contains(forbidden), "bijux-dev-dag must not depend on {forbidden}");
    }
}

#[test]
fn cli_main_stays_thin_wiring_only() {
    let cli_main = root().join("crates/bijux-dag-cli/src/main.rs");
    let text = fs::read_to_string(cli_main).expect("read cli main");
    let lines = text.lines().count();
    assert!(lines <= 120, "cli main grew beyond thin wiring budget: {lines}");
    assert!(
        !text.contains("std::fs::") && !text.contains("Command::new("),
        "cli main must not contain business logic side-effect plumbing"
    );
}
