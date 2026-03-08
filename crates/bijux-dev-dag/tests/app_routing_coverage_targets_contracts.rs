use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[test]
fn app_routing_coverage_target_policy_is_present_and_nontrivial() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root");
    let path = root.join("configs/policy/app_routing_coverage_targets.json");
    assert!(path.exists(), "missing policy file: {}", path.display());

    let raw = fs::read_to_string(path).expect("read app routing coverage policy");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("valid policy json");
    let min_routes = value
        .get("line_coverage_targets")
        .and_then(|v| v.as_object())
        .expect("line_coverage_targets object");

    assert!(
        min_routes
            .get("crates/bijux-dag-app/src/routes")
            .and_then(|v| v.as_f64())
            .map(|v| v >= 0.70)
            .unwrap_or(false),
        "app routes aggregate target must be >= 0.70"
    );

    for rel in [
        "crates/bijux-dag-app/src/routes/inspect_routes.rs",
        "crates/bijux-dag-app/src/routes/plan_routes.rs",
        "crates/bijux-dag-app/src/routes/diagnostics_routes.rs",
        "crates/bijux-dag-app/src/routes/output_selection.rs",
        "crates/bijux-dag-app/src/routes/response.rs",
        "crates/bijux-dag-app/src/routes/run_lookup.rs",
        "crates/bijux-dag-app/src/routes/surface_routes.rs",
    ] {
        let target = min_routes
            .get(rel)
            .and_then(|v| v.as_f64())
            .unwrap_or_default();
        assert!(target >= 0.70, "coverage target must be >= 0.70 for {rel}");
    }
}
