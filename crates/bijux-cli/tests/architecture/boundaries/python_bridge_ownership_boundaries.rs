#![forbid(unsafe_code)]
//! Guards that keep Python bridge ownership out of bijux-cli runtime code.

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
fn bijux_cli_source_never_imports_python_bridge_crate() {
    let src_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();

    for file in rs_files_under(&src_root) {
        let source = fs::read_to_string(&file).expect("read source");
        if source.contains("bijux_cli_python") {
            offenders.push(file.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "bijux-cli must not import bijux-cli-python; keep bridge ownership in dedicated crate: {offenders:?}"
    );
}
