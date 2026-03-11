#![forbid(unsafe_code)]
//! Architecture boundaries between interface orchestration, kernel, and domain modules.

use std::collections::BTreeSet;
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

fn immediate_directories(root: &Path) -> BTreeSet<String> {
    let Ok(entries) = fs::read_dir(root) else {
        return BTreeSet::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .collect()
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

fn collect_path_import_offenders(root: &Path, forbidden_paths: &[&str]) -> Vec<String> {
    let mut offenders = Vec::new();
    for file in rs_files_under(root) {
        let source = fs::read_to_string(&file).expect("read source");
        let cleaned = strip_comments_and_strings(&source);
        if forbidden_paths
            .iter()
            .any(|needle| cleaned.contains(needle))
        {
            offenders.push(file.display().to_string());
        }
    }
    offenders
}

#[test]
fn feature_root_inventory_is_explicit_and_exhaustive() {
    let features_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/features");
    let observed = immediate_directories(&features_root);
    let expected: BTreeSet<String> = [
        "config",
        "diagnostics",
        "history",
        "install",
        "memory",
        "plugins",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();

    assert_eq!(
        observed, expected,
        "feature root inventory drifted; update boundary laws intentionally"
    );
}

#[test]
fn domain_modules_do_not_depend_on_interface_bootstrap_or_kernel_layers() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let features_root = src_root.join("features");
    let mut domain_roots: Vec<PathBuf> = immediate_directories(&features_root)
        .into_iter()
        .map(|name| features_root.join(name))
        .collect();
    domain_roots.push(src_root.join("routing"));

    let mut offenders = Vec::new();
    for module_root in domain_roots {
        offenders.extend(collect_path_import_offenders(
            &module_root,
            &[
                "crate::interface::",
                "crate::bootstrap::",
                "crate::kernel::",
            ],
        ));
    }

    assert!(
        offenders.is_empty(),
        "domain modules must not import interface/bootstrap/kernel layers: {offenders:?}"
    );
}

#[test]
fn kernel_layer_does_not_depend_on_interface_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/kernel");
    let offenders = collect_path_import_offenders(&root, &["crate::interface::"]);
    assert!(
        offenders.is_empty(),
        "kernel layer must not import interface layer: {offenders:?}"
    );
}

#[test]
fn cli_interface_does_not_depend_on_kernel_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/interface/cli");
    let offenders = collect_path_import_offenders(&root, &["crate::kernel::"]);
    assert!(
        offenders.is_empty(),
        "cli interface must not import kernel layer: {offenders:?}"
    );
}

#[test]
fn feature_modules_do_not_depend_on_interface_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/features");
    let offenders = collect_path_import_offenders(&root, &["crate::interface::"]);
    assert!(
        offenders.is_empty(),
        "feature modules must not import interface layer: {offenders:?}"
    );
}

#[test]
fn infrastructure_layer_does_not_depend_on_feature_or_interface_layers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/infrastructure");
    let offenders =
        collect_path_import_offenders(&root, &["crate::features::", "crate::interface::"]);
    assert!(
        offenders.is_empty(),
        "infrastructure adapters must not import feature/interface modules: {offenders:?}"
    );
}
