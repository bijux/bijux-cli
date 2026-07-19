#![forbid(unsafe_code)]
//! Memory command behavior coverage.
//! test_type: memory-command-stability

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir()
        .join(format!("bijux-memory-coverage-{name}-{}-{nanos}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[test]
fn memory_root_and_list_missing_empty_valid_text_json_yaml() {
    let root = temp_dir("root-list");
    let home = root.join("home");
    fs::create_dir_all(home.join(".bijux")).expect("mkdir");

    let missing = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(missing.status.code(), Some(0));
    let missing_json: Value = serde_json::from_slice(&missing.stdout).expect("json");
    assert_eq!(missing_json["keys"], serde_json::json!([]));

    let mem_file = home.join(".bijux").join(".memory.json");
    fs::write(&mem_file, "{}").expect("empty memory");
    let empty = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(empty.status.code(), Some(0));
    let empty_json: Value = serde_json::from_slice(&empty.stdout).expect("json");
    assert_eq!(empty_json["count"], 0);

    fs::write(&mem_file, r#"{"beta":2,"alpha":1}"#).expect("populated memory");
    let populated = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(populated.status.code(), Some(0));
    let populated_json: Value = serde_json::from_slice(&populated.stdout).expect("json");
    assert_eq!(populated_json["count"], 2);

    let text = run_with_env(
        &["memory", "list", "--format", "text"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(text.status.code(), Some(0));
    assert!(String::from_utf8(text.stdout).expect("utf-8").contains("alpha"));

    let yaml = run_with_env(
        &["memory", "list", "--format", "yaml", "--pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(yaml.status.code(), Some(0));
    assert!(String::from_utf8(yaml.stdout).expect("utf-8").contains("keys:"));

    let root_json = run_with_env(
        &["memory", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(root_json.status.code(), Some(0));
    let _: Value = serde_json::from_slice(&root_json.stdout).expect("json");
}

#[test]
fn memory_malformed_wrong_type_missing_required_and_extra_fields() {
    let root = temp_dir("corruption");
    let home = root.join("home");
    let bijux_dir = home.join(".bijux");
    fs::create_dir_all(&bijux_dir).expect("mkdir");
    let mem_file = bijux_dir.join(".memory.json");

    fs::write(&mem_file, "{broken").expect("malformed");
    let malformed = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(malformed.status.code(), Some(1));
    assert!(malformed.stdout.is_empty());
    let malformed_error = String::from_utf8(malformed.stderr).expect("utf-8");
    assert!(malformed_error.contains("Malformed memory state"));

    fs::write(
        &mem_file,
        r#"{"alpha":1,"beta":{"value":2},"gamma":null,"delta":{"nested":true,"extra":"x"}}"#,
    )
    .expect("wrong type/extra fields");
    let mixed = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(mixed.status.code(), Some(0));
    let mixed_json: Value = serde_json::from_slice(&mixed.stdout).expect("json");
    let keys = mixed_json["keys"].as_array().expect("keys array");
    assert!(keys.iter().any(|k| k == "beta"));
    assert!(keys.iter().any(|k| k == "delta"));

    fs::write(&mem_file, "[]").expect("non-object json");
    let non_object = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(non_object.status.code(), Some(1));
    assert!(non_object.stdout.is_empty());
    assert!(!non_object.stderr.is_empty());
}

#[test]
fn memory_set_rejects_empty_keys() {
    let root = temp_dir("empty-key");
    let home = root.join("home");
    fs::create_dir_all(home.join(".bijux")).expect("mkdir");

    let out = run_with_env(
        &["memory", "set", "=abc", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    let memory_file = home.join(".bijux").join(".memory.json");
    if memory_file.exists() {
        let payload = fs::read_to_string(&memory_file).expect("read memory file");
        assert!(!payload.contains("\"\""), "empty key must never be persisted in memory state");
    }
}

#[test]
fn memory_quiet_no_color_and_deterministic_repeated_runs() {
    let root = temp_dir("rendering-determinism");
    let home = root.join("home");
    let bijux_dir = home.join(".bijux");
    fs::create_dir_all(&bijux_dir).expect("mkdir");
    let mem_file = bijux_dir.join(".memory.json");
    fs::write(&mem_file, r#"{"alpha":1,"beta":2}"#).expect("seed memory");

    let quiet =
        run_with_env(&["memory", "list", "--quiet"], &[("HOME", home.to_str().expect("utf-8"))]);
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let no_color = run_with_env(
        &["memory", "list", "--format", "text"],
        &[("HOME", home.to_str().expect("utf-8")), ("NO_COLOR", "1")],
    );
    assert_eq!(no_color.status.code(), Some(0));
    let no_color_text = String::from_utf8(no_color.stdout).expect("utf-8");
    assert!(!no_color_text.contains("\u{1b}["));

    let first = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    let second = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
#[cfg(unix)]
fn memory_unwritable_storage_conditions_for_read_and_write_paths() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("unwritable");
    let home = root.join("home");
    let bijux_dir = home.join(".bijux");
    fs::create_dir_all(&bijux_dir).expect("mkdir");
    let mem_file = bijux_dir.join(".memory.json");
    fs::write(&mem_file, r#"{"alpha":1}"#).expect("seed");

    let list_before = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(list_before.status.code(), Some(0));

    fs::set_permissions(&bijux_dir, fs::Permissions::from_mode(0o555)).expect("chmod readonly");

    let set = run_with_env(
        &["memory", "set", "beta=2", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(set.status.code(), Some(1));
    assert!(set.stdout.is_empty());
    assert!(!set.stderr.is_empty());

    let list_after = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.to_str().expect("utf-8"))],
    );
    assert_eq!(list_after.status.code(), Some(0));

    fs::set_permissions(&bijux_dir, fs::Permissions::from_mode(0o755)).expect("restore");
}

#[test]
fn memory_config_path_override_does_not_change_home_memory_resolution() {
    let root = temp_dir("path-override");
    let home = root.join("home");
    let bijux_dir = home.join(".bijux");
    fs::create_dir_all(&bijux_dir).expect("mkdir");
    let mem_file = bijux_dir.join(".memory.json");
    fs::write(&mem_file, r#"{"home_only":true}"#).expect("seed memory");

    let fake_config = root.join("fake.env");
    fs::write(&fake_config, "BIJUXCLI_ALPHA=config\n").expect("fake config");

    let args = [
        "memory",
        "list",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        fake_config.to_str().expect("utf-8"),
    ];
    let out = run_with_env(
        &args,
        &[
            ("HOME", home.to_str().expect("utf-8")),
            ("BIJUXCLI_CONFIG", fake_config.to_str().expect("utf-8")),
        ],
    );
    let out_repeat = run_with_env(
        &args,
        &[
            ("HOME", home.to_str().expect("utf-8")),
            ("BIJUXCLI_CONFIG", fake_config.to_str().expect("utf-8")),
        ],
    );
    assert_eq!(out_repeat.status.code(), Some(0));
    assert_eq!(out.stdout, out_repeat.stdout);
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("keys").and_then(Value::as_array).is_some());
    assert_eq!(out.status.code(), Some(0));
}
