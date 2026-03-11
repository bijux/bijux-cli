#![forbid(unsafe_code)]
//! Prevents maintainer report assembly from drifting back into runtime core.

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
fn core_adapter_is_runtime_query_only_and_delegates_dispatch() {
    let source = strip_comments_and_strings(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/features/developer/runtime_query_adapter.rs"
        ))
        .expect("read dev cli adapter source"),
    );

    assert!(
        source.contains("impl RuntimeQueryProvider for RuntimeQueryAdapter"),
        "core dev cli module must only provide runtime query adapter implementation"
    );
    assert!(
        source.contains("dev_dispatch::try_handle"),
        "core dev cli module must delegate dispatch to bijux-dev-cli"
    );
    assert!(
        !source.contains("match normalized_path"),
        "core dev cli module must not own command branch dispatch"
    );
    assert!(
        !source.contains("dev_control_plane::build_doctor_report"),
        "core dev cli module must not assemble maintainer report payloads"
    );
    assert!(
        !source.contains("dev_routes::build_report_from_query"),
        "core dev cli module must not call route report builders"
    );
}

#[test]
fn dev_cli_dispatch_owns_report_assembly_and_command_branches() {
    let source = strip_comments_and_strings(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../bijux-dev-cli/src/dispatch.rs"
        ))
        .expect("read dev cli dispatch source"),
    );

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
        "dev_script_audit::build_report",
        "dev_docs_audit::build_report",
        "dev_crate_health::build_report",
        "dev_runtime_identity::build_report",
        "dev_package_health::build_report",
        "dev_state_audit::build_report",
        "dev_state_audit::build_doctor_report",
    ];

    for needle in delegated {
        assert!(source.contains(needle), "dispatch must delegate report assembly for {needle}");
    }
    assert!(
        !source.contains("render_value("),
        "dev-cli dispatch must return structured payloads and avoid terminal rendering"
    );
}
