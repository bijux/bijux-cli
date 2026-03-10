#![forbid(unsafe_code)]
//! Inspect and developer diagnostics parity coverage.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use bijux_cli_python as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-diagnostics-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn inspect_schema_consistency_holds_across_text_json_yaml_modes() {
    let json = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert!(json.status.success());
    let payload = parse_json(&json.stdout);
    assert_eq!(payload["status"], "ok");
    assert!(payload["route_sources"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty()));
    assert!(payload["builtins"].is_array());
    assert!(payload.get("contracts").is_some());

    let yaml = run(&["inspect", "--format", "yaml", "--pretty"]);
    assert!(yaml.status.success());
    let yaml_text = String::from_utf8(yaml.stdout).expect("yaml utf-8");
    assert!(yaml_text.contains("status: ok"));
    assert!(yaml_text.contains("route_sources:"));
    assert!(yaml_text.contains("contracts:"));

    let text = run(&["inspect", "--format", "text"]);
    assert!(text.status.success());
    let text_body = String::from_utf8(text.stdout).expect("text utf-8");
    assert!(text_body.contains("status"));
    assert!(text_body.contains("route_sources"));
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
    let routes =
        parse_json(&run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);

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
    let registry =
        parse_json(&run(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    assert!(registry["registry"].is_array());
    assert!(registry["ownership"].is_object());
    assert!(registry["precedence"].is_array());

    let env = parse_json(&run(&["dev", "cli", "env", "--format", "json", "--no-pretty"]).stdout);
    assert!(env["source_precedence"].is_array());
    assert!(env["active"]["config_file"].is_string());

    let doctor =
        parse_json(&run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]).stdout);
    assert!(doctor["issues"]["config"].is_array());
    assert!(doctor["issues"]["paths"].is_array());
    assert!(doctor["issues"]["plugins"].is_array());

    let contracts =
        parse_json(&run(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]).stdout);
    assert!(contracts["contracts"].is_array());
    assert!(contracts["schema_version"].is_string());
    assert!(contracts["runtime_version"].is_string());

    let state_audit = parse_json(
        &run(&[
            "dev",
            "cli",
            "state-audit",
            "--format",
            "json",
            "--no-pretty",
        ])
        .stdout,
    );
    assert!(state_audit["paths"]["config"]["path"].is_string());
    assert!(state_audit["paths"]["history"]["path"].is_string());
    assert!(state_audit["paths"]["plugins_registry"]["path"].is_string());
    assert!(state_audit["paths"]["memory"]["path"].is_string());

    let state_doctor = parse_json(
        &run(&[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ])
        .stdout,
    );
    assert!(state_doctor["doctor"]["status"].is_string());
    assert!(state_doctor["doctor"]["issues"].is_array());
}

#[test]
fn state_doctor_reports_config_duplicate_keys() {
    let temp = make_temp_dir("state-doctor-duplicates");
    let config = temp.join("dupes.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=2\n").expect("write config");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_CONFIG", config.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues
        .iter()
        .any(|item| item["message"] == "duplicate config keys found"));
}

#[test]
fn state_doctor_reports_history_malformed_storage() {
    let temp = make_temp_dir("state-doctor-history-malformed");
    let history = temp.join("malformed.history");
    fs::write(&history, "{\"oops\":true}").expect("write malformed history");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|item| item["area"] == "history"));
}

#[test]
fn state_doctor_reports_config_corruption_and_partial_write_artifact() {
    let temp = make_temp_dir("state-doctor-config-corrupt");
    let config = temp.join("corrupt.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN_LINE\n").expect("write malformed config");
    fs::write(config.with_extension("tmp"), "BIJUXCLI_ALPHA=stale\n").expect("write stale tmp");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_CONFIG", config.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|item| item["area"] == "config"));
    assert!(issues
        .iter()
        .any(|item| item["message"] == "partial-write rollback artifact detected"));
}

#[test]
fn state_doctor_reports_memory_wrong_type_entries() {
    let temp = make_temp_dir("state-doctor-memory-wrong-type");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "{\"alpha\":1,\"beta\":{\"v\":1}}").expect("seed memory");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("HOME", home.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|item| item["area"] == "memory"));
}

#[test]
fn state_doctor_recovers_partial_registry_and_cleans_stale_backup() {
    let temp = make_temp_dir("state-doctor-registry-repair");
    let plugins = temp.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    let registry = plugins.join("registry.json");
    let backup = plugins.join("registry.bak");
    fs::write(&registry, "{\"version\":\"v1\",").expect("write partial registry");
    fs::write(&backup, "{}\n").expect("write stale backup");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert!(payload["doctor"]["repairs"].is_array());
    assert!(!backup.exists(), "state doctor should clean stale backup");
}

#[test]
fn state_doctor_json_and_text_contracts_are_stable() {
    let json_out = run(&[
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert!(json_out.status.success());
    let json_payload = parse_json(&json_out.stdout);
    assert_eq!(json_payload["runtime"], "dev-cli");
    assert!(json_payload["doctor"]["status"].is_string());
    assert!(json_payload["doctor"]["issues"].is_array());
    assert!(json_payload["doctor"]["repairs"].is_array());

    let text_out = run(&["dev", "cli", "state-doctor", "--format", "text"]);
    assert!(text_out.status.success());
    let text = String::from_utf8(text_out.stdout).expect("text utf-8");
    assert!(text.contains("\"runtime\": \"dev-cli\""));
    assert!(text.contains("\"doctor\""));
}

#[test]
fn state_doctor_failure_routes_output_to_stderr_only() {
    let out = run(&["dev", "cli", "state-doctor", "--format", "nope"]);
    assert_ne!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn state_doctor_exit_codes_cover_healthy_degraded_and_usage_failure() {
    let healthy = run(&[
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(healthy.status.code(), Some(0));

    let temp = make_temp_dir("state-doctor-exit-degraded");
    let config = temp.join("corrupt.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write malformed config");
    let degraded = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_CONFIG", config.display().to_string())],
    );
    assert_eq!(degraded.status.code(), Some(0));
    let payload = parse_json(&degraded.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");

    let usage = run(&["dev", "cli", "state-doctor", "--format", "invalid"]);
    assert_ne!(usage.status.code(), Some(0));
    assert!(usage.stdout.is_empty());
    assert!(!usage.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn state_doctor_reports_config_invalid_encoding() {
    let temp = make_temp_dir("state-doctor-config-invalid-encoding");
    let config = temp.join("invalid-encoding.env");
    fs::write(&config, vec![0xff, 0xfe, b'=', b'1', b'\n']).expect("write invalid bytes");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_CONFIG", config.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|item| {
        item["area"] == "config"
            && item["message"]
                .as_str()
                .map(|msg| msg.contains("stream did not contain valid UTF-8"))
                .unwrap_or(false)
    }));
}

#[cfg(unix)]
#[test]
fn state_doctor_reports_plugin_registry_invalid_encoding() {
    let temp = make_temp_dir("state-doctor-registry-invalid-encoding");
    let plugins = temp.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    let registry = plugins.join("registry.json");
    fs::write(&registry, vec![0xff, 0xfe, b'{', b'}']).expect("write invalid bytes");

    let out = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"]
        .as_array()
        .expect("issues array");
    assert!(issues.iter().any(|item| {
        item["area"] == "plugins"
            && item["message"]
                .as_str()
                .map(|msg| msg.contains("stream did not contain valid UTF-8"))
                .unwrap_or(false)
    }));
}
