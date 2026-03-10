#![forbid(unsafe_code)]
//! REPL history write interruption resilience.

use std::fs;
use std::path::PathBuf;

use bijux_cli as _;
use bijux_cli_python as _;
use bijux_cli::repl::{
    configure_history, execute_repl_line, flush_history, load_history, startup_repl,
};
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_history_path(name: &str) -> PathBuf {
    std::env::temp_dir()
        .join(format!("bijux-repl-history-resilience-{name}-{}.json", std::process::id()))
}

#[test]
fn repl_exit_flush_reports_write_interruption_without_crashing_session() {
    let bad_dir = std::env::temp_dir()
        .join(format!("bijux-repl-history-write-blocked-{}", std::process::id()));
    let _ = fs::remove_dir_all(&bad_dir);
    fs::create_dir_all(&bad_dir).expect("mkdir bad dir");

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(bad_dir.clone()), true, 64);
    execute_repl_line(&mut session, "status").expect("run status");
    let result = flush_history(&session);
    assert!(result.is_err());

    let _ = fs::remove_dir_all(bad_dir);
}

#[test]
fn repl_command_recording_survives_flush_failure_and_recovers_on_retry() {
    let bad_dir = std::env::temp_dir()
        .join(format!("bijux-repl-history-write-blocked-retry-{}", std::process::id()));
    let _ = fs::remove_dir_all(&bad_dir);
    fs::create_dir_all(&bad_dir).expect("mkdir bad dir");
    let good_path = temp_history_path("recording-retry-good");
    let _ = fs::remove_file(&good_path);

    let (mut session, _) = startup_repl("default", None);
    configure_history(&mut session, Some(bad_dir.clone()), true, 64);
    execute_repl_line(&mut session, "status").expect("status");
    execute_repl_line(&mut session, "doctor").expect("doctor");
    let first = flush_history(&session);
    assert!(first.is_err());

    configure_history(&mut session, Some(good_path.clone()), true, 64);
    flush_history(&session).expect("flush retry should succeed");

    let (mut reloaded, _) = startup_repl("default", None);
    configure_history(&mut reloaded, Some(good_path.clone()), true, 64);
    load_history(&mut reloaded).expect("load persisted history");
    assert!(reloaded.history.iter().any(|item| item == "status"));
    assert!(reloaded.history.iter().any(|item| item == "doctor"));

    let _ = fs::remove_file(good_path);
    let _ = fs::remove_dir_all(bad_dir);
}
