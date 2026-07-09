#![forbid(unsafe_code)]
//! Memory output stability checks for state handling and diagnostics consistency.
//! test_type: memory-output-stability

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

fn temp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir()
            .join(format!("bijux-memory-output-stability-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn memory_state_parsing_is_stable_under_field_reordering_and_unknown_fields() {
    let root = temp_dir("field-reorder");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");

    fs::write(&memory, r#"{"beta":{"v":2,"extra":"x"},"alpha":{"v":1}}"#).expect("write one");
    let envs = [("HOME", home.display().to_string())];
    let first = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    let first_json = parse_json(&first.stdout);

    fs::write(&memory, r#"{"alpha":{"v":1},"beta":{"extra":"x","v":2}}"#).expect("write two");
    let second = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(second.status.code(), Some(0));
    let second_json = parse_json(&second.stdout);

    assert_eq!(
        first_json["keys"], second_json["keys"],
        "field reorder should not change parsed keys"
    );
}

#[test]
fn missing_and_empty_memory_states_are_intentionally_consistent() {
    let root = temp_dir("missing-empty");
    let home = root.join("home");
    fs::create_dir_all(home.join(".bijux")).expect("mkdir");
    let envs = [("HOME", home.display().to_string())];

    let missing = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(missing.status.code(), Some(0));
    let missing_json = parse_json(&missing.stdout);

    let memory = home.join(".bijux").join(".memory.json");
    fs::write(&memory, "{}").expect("write empty");
    let empty = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(empty.status.code(), Some(0));
    let empty_json = parse_json(&empty.stdout);

    assert_eq!(
        missing_json, empty_json,
        "missing and empty memory states should be intentionally consistent"
    );
}

#[test]
fn memory_json_and_yaml_outputs_keep_stable_field_ordering_and_byte_stability() {
    let root = temp_dir("ordering");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    fs::write(&memory, r#"{"beta":{"v":2},"alpha":{"v":1}}"#).expect("write");
    let envs = [("HOME", home.display().to_string())];

    let json_a = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    let json_b = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(json_a.status.code(), Some(0));
    assert_eq!(json_b.status.code(), Some(0));
    assert_eq!(json_a.stdout, json_b.stdout, "json output should be byte stable");

    let yaml_a = run_with_env(&["memory", "list", "--format", "yaml", "--pretty"], &envs);
    let yaml_b = run_with_env(&["memory", "list", "--format", "yaml", "--pretty"], &envs);
    assert_eq!(yaml_a.status.code(), Some(0));
    assert_eq!(yaml_b.status.code(), Some(0));
    assert_eq!(yaml_a.stdout, yaml_b.stdout, "yaml output should be stable");
}

#[test]
fn memory_wrong_type_and_missing_required_shape_failures_are_stable() {
    let root = temp_dir("wrong-type");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    let envs = [("HOME", home.display().to_string())];

    fs::write(&memory, "[]").expect("write non-object");
    let wrong_type = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(wrong_type.status.code(), Some(1));
    let wrong_type_err = parse_json(&wrong_type.stderr);
    assert_eq!(wrong_type_err["status"], "error");

    fs::write(&memory, "{\"only\":\"string\"}").expect("write missing-required-shape");
    let missing_required =
        run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(missing_required.status.code(), Some(0));
    let payload = parse_json(&missing_required.stdout);
    assert!(payload["keys"].as_array().expect("keys").iter().any(|v| v == "only"));
}

#[test]
fn memory_path_override_and_quiet_mode_keep_functional_semantics() {
    let root = temp_dir("path-quiet");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    let fake_config = root.join("fake.env");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    fs::write(&memory, r#"{"alpha":{"v":1}}"#).expect("seed memory");
    fs::write(&fake_config, "BIJUXCLI_ALPHA=config\n").expect("seed config");

    let envs = [
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_CONFIG", fake_config.display().to_string()),
    ];
    let normal = run_with_env(
        &[
            "memory",
            "list",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            fake_config.to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert_eq!(normal.status.code(), Some(0));
    let normal_json = parse_json(&normal.stdout);

    let quiet = run_with_env(
        &["memory", "list", "--quiet", "--config-path", fake_config.to_str().expect("utf-8")],
        &envs,
    );
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let repeat = run_with_env(
        &[
            "memory",
            "list",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            fake_config.to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert_eq!(repeat.status.code(), Some(0));
    let repeat_json = parse_json(&repeat.stdout);
    assert_eq!(normal_json, repeat_json, "path override should keep memory semantics stable");
}
