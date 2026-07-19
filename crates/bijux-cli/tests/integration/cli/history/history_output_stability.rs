#![forbid(unsafe_code)]
//! History output stability checks for diagnostics consistency and metadata-insensitive rendering.
//! test_type: history-output-stability

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
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

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir()
        .join(format!("bijux-history-output-stability-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[cfg(unix)]
#[test]
fn history_output_is_stable_under_filesystem_metadata_changes() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("metadata-stability");
    let history = root.join("history.json");
    fs::write(
        &history,
        serde_json::to_string(&vec![
            serde_json::json!({"command":"status","timestamp":1.0}),
            serde_json::json!({"command":"doctor","timestamp":2.0}),
        ])
        .expect("json"),
    )
    .expect("write history");

    let envs = [("BIJUXCLI_HISTORY_FILE", history.display().to_string())];
    let first = run_with_env(&["history", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));

    let mut perms = fs::metadata(&history).expect("metadata").permissions();
    perms.set_mode(0o640);
    fs::set_permissions(&history, perms).expect("chmod");

    let second = run_with_env(&["history", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(
        first.stdout, second.stdout,
        "history output should not drift on metadata-only change"
    );
}
