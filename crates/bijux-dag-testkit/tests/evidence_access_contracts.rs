use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit::{
    evidence_asset_ids, evidence_registry_path, load_evidence_registry_checked,
    resolve_evidence_asset_by_id_checked, workspace_root_from_manifest_dir,
};
use serde as _;
use serde_json as _;
use tempfile as _;

fn workspace_root() -> std::path::PathBuf {
    workspace_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn missing_evidence_asset_error_is_actionable() {
    let root = workspace_root();
    let registry = load_evidence_registry_checked(&root).expect("load evidence registry");
    let error = resolve_evidence_asset_by_id_checked(&registry, "missing.asset.id")
        .expect_err("missing id should return an actionable error");
    assert!(error.contains("missing.asset.id"), "error should include missing asset id: {error}");
    assert!(
        error.contains("ownership") && error.contains("consumer mapping"),
        "error should include next-step guidance: {error}"
    );
}

#[test]
fn registry_asset_ids_are_stable_across_reload() {
    let root = workspace_root();
    let first = load_evidence_registry_checked(&root).expect("first load");
    let second = load_evidence_registry_checked(&root).expect("second load");
    assert_eq!(
        evidence_asset_ids(&first),
        evidence_asset_ids(&second),
        "registry asset ids changed across reload without source changes"
    );
}

#[test]
fn checked_registry_loader_reports_path_on_parse_or_read_failure() {
    let root = workspace_root();
    let path = evidence_registry_path(&root);
    assert!(path.exists(), "evidence registry path should exist: {}", path.display());
}
