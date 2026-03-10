#![forbid(unsafe_code)]
//! Architecture boundaries for runtime query providers used by dev-cli.

use std::fs;
use std::path::{Path, PathBuf};

fn rs_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn runtime_crates_do_not_import_dev_cli_crate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let runtime_crates = ["crates/bijux-cli/src/routing", "crates/bijux-cli-python/src"];

    let mut offenders = Vec::<String>::new();
    for crate_src in runtime_crates {
        for file in rs_files_under(&root.join(crate_src)) {
            let source = fs::read_to_string(&file).expect("read source");
            if source.contains("bijux_dev_cli") {
                offenders.push(file.display().to_string());
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "runtime crates must not import bijux-dev-cli directly: {offenders:?}"
    );
}

#[test]
fn query_interfaces_remain_structured_data_only_without_ui_rendering() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let query_files = [
        "crates/bijux-cli/src/features/diagnostics/mod.rs",
        "crates/bijux-cli/src/features/diagnostics/state_diagnostics.rs",
        "crates/bijux-cli/src/features/diagnostics/parity_status.rs",
        "crates/bijux-cli/src/features/install/query.rs",
        "crates/bijux-cli/src/routing/query.rs",
        "crates/bijux-cli/src/routing/inventory.rs",
    ];

    for file in query_files {
        let source = fs::read_to_string(root.join(file)).expect("read query provider");
        assert!(!source.contains("println!"), "query provider must not print: {file}");
        assert!(!source.contains("eprintln!"), "query provider must not print: {file}");
        assert!(
            !source.contains("render_value("),
            "query provider must not format terminal output: {file}"
        );
        assert!(
            !source.contains("serde_json::json!"),
            "query provider must not assemble presentation payloads: {file}"
        );
    }
}
