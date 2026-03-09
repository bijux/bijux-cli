#![forbid(unsafe_code)]
//! Golden snapshots for newly ported command outputs.

use std::process::Command;

use bijux_cli_core as _;
use libc as _;
use serde_json::Value;

fn run_json(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "command failed: {args:?}");
    serde_json::from_slice(&output.stdout).expect("stdout should be json")
}

fn normalize_paths(value: Value) -> Value {
    let home = std::env::var("HOME").unwrap_or_default();
    normalize_node(value, &home)
}

fn normalize_node(value: Value, home: &str) -> Value {
    match value {
        Value::String(text) => Value::String(text.replace(home, "<HOME>")),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(|item| normalize_node(item, home)).collect())
        }
        Value::Object(map) => Value::Object(
            map.into_iter().map(|(key, val)| (key, normalize_node(val, home))).collect(),
        ),
        other => other,
    }
}

fn assert_snapshot(args: &[&str], snapshot_path: &str) {
    let actual = normalize_paths(run_json(args));
    let expected: Value = serde_json::from_str(
        &std::fs::read_to_string(snapshot_path).expect("snapshot file should exist"),
    )
    .expect("snapshot should parse as json");
    assert_eq!(actual, expected, "golden mismatch for args: {args:?}");
}

#[test]
fn ported_command_outputs_match_goldens() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let golden_config_path = format!("{home}/.bijux/golden-config.env");
    let _ = std::fs::remove_file(&golden_config_path);

    let set_args = [
        "cli",
        "config",
        "set",
        "golden_key=golden-value",
        "--config-path",
        golden_config_path.as_str(),
    ];
    assert_snapshot(&set_args, "tests/snapshots/ported/cli_config_set.json");

    let get_args =
        ["cli", "config", "get", "golden_key", "--config-path", golden_config_path.as_str()];
    assert_snapshot(&get_args, "tests/snapshots/ported/cli_config_get.json");

    let cases: [(&[&str], &str); 12] = [
        (&["status"], "tests/snapshots/ported/root_status.json"),
        (&["audit"], "tests/snapshots/ported/root_audit.json"),
        (&["docs"], "tests/snapshots/ported/root_docs.json"),
        (&["sleep", "0"], "tests/snapshots/ported/root_sleep.json"),
        (&["cli", "self-test"], "tests/snapshots/ported/cli_self_test.json"),
        (&["cli", "plugins", "list"], "tests/snapshots/ported/cli_plugins_list.json"),
        (&["cli", "plugins", "inspect"], "tests/snapshots/ported/cli_plugins_inspect.json"),
        (&["dev", "cli", "routes"], "tests/snapshots/ported/dev_cli_routes.json"),
        (&["dev", "cli", "registry"], "tests/snapshots/ported/dev_cli_registry.json"),
        (&["dev", "cli", "env"], "tests/snapshots/ported/dev_cli_env.json"),
        (&["dev", "cli", "doctor"], "tests/snapshots/ported/dev_cli_doctor.json"),
        (&["dev", "cli", "contracts"], "tests/snapshots/ported/dev_cli_contracts.json"),
    ];

    for (args, snapshot) in cases {
        assert_snapshot(args, snapshot);
    }
}
