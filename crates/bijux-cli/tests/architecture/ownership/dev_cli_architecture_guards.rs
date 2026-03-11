#![forbid(unsafe_code)]
//! Workspace-level architecture guards for dev-cli ownership.

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
    let adapter_source_raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/features/developer/runtime_query_adapter.rs"
    ))
    .expect("read dev cli command source");
    let dispatch_source_raw = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../bijux-dev-cli/src/dispatch.rs"
    ))
    .expect("read dev cli dispatch source");
    let adapter_source = strip_comments_and_strings(&adapter_source_raw);
    let dispatch_source = strip_comments_and_strings(&dispatch_source_raw);

    assert!(
        adapter_source.contains("dev_dispatch::try_handle"),
        "core adapter must delegate dev cli command routing to bijux-dev-cli dispatch"
    );
    assert!(
        !adapter_source.contains("dev_control_plane::build_doctor_report"),
        "core adapter must not assemble dev cli report payloads directly"
    );
    assert!(
        !adapter_source.contains("match normalized_path"),
        "core adapter must not own command branch dispatch tables"
    );
    assert!(
        !adapter_source.contains("::build_"),
        "core adapter must not call report builder functions directly"
    );

    let delegated = [
        "dev_control_plane::build_snapshots_audit_report",
        "dev_control_plane::build_fixture_audit_report",
        "dev_control_plane::build_plugin_health_report",
        "dev_control_plane::build_doctor_report",
    ];
    for needle in delegated {
        assert!(dispatch_source.contains(needle), "missing delegation for {needle}");
    }
    assert!(
        dispatch_source.contains("pub fn owns_path"),
        "dev-cli dispatch must own path classification contract"
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
