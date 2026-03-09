//! Config parity integration coverage for Rust core behavior.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core::app::run_app;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_routing as _;
use clap as _;
use futures as _;
use serde_json::Value;

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("valid json")
}

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-cli-core-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn config_set_and_get_match_normalization_rules() {
    let temp = make_temp_dir("normalize");
    let path = temp.join("custom").join(".env");

    let set = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "BIJUXCLI_My_Key=hello".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(set.exit_code, 0);

    let get = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "my_key".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(get.exit_code, 0);
    let payload = parse_json(&get.stdout);
    assert_eq!(payload["value"], "hello");
}

#[test]
fn config_get_missing_key_is_deterministic() {
    let temp = make_temp_dir("missing");
    let path = temp.join("missing").join(".env");

    let get = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "unknown".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(get.exit_code, 2);
    assert!(get.stdout.is_empty());
    let payload = parse_json(&get.stderr);
    assert_eq!(payload["code"], 2);
}

#[test]
fn config_set_creates_parent_directory_and_preserves_unrelated_keys() {
    let temp = make_temp_dir("preserve");
    let path = temp.join("nested").join("env").join("state.env");

    let first = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "alpha=1".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(first.exit_code, 0);

    let second = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "beta=2".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(second.exit_code, 0);

    let content = fs::read_to_string(path).expect("config file");
    assert!(content.contains("BIJUXCLI_ALPHA=1"));
    assert!(content.contains("BIJUXCLI_BETA=2"));
}

#[test]
fn config_set_repeated_write_is_idempotent() {
    let temp = make_temp_dir("idempotent");
    let path = temp.join("idempotent.env");

    for _ in 0..2 {
        let out = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "config".to_string(),
            "set".to_string(),
            "same=1".to_string(),
            "--config-path".to_string(),
            path.display().to_string(),
        ])
        .expect("run_app");
        assert_eq!(out.exit_code, 0);
    }

    let content = fs::read_to_string(path).expect("config file");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.iter().filter(|line| line.contains("BIJUXCLI_SAME=")).count(), 1);
}

#[test]
fn config_set_rejects_invalid_key_and_controls() {
    let temp = make_temp_dir("invalid");
    let path = temp.join("invalid.env");

    let invalid_key = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "bad-key=1".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(invalid_key.exit_code, 2);

    let unknown_section = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "group.key=1".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(unknown_section.exit_code, 2);

    let invalid_value = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        "ok=bad\tvalue".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(invalid_value.exit_code, 3);
}

#[test]
fn config_get_errors_on_malformed_file() {
    let temp = make_temp_dir("malformed");
    let path = temp.join("malformed.env");
    fs::write(&path, "BIJUXCLI_OK=1\nMALFORMED_LINE\n").expect("write malformed");

    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "ok".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 1);
    assert!(out.stdout.is_empty());
}

#[test]
fn config_output_supports_json_yaml_and_text() {
    let temp = make_temp_dir("formats");
    let path = temp.join("formats.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let json = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "alpha".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(json.exit_code, 0);
    assert!(json.stdout.trim_start().starts_with('{'));

    let yaml = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "alpha".to_string(),
        "--format".to_string(),
        "yaml".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(yaml.exit_code, 0);
    assert!(yaml.stdout.contains("value:"));

    let text = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "alpha".to_string(),
        "--format".to_string(),
        "text".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(text.exit_code, 0);
    assert!(text.stdout.contains("alpha"));
}

#[test]
fn config_set_reports_error_for_unwritable_parent() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp = make_temp_dir("readonly");
        let parent = temp.join("readonly");
        fs::create_dir_all(&parent).expect("mkdir");
        fs::set_permissions(&parent, fs::Permissions::from_mode(0o555)).expect("chmod");

        let target: PathBuf = parent.join("config.env");
        let out = run_app(&[
            "bijux".to_string(),
            "cli".to_string(),
            "config".to_string(),
            "set".to_string(),
            "alpha=1".to_string(),
            "--config-path".to_string(),
            target.display().to_string(),
        ])
        .expect("run_app");

        assert_eq!(out.exit_code, 1);
    }
}
