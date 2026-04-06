//! Maintainer documentation audit report assembly.

use std::path::Path;

use serde_json::{json, Value};

use crate::infra::artifacts::{
    collect_files_recursive, json_artifact_state, read_json_if_exists, relative_to_root,
};

fn has_extension(path: &str, extension: &str) -> bool {
    Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

/// Builds the maintainer documentation audit report payload.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    let artifact_summary =
        read_json_if_exists(&workspace_root.join("artifacts/status/docs_audit.json"));
    let all_docs_files = collect_files_recursive(&workspace_root.join("docs"))
        .into_iter()
        .map(|p| relative_to_root(&p, workspace_root))
        .collect::<Vec<_>>();
    let contract_assets = collect_files_recursive(&workspace_root.join("contracts"))
        .into_iter()
        .map(|p| relative_to_root(&p, workspace_root))
        .collect::<Vec<_>>();
    let docs_files =
        all_docs_files.iter().filter(|path| has_extension(path, "md")).cloned().collect::<Vec<_>>();
    let machine_readable_docs = all_docs_files
        .iter()
        .filter(|path| has_extension(path, "json") && path.starts_with("docs/assets/"))
        .cloned()
        .collect::<Vec<_>>();
    let site_assets = all_docs_files
        .iter()
        .filter(|path| {
            has_extension(path, "css")
                || has_extension(path, "js")
                || has_extension(path, "html")
                || has_extension(path, "png")
        })
        .cloned()
        .collect::<Vec<_>>();

    json!({
        "docs_audit": {
            "status": "ok",
            "artifact_state": json_artifact_state(&artifact_summary),
            "docs_count": docs_files.len(),
            "markdown_count": docs_files.len(),
            "machine_readable_count": machine_readable_docs.len(),
            "contract_asset_count": contract_assets.len(),
            "site_asset_count": site_assets.len(),
            "total_file_count": all_docs_files.len(),
        },
        "docs": docs_files,
        "docs_count": docs_files.len(),
        "machine_readable_docs": machine_readable_docs,
        "machine_readable_count": machine_readable_docs.len(),
        "contract_assets": contract_assets,
        "contract_asset_count": contract_assets.len(),
        "site_assets": site_assets,
        "site_asset_count": site_assets.len(),
        "total_docs_file_count": all_docs_files.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::Value;

    use super::build_report;

    #[test]
    fn docs_audit_report_shape_is_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&root);
        assert!(report.get("docs").is_some());
        assert!(report.get("docs_count").is_some());
        assert!(report.get("docs_audit").is_some());
        assert!(report.get("machine_readable_docs").is_some());
        assert!(report.get("contract_assets").is_some());
        assert!(report.get("site_assets").is_some());
    }

    #[test]
    fn docs_audit_tracks_non_markdown_files_under_docs_and_contract_roots() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&root);
        let machine_readable_docs =
            report["machine_readable_docs"].as_array().expect("machine-readable docs");
        let contract_assets = report["contract_assets"].as_array().expect("contract assets");
        let site_assets = report["site_assets"].as_array().expect("site assets");
        assert!(contract_assets
            .iter()
            .filter_map(Value::as_str)
            .any(|path| path == "contracts/schemas/output-envelope-v1.schema.json"));
        assert!(machine_readable_docs.is_empty());
        assert!(site_assets
            .iter()
            .filter_map(Value::as_str)
            .any(|path| path == "docs/assets/javascripts/mermaid-init.js"));
    }
}
