#![forbid(unsafe_code)]
//! Integration coverage for implemented built-in and developer commands.

use std::process::Command;
use std::{env, fs};

use bijux_cli as _;
use bijux_cli_python as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "process failed for args: {args:?}");
    assert!(
        output.stderr.is_empty(),
        "successful command must not write to stderr: {args:?}"
    );
    assert!(
        !output.stdout.is_empty(),
        "successful command must produce stdout payload: {args:?}"
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

fn run_raw(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_with_env(args: &[&str], key: &str, value: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env(key, value)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "process failed for args: {args:?}");
    assert!(
        output.stderr.is_empty(),
        "successful command must not write to stderr: {args:?}"
    );
    assert!(
        !output.stdout.is_empty(),
        "successful command must produce stdout payload: {args:?}"
    );
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn executes_root_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["status"], "status"),
        (vec!["version"], "version"),
        (vec!["doctor"], "status"),
        (vec!["audit"], "checks"),
        (vec!["docs"], "topics"),
        (vec!["atlas"], "mount"),
        (vec!["sleep", "0"], "slept_seconds"),
        (vec!["history"], "entries"),
        (vec!["memory"], "count"),
        (vec!["memory", "list"], "keys"),
        (vec!["plugins", "list"], "plugins"),
        (vec!["plugins"], "plugins"),
        (vec!["plugins", "info"], "plugins"),
        (vec!["plugins", "inspect"], "status"),
        (vec!["plugins", "doctor"], "doctor"),
        (vec!["plugins", "reserved-names"], "reserved_namespaces"),
        (vec!["plugins", "where"], "plugins_dir"),
        (vec!["plugins", "explain"], "diagnostics"),
        (vec!["plugins", "schema"], "schema"),
        (vec!["repl"], "mode"),
        (vec!["completion"], "shells"),
        (vec!["inspect"], "route_sources"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("root command must emit valid json");
        if args == vec!["config"] {
            assert!(
                payload.is_object(),
                "config root should return object payload"
            );
            continue;
        }
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn executes_cli_namespace_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["cli", "status"], "runtime"),
        (vec!["cli", "paths"], "path_binaries"),
        (vec!["cli", "config", "set", "TEST_KEY=1"], "status"),
        (vec!["cli", "self-test"], "checks"),
        (vec!["cli", "plugins", "list"], "plugins"),
        (vec!["cli", "plugins", "info"], "plugins"),
        (vec!["cli", "plugins", "inspect"], "compatibility_warnings"),
        (vec!["cli", "plugins", "doctor"], "doctor"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("cli command must emit valid json");
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn cli_paths_reports_active_binary_metadata() {
    let stdout = run(&["cli", "paths"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload.get("active_binary").is_some());
    assert!(payload.get("path_binaries").is_some());
    assert!(payload.get("post_install_hint").is_some());
}

#[test]
fn cli_doctor_reports_install_diagnostics() {
    let stdout = run(&["doctor"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let install = payload.get("install").expect("install diagnostics");
    assert!(install.get("has_path_shadowing").is_some());
    assert!(install.get("has_duplicate_installs").is_some());
    assert!(install.get("stale_wrapper_scripts").is_some());
    assert!(install.get("legacy_installer_conflicts").is_some());
    assert!(install
        .get("has_mismatched_wheel_binary_versions")
        .is_some());
}

#[test]
fn executes_dev_cli_namespace_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["dev", "cli", "inventory"], "scripts"),
        (vec!["dev", "cli", "routes"], "routes"),
        (vec!["dev", "cli", "route-audit"], "summary"),
        (vec!["dev", "cli", "registry"], "registry"),
        (vec!["dev", "cli", "parity"], "rust_python"),
        (vec!["dev", "cli", "docs"], "docs_count"),
        (vec!["dev", "cli", "docs-audit"], "docs_audit"),
        (vec!["dev", "cli", "plugin-health"], "machine_report"),
        (vec!["dev", "cli", "status"], "current_rust_state"),
        (vec!["dev", "cli", "script-audit"], "scripts"),
        (
            vec!["dev", "cli", "scripts", "remaining"],
            "remaining_root_scripts",
        ),
        (vec!["dev", "cli", "scripts", "migrated"], "migrated"),
        (vec!["dev", "cli", "scripts", "diff"], "remaining"),
        (vec!["dev", "cli", "scripts", "audit"], "migrated"),
        (vec!["dev", "cli", "scripts", "package-metadata"], "status"),
        (vec!["dev", "cli", "scripts", "e2e-contract"], "status"),
        (vec!["dev", "cli", "scripts", "pip-audit"], "status"),
        (
            vec!["dev", "cli", "scripts", "capture-python-behavior"],
            "status",
        ),
        (vec!["dev", "cli", "rustdoc", "audit"], "coverage"),
        (vec!["dev", "cli", "rustdoc", "coverage"], "coverage"),
        (
            vec!["dev", "cli", "rustdoc", "broken-links"],
            "broken_links",
        ),
        (
            vec!["dev", "cli", "rustdoc", "public-api"],
            "missing_public_docs",
        ),
        (vec!["dev", "cli", "rustdoc", "examples"], "example_sources"),
        (
            vec!["dev", "cli", "release", "status"],
            "release_status_manifest",
        ),
        (vec!["dev", "cli", "release", "evidence"], "bundle"),
        (vec!["dev", "cli", "release", "readiness"], "release_ready"),
        (vec!["dev", "cli", "release", "diff"], "done"),
        (vec!["dev", "cli", "release", "gaps"], "missing_evidence"),
        (
            vec!["dev", "cli", "release", "behavior-changes"],
            "commands",
        ),
        (
            vec!["dev", "cli", "release", "intentional-differences"],
            "items",
        ),
        (vec!["dev", "cli", "release", "unresolved-gaps"], "items"),
        (
            vec!["dev", "cli", "release", "compatibility-leftovers"],
            "series",
        ),
        (vec!["dev", "cli", "evidence", "list"], "records"),
        (
            vec![
                "dev",
                "cli",
                "evidence",
                "show",
                "--id",
                "EVIDENCE-1001-RELEASE-TRUTH",
            ],
            "found",
        ),
        (vec!["dev", "cli", "evidence", "audit"], "status"),
        (vec!["dev", "cli", "evidence", "stale"], "stale"),
        (vec!["dev", "cli", "evidence", "matrix"], "status_matrix"),
        (
            vec!["dev", "cli", "evidence", "website-export"],
            "website_export",
        ),
        (vec!["dev", "cli", "evidence", "ci-export"], "ci_export"),
        (
            vec!["dev", "cli", "evidence", "release-export"],
            "release_export",
        ),
        (vec!["dev", "cli", "evidence", "command-map"], "command_map"),
        (vec!["dev", "cli", "evidence", "parity-map"], "parity_map"),
        (vec!["dev", "cli", "config", "rust-owner"], "rust_owner"),
        (vec!["dev", "cli", "config", "python-owner"], "python_owner"),
        (vec!["dev", "cli", "config", "ownership"], "owners"),
        (vec!["dev", "cli", "config", "drift"], "drift"),
        (vec!["dev", "cli", "config", "shape"], "schemas"),
        (vec!["dev", "cli", "config", "evidence-map"], "evidence_ids"),
        (
            vec!["dev", "cli", "python", "bridge-status"],
            "bridge_status",
        ),
        (
            vec!["dev", "cli", "python", "surface-status"],
            "surface_status",
        ),
        (
            vec!["dev", "cli", "python", "sovereignty-audit"],
            "python_sovereignty_audit",
        ),
        (vec!["dev", "cli", "python", "drift"], "drift"),
        (vec!["dev", "cli", "python", "packaging"], "packaging"),
        (vec!["dev", "cli", "repo", "health"], "repo_health"),
        (
            vec!["dev", "cli", "repo", "drift"],
            "dead_scripts_references",
        ),
        (
            vec!["dev", "cli", "repo", "inventories"],
            "stale_inventories",
        ),
        (
            vec!["dev", "cli", "repo", "generated"],
            "stale_generated_artifacts",
        ),
        (vec!["dev", "cli", "repo", "stale"], "stale_snapshots"),
        (vec!["dev", "cli", "dashboard"], "dashboard"),
        (vec!["dev", "cli", "quickcheck"], "quickcheck"),
        (vec!["dev", "cli", "truth"], "truth"),
        (vec!["dev", "cli", "blockers"], "blockers"),
        (vec!["dev", "cli", "next"], "next"),
        (vec!["dev", "cli", "snapshots-audit"], "snapshots"),
        (vec!["dev", "cli", "fixture-audit"], "parity_fixtures"),
        (vec!["dev", "cli", "crate-health"], "crate_metrics"),
        (
            vec!["dev", "cli", "package-health"],
            "install_state_assumptions",
        ),
        (vec!["dev", "cli", "env"], "source_precedence"),
        (vec!["dev", "cli", "doctor"], "issues"),
        (vec!["dev", "cli", "contracts"], "contracts"),
        (vec!["dev", "cli", "runtime-identity"], "entrypoints"),
        (vec!["dev", "cli", "docs-prune-plan"], "target_cap"),
        (vec!["dev", "cli", "state-audit"], "paths"),
        (vec!["dev", "cli", "state-doctor"], "doctor"),
        (vec!["dev", "cli", "atlas"], "mount"),
        (vec!["dev", "cli", "di"], "container"),
        (vec!["dev", "cli", "list-products"], "products"),
        (vec!["dev", "cli", "list-plugins"], "plugins"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("dev cli command must emit valid json");
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn unsupported_config_set_input_returns_usage_error() {
    let output = run_raw(&["cli", "config", "set", "INVALID_PAIR"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        output.stdout.is_empty(),
        "usage failures must not write to stdout"
    );
    let stderr: serde_json::Value =
        serde_json::from_slice(&output.stderr).expect("usage failure stderr json");
    assert_eq!(stderr["status"], "error");
    assert_eq!(stderr["code"], 2);
    assert!(
        stderr["message"]
            .as_str()
            .is_some_and(|msg| msg.to_ascii_lowercase().contains("argument")),
        "usage failure should explain invalid argument"
    );
}

#[test]
fn runtime_identity_reports_ambiguous_active_binary_selection() {
    let temp = env::temp_dir().join(format!("bijux-runtime-identity-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    let first = temp.join("first");
    let second = temp.join("second");
    fs::create_dir_all(&first).expect("create first dir");
    fs::create_dir_all(&second).expect("create second dir");
    fs::write(first.join("bijux"), b"#!/bin/sh\n").expect("write first binary");
    fs::write(second.join("bijux"), b"#!/bin/sh\n").expect("write second binary");
    let path_value = env::join_paths([&first, &second]).expect("join path");

    let stdout = run_with_env(
        &["dev", "cli", "runtime-identity"],
        "PATH",
        path_value.to_str().expect("utf-8 path"),
    );
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(payload["active_binary_selection_is_ambiguous"], true);
    assert_eq!(payload["diagnostics"]["path_shadowing_detected"], true);
    let binaries = payload["path_binaries"].as_array().expect("array");
    assert!(binaries.len() >= 2);

    fs::remove_dir_all(&temp).expect("cleanup temp");
}

#[test]
fn runtime_identity_reports_python_bridge_support_diagnostic() {
    let stdout = run_with_env(
        &["dev", "cli", "runtime-identity"],
        "BIJUX_PYTHON_BRIDGE_SUPPORTED",
        "0",
    );
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(payload["diagnostics"]["python_bridge_supported"], false);
}

#[test]
fn crate_health_reports_usage_error_for_unknown_flag() {
    let output = run_raw(&["dev", "cli", "crate-health", "--unknown-flag"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    assert!(stderr.contains("Usage: bijux"));
    assert!(stderr.contains("Commands:"));
}

#[test]
fn scripts_provenance_statement_generates_output_file() {
    let temp = env::temp_dir().join(format!("bijux-provenance-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("create temp dir");
    let stdout = run(&[
        "dev",
        "cli",
        "scripts",
        "provenance-statement",
        "--tag",
        "v0.0.0-test",
        "--output-dir",
        temp.to_str().expect("utf-8 temp"),
    ]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(payload["status"], "ok");
    let file = payload["file"].as_str().expect("file path");
    assert!(
        std::path::Path::new(file).exists(),
        "provenance file should exist"
    );
    fs::remove_dir_all(&temp).expect("cleanup temp");
}

#[test]
fn dev_cli_status_is_deterministic_across_repeated_runs() {
    let first = run(&["dev", "cli", "status", "--format", "json", "--no-pretty"]);
    let second = run(&["dev", "cli", "status", "--format", "json", "--no-pretty"]);
    let first_payload: serde_json::Value = serde_json::from_str(&first).expect("valid json");
    let second_payload: serde_json::Value = serde_json::from_str(&second).expect("valid json");
    assert_eq!(
        first_payload["next_phase_priorities"],
        second_payload["next_phase_priorities"]
    );
    assert_eq!(
        first_payload["next_phase_summary_text"],
        second_payload["next_phase_summary_text"]
    );
    assert_eq!(
        first_payload["command_migration"],
        second_payload["command_migration"]
    );
}

#[test]
fn dev_cli_parity_surfaces_dashboard_artifact() {
    let stdout = run(&["dev", "cli", "parity", "--format", "json", "--no-pretty"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload["parity_dashboard"].is_object());
    assert!(payload["parity_dashboard_text"].is_string());
}
