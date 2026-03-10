#![forbid(unsafe_code)]
//! Architecture boundaries between CLI orchestration, kernel, and domain modules.

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

#[test]
fn domain_modules_do_not_depend_on_cli_or_kernel_layers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let domain_roots = ["config", "install", "plugin", "query", "routing"];

    let mut cli_offenders = Vec::new();
    let mut kernel_offenders = Vec::new();

    for module_root in domain_roots {
        for file in rs_files_under(&root.join(module_root)) {
            let source = fs::read_to_string(&file).expect("read source");
            if source.contains("crate::cli::") {
                cli_offenders.push(file.display().to_string());
            }
            if source.contains("crate::kernel::") {
                kernel_offenders.push(file.display().to_string());
            }
        }
    }

    assert!(
        cli_offenders.is_empty(),
        "domain modules must not import cli layer: {cli_offenders:?}"
    );
    assert!(
        kernel_offenders.is_empty(),
        "domain modules must not import kernel layer: {kernel_offenders:?}"
    );
}

#[test]
fn kernel_layer_does_not_depend_on_cli_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/kernel");
    let mut offenders = Vec::new();

    for file in rs_files_under(&root) {
        let source = fs::read_to_string(&file).expect("read source");
        if source.contains("crate::cli::") {
            offenders.push(file.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "kernel layer must not import cli layer: {offenders:?}"
    );
}

#[test]
fn cli_layer_does_not_depend_on_kernel_layer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli");
    let mut offenders = Vec::new();

    for file in rs_files_under(&root) {
        let source = fs::read_to_string(&file).expect("read source");
        let is_allowed_entrypoint = file.ends_with("src/cli/entrypoint.rs");
        if source.contains("crate::kernel::") && !is_allowed_entrypoint {
            offenders.push(file.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "cli layer must not import kernel layer: {offenders:?}"
    );
}
