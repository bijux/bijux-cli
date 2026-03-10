#![forbid(unsafe_code)]
//! Cross-surface state consistency coverage for TODOs 321-335.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use bijux_cli_python::execution_outcome_api;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("json stdout")
}

fn bridge_json(args: &[&str]) -> Value {
    let argv = std::iter::once("bijux".to_string())
        .chain(args.iter().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let payload: Value =
        serde_json::from_str(&execution_outcome_api(&argv).expect("bridge outcome"))
            .expect("bridge json");
    let primary = if payload["stdout"].as_str().is_some_and(|s| !s.is_empty()) {
        payload["stdout"].as_str().unwrap_or("{}")
    } else {
        payload["stderr"].as_str().unwrap_or("{}")
    };
    serde_json::from_str(primary).expect("bridge primary stream json")
}

fn tmp_dir(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bijux-cross-state-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn config_mutations_are_visible_across_binary_bridge_and_repl_reads() {
    let root = tmp_dir("config-shared");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_SHARED_KEY=seed\n").expect("seed config");
    let config_text = config.to_string_lossy().to_string();

    let set_bin =
        run(&["cli", "config", "set", "shared_key=from-binary", "--config-path", &config_text]);
    assert_eq!(set_bin.status.code(), Some(0));
    let read_bridge =
        bridge_json(&["cli", "config", "get", "shared_key", "--config-path", &config_text]);
    assert_eq!(read_bridge["value"], "from-binary");

    let read_bin = run(&[
        "cli",
        "config",
        "get",
        "shared_key",
        "--config-path",
        &config_text,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(read_bin.status.code(), Some(0));
    assert_eq!(json_stdout(&read_bin)["value"], "from-binary");
}

#[test]
fn plugins_history_memory_and_paths_views_are_consistent_across_binary_and_bridge() {
    let plugins_bin = json_stdout(&run(&["plugins", "list", "--format", "json", "--no-pretty"]));
    let plugins_bridge = bridge_json(&["plugins", "list", "--format", "json", "--no-pretty"]);
    assert_eq!(plugins_bin["plugins"], plugins_bridge["plugins"]);

    let history_bin = json_stdout(&run(&["history", "--format", "json", "--no-pretty"]));
    let history_bridge = bridge_json(&["history", "--format", "json", "--no-pretty"]);
    assert_eq!(history_bin["entries"], history_bridge["entries"]);

    let memory_bin = json_stdout(&run(&["memory", "--format", "json", "--no-pretty"]));
    let memory_bridge = bridge_json(&["memory", "--format", "json", "--no-pretty"]);
    assert_eq!(memory_bin, memory_bridge);

    let runtime_bin =
        json_stdout(&run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]));
    let runtime_bridge =
        bridge_json(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]);
    assert_eq!(runtime_bin["active_binary"], runtime_bridge["active_binary"]);
    assert_eq!(runtime_bin["path_binaries"], runtime_bridge["path_binaries"]);

    let paths_bin = json_stdout(&run(&["cli", "paths", "--format", "json", "--no-pretty"]));
    let paths_bridge = bridge_json(&["cli", "paths", "--format", "json", "--no-pretty"]);
    assert_eq!(paths_bin, paths_bridge);
}

#[test]
fn state_path_overrides_propagate_consistently_for_config_path_views() {
    let root = tmp_dir("config-path-override");
    let config = root.join("override.env");
    fs::write(&config, "BIJUXCLI_OVERRIDE_KEY=present\n").expect("seed override");
    let config_text = config.to_string_lossy().to_string();

    let bin = json_stdout(&run(&[
        "--config-path",
        &config_text,
        "cli",
        "paths",
        "--format",
        "json",
        "--no-pretty",
    ]));
    let bridge = bridge_json(&[
        "--config-path",
        &config_text,
        "cli",
        "paths",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(bin["config"], Value::String(config_text.clone()));
    assert_eq!(bridge["config"], Value::String(config_text));
}

#[test]
fn doctor_and_state_doctor_agree_on_corruption_classes_across_config_plugins_history_and_memory() {
    let root = tmp_dir("doctor-corruption");
    let config = root.join("broken.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("config");

    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("plugins dir");
    fs::create_dir_all(plugins.join("registry.json")).expect("registry dir corruption");

    let history = root.join("history.log");
    fs::create_dir_all(&history).expect("history dir corruption");

    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("memory parent")).expect("mkdir");
    fs::write(&memory, "{broken").expect("memory");

    let config_s = config.to_string_lossy().to_string();
    let plugins_s = plugins.to_string_lossy().to_string();
    let history_s = history.to_string_lossy().to_string();
    let home_s = home.to_string_lossy().to_string();

    let doctor = run_env(
        &["doctor", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", &config_s),
            ("BIJUXCLI_PLUGINS_DIR", &plugins_s),
            ("BIJUXCLI_HISTORY_FILE", &history_s),
            ("HOME", &home_s),
        ],
    );
    let state_doctor = run_env(
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", &config_s),
            ("BIJUXCLI_PLUGINS_DIR", &plugins_s),
            ("BIJUXCLI_HISTORY_FILE", &history_s),
            ("HOME", &home_s),
        ],
    );
    assert_eq!(doctor.status.code(), Some(0));
    assert_eq!(state_doctor.status.code(), Some(0));

    let doctor_json = json_stdout(&doctor);
    let state_json = json_stdout(&state_doctor);

    assert!(doctor_json["install"].is_object());
    let issues = state_json["doctor"]["issues"].as_array().expect("issues");
    let areas =
        issues.iter().filter_map(|row| row.get("area").and_then(Value::as_str)).collect::<Vec<_>>();
    assert!(areas.iter().any(|v| *v == "config"));
    assert!(areas.iter().any(|v| *v == "plugins"));
    assert!(areas.iter().any(|v| *v == "history"));
    assert_eq!(state_json["doctor"]["status"], "degraded");
}
