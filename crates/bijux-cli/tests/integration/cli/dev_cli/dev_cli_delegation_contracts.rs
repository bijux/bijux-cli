#![forbid(unsafe_code)]
//! Process-boundary contracts for delegated `dev` commands.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn tmp_dir(name: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("bijux-dev-delegation-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn write_mock_delegate(path: &Path) {
    #[cfg(unix)]
    {
        fs::write(path, "#!/bin/sh\nprintf '%s\\n' \"$@\"\n").expect("write mock delegate");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod +x");
    }

    #[cfg(windows)]
    {
        fs::write(path, "@echo off\r\nfor %%A in (%*) do echo %%~A\r\n")
            .expect("write mock delegate");
    }
}

fn run(args: &[&str], envs: &[(&str, &Path)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("run")
}

fn stdout_lines(output: &std::process::Output) -> Vec<String> {
    String::from_utf8(output.stdout.clone())
        .expect("utf-8")
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[test]
fn canonical_dev_cli_route_forwards_only_dev_cli_tail_arguments() {
    let root = tmp_dir("canonical");
    let delegate = root.join(if cfg!(windows) { "mock-dev-cli.cmd" } else { "mock-dev-cli" });
    write_mock_delegate(&delegate);

    let output = run(
        &["dev", "cli", "routes", "--format", "json", "--no-pretty"],
        &[("BIJUX_DEV_CLI_BIN", &delegate)],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout_lines(&output),
        vec![
            "routes".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--no-pretty".to_string(),
        ]
    );
}

#[test]
fn legacy_dev_entry_route_forwards_namespace_as_first_argument() {
    let root = tmp_dir("legacy-entry");
    let delegate = root.join(if cfg!(windows) { "mock-dev-cli.cmd" } else { "mock-dev-cli" });
    write_mock_delegate(&delegate);

    let output = run(&["dev", "registry", "--format", "text"], &[("BIJUX_DEV_CLI_BIN", &delegate)]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout_lines(&output),
        vec!["registry".to_string(), "--format".to_string(), "text".to_string()]
    );
}

#[test]
fn missing_delegate_binary_returns_actionable_error() {
    let root = tmp_dir("missing");
    let missing = root.join("does-not-exist");

    let output = run(&["dev", "cli", "routes"], &[("BIJUX_DEV_CLI_BIN", &missing)]);

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert!(stderr.contains("failed to run `bijux dev cli`"));
    assert!(stderr.contains("attempted binaries:"));
    assert!(stderr.contains("install with `cargo install bijux-dev-cli`"));
}

#[test]
fn workspace_source_wins_over_stale_path_delegate_without_explicit_override() {
    let root = tmp_dir("path-shadow");
    let delegate = root.join(if cfg!(windows) { "bijux-dev-cli.cmd" } else { "bijux-dev-cli" });
    write_mock_delegate(&delegate);

    let original_path = env::var("PATH").unwrap_or_default();
    let separator = if cfg!(windows) { ";" } else { ":" };
    let combined_path = if original_path.is_empty() {
        root.to_string_lossy().to_string()
    } else {
        format!("{}{}{}", root.display(), separator, original_path)
    };

    let output = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(["dev", "cli", "contracts", "--format", "json", "--no-pretty"])
        .env("PATH", combined_path)
        .output()
        .expect("run");

    assert_eq!(output.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&output.stdout).expect("json payload");
    assert!(payload["contracts"].is_array());
    assert!(payload["runtime_version"].is_string());
}
