#![forbid(unsafe_code)]
//! Ownership boundary contracts for the maintainer command registry and runtime law separation.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_dev_cli::schema::command_registry::{
    command_registry, DevCliCommand, MAINTAINER_COMMAND_NAMESPACE,
};

fn rs_files_under(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
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
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

fn strip_comments_and_strings(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut idx = 0;
    let mut block_comment_depth = 0_usize;
    let mut in_line_comment = false;
    let mut in_string = false;
    let mut in_raw_string_hashes: Option<usize> = None;
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

        if let Some(raw_hashes) = in_raw_string_hashes {
            if current == b'"' {
                let mut matches = true;
                for offset in 0..raw_hashes {
                    if bytes.get(idx + 1 + offset).copied() != Some(b'#') {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    in_raw_string_hashes = None;
                    idx += 1 + raw_hashes;
                    out.push(' ');
                    continue;
                }
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

        if current == b'r' {
            let mut hashes = 0_usize;
            let mut cursor = idx + 1;
            while bytes.get(cursor).copied() == Some(b'#') {
                hashes += 1;
                cursor += 1;
            }
            if bytes.get(cursor).copied() == Some(b'"') {
                in_raw_string_hashes = Some(hashes);
                idx = cursor + 1;
                out.push(' ');
                continue;
            }
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
fn command_registry_covers_all_known_maintainer_subcommands() {
    let fixture = include_str!("../data/fixtures/routing/maintainer_subcommands.txt");
    let known: BTreeSet<String> = fixture
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();
    let registered: BTreeSet<String> =
        command_registry().iter().map(|entry| entry.command.as_str().to_string()).collect();

    assert_eq!(registered, known);
}

#[test]
fn command_registry_entries_are_canonical_and_unique() {
    let mut seen = BTreeSet::<&'static str>::new();
    for entry in command_registry() {
        assert_eq!(entry.owner, "bijux-dev-cli");
        assert!(entry.command.as_str().starts_with("bijux-dev-cli "));
        assert!(seen.insert(entry.command.as_str()));
    }
    assert_eq!(MAINTAINER_COMMAND_NAMESPACE, "bijux-dev-cli");
    assert!(command_registry().iter().any(|entry| matches!(entry.command, DevCliCommand::Status)));
}

#[test]
fn crate_scope_rejects_runtime_command_law_and_root_alias_reexports() {
    let lib_source = include_str!("../../../src/maintainer/mod.rs");
    assert!(lib_source.contains("Runtime command law remains in runtime crates"));
    assert!(!lib_source.contains("pub use report"));
    assert!(!lib_source.contains("pub use contract_engine"));

    let runtime_law_signatures = [
        "cli plugins",
        "cli config",
        "history clear",
        "memory set",
        "route_response(",
        "parse_intent(",
    ];

    for signature in runtime_law_signatures {
        let present = include_str!("../../../src/maintainer/reports/control_plane.rs")
            .contains(signature)
            || include_str!("../../../src/maintainer/reports/repository_health/status/mod.rs")
                .contains(signature)
            || include_str!("../../../src/maintainer/reports/runtime_surface/parity.rs")
                .contains(signature)
            || include_str!("../../../src/maintainer/reports/runtime_surface/runtime_identity.rs")
                .contains(signature);
        assert!(
            !present,
            "runtime law signature leaked into maintainer control-plane crate: {signature}"
        );
    }
}

#[test]
fn maintainer_sources_only_import_product_api_layers_not_runtime_internals() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden = [
        "bijux_cli::bootstrap::",
        "bijux_cli::features::",
        "bijux_cli::infrastructure::",
        "bijux_cli::interface::",
        "bijux_cli::kernel::",
        "bijux_cli::routing::",
        "bijux_cli::shared::",
        "bijux_dag_runtime::runtime_core::execution::engine::",
    ];

    let mut offenders = Vec::new();
    for file in rs_files_under(&source_root) {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", file.display()));
        let cleaned = strip_comments_and_strings(&source);
        if forbidden.iter().any(|needle| cleaned.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "maintainer sources imported forbidden product internals: {offenders:?}"
    );
}
