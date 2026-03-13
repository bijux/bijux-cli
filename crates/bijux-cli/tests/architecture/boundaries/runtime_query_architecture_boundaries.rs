#![forbid(unsafe_code)]
//! Architecture boundaries for runtime query providers used by dev-cli.

use std::fs;
use std::path::{Path, PathBuf};

fn rs_files_under(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }

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

fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut idx = 0;
    let mut line_comment = false;
    let mut block_comment_depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let current = bytes[idx];
        let next = bytes.get(idx + 1).copied();

        if line_comment {
            if current == b'\n' {
                line_comment = false;
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if block_comment_depth > 0 {
            if current == b'/' && next == Some(b'*') {
                block_comment_depth += 1;
                idx += 2;
                continue;
            }
            if current == b'*' && next == Some(b'/') {
                block_comment_depth -= 1;
                idx += 2;
                continue;
            }
            if current == b'\n' {
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
                idx += 1;
                continue;
            }
            if current == b'\\' {
                escaped = true;
                idx += 1;
                continue;
            }
            if current == b'"' {
                in_string = false;
                out.push(' ');
                idx += 1;
                continue;
            }
            if current == b'\n' {
                out.push('\n');
            }
            idx += 1;
            continue;
        }

        if current == b'/' && next == Some(b'/') {
            line_comment = true;
            idx += 2;
            continue;
        }
        if current == b'/' && next == Some(b'*') {
            block_comment_depth = 1;
            idx += 2;
            continue;
        }
        if current == b'"' {
            in_string = true;
            out.push(' ');
            idx += 1;
            continue;
        }

        out.push(current as char);
        idx += 1;
    }

    out
}

#[test]
fn runtime_crates_do_not_import_maintainer_crate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let runtime_crates = ["crates/bijux-cli/src/routing", "crates/bijux-cli-python/src"];

    let mut offenders = Vec::<String>::new();
    for crate_src in runtime_crates {
        for file in rs_files_under(&root.join(crate_src)) {
            let source = fs::read_to_string(&file).expect("read source");
            let cleaned = strip_comments_and_strings(&source);
            if cleaned.contains("bijux_dev_cli") {
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
fn runtime_query_provider_inventory_is_explicit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let expected_files = [
        "crates/bijux-cli/src/features/diagnostics/mod.rs",
        "crates/bijux-cli/src/features/diagnostics/state_diagnostics.rs",
        "crates/bijux-cli/src/features/diagnostics/parity_status.rs",
        "crates/bijux-cli/src/features/install/query.rs",
        "crates/bijux-cli/src/contracts/query.rs",
        "crates/bijux-cli/src/features/diagnostics/routing_inventory.rs",
    ];

    for file in expected_files {
        assert!(
            root.join(file).is_file(),
            "expected runtime query provider file is missing: {file}"
        );
    }
}

#[test]
fn query_interfaces_remain_data_only_without_ui_or_side_effecting_writes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let query_files = [
        "crates/bijux-cli/src/features/diagnostics/mod.rs",
        "crates/bijux-cli/src/features/diagnostics/state_diagnostics.rs",
        "crates/bijux-cli/src/features/diagnostics/parity_status.rs",
        "crates/bijux-cli/src/features/install/query.rs",
        "crates/bijux-cli/src/contracts/query.rs",
        "crates/bijux-cli/src/features/diagnostics/routing_inventory.rs",
    ];

    let forbidden_tokens = [
        "println!(",
        "eprintln!(",
        "render_value(",
        "EmitterConfig",
        "OutputFormat",
        "std::process::Command",
        "Command::new(",
        "fs::write(",
        "fs::remove_file(",
        "fs::create_dir_all(",
    ];

    for file in query_files {
        let source = fs::read_to_string(root.join(file)).expect("read query provider");
        let cleaned = strip_comments_and_strings(&source);
        for forbidden in forbidden_tokens {
            assert!(
                !cleaned.contains(forbidden),
                "query provider must stay read-only and rendering-free; found `{forbidden}` in {file}"
            );
        }
    }
}
