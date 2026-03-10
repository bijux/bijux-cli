//! Config parity integration coverage for Rust core behavior.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow as _;
use bijux_cli_routing as _;
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
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
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

#[test]
#[cfg(unix)]
fn config_get_reports_error_for_unreadable_file() {
    use std::os::unix::fs::PermissionsExt;

    let temp = make_temp_dir("unreadable");
    let path = temp.join("unreadable.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\n").expect("write");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).expect("chmod");

    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        "alpha".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 1);
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("restore");
}

#[test]
fn config_unset_removes_existing_key_and_is_safe_for_missing_key() {
    let temp = make_temp_dir("unset");
    let path = temp.join("unset.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed");

    let removed = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "unset".to_string(),
        "alpha".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(removed.exit_code, 0);
    let removed_payload = parse_json(&removed.stdout);
    assert_eq!(removed_payload["removed"], true);

    let missing = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "unset".to_string(),
        "missing".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(missing.exit_code, 0);
    let missing_payload = parse_json(&missing.stdout);
    assert_eq!(missing_payload["removed"], false);
}

#[test]
fn config_unset_rejects_malformed_key() {
    let temp = make_temp_dir("unset-invalid");
    let path = temp.join("unset.env");

    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "unset".to_string(),
        "bad-key".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(out.exit_code, 2);
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn config_clear_handles_non_empty_empty_and_missing_files() {
    let temp = make_temp_dir("clear");
    let non_empty = temp.join("non-empty.env");
    fs::write(&non_empty, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed");

    let cleared_non_empty = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "clear".to_string(),
        "--config-path".to_string(),
        non_empty.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(cleared_non_empty.exit_code, 0);
    let payload = parse_json(&cleared_non_empty.stdout);
    assert_eq!(payload["removed_keys"], 2);
    assert_eq!(payload["removed_file"], true);
    assert!(!non_empty.exists());

    let missing = temp.join("missing.env");
    let cleared_missing = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "clear".to_string(),
        "--config-path".to_string(),
        missing.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(cleared_missing.exit_code, 0);
    let missing_payload = parse_json(&cleared_missing.stdout);
    assert_eq!(missing_payload["removed_keys"], 0);
    assert_eq!(missing_payload["removed_file"], false);
}

#[test]
#[cfg(unix)]
fn config_clear_reports_write_failure_for_read_only_dir() {
    use std::os::unix::fs::PermissionsExt;

    let temp = make_temp_dir("clear-ro");
    let dir = temp.join("readonly");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("clear.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\n").expect("seed");
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).expect("chmod");

    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "clear".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 1);
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("restore");
}

#[test]
fn config_reload_success_missing_and_malformed_behavior() {
    let temp = make_temp_dir("reload");
    let good = temp.join("good.env");
    let missing = temp.join("missing.env");
    let malformed = temp.join("malformed.env");

    fs::write(&good, "BIJUXCLI_ALPHA=1\n").expect("seed");
    fs::write(&malformed, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("seed malformed");

    let ok = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "reload".to_string(),
        "--config-path".to_string(),
        good.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(ok.exit_code, 0);
    let ok_payload = parse_json(&ok.stdout);
    assert_eq!(ok_payload["status"], "reloaded");
    assert_eq!(ok_payload["entry_count"], 1);

    let missing_out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "reload".to_string(),
        "--config-path".to_string(),
        missing.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(missing_out.exit_code, 0);
    let missing_payload = parse_json(&missing_out.stdout);
    assert_eq!(missing_payload["entry_count"], 0);

    let malformed_out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "reload".to_string(),
        "--config-path".to_string(),
        malformed.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(malformed_out.exit_code, 1);
    assert!(malformed_out.stdout.is_empty());
    assert!(!malformed_out.stderr.is_empty());
}
