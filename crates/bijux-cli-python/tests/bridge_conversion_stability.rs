#![forbid(unsafe_code)]
//! Python bridge conversion stability checks for success and error envelope handling.
//! test_type: bridge-conversion-stability

use bijux_cli as _;
use bijux_cli_python::{execution_facade_api, execution_outcome_api};
use serde_json::Value;

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("json payload")
}

#[test]
fn fuzz_bridge_conversion_of_success_envelopes_is_stable() {
    let argv = vec![
        "bijux".to_string(),
        "cli".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ];
    let a = parse_json(&execution_facade_api(&argv).expect("facade a"));
    let b = parse_json(&execution_facade_api(&argv).expect("facade b"));
    assert_eq!(a, b);
    let status = a["status"].as_str().expect("status should be a string");
    assert!(matches!(status, "ok" | "warning" | "degraded"));
    assert!(a.as_object().map(|o| o.len()).unwrap_or(0) > 1);
}

#[test]
fn fuzz_bridge_conversion_of_error_envelopes_is_stable() {
    let argv =
        vec!["bijux".to_string(), "cli".to_string(), "plugins".to_string(), "check".to_string()];
    let a = parse_json(&execution_outcome_api(&argv).expect("outcome a"));
    let b = parse_json(&execution_outcome_api(&argv).expect("outcome b"));
    assert_eq!(a, b);
    assert_eq!(a["error_kind"], "UsageError");
    assert!(a["exit_code"].as_i64().unwrap_or(0) != 0);
    assert!(a.get("stderr").is_some());
}
