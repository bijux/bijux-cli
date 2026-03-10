#![forbid(unsafe_code)]
//! Contracts for top-level dev-cli cockpit commands.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args, &[]);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("valid json")
}

#[test]
fn cockpit_json_contracts_are_stable() {
    let commands = [
        (["dev", "cli", "dashboard"], "dashboard"),
        (["dev", "cli", "quickcheck"], "quickcheck"),
        (["dev", "cli", "truth"], "truth"),
        (["dev", "cli", "blockers"], "blockers"),
        (["dev", "cli", "next"], "next"),
    ];
    for (command, key) in commands {
        let first = run_ok_json(&command);
        let second = run_ok_json(&command);
        assert!(first.get(key).is_some(), "missing key {key} for {:?}", command);
        assert_eq!(
            first.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
            second.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()),
            "top-level key drift for {:?}",
            command
        );
    }
}

#[test]
fn cockpit_text_heads_match_snapshots() {
    let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("cli_surface")
        .join("snapshots")
        .join("dev_cli_cockpit_text_heads.json");
    let expected: BTreeMap<String, String> =
        serde_json::from_str(&fs::read_to_string(snapshot_path).expect("read snapshot"))
            .expect("parse snapshot");
    for (command, prefix) in expected {
        let mut args: Vec<&str> = command.split_whitespace().collect();
        args.push("--format");
        args.push("text");
        let out = run(&args, &[]);
        assert!(out.status.success(), "text command failed for {command}");
        let text = String::from_utf8(out.stdout).expect("utf8");
        assert!(text.starts_with(&prefix), "snapshot drift for {command}");
    }
}

#[test]
fn cockpit_repeated_run_is_deterministic() {
    let commands = [
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "quickcheck"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ];
    for command in commands {
        let mut args = command.to_vec();
        args.push("--format");
        args.push("json");
        args.push("--no-pretty");
        let first = run(&args, &[]);
        let second = run(&args, &[]);
        assert!(first.status.success(), "first run failed for {:?}", args);
        assert!(second.status.success(), "second run failed for {:?}", args);
        assert_eq!(first.stdout, second.stdout, "stdout drift for {:?}", args);
    }
}

#[test]
fn cockpit_commands_work_with_corrupted_state() {
    let root = std::env::temp_dir().join(format!("bijux-cockpit-corrupt-{}", std::process::id()));
    fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    let history = root.join("history.json");
    let memory = root.join("memory.json");
    fs::write(&config, "BROKEN=\0\n").expect("write config");
    fs::write(&history, "{not-json").expect("write history");
    fs::write(&memory, "{not-json").expect("write memory");
    let envs = [
        ("BIJUX_CONFIG_PATH", config.to_string_lossy().to_string()),
        ("BIJUX_HISTORY_PATH", history.to_string_lossy().to_string()),
        ("BIJUX_MEMORY_PATH", memory.to_string_lossy().to_string()),
    ];
    for command in [
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "quickcheck"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(out.status.success(), "command failed under corrupted state: {:?}", command);
    }
}

#[test]
fn cockpit_commands_work_with_mixed_install_ambiguity() {
    let path = format!("/tmp/bijux-a:/tmp/bijux-b:{}", std::env::var("PATH").unwrap_or_default());
    let envs = [("PATH", path)];
    for command in [
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "quickcheck"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(
            out.status.success(),
            "command failed under mixed install ambiguity: {:?}",
            command
        );
    }
}

#[test]
fn cockpit_commands_work_with_plugin_failures_present() {
    let root = std::env::temp_dir().join(format!("bijux-cockpit-plugin-{}", std::process::id()));
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir");
    fs::write(plugins.join("broken.toml"), "not a plugin manifest").expect("write plugin");
    let envs = [("BIJUX_PLUGINS_DIR", plugins.to_string_lossy().to_string())];
    for command in [
        ["dev", "cli", "dashboard"],
        ["dev", "cli", "quickcheck"],
        ["dev", "cli", "truth"],
        ["dev", "cli", "blockers"],
        ["dev", "cli", "next"],
    ] {
        let out = run(&command, &envs);
        assert!(out.status.success(), "command failed with plugin failures present: {:?}", command);
    }
}
