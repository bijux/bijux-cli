//! Maintainer documentation audit report assembly.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn collect_files(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !base.exists() {
        return out;
    }
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

fn rel_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn read_json_if_exists(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| json!({}))
}

/// Builds the maintainer documentation audit report payload.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    let docs_audit = read_json_if_exists(&workspace_root.join("artifacts/status/docs_audit.json"));
    let docs_files: Vec<String> = collect_files(&workspace_root.join("docs"))
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .map(|p| rel_to_root(&p, workspace_root))
        .collect();

    json!({
        "docs_audit": docs_audit,
        "docs": docs_files,
        "docs_count": docs_files.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::build_report;

    #[test]
    fn docs_audit_report_shape_is_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&root);
        assert!(report.get("docs").is_some());
        assert!(report.get("docs_count").is_some());
        assert!(report.get("docs_audit").is_some());
    }
}
