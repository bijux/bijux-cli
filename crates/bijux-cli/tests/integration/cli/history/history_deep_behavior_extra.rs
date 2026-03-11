#![forbid(unsafe_code)]
//! Additional deep history checks for diagnostics consistency and metadata-insensitive output stability.
//! test_type: history-deep-behavior

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bijux-history-extra-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

#[test]
fn history_doctor_and_state_doctor_agree_on_history_corruption_findings() {
    let root = temp_dir("doctor-consistency");
    let history = root.join("history.log");
    fs::write(&history, "{oops:true}\n").expect("write malformed history");
    let envs = [("BIJUXCLI_HISTORY_FILE", history.display().to_string())];

    let doctor = run_with_env(
        &["dev", "cli", "doctor", "--format", "json", "--no-pretty"],
        &envs,
    );
    let state_doctor = run_with_env(
        &[
            "dev",
            "cli",
            "state-doctor",
            "--format",
            "json",
            "--no-pretty",
        ],
        &envs,
    );
    assert_eq!(doctor.status.code(), Some(0));
    assert_eq!(state_doctor.status.code(), Some(0));

    let doctor_json = parse_json(&doctor.stdout);
    let state_json = parse_json(&state_doctor.stdout);

    let doctor_has_history_issue = doctor_json["issues"]["history"]
        .as_array()
        .map(|rows| !rows.is_empty())
        .unwrap_or(false);
    let state_has_history_issue = state_json["doctor"]["issues"]
        .as_array()
        .map(|rows| rows.iter().any(|row| row["area"] == "history"))
        .unwrap_or(false);
    assert_eq!(
        doctor_has_history_issue, state_has_history_issue,
        "doctor and state-doctor should agree on presence of history corruption findings"
    );
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
