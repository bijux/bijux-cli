#![forbid(unsafe_code)]
//! History command behavior coverage.
//! test_type: history-command-stability

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir()
            .join(format!("bijux-history-coverage-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn entry(cmd: &str, ts: f64) -> serde_json::Value {
    serde_json::json!({
        "command": cmd,
        "params": [],
        "success": true,
        "return_code": 0,
        "duration_ms": 1.0,
        "timestamp": ts,
        "raw": {}
    })
}

#[test]
fn history_root_listing_no_file_one_record_many_records_and_ordering() {
    let root = temp_dir("root-list");
    let missing = root.join("missing.history");
    let one = root.join("one.history");
    let many = root.join("many.history");

    let out_missing = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", missing.to_str().expect("utf-8"))],
    );
    assert_eq!(out_missing.status.code(), Some(0));
    let missing_json: Value = serde_json::from_slice(&out_missing.stdout).expect("json");
    assert_eq!(missing_json["entries"], serde_json::json!([]));

    fs::write(&one, serde_json::to_string(&vec![entry("status", 1.0)]).expect("json"))
        .expect("write one");
    let out_one = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", one.to_str().expect("utf-8"))],
    );
    assert_eq!(out_one.status.code(), Some(0));
    let one_json: Value = serde_json::from_slice(&out_one.stdout).expect("json");
    assert_eq!(one_json["entries"].as_array().expect("array").len(), 1);

    let many_entries =
        vec![entry("version", 1.0), entry("status", 2.0), entry("plugins list", 3.0)];
    fs::write(&many, serde_json::to_string(&many_entries).expect("json")).expect("write many");
    let out_many = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", many.to_str().expect("utf-8"))],
    );
    assert_eq!(out_many.status.code(), Some(0));
    let many_json: Value = serde_json::from_slice(&out_many.stdout).expect("json");
    let rows = many_json["entries"].as_array().expect("array");
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["command"], "version");
    assert_eq!(rows[1]["command"], "status");
    assert_eq!(rows[2]["command"], "plugins list");
}

#[test]
fn history_text_json_yaml_quiet_and_no_color_modes() {
    let root = temp_dir("formats");
    let path = root.join("history.json");
    fs::write(&path, serde_json::to_string(&vec![entry("status", 1.0)]).expect("json"))
        .expect("write");
    let h = path.to_str().expect("utf-8");

    let text = run_with_env(&["history", "--format", "text"], &[("BIJUXCLI_HISTORY_FILE", h)]);
    assert_eq!(text.status.code(), Some(0));
    assert!(String::from_utf8(text.stdout).expect("utf-8").contains("status"));

    let json = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", h)],
    );
    assert_eq!(json.status.code(), Some(0));
    let _: Value = serde_json::from_slice(&json.stdout).expect("json");

    let yaml =
        run_with_env(&["history", "--format", "yaml", "--pretty"], &[("BIJUXCLI_HISTORY_FILE", h)]);
    assert_eq!(yaml.status.code(), Some(0));
    assert!(String::from_utf8(yaml.stdout).expect("utf-8").contains("entries:"));

    let quiet = run_with_env(&["history", "--quiet"], &[("BIJUXCLI_HISTORY_FILE", h)]);
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let no_color = run_with_env(
        &["history", "--format", "text"],
        &[("BIJUXCLI_HISTORY_FILE", h), ("NO_COLOR", "1")],
    );
    assert_eq!(no_color.status.code(), Some(0));
    let body = String::from_utf8(no_color.stdout).expect("utf-8");
    assert!(!body.contains("\u{1b}["));
}

#[test]
fn history_malformed_and_mixed_valid_invalid_tolerance_and_duplicates() {
    let root = temp_dir("corruption");
    let malformed = root.join("malformed.history");
    fs::write(&malformed, "{oops:true}").expect("write malformed");

    let malformed_out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", malformed.to_str().expect("utf-8"))],
    );
    assert_eq!(malformed_out.status.code(), Some(1));
    assert!(malformed_out.stdout.is_empty());
    assert!(!malformed_out.stderr.is_empty());

    let mixed = root.join("mixed.history");
    fs::write(&mixed, "status\nBROKEN-LINE\nstatus\nplugins list\n").expect("write mixed");
    let mixed_out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", mixed.to_str().expect("utf-8"))],
    );
    assert_eq!(mixed_out.status.code(), Some(0));
    let mixed_json: Value = serde_json::from_slice(&mixed_out.stdout).expect("json");
    let rows = mixed_json["entries"].as_array().expect("array");
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0]["command"], "status");
    assert_eq!(rows[1]["command"], "BROKEN-LINE");
    assert_eq!(rows[2]["command"], "status");
}

#[test]
fn history_filter_and_sort_apply_before_limit() {
    let root = temp_dir("filter-sort-before-limit");
    let path = root.join("history.json");
    fs::write(
        &path,
        serde_json::to_string(&vec![
            entry("match-01", 1.0),
            entry("noise-01", 100.0),
            entry("noise-02", 101.0),
            entry("noise-03", 102.0),
            entry("match-02", 2.0),
            entry("match-03", 3.0),
        ])
        .expect("json"),
    )
    .expect("write");

    let out = run_with_env(
        &[
            "history",
            "--format",
            "json",
            "--no-pretty",
            "--filter",
            "match",
            "--sort",
            "timestamp",
            "--limit",
            "2",
        ],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    let rows = payload["entries"].as_array().expect("array");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["command"], "match-02");
    assert_eq!(rows[1]["command"], "match-03");
}

#[test]
fn history_limit_path_override_and_repeated_run_determinism() {
    let root = temp_dir("limit-determinism");
    let path = root.join("history.json");
    fs::write(
        &path,
        serde_json::to_string(&vec![entry("one", 1.0), entry("two", 2.0), entry("three", 3.0)])
            .expect("json"),
    )
    .expect("write");

    let first = run_with_env(
        &["history", "--format", "json", "--no-pretty", "--limit", "2"],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    let second = run_with_env(
        &["history", "--format", "json", "--no-pretty", "--limit", "2"],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);

    let payload: Value = serde_json::from_slice(&first.stdout).expect("json");
    let rows = payload["entries"].as_array().expect("array");
    assert_eq!(rows.len(), 2);
}

#[test]
fn history_equals_form_options_apply_filter_sort_and_limit() {
    let root = temp_dir("equals-options");
    let path = root.join("history.json");
    fs::write(
        &path,
        serde_json::to_string(&vec![
            entry("match_new", 30.0),
            entry("other", 10.0),
            entry("match_old", 20.0),
        ])
        .expect("json"),
    )
    .expect("write");

    let out = run_with_env(
        &[
            "history",
            "--format",
            "json",
            "--no-pretty",
            "--filter=match",
            "--sort=timestamp",
            "--limit=3",
        ],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    let rows = payload["entries"].as_array().expect("array");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["command"], "match_old");
    assert_eq!(rows[1]["command"], "match_new");
}

#[test]
fn history_invalid_limit_and_sort_values_fail_with_usage_exit() {
    let invalid_limit = run(&["history", "--limit", "not-a-number"]);
    assert_eq!(invalid_limit.status.code(), Some(2));
    assert!(invalid_limit.stdout.is_empty());
    assert!(!invalid_limit.stderr.is_empty());

    let zero_limit = run(&["history", "--limit", "0"]);
    assert_eq!(zero_limit.status.code(), Some(2));
    assert!(zero_limit.stdout.is_empty());
    assert!(!zero_limit.stderr.is_empty());

    let oversized_limit = run(&["history", "--limit", "10001"]);
    assert_eq!(oversized_limit.status.code(), Some(2));
    assert!(oversized_limit.stdout.is_empty());
    assert!(!oversized_limit.stderr.is_empty());

    let empty_filter = run(&["history", "--filter", "   "]);
    assert_eq!(empty_filter.status.code(), Some(2));
    assert!(empty_filter.stdout.is_empty());
    assert!(!empty_filter.stderr.is_empty());

    let invalid_sort = run(&["history", "--sort", "lexical"]);
    assert_eq!(invalid_sort.status.code(), Some(2));
    assert!(invalid_sort.stdout.is_empty());
    assert!(!invalid_sort.stderr.is_empty());
}

#[test]
#[cfg(unix)]
fn history_clear_with_unwritable_parent_fails_stably() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("unwritable");
    let dir = root.join("readonly");
    fs::create_dir_all(&dir).expect("mkdir readonly");
    let path = dir.join("history.json");
    fs::write(&path, serde_json::to_string(&vec![entry("status", 1.0)]).expect("json"))
        .expect("seed history");

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).expect("chmod");

    let out = run_with_env(
        &["history", "clear", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("restore");
}

#[test]
fn history_clear_rejects_malformed_history_payloads() {
    let root = temp_dir("clear-corruption");
    let path = root.join("history.json");
    fs::write(&path, "{\"oops\":true}").expect("seed malformed history");

    let out = run_with_env(
        &["history", "clear", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn history_clear_force_recovers_malformed_history_payloads() {
    let root = temp_dir("clear-corruption-force");
    let path = root.join("history.json");
    fs::write(&path, "{\"oops\":true}").expect("seed malformed history");

    let out = run_with_env(
        &["history", "clear", "--force", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", path.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["status"], "cleared");
    assert_eq!(payload["force_applied"], true);
    assert_eq!(payload["corruption_ignored"], true);
}

#[test]
fn history_help_and_exit_discipline_for_root_and_clear() {
    let root_help = run(&["history", "--help"]);
    assert_eq!(root_help.status.code(), Some(0));
    assert!(String::from_utf8(root_help.stdout).expect("utf-8").contains("Usage: bijux history"));
    assert!(root_help.stderr.is_empty());

    let clear_help = run(&["history", "clear", "--help"]);
    assert_eq!(clear_help.status.code(), Some(0));
    let clear_help_text = String::from_utf8(clear_help.stdout).expect("utf-8");
    assert!(clear_help_text.contains("Usage: bijux history clear"));
    assert!(clear_help_text.contains("--force"));
    assert!(clear_help.stderr.is_empty());

    let malformed = run(&["history", "--unknown-flag"]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());
}
