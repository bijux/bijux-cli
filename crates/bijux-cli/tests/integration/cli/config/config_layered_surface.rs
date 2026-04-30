#![forbid(unsafe_code)]
//! Layered config operator surface tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!("bijux-config-layered-{name}-{}-{counter}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn run_in(cwd: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.current_dir(cwd).args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn assert_success_json(output: &Output) -> Value {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("json output")
}

#[test]
fn config_schema_and_validate_cover_project_profile_and_env_override() {
    let root = temp_dir("schema-validate");
    let global = root.join("global.env");
    let project = root.join("project");
    fs::create_dir_all(project.join(".bijux/profiles")).expect("mkdir");
    fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\n").expect("global");
    fs::write(project.join(".bijux/config.toml"), "[dag]\njobs = 3\n").expect("project config");
    fs::write(project.join(".bijux/profiles/dev.toml"), "[dag]\ncache_mode = 'strict'\n")
        .expect("project profile");

    let schema = assert_success_json(&run_in(
        &project,
        &["config", "schema", "dag", "--format", "json", "--no-pretty"],
        &[],
    ));
    assert_eq!(schema["scope"], "dag");

    let validate = assert_success_json(&run_in(
        &project,
        &[
            "config",
            "validate",
            "--profile",
            "dev",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[("BIJUX_DAG_JOBS", "8")],
    ));
    assert_eq!(validate["valid"], true);
    assert_eq!(validate["effective"]["dag.jobs"]["value"], "8");
    assert_eq!(validate["effective"]["dag.cache_mode"]["value"], "strict");
    assert!(validate["project_discovery"]["config_path"]
        .as_str()
        .is_some_and(|value| value.ends_with(".bijux/config.toml")));
}

#[test]
fn config_docs_emits_generated_markdown_reference() {
    let root = temp_dir("docs");
    let docs = assert_success_json(&run_in(
        &root,
        &["config", "docs", "cli", "--format", "json", "--no-pretty"],
        &[],
    ));
    let markdown = docs["markdown"].as_str().expect("markdown");
    assert!(markdown.contains("# Generated Config Reference"));
    assert!(markdown.contains("## `cli`"));
    assert!(markdown.contains("`cli.log_level`"));
}

#[test]
fn config_explain_redacts_sensitive_values_and_reports_candidates() {
    let root = temp_dir("explain");
    let global = root.join("global.env");
    fs::write(&global, "BIJUXCLI_CLI_ACCESS_TOKEN=secret-token\n").expect("global");

    let explain = assert_success_json(&run_in(
        &root,
        &[
            "config",
            "explain",
            "cli.access_token",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[],
    ));
    assert_eq!(explain["effective"]["value"], "[redacted]");
    assert!(explain["environment"]["candidates"]
        .as_array()
        .is_some_and(|items| items.iter().any(|entry| entry == "BIJUXCLI_ACCESS_TOKEN")));
}

#[test]
fn config_repair_writes_backup_and_sanitizes_file() {
    let root = temp_dir("repair");
    let global = root.join("global.env");
    fs::write(&global, "BIJUXCLI_ALPHA=1\nBROKEN\nBIJUXCLI_BÄD=2\nBIJUXCLI_ALPHA=2\n")
        .expect("global");

    let repair = assert_success_json(&run_in(
        &root,
        &[
            "config",
            "repair",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[],
    ));
    assert_eq!(repair["changed"], true);
    assert_eq!(repair["dropped_line_count"], 2);
    assert!(repair["issues"]
        .as_array()
        .is_some_and(|items| items.iter().any(|entry| entry["issue"] == "malformed-line")));
    assert!(repair["issues"]
        .as_array()
        .is_some_and(|items| items.iter().any(|entry| entry["issue"] == "invalid-key")));
    assert!(repair["remediation"].as_array().is_some_and(|items| items
        .iter()
        .any(|entry| entry == "Use KEY=VALUE format for each non-comment config line.")));
    let repaired_text = fs::read_to_string(&global).expect("repaired file");
    assert_eq!(repaired_text, "BIJUXCLI_ALPHA=2\n");
    assert!(global.with_extension("bak").exists());
}

#[test]
fn config_export_and_load_portable_round_trip() {
    let root = temp_dir("portable");
    let global = root.join("global.env");
    let imported = root.join("imported.env");
    let bundle = root.join("bundle.json");
    let project = root.join("project");
    fs::create_dir_all(project.join(".bijux")).expect("mkdir");
    fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\n").expect("global");
    fs::write(project.join(".bijux/config.toml"), "[dag]\njobs = 4\n").expect("project");

    let export = assert_success_json(&run_in(
        &project,
        &[
            "config",
            "export",
            bundle.to_str().expect("utf-8"),
            "--portable",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[],
    ));
    assert_eq!(export["file_format"], "portable_json");

    let load = assert_success_json(&run_in(
        &project,
        &[
            "config",
            "load",
            bundle.to_str().expect("utf-8"),
            "--portable",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            imported.to_str().expect("utf-8"),
        ],
        &[],
    ));
    assert_eq!(load["file_format"], "portable_json");

    let imported_text = fs::read_to_string(imported).expect("imported");
    assert!(imported_text.contains("BIJUXCLI_CLI_LOG_LEVEL=info"));
    assert!(imported_text.contains("BIJUXCLI_DAG_JOBS=4"));
}

#[test]
fn config_validate_reports_invalid_typed_value() {
    let root = temp_dir("validate-error");
    let global = root.join("global.env");
    fs::write(&global, "BIJUXCLI_DAG_JOBS=abc\n").expect("global");

    let output = run_in(
        &root,
        &[
            "cli",
            "config",
            "validate",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[],
    );
    assert_eq!(output.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(payload["valid"], false);
    assert!(payload["errors"].as_array().is_some_and(|items| items.iter().any(|entry| entry
        .as_str()
        .is_some_and(|text| text.contains("expects an integer value")))));
}

#[test]
fn config_validate_override_takes_highest_precedence() {
    let root = temp_dir("validate-override");
    let global = root.join("global.env");
    let project = root.join("project");
    fs::create_dir_all(project.join(".bijux")).expect("mkdir");
    fs::write(&global, "BIJUXCLI_CLI_LOG_LEVEL=info\n").expect("global");
    fs::write(project.join(".bijux/config.toml"), "[cli]\nlog_level = 'warn'\n").expect("project");

    let payload = assert_success_json(&run_in(
        &project,
        &[
            "config",
            "validate",
            "--override",
            "cli.log_level=debug",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            global.to_str().expect("utf-8"),
        ],
        &[("BIJUXCLI_CLI_LOG_LEVEL", "error")],
    ));
    assert_eq!(payload["effective"]["cli.log_level"]["value"], "debug");
    assert_eq!(
        payload["precedence"],
        serde_json::json!([
            "defaults",
            "global_file",
            "global_profile",
            "project_file",
            "project_profile",
            "environment",
            "cli_overrides"
        ])
    );
}
