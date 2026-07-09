#![forbid(unsafe_code)]
//! Cross-surface parity laws owned by the Python bridge crate.

use std::collections::BTreeSet;

use bijux_cli::api::runtime::run_app;
use bijux_cli_python::{command_tree_introspection_api, execution_outcome_api};
use serde_json::Value;

fn bridge_outcome(args: &[&str]) -> Value {
    let argv = std::iter::once("bijux".to_string())
        .chain(args.iter().map(|value| (*value).to_string()))
        .collect::<Vec<_>>();
    serde_json::from_str(&execution_outcome_api(&argv).expect("bridge outcome json"))
        .expect("bridge outcome payload")
}

fn core_outcome(args: &[&str]) -> Value {
    let argv = std::iter::once("bijux".to_string())
        .chain(args.iter().map(|value| (*value).to_string()))
        .collect::<Vec<_>>();
    let result = run_app(&argv).expect("core run");
    serde_json::json!({
        "exit_code": result.exit_code,
        "stdout": result.stdout,
        "stderr": result.stderr,
    })
}

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("json payload")
}

fn root_namespaces_from_inspect(payload: &Value) -> BTreeSet<String> {
    payload["builtins"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect()
}

#[test]
fn bridge_and_core_agree_on_exit_codes_for_representative_routes() {
    let cases = [
        vec!["status", "--format", "json", "--no-pretty"],
        vec!["doctor", "--format", "json", "--no-pretty"],
        vec!["config", "get"],
        vec!["plugins", "uninstall"],
        vec!["atlas", "does-not-exist"],
    ];

    for args in cases {
        let bridge = bridge_outcome(&args);
        let core = core_outcome(&args);
        assert_eq!(bridge["exit_code"], core["exit_code"], "exit drift for {args:?}");
    }
}

#[test]
fn bridge_and_core_agree_on_stream_routing_for_success_and_usage_failure() {
    let success_args = ["status", "--format", "json", "--no-pretty"];
    let success_bridge = bridge_outcome(&success_args);
    let success_core = core_outcome(&success_args);
    assert_eq!(success_bridge["stdout"], success_core["stdout"]);
    assert_eq!(success_bridge["stderr"], success_core["stderr"]);
    assert!(success_bridge["stderr"].as_str().unwrap_or_default().is_empty());

    let usage_args = ["unknown-command"];
    let usage_bridge = bridge_outcome(&usage_args);
    let usage_core = core_outcome(&usage_args);
    assert_eq!(usage_bridge["stdout"], usage_core["stdout"]);
    assert_eq!(usage_bridge["stderr"], usage_core["stderr"]);
    assert!(usage_bridge["stdout"].as_str().unwrap_or_default().is_empty());
    assert!(!usage_bridge["stderr"].as_str().unwrap_or_default().is_empty());
}

#[test]
fn bridge_and_core_agree_on_payload_shape_for_status_doctor_and_inspect() {
    let cases = [
        vec!["status", "--format", "json", "--no-pretty"],
        vec!["doctor", "--format", "json", "--no-pretty"],
        vec!["inspect", "--format", "json", "--no-pretty"],
    ];

    for args in cases {
        let bridge = bridge_outcome(&args);
        let core = core_outcome(&args);
        assert_eq!(bridge["exit_code"], core["exit_code"], "exit drift for {args:?}");
        let bridge_payload = parse_json(bridge["stdout"].as_str().unwrap_or_default());
        let core_payload = parse_json(core["stdout"].as_str().unwrap_or_default());
        assert_eq!(bridge_payload, core_payload, "payload drift for {args:?}");
    }
}

#[test]
fn bridge_command_tree_introspection_matches_core_root_namespaces() {
    let core_inspect = core_outcome(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(core_inspect["exit_code"], 0);
    let core_payload = parse_json(core_inspect["stdout"].as_str().unwrap_or_default());
    let core_namespaces = root_namespaces_from_inspect(&core_payload);

    let bridge_tree = parse_json(&command_tree_introspection_api());
    let bridge_namespaces: BTreeSet<String> = bridge_tree["namespaces"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();

    assert_eq!(bridge_tree["root"], "bijux");
    assert_eq!(bridge_tree["source"], "runtime-inspect");
    assert_eq!(bridge_namespaces, core_namespaces);
}
