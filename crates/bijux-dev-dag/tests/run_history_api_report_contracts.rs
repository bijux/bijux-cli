use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

#[test]
fn generated_run_history_api_report_tracks_operator_run_schemas() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let report_path = root.join("docs/reports/foundation/run_history_api_report.json");
    let payload: Value = serde_json::from_str(&std::fs::read_to_string(report_path).expect("report"))
        .expect("json");

    assert_eq!(payload["generated_from"], "configs/schema/operator");
    let surfaces = payload["surfaces"].as_object().expect("surfaces");
    for required in [
        "run_summary.schema.json",
        "run_show.schema.json",
        "run_inspect.schema.json",
        "run_history.schema.json",
    ] {
        assert!(surfaces.contains_key(required), "missing schema entry: {required}");
        assert!(
            surfaces[required].as_array().map(|v| !v.is_empty()).unwrap_or(false),
            "schema entry should list required fields: {required}"
        );
    }
}
