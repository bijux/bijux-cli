use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn comparison_scenarios_have_required_ids_and_unique_names() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scenarios_dir = root.join("comparisons/scenarios");
    let entries = fs::read_dir(&scenarios_dir).expect("read scenarios dir");
    let mut ids = BTreeSet::new();
    for entry in entries {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(&path).expect("read scenario");
        let val: Value = serde_json::from_str(&payload).expect("parse scenario");
        let id = val.get("id").and_then(Value::as_str).expect("id");
        assert!(ids.insert(id.to_string()), "duplicate scenario id: {id}");
    }
    for required in [
        "chain",
        "diamond",
        "retry-timeout",
        "cache-reuse-shape",
        "replay-equivalence",
        "failure-propagation",
        "determinism",
        "operator-inspectability",
        "failure-diagnostics",
        "scheduler-tiny-tasks-overhead",
        "artifact-inspectability",
    ] {
        assert!(ids.contains(required), "missing required scenario id {required}");
    }
}

#[test]
fn bijux_baseline_covers_all_scenarios() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let baseline_path = root.join("comparisons/bijux/baselines/v1.json");
    let baseline_payload = fs::read_to_string(baseline_path).expect("read baseline");
    let baseline: Value = serde_json::from_str(&baseline_payload).expect("parse baseline");
    let items = baseline
        .get("scenarios")
        .and_then(Value::as_array)
        .expect("scenarios");
    let mut ids = BTreeSet::new();
    for item in items {
        let id = item.get("id").and_then(Value::as_str).expect("id");
        ids.insert(id.to_string());
    }
    assert_eq!(ids.len(), 11);
    assert!(ids.contains("determinism"));
    assert!(ids.contains("operator-inspectability"));
    assert!(ids.contains("failure-diagnostics"));
    assert!(ids.contains("scheduler-tiny-tasks-overhead"));
    assert!(ids.contains("artifact-inspectability"));
}
