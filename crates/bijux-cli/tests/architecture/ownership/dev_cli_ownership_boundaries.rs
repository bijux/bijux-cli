#![forbid(unsafe_code)]
//! Prevents maintainer report assembly from drifting back into runtime core.

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
fn core_runtime_delegates_dev_cli_at_process_boundary() {
    let source = strip_comments_and_strings(&read_runtime_dispatch_source());

    assert!(
        source.contains("let forwarded = if tool_namespace ==  ") && source.contains("&argv[3..]"),
        "runtime dispatch must normalize canonical `dev cli` delegation arguments"
    );
    assert!(
        source.contains("else { &argv[2..] }"),
        "runtime dispatch must normalize `dev <subcommand>` delegation arguments"
    );
    assert!(
        source.contains("return Some(delegate_dev_cli(forwarded));"),
        "runtime dispatch must delegate non-product `dev` routes to external binary"
    );
    assert!(
        !source.contains("runtime_query_adapter::try_handle"),
        "runtime dispatch must not keep in-process dev-cli adapters"
    );
    assert!(
        !source.contains("bijux_dev_cli"),
        "runtime dispatch must not import dev-cli crate directly"
    );
}

#[test]
fn dev_cli_dispatch_owns_report_assembly_and_command_branches() {
    let source = strip_comments_and_strings(&read_dev_cli_source());

    assert!(
        source.contains("match normalized_path"),
        "dev cli crate must own command dispatch branching"
    );

    let delegated = [
        "dev_routes::build_report_from_query",
        "dev_registry::build_report_from_query",
        "dev_route_audit::build_report_from_query",
        "dev_env::build_report",
        "dev_contracts::build_report_from_query",
        "dev_parity::build_report",
        "dev_status::build_report",
        "dev_control_plane::build_atlas_report",
        "dev_control_plane::build_dependency_injection_report",
        "dev_maintenance_audit::build_report",
        "dev_docs_audit::build_report",
        "dev_crate_health::build_report",
        "dev_runtime_identity::build_report",
        "dev_package_health::build_report",
        "dev_state_audit::build_report",
        "dev_state_audit::build_doctor_report",
    ];

    for needle in delegated {
        assert!(
            source.contains(needle),
            "dispatch must delegate report assembly for {needle}"
        );
    }
    assert!(
        !source.contains("render_value("),
        "dev-cli dispatch must return structured payloads and avoid terminal rendering"
    );
}
