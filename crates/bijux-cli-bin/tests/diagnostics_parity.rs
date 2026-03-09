#![forbid(unsafe_code)]
//! Inspect and developer diagnostics parity coverage.

use std::collections::BTreeSet;
use std::process::Command;

use bijux_cli_core as _;
use libc as _;
use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

#[test]
fn inspect_supports_text_json_yaml_modes() {
    let json = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert!(json.status.success());
    let payload = parse_json(&json.stdout);
    assert_eq!(payload["status"], "ok");
    assert!(payload["route_sources"].is_array());

    let yaml = run(&["inspect", "--format", "yaml", "--pretty"]);
    assert!(yaml.status.success());
    let yaml_text = String::from_utf8(yaml.stdout).expect("yaml utf-8");
    assert!(yaml_text.contains("status: ok"));
    assert!(yaml_text.contains("route_sources:"));

    let text = run(&["inspect", "--format", "text"]);
    assert!(text.status.success());
    let text_body = String::from_utf8(text.stdout).expect("text utf-8");
    assert!(text_body.contains('"'));
}

#[test]
fn inspect_trace_and_quiet_behaviors_are_stable() {
    let plain = run(&["inspect", "--format", "json", "--no-pretty"]);
    let trace = run(&[
        "--log-level",
        "trace",
        "inspect",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert!(plain.status.success());
    assert!(trace.status.success());

    let plain_value = parse_json(&plain.stdout);
    let trace_value = parse_json(&trace.stdout);
    assert_eq!(plain_value["status"], trace_value["status"]);
    assert_eq!(plain_value["route_sources"], trace_value["route_sources"]);

    let quiet = run(&["inspect", "--quiet"]);
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());
}

#[test]
fn inspect_failure_normalization_routes_to_stderr() {
    let out = run(&["inspect", "unexpected", "--format", "json", "--no-pretty"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(stderr.contains("Usage: bijux"));
    assert!(stderr.contains("inspect"));
}

#[test]
fn inspect_and_dev_routes_are_internally_consistent() {
    let inspect = parse_json(&run(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let routes = parse_json(&run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);

    let inspect_set: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route_sources array")
        .iter()
        .map(|item| {
            item["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|seg| seg.as_str().expect("string"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let route_set: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes array")
        .iter()
        .map(|item| {
            item["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|seg| seg.as_str().expect("string"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    assert_eq!(inspect_set, route_set);
}

#[test]
fn dev_diagnostics_payloads_expose_metadata_contracts() {
    let registry = parse_json(&run(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    assert!(registry["registry"].is_array());
    assert!(registry["ownership"].is_object());
    assert!(registry["precedence"].is_array());

    let env = parse_json(&run(&["dev", "cli", "env", "--format", "json", "--no-pretty"]).stdout);
    assert!(env["source_precedence"].is_array());
    assert!(env["active"]["config_file"].is_string());

    let doctor = parse_json(&run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]).stdout);
    assert!(doctor["issues"]["config"].is_array());
    assert!(doctor["issues"]["paths"].is_array());
    assert!(doctor["issues"]["plugins"].is_array());

    let contracts = parse_json(&run(&[
        "dev",
        "cli",
        "contracts",
        "--format",
        "json",
        "--no-pretty",
    ]).stdout);
    assert!(contracts["contracts"].is_array());
    assert!(contracts["schema_version"].is_string());
    assert!(contracts["runtime_version"].is_string());
}
