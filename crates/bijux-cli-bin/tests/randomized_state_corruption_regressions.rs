#![forbid(unsafe_code)]
//! Replays minimized corrupted-state reproducers to prevent regressions.

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use libc as _;
use serde_json::Value;

struct Reproducer {
    command: Vec<String>,
    config_text: Option<String>,
    history_text: Option<String>,
    memory_text: Option<String>,
    registry_text: Option<String>,
}

fn run_case(path: &Path) {
    let text = fs::read_to_string(path).expect("read reproducer");
    let json: Value = serde_json::from_str(&text).expect("parse reproducer");
    let command = json["command"]
        .as_array()
        .expect("command array")
        .iter()
        .map(|v| v.as_str().expect("command token").to_owned())
        .collect();
    let case = Reproducer {
        command,
        config_text: json["config_text"].as_str().map(ToOwned::to_owned),
        history_text: json["history_text"].as_str().map(ToOwned::to_owned),
        memory_text: json["memory_text"].as_str().map(ToOwned::to_owned),
        registry_text: json["registry_text"].as_str().map(ToOwned::to_owned),
    };

    let temp = std::env::temp_dir().join(format!(
        "bijux-state-corruption-reproducer-{}-{}",
        path.file_stem().and_then(|s| s.to_str()).unwrap_or("case"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("mkdir temp");

    let home = temp.join("home");
    let plugins = temp.join("plugins");
    let config = temp.join("config.env");
    let history = temp.join("history.log");
    let memory = home.join(".bijux").join(".memory.json");

    fs::create_dir_all(home.join(".bijux")).expect("mkdir home");
    fs::create_dir_all(&plugins).expect("mkdir plugins");

    if let Some(v) = case.config_text {
        fs::write(&config, v).expect("write config");
    }
    if let Some(v) = case.history_text {
        fs::write(&history, v).expect("write history");
    }
    if let Some(v) = case.memory_text {
        fs::write(&memory, v).expect("write memory");
    }
    if let Some(v) = case.registry_text {
        fs::write(plugins.join("registry.json"), v).expect("write registry");
    }

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(&case.command)
        .env("HOME", &home)
        .env("BIJUXCLI_PLUGINS_DIR", &plugins)
        .env("BIJUXCLI_HISTORY_FILE", &history)
        .env("BIJUXCLI_CONFIG_PATH", &config);

    let out = cmd.output().expect("run reproducer command");
    assert!(
        matches!(out.status.code(), Some(0) | Some(1)),
        "reproducer {} crashed with status {:?}",
        path.display(),
        out.status.code()
    );
}

#[test]
fn minimized_corrupted_state_reproducers_replay_without_crashing() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fuzz/state_corruption_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read minimized directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "at least one minimized reproducer must be retained");

    for file in files {
        run_case(&file);
    }
}
