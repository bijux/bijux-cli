#![forbid(unsafe_code)]
//! Replay minimized plugin/history/memory corruption campaign cases.

use std::fs;
use std::path::Path;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_case(case_file: &Path) {
    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(case_file).expect("read case"))
            .expect("parse case json");

    let args: Vec<String> = json["args"]
        .as_array()
        .expect("args array")
        .iter()
        .map(|v| v.as_str().expect("arg str").to_owned())
        .collect();

    let temp = std::env::temp_dir().join(format!(
        "bijux-plugin-state-campaign-repro-{}-{}",
        case_file.file_stem().and_then(|s| s.to_str()).unwrap_or("case"),
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("mkdir temp");

    let home = temp.join("home");
    let plugins = temp.join("plugins");
    let history = temp.join("history.log");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("memory parent")).expect("mkdir memory parent");
    fs::create_dir_all(&plugins).expect("mkdir plugins");

    fs::write(plugins.join("registry.json"), json["registry_text"].as_str().unwrap_or_default())
        .expect("write registry");
    fs::write(&history, json["history_text"].as_str().unwrap_or_default()).expect("write history");
    fs::write(&memory, json["memory_text"].as_str().unwrap_or_default()).expect("write memory");

    let out = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .env("HOME", &home)
        .env("BIJUXCLI_PLUGINS_DIR", &plugins)
        .env("BIJUXCLI_HISTORY_FILE", &history)
        .output()
        .expect("run case");

    assert!(
        matches!(out.status.code(), Some(0) | Some(1) | Some(2)),
        "case {} crashed with status {:?}",
        case_file.display(),
        out.status.code()
    );
}

#[test]
fn minimized_plugin_state_corruption_cases_replay_without_crashing() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fuzz/minimized_cases/plugin_state_corruption_minimized_cases");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("read case directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "must retain at least one minimized corruption case");

    for file in files {
        run_case(&file);
    }
}
