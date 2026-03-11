//! Maintainer documentation audit report assembly.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{collect_files_recursive, read_json_if_exists, relative_to_root};

/// Builds the maintainer documentation audit report payload.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    let docs_audit = read_json_if_exists(&workspace_root.join("artifacts/status/docs_audit.json"));
    let docs_files: Vec<String> = collect_files_recursive(&workspace_root.join("docs"))
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
        .map(|p| relative_to_root(&p, workspace_root))
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
