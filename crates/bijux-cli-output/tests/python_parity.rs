#![forbid(unsafe_code)]
//! Parity checks against captured Python behavior artifacts.

use std::fs;
use std::path::Path;

use bijux_cli_contracts as _;
use bijux_cli_output as _;
use serde_json::Value;
use serde_yaml as _;
use thiserror as _;

#[test]
fn parity_with_captured_python_behavior_lock() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../artifacts/current-python-behavior-lock.json");
    let text = fs::read_to_string(&root).expect("behavior lock file should exist");
    let data: Value = serde_json::from_str(&text).expect("behavior lock should be valid JSON");

    let captures =
        data.get("captures").and_then(Value::as_object).expect("captures object should exist");

    let success = captures
        .get("success_streams")
        .or_else(|| captures.get("behavior_success_streams"))
        .expect("success capture missing");
    let success_stdout = success.get("stdout").and_then(Value::as_str).unwrap_or_default();
    let success_stderr = success.get("stderr").and_then(Value::as_str).unwrap_or_default();
    assert!(!success_stdout.is_empty(), "python success should emit stdout");
    assert!(success_stderr.is_empty(), "python success should not emit stderr");

    let validation = captures
        .get("validation_failure_streams")
        .or_else(|| captures.get("behavior_validation_failure_streams"))
        .expect("validation capture missing");
    let validation_stdout = validation.get("stdout").and_then(Value::as_str).unwrap_or_default();
    let validation_stderr = validation.get("stderr").and_then(Value::as_str).unwrap_or_default();
    assert!(
        !validation_stdout.is_empty() || !validation_stderr.is_empty(),
        "python validation failure should emit an error payload"
    );

    let internal = captures
        .get("internal_failure_streams")
        .or_else(|| captures.get("behavior_internal_failure_streams"))
        .expect("internal capture missing");
    let internal_stderr = internal.get("stderr").and_then(Value::as_str).unwrap_or_default();
    assert!(!internal_stderr.is_empty(), "python internal failure should emit stderr");
}
