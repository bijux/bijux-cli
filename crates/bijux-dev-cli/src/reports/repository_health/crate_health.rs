//! Maintainer crate health report assembly.

use std::path::Path;

use serde_json::{json, Value};

use crate::infrastructure::artifacts::read_json_if_exists;

/// Builds the maintainer crate health report payload.
#[must_use]
pub fn build_report(workspace_root: &Path) -> Value {
    let metrics =
        read_json_if_exists(&workspace_root.join("artifacts/status/crate_boundary_metrics.json"));
    let report =
        read_json_if_exists(&workspace_root.join("artifacts/status/crate_boundary_report.json"));
    let state =
        read_json_if_exists(&workspace_root.join("artifacts/status/current_rust_state.json"));
    let public_api_by_crate =
        read_json_if_exists(&workspace_root.join("artifacts/status/public_api_by_crate.json"));
    let internal_only_candidates = read_json_if_exists(
        &workspace_root.join("artifacts/status/internal_only_candidates_by_crate.json"),
    );
    let cross_crate_api_usage =
        read_json_if_exists(&workspace_root.join("artifacts/status/cross_crate_api_usage.json"));
    let duplication_hotspots =
        read_json_if_exists(&workspace_root.join("artifacts/status/duplication_hotspots.json"));

    json!({
        "crate_metrics": metrics,
        "crate_report": report,
        "public_api_counts": state.get("crates_public_api_counts").cloned().unwrap_or_else(|| json!([])),
        "dependency_edges": state.get("crate_dependency_edges").cloned().unwrap_or_else(|| json!([])),
        "public_api_by_crate": public_api_by_crate,
        "internal_only_candidates_by_crate": internal_only_candidates,
        "cross_crate_api_usage": cross_crate_api_usage,
        "duplication_hotspots": duplication_hotspots,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::build_report;

    #[test]
    fn crate_health_report_shape_is_stable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = build_report(&root);
        assert!(report.get("crate_metrics").is_some());
        assert!(report.get("crate_report").is_some());
        assert!(report.get("duplication_hotspots").is_some());
    }
}
