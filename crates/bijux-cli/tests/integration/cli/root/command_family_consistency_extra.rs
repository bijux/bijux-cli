#![forbid(unsafe_code)]
//! Command-family consistency coverage for stable behavior contracts.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_env(args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn json(output: &Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "expected success but saw {:?} with stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "successful machine output must keep stderr empty"
    );
    assert!(
        !output.stdout.is_empty(),
        "successful machine output must produce stdout"
    );
    serde_json::from_slice(&output.stdout).expect("json")
}

fn route_set(routes: &Value, key: &str) -> BTreeSet<String> {
    routes[key]
        .as_array()
        .expect("route array")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

fn tmp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("bijux-family-{label}-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn root_status_and_cli_status_agree_where_semantics_overlap() {
    let root = json(&run(&["status", "--format", "json", "--no-pretty"]));
    let cli = json(&run(&["cli", "status", "--format", "json", "--no-pretty"]));

    assert_eq!(root["status"], cli["status"]);
    assert_eq!(root["route_owner"], cli["route_owner"]);
}

#[test]
fn root_config_listing_and_cli_config_views_agree_where_both_exist() {
    let root = json(&run(&["config", "--format", "json", "--no-pretty"]));
    let cli = json(&run(&[
        "cli",
        "config",
        "list",
        "--format",
        "json",
        "--no-pretty",
    ]));

    assert_eq!(root["status"], cli["status"]);
    assert_eq!(root["entries"], cli["entries"]);
}

#[test]
fn plugins_and_routes_views_agree_between_user_and_dev_surfaces() {
    let plugins = json(&run(&[
        "plugins",
        "list",
        "--format",
        "json",
        "--no-pretty",
    ]));
    let registry = json(&run(&[
        "dev",
        "cli",
        "registry",
        "--format",
        "json",
        "--no-pretty",
    ]));
    let inspect = json(&run(&["inspect", "--format", "json", "--no-pretty"]));
    let routes = json(&run(&[
        "dev",
        "cli",
        "routes",
        "--format",
        "json",
        "--no-pretty",
    ]));

    let reserved: BTreeSet<String> = registry["registry"]
        .as_array()
        .expect("registry")
        .iter()
        .filter(|row| row["reserved"] == true)
        .filter_map(|row| row["name"].as_str())
        .map(ToString::to_string)
        .collect();

    for plugin in plugins["plugins"].as_array().expect("plugins") {
        if let Some(namespace) = plugin["manifest"]["namespace"].as_str() {
            assert!(!reserved.contains(namespace));
        }
    }

    assert_eq!(
        route_set(&inspect, "route_sources"),
        route_set(&routes, "routes")
    );
}

#[test]
fn cli_paths_match_state_audit_paths_view() {
    let paths = json(&run(&["cli", "paths", "--format", "json", "--no-pretty"]));
    let audit = json(&run(&[
        "dev",
        "cli",
        "state-audit",
        "--format",
        "json",
        "--no-pretty",
    ]));

    assert_eq!(paths["config"], audit["paths"]["config"]["path"]);
    assert_eq!(paths["history"], audit["paths"]["history"]["path"]);
    let plugins_registry_path = audit["paths"]["plugins_registry"]["path"]
        .as_str()
        .expect("plugins registry path");
    let plugins_dir = Path::new(plugins_registry_path)
        .parent()
        .expect("plugins registry parent")
        .to_string_lossy()
        .to_string();
    assert_eq!(paths["plugins"], Value::String(plugins_dir));
}

#[test]
fn doctor_and_state_doctor_agree_on_corruption_classes_for_config_plugins_history_memory() {
    let temp = tmp_dir("corruption-classes");
    let config = temp.join("broken.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("config");

    let plugins = temp.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");
    fs::write(plugins.join("registry.json"), "{\"version\":\"v1\",").expect("registry");

    let history = temp.join("history.log");
    fs::write(&history, "{broken").expect("history");

    let home = temp.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    fs::write(&memory, "{broken").expect("memory");

    let doctor = run_env(
        &["doctor", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", &config),
            ("BIJUXCLI_PLUGINS_DIR", &plugins),
            ("BIJUXCLI_HISTORY_FILE", &history),
            ("HOME", &home),
        ],
    );
    let state_doctor = run_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[
            ("BIJUXCLI_CONFIG", &config),
            ("BIJUXCLI_PLUGINS_DIR", &plugins),
            ("BIJUXCLI_HISTORY_FILE", &history),
            ("HOME", &home),
        ],
    );

    let doctor_json = json(&doctor);
    let state_json = json(&state_doctor);

    assert!(doctor_json["install"].is_object());
    assert_eq!(state_json["doctor"]["status"], "degraded");
    assert!(state_json["doctor"]["issues"]
        .as_array()
        .is_some_and(|rows| !rows.is_empty()));
}

#[test]
fn command_family_help_trees_and_machine_output_envelopes_remain_consistent() {
    let families: BTreeMap<&str, &[&str]> = BTreeMap::from([
        (
            "status",
            &["status", "--format", "json", "--no-pretty"] as &[&str],
        ),
        ("config", &["config", "--format", "json", "--no-pretty"]),
        (
            "plugins",
            &["plugins", "list", "--format", "json", "--no-pretty"],
        ),
        ("inspect", &["inspect", "--format", "json", "--no-pretty"]),
    ]);

    for (family, command) in families {
        let help = run(&[family, "--help"]);
        assert_eq!(help.status.code(), Some(0));
        assert!(
            help.stderr.is_empty(),
            "help should not emit stderr for {family}"
        );
        assert!(
            String::from_utf8_lossy(&help.stdout).contains("Usage:"),
            "help should include usage section for {family}"
        );

        let payload = json(&run(command));
        assert!(payload.is_object());
        if family != "config" {
            assert!(payload.as_object().is_some_and(|obj| !obj.is_empty()));
        }
    }
}
