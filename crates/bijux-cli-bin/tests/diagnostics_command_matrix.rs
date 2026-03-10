#![forbid(unsafe_code)]
//! Diagnostics and inspect command behavior matrix coverage.
//! test_type: diagnostics-structured-truth

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-diagnostics-matrix-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn inspect_text_json_yaml_quiet_and_trace_modes() {
    let text = run(&["inspect", "--format", "text"]);
    assert_eq!(text.status.code(), Some(0));
    assert!(String::from_utf8(text.stdout).expect("utf-8").contains('"'));

    let json_out = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(json_out.status.code(), Some(0));
    let json_payload = json(&json_out.stdout);
    assert_eq!(json_payload["status"], "ok");

    let yaml = run(&["inspect", "--format", "yaml", "--pretty"]);
    assert_eq!(yaml.status.code(), Some(0));
    assert!(String::from_utf8(yaml.stdout).expect("utf-8").contains("status: ok"));

    let quiet = run(&["inspect", "--quiet"]);
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let trace = run(&["--log-level", "trace", "inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(trace.status.code(), Some(0));
    let trace_payload = json(&trace.stdout);
    assert_eq!(json_payload["route_sources"], trace_payload["route_sources"]);
}

#[test]
fn doctor_text_json_and_corrupted_state_coverage() {
    let doctor_text = run(&["doctor", "--format", "text"]);
    assert_eq!(doctor_text.status.code(), Some(0));
    assert!(!doctor_text.stdout.is_empty());

    let doctor_json = run(&["doctor", "--format", "json", "--no-pretty"]);
    assert_eq!(doctor_json.status.code(), Some(0));
    let doctor_json_payload = json(&doctor_json.stdout);
    assert!(doctor_json_payload["status"].is_string());

    let temp = make_temp_dir("doctor-corruptions");
    let config = temp.join("corrupt.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write config");
    let plugins = temp.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(plugins.join("registry.json"), "{\"version\":\"v1\",").expect("partial registry");
    let history = temp.join("bad.history");
    fs::write(&history, "{oops:true}").expect("write history");
    let home = temp.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir memory");
    fs::write(&memory, "{broken").expect("write memory");

    let out = run_with_env(
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", config.display().to_string()),
            ("BIJUXCLI_PLUGINS_DIR", plugins.display().to_string()),
            ("BIJUXCLI_HISTORY_FILE", history.display().to_string()),
            ("HOME", home.display().to_string()),
        ],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload = json(&out.stdout);
    assert_eq!(payload["doctor"]["status"], "degraded");
    let issues = payload["doctor"]["issues"].as_array().expect("issues");
    assert!(issues.iter().any(|i| i["area"] == "config"));
}

#[test]
fn dev_cli_routes_registry_env_contracts_json_shape_stability() {
    let routes = json(&run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);
    assert!(routes["routes"].is_array());
    assert!(routes["aliases"].is_array());

    let registry =
        json(&run(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    assert!(registry["registry"].is_array());
    assert!(registry["ownership"].is_object());

    let env = json(&run(&["dev", "cli", "env", "--format", "json", "--no-pretty"]).stdout);
    assert!(env["active"].is_object());
    assert!(env["source_precedence"].is_array());

    let contracts =
        json(&run(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]).stdout);
    assert!(contracts["contracts"].is_array());
    assert!(contracts["schema_version"].is_string());
}

#[test]
fn diagnostics_consistency_across_inspect_doctor_and_dev_surfaces() {
    let inspect = json(&run(&["inspect", "--format", "json", "--no-pretty"]).stdout);
    let routes = json(&run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]).stdout);
    let doctor = json(&run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]).stdout);

    let inspect_routes: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route sources")
        .iter()
        .map(|item| {
            item["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|s| s.as_str().expect("segment str"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let dev_routes: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .map(|item| {
            item["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .map(|s| s.as_str().expect("segment str"))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    assert_eq!(inspect_routes, dev_routes);
    assert_eq!(inspect["status"], "ok");
    assert!(doctor["issues"].is_object());
}
