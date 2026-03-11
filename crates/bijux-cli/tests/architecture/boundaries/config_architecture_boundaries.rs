#![forbid(unsafe_code)]
//! Architecture boundaries specific to config layering.

use std::fs;
use std::path::Path;

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
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
fn bin_entrypoint_stays_free_of_config_business_logic() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let bin_rs = src_root.join("bin/bijux-rs.rs");
    let cleaned = strip_comments_and_strings(&read(&bin_rs));

    for forbidden in [
        "crate::features::config::",
        "config_operations::",
        "execute_config_command(",
        "run_config_migrations(",
        "BIJUXCLI_CONFIG",
    ] {
        assert!(
            !cleaned.contains(forbidden),
            "binary entrypoint must stay thin and config-agnostic, found `{forbidden}` in {}",
            bin_rs.display()
        );
    }
}

#[test]
fn config_feature_storage_and_validation_stay_free_of_interface_or_output_rendering() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let feature_files = [
        src_root.join("features/config/storage.rs"),
        src_root.join("features/config/validation.rs"),
        src_root.join("features/config/serialization.rs"),
    ];

    for file in feature_files {
        let cleaned = strip_comments_and_strings(&read(&file));
        for forbidden in [
            "crate::interface::",
            "crate::shared::output::",
            "render_value(",
            "EmitterConfig",
            "OutputFormat",
            "println!(",
            "eprintln!(",
        ] {
            assert!(
                !cleaned.contains(forbidden),
                "config internals must stay runtime-only, found `{forbidden}` in {}",
                file.display()
            );
        }
    }
}

#[test]
fn config_cli_handler_depends_on_operations_boundary_only() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let handler_file = src_root.join("interface/cli/handlers/config.rs");
    let operations_file = src_root.join("features/config/operations.rs");

    let handler = strip_comments_and_strings(&read(&handler_file));
    let operations = strip_comments_and_strings(&read(&operations_file));

    assert!(
        handler.contains("crate::features::config::operations as config_operations"),
        "config CLI handler must target operations boundary"
    );
    for forbidden in [
        "crate::features::config::service::",
        "crate::features::config::storage::",
        "crate::features::config::validation::",
        "crate::features::config::serialization::",
    ] {
        assert!(
            !handler.contains(forbidden),
            "config CLI handler must not bypass operations boundary, found `{forbidden}`"
        );
    }

    assert!(
        operations.contains("crate::features::config::service::"),
        "operations boundary must own config service wiring"
    );
    assert!(
        operations.contains("crate::features::config::storage::FileConfigRepository"),
        "operations boundary must own repository wiring"
    );
}
