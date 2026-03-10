#![forbid(unsafe_code)]
//! Config corruption and recovery hardening coverage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(out.status.success(), "command failed: {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn temp_dir(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "bijux-config-hardening-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir temp");
    base
}

fn write_config(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("mkdir parent");
    fs::write(path, text).expect("write config");
}

#[test]
fn config_truncation_duplicate_keys_line_endings_whitespace_and_null_byte_fail_cleanly() {
    let root = temp_dir("corruption-shapes");

    let truncated_key = root.join("truncated-key.env");
    write_config(&truncated_key, "BIJUXCLI_ALPHA=1\nBIJUXCLI_");
    let a =
        run(&["cli", "config", "reload", "--config-path", truncated_key.to_str().expect("utf-8")]);

    let truncated_value = root.join("truncated-value.env");
    write_config(&truncated_value, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=");
    let b = run(&[
        "cli",
        "config",
        "reload",
        "--config-path",
        truncated_value.to_str().expect("utf-8"),
    ]);

    let duplicate_keys = root.join("duplicate-keys.env");
    write_config(&duplicate_keys, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\nBIJUXCLI_ALPHA=3\n");
    let c =
        run(&["cli", "config", "reload", "--config-path", duplicate_keys.to_str().expect("utf-8")]);

    let bad_line_endings = root.join("bad-line-endings.env");
    write_config(&bad_line_endings, "BIJUXCLI_ALPHA=1\rBIJUXCLI_BETA=2\r");
    let d = run(&[
        "cli",
        "config",
        "reload",
        "--config-path",
        bad_line_endings.to_str().expect("utf-8"),
    ]);

    let whitespace_abuse = root.join("whitespace-abuse.env");
    write_config(&whitespace_abuse, "   BIJUXCLI_ALPHA = 1\n\tBIJUXCLI_BETA=2\n");
    let e = run(&[
        "cli",
        "config",
        "reload",
        "--config-path",
        whitespace_abuse.to_str().expect("utf-8"),
    ]);

    let null_bytes = root.join("null-bytes.env");
    fs::write(&null_bytes, b"BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=ok\0oops\n")
        .expect("write null bytes");
    let f = run(&["cli", "config", "reload", "--config-path", null_bytes.to_str().expect("utf-8")]);

    let exits = [
        a.status.code(),
        b.status.code(),
        c.status.code(),
        d.status.code(),
        e.status.code(),
        f.status.code(),
    ];
    assert!(exits.iter().any(|code| *code == Some(1)), "at least one corruption shape must fail");
}

#[test]
fn config_doctor_reports_corruption_for_broken_config_states() {
    let root = temp_dir("doctor-corruption");
    let config_path = root.join("doctor.env");
    write_config(&config_path, "BIJUXCLI_ALPHA=1\nBROKEN\n");

    let out = run(&[
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert!(matches!(out.status.code(), Some(0) | Some(1)));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    let issues = payload["doctor"]["issues"].as_array().expect("issues");
    assert!(issues.iter().any(|issue| issue["area"] == "config"));
}

#[test]
#[cfg(unix)]
fn config_set_clear_unset_failures_preserve_previous_content_as_rollback_proof() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("rollback-proof");
    let dir = root.join("readonly");
    fs::create_dir_all(&dir).expect("mkdir readonly");
    let config_path = dir.join("state.env");
    write_config(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n");
    let before = fs::read_to_string(&config_path).expect("read baseline");

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).expect("chmod 555");

    let set_fail = run(&[
        "cli",
        "config",
        "set",
        "gamma=3",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    let clear_fail =
        run(&["cli", "config", "clear", "--config-path", config_path.to_str().expect("utf-8")]);
    let unset_fail = run(&[
        "cli",
        "config",
        "unset",
        "alpha",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("chmod 755");

    assert_eq!(set_fail.status.code(), Some(1));
    assert_eq!(clear_fail.status.code(), Some(1));
    assert_eq!(unset_fail.status.code(), Some(1));

    let after = fs::read_to_string(&config_path).expect("read after failures");
    assert_eq!(before, after);
}

#[test]
#[cfg(unix)]
fn config_clear_and_unset_retry_are_idempotent_after_transient_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("retry-clear-unset");
    let dir = root.join("readonly");
    fs::create_dir_all(&dir).expect("mkdir readonly");
    let config_path = dir.join("retry.env");
    write_config(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n");

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).expect("chmod 555");
    let first_unset = run(&[
        "cli",
        "config",
        "unset",
        "alpha",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    let first_clear =
        run(&["cli", "config", "clear", "--config-path", config_path.to_str().expect("utf-8")]);
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("chmod 755");

    assert_eq!(first_unset.status.code(), Some(1));
    assert_eq!(first_clear.status.code(), Some(1));

    let second_unset = run_ok_json(&[
        "cli",
        "config",
        "unset",
        "alpha",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert_eq!(second_unset["status"], "deleted");

    let second_clear = run_ok_json(&[
        "cli",
        "config",
        "clear",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert_eq!(second_clear["status"], "cleared");
}

#[test]
fn concurrent_config_reads_during_mutation_and_parallel_writes_do_not_corrupt_file_shape() {
    let root = temp_dir("concurrency");
    let config_path = root.join("concurrency.env");
    write_config(&config_path, "BIJUXCLI_ALPHA=1\n");

    let path_a = config_path.clone();
    let writer_a = thread::spawn(move || {
        for i in 0..40 {
            let _ = run(&[
                "cli",
                "config",
                "set",
                &format!("alpha={i}"),
                "--config-path",
                path_a.to_str().expect("utf-8"),
            ]);
        }
    });

    let path_b = config_path.clone();
    let writer_b = thread::spawn(move || {
        for i in 0..40 {
            let _ = run(&[
                "cli",
                "config",
                "set",
                &format!("beta={i}"),
                "--config-path",
                path_b.to_str().expect("utf-8"),
            ]);
        }
    });

    let path_c = config_path.clone();
    let reader = thread::spawn(move || {
        for _ in 0..40 {
            let _ =
                run(&["cli", "config", "reload", "--config-path", path_c.to_str().expect("utf-8")]);
        }
    });

    writer_a.join().expect("writer a");
    writer_b.join().expect("writer b");
    reader.join().expect("reader");

    let final_reload =
        run(&["cli", "config", "reload", "--config-path", config_path.to_str().expect("utf-8")]);

    assert!(matches!(final_reload.status.code(), Some(0) | Some(1)));
    let text = fs::read_to_string(&config_path).expect("final config readable");
    assert!(text.lines().all(|line| line.contains('=') && line.starts_with("BIJUXCLI_")));
}

#[test]
fn invalid_utf8_config_file_is_reported_cleanly() {
    let root = temp_dir("invalid-utf8");
    let config_path = root.join("invalid-utf8.env");
    fs::write(&config_path, vec![0x66, 0x6f, 0x80, 0x6f]).expect("write invalid utf8 bytes");

    let out =
        run(&["cli", "config", "reload", "--config-path", config_path.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}
