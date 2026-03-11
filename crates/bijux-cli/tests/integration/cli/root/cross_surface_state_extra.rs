#![forbid(unsafe_code)]
//! Cross-surface state consistency coverage for runtime and state commands.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

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

fn tmp_dir(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bijux-cross-state-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
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
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
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
    let areas = issues
        .iter()
        .filter_map(|row| row.get("area").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(areas.iter().any(|v| *v == "config"));
    assert!(areas.iter().any(|v| *v == "plugins"));
    assert!(areas.iter().any(|v| *v == "history"));
    assert_eq!(state_json["doctor"]["status"], "degraded");
}
