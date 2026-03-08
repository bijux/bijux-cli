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
fn replay_diff_benchmark_focus_report_is_generated_from_scenario_registry() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let report_path = root.join("docs/reports/foundation/replay_diff_benchmark_focus_report.json");
    let payload: Value = serde_json::from_str(&std::fs::read_to_string(report_path).expect("report"))
        .expect("json");

    assert_eq!(payload["generated_from"], "evidence/perf/scenario_registry.json");
    assert_eq!(payload["focus"], "replay-diff");
    assert!(payload["scenario_count"].as_u64().unwrap_or(0) >= 2);

    let scenarios = payload["scenarios"].as_array().expect("scenarios");
    assert!(
        scenarios
            .iter()
            .any(|entry| entry["id"].as_str().unwrap_or_default().contains("replay"))
    );
    assert!(
        scenarios
            .iter()
            .any(|entry| entry["id"].as_str().unwrap_or_default().contains("diff"))
    );
}
