#![forbid(unsafe_code)]
//! Replay minimized adversarial fs/process reproducers.

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_case(path: &Path) {
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(path).expect("read case")).expect("parse case");

    let args: Vec<String> = json["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|v| v.as_str().expect("arg str").to_owned())
        .collect();

    let temp = std::env::temp_dir().join(format!(
        "bijux-fs-process-repro-{}-{}",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("case"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("mkdir temp");

    let config = temp.join("active.env");
    let plugins = temp.join("plugins");
    let history = temp.join("history.log");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(&config, json["config_text"].as_str().unwrap_or("BIJUXCLI_ALPHA=1\n"))
        .expect("write config");
    fs::write(
        plugins.join("registry.json"),
        json["registry_text"].as_str().unwrap_or("{\"plugins\":[]}"),
    )
    .expect("write registry");
    fs::write(&history, json["history_text"].as_str().unwrap_or("status\n"))
        .expect("write history");

    let mut expanded = Vec::new();
    for arg in args {
        match arg.as_str() {
            "<CONFIG_PATH>" => expanded.push(config.display().to_string()),
            _ => expanded.push(arg),
        }
    }

    let out = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(expanded)
        .env("BIJUXCLI_CONFIG_PATH", &config)
        .env("BIJUXCLI_PLUGINS_DIR", &plugins)
        .env("BIJUXCLI_HISTORY_FILE", &history)
        .output()
        .expect("run repro case");

    assert!(
        matches!(out.status.code(), Some(0) | Some(1) | Some(2)),
        "reproducer {} returned unexpected status {:?}",
        path.display(),
        out.status.code()
    );
}

#[test]
fn minimized_adversarial_cases_replay_without_panics() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fuzz/adversarial_fs_process_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read minimized adversarial cases")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "retain at least one adversarial minimized case");

    for file in files {
        run_case(&file);
    }
}
