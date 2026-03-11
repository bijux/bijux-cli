#![forbid(unsafe_code)]
//! Workspace-level architecture guards for dev-cli ownership.

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

fn read_dev_cli_source() -> String {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let router_root = crate_root.join("../bijux-dev-cli/src/cli");
    assert!(
        router_root.is_dir(),
        "expected modular dev-cli source directory at {}",
        router_root.display()
    );
    let mut files = Vec::<PathBuf>::new();
    collect_rs_files(&router_root, &mut files);
    files.sort();
    assert!(
        !files.is_empty(),
        "expected dev-cli source files under {}",
        router_root.display()
    );

    let mut source = String::new();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("read dev cli dispatch source {}: {err}", file.display()));
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
fn runtime_crates_do_not_import_bijux_dev_cli() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let runtime_crates = ["crates/bijux-cli/src/routing", "crates/bijux-cli-python/src"];

    for crate_src in runtime_crates {
        for path in walk_rs_files(&root.join(crate_src)) {
            let source = std::fs::read_to_string(&path).expect("read source file");
            let cleaned = strip_comments_and_strings(&source);
            assert!(
                !cleaned.contains("bijux_dev_cli"),
                "runtime crate source must not import bijux-dev-cli: {}",
                path.display()
            );
        }
    }
}

#[test]
fn core_dev_cli_routes_delegate_to_dev_cli_module_helpers() {
    let dispatch_source_raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/interface/cli/dispatch.rs"
    ))
    .expect("read runtime dispatch source");
    let dev_cli_source_raw = read_dev_cli_source();
    let dispatch_source = strip_comments_and_strings(&dispatch_source_raw);
    let dev_cli_source = strip_comments_and_strings(&dev_cli_source_raw);

    assert!(
        dispatch_source.contains("delegate_dev_cli(&argv[3..])"),
        "runtime dispatch must delegate canonical dev cli namespace to external dev-cli binary"
    );
    assert!(
        dispatch_source.contains("delegate_dev_cli(&argv[2..])"),
        "runtime dispatch must delegate legacy `dev <alias>` routes to external dev-cli binary"
    );
    assert!(
        !dispatch_source.contains("runtime_query_adapter::try_handle"),
        "runtime dispatch must not retain in-process dev-cli routing adapters"
    );
    assert!(
        !dispatch_source.contains("bijux_dev_cli"),
        "runtime dispatch must not import dev-cli crate symbols directly"
    );

    let delegated = [
        "root::try_handle",
        "maintenance::try_handle",
        "rustdoc::try_handle",
        "release::try_handle",
    ];
    for needle in delegated {
        assert!(dev_cli_source.contains(needle), "missing dev-cli dispatch delegation for {needle}");
    }
    assert!(
        dev_cli_source.contains("pub fn owns_path"),
        "dev-cli dispatch must own path classification contract"
    );
}

#[test]
fn workspace_automation_does_not_execute_status_contracts_directly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut offenders = Vec::<String>::new();

    let scan_roots = [".github", "crates", "docs", "makes", "scripts", "tests"];
    for scan_root in scan_roots {
        for path in walk_files(&root.join(scan_root)) {
            let rel =
                path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            if rel == "crates/bijux-cli/tests/architecture/ownership/dev_cli_architecture_guards.rs"
            {
                continue;
            }

            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            let direct_source_arg = source.contains("maintenance status run --source")
                || source.contains("maintenance\", \"status\", \"run\", \"--source")
                || source.contains("maintenance status run -- source");
            if direct_source_arg {
                offenders.push(rel);
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "status contracts must run through `bijux dev cli maintenance status run --id ...`; direct execution found in:\n{}",
        offenders.join("\n")
    );
}

fn walk_rs_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
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
    out
}

fn walk_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    let mut stack = vec![root.to_path_buf()];
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

            let include = if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
                name == "Makefile"
            } else {
                false
            } || path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| {
                matches!(ext, "rs" | "py" | "sh" | "yml" | "yaml" | "md" | "toml" | "txt" | "mk")
            });
            if include {
                out.push(path);
            }
        }
    }
    out
}
