#![forbid(unsafe_code)]
//! Source-level invariants for dev-cli dispatch and failure handling.

use std::path::{Path, PathBuf};

fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut idx = 0;
    let mut in_line_comment = false;
    let mut block_comment_depth = 0_usize;
    let mut in_string = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let current = bytes[idx];
        let next = bytes.get(idx + 1).copied();

        if in_line_comment {
            if current == b'\n' {
                in_line_comment = false;
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
            in_line_comment = true;
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

fn read_runtime_dispatch_source() -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dispatch_file = crate_root.join("src/interface/cli/dispatch.rs");
    let dispatch_dir = crate_root.join("src/interface/cli/dispatch");
    assert!(
        dispatch_file.is_file(),
        "expected runtime dispatch entry file at {}",
        dispatch_file.display()
    );
    assert!(
        dispatch_dir.is_dir(),
        "expected runtime dispatch module directory at {}",
        dispatch_dir.display()
    );

    let mut files = vec![dispatch_file];
    collect_rs_files(&dispatch_dir, &mut files);
    files.sort();

    let mut source = String::new();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read runtime dispatch source {}: {err}", file.display()));
        source.push_str(&text);
        source.push('\n');
    }
    source
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
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
fn dev_cli_dispatch_uses_shared_envelope_and_exit_mapping() {
    let source = strip_comments_and_strings(&read_runtime_dispatch_source());
    assert!(source.contains("render_value("), "core app must use shared report envelope renderer");
    assert!(source.contains("AppRunResult"), "core app must return a normalized run envelope");
    assert!(
        source.contains("try_delegate_known_bijux_tool"),
        "core dispatch must route known external tool delegation before local handling"
    );
}

#[test]
fn dev_cli_dispatch_remains_core_only_and_bin_stays_thin() {
    let core_source = strip_comments_and_strings(&read_runtime_dispatch_source());
    let bin_source = strip_comments_and_strings(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bin/bijux.rs"))
            .expect("read core bin source"),
    );

    assert!(
        core_source.contains("delegate_dev_cli("),
        "core dispatch must delegate dev-cli commands through external binary boundary"
    );
    assert!(
        !core_source.contains("handlers::developer"),
        "core dispatch must not own dedicated developer handler modules"
    );
    assert!(
        !core_source.contains("handlers::developer_runtime"),
        "core dispatch must not route dev cli through interface handler facades"
    );
    assert!(!core_source.contains("runtime_query_adapter::try_handle"));
    assert!(!bin_source.contains("dev cli"), "bin must not own dev cli dispatch");
    assert!(!bin_source.contains("bijux_dev_cli"), "bin must not import bijux-dev-cli crate");
}
