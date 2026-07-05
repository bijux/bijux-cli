use bijux_dag_app as _;
use serde_json as _;
use tempfile::tempdir;

use std::process::Command;

fn dag_command() -> Command {
    let path = env!("CARGO_BIN_EXE_bijux-dag");
    assert!(
        std::path::Path::new(path).exists(),
        "resolved bijux test binary path does not exist: {path}"
    );
    Command::new(path)
}

fn write_schedule_submit_fixtures() -> (tempfile::TempDir, String, String, String, String) {
    let dir = tempdir().expect("tempdir");
    let registry = dir.path().join("schedule-registry.json");
    let inputs = dir.path().join("schedule-inputs.json");
    let ledger = dir.path().join("schedule-ledger.json");
    let out = dir.path().join("schedule-ledger-updated.json");

    std::fs::write(
        &registry,
        r#"{
  "definitions": [
    {
      "id": "manual-ops",
      "dag_name": "atlas.manual-ops",
      "dag_version_policy": "run-latest",
      "trigger": "Manual",
      "queue": {"queue_name": "catalog", "tenant": "atlas"},
      "priority": "High",
      "concurrency": {
        "per_dag": 2,
        "per_queue": 4,
        "per_tenant": 4,
        "per_node_group": null
      },
      "catch_up": {"enabled": false, "max_catch_up_runs": 0}
    }
  ]
}"#,
    )
    .expect("write registry");
    std::fs::write(
        &inputs,
        r#"{
  "now_unix_ms": 200000,
  "manual_requests": [
    {
      "request_id": "manual-001",
      "schedule_id": "manual-ops",
      "requested_unix_ms": 175000
    }
  ]
}"#,
    )
    .expect("write inputs");
    std::fs::write(&ledger, r#"{"entries":[]}"#).expect("write ledger");

    (
        dir,
        registry.to_string_lossy().into_owned(),
        inputs.to_string_lossy().into_owned(),
        ledger.to_string_lossy().into_owned(),
        out.to_string_lossy().into_owned(),
    )
}

fn write_invalid_timezone_registry_fixture() -> (tempfile::TempDir, String) {
    let dir = tempdir().expect("tempdir");
    let registry = dir.path().join("schedule-registry-invalid-timezone.json");

    std::fs::write(
        &registry,
        r#"{
  "definitions": [
    {
      "id": "broken-timezone",
      "dag_name": "atlas.catalog",
      "dag_version_policy": "run-latest",
      "trigger": {
        "Cron": {
          "expression": "0 2 * * *",
          "timezone": "Mars/Olympus"
        }
      },
      "queue": {"queue_name": "catalog", "tenant": "atlas"},
      "priority": "High",
      "concurrency": {
        "per_dag": 2,
        "per_queue": 4,
        "per_tenant": 4,
        "per_node_group": null
      },
      "catch_up": {"enabled": true, "max_catch_up_runs": 3}
    }
  ]
}"#,
    )
    .expect("write invalid timezone registry");

    (dir, registry.to_string_lossy().into_owned())
}

#[test]
fn schedule_submit_help_explains_inputs_and_ledger_flags() {
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["schedule", "submit", "--help"])
        .output()
        .expect("schedule submit help");
    assert!(output.status.success());

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains(
        "evaluate internal schedule trigger inputs into deterministic submission records"
    ));
    assert!(text.contains("json file containing now_unix_ms"));
    assert!(text.contains("--ledger"));
    assert!(text.contains("--out"));
}

#[test]
fn schedule_submit_writes_updated_ledger_through_binary() {
    let (_dir, registry, inputs, ledger, out) = write_schedule_submit_fixtures();
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args([
            "--json", "schedule", "submit", &registry, &inputs, "--ledger", &ledger, "--out", &out,
        ])
        .output()
        .expect("schedule submit");
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schedule submit json");
    assert_eq!(payload["command"], "dag.schedule.submit");
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["generated_requests"].as_array().map(Vec::len), Some(1));
    assert_eq!(payload["data"]["recorded_submissions"].as_array().map(Vec::len), Some(1));

    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("read updated ledger"))
            .expect("parse updated ledger");
    assert_eq!(written["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(written["entries"][0]["schedule_id"], "manual-ops");
}

#[test]
fn schedule_validate_reports_invalid_timezones_through_binary() {
    let (_dir, registry) = write_invalid_timezone_registry_fixture();
    let output = dag_command()
        .env("BIJUX_DAG_ENABLE_INTERNAL", "1")
        .args(["--json", "schedule", "validate", &registry])
        .output()
        .expect("schedule validate");
    assert!(!output.status.success());

    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("schedule validate json");
    assert_eq!(payload["command"], "dag.schedule.validate");
    assert_eq!(payload["ok"], false);
    assert!(payload["diagnostics"][0]["message"]
        .as_str()
        .expect("diagnostic message")
        .contains("unsupported cron timezone"));
}
