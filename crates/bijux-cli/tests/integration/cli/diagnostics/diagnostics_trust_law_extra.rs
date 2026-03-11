#![forbid(unsafe_code)]
//! Diagnostics trust law coverage for TODOs 361-374.
//! test_type: diagnostics-trust

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_env(args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir()
        .join(format!("bijux-diagnostics-trust-{label}-{}-{nanos}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir");
    path
}

#[test]
fn dev_cli_contracts_and_routes_match_snapshot_semantics_and_are_byte_stable() {
    let contracts_a = run(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]);
    let contracts_b = run(&["dev", "cli", "contracts", "--format", "json", "--no-pretty"]);
    assert_eq!(contracts_a.status.code(), Some(0));
    assert_eq!(contracts_a.stdout, contracts_b.stdout);

    let routes_a = run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let routes_b = run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    assert_eq!(routes_a.status.code(), Some(0));
    assert_eq!(routes_a.stdout, routes_b.stdout);

    let contracts_live = parse_json(&contracts_a.stdout);
    let contracts_snapshot: Value =
        serde_json::from_str(include_str!("../../../data/golden/ported/dev_cli_contracts.json"))
            .expect("contracts snapshot");
    assert_eq!(contracts_live, contracts_snapshot);

    let routes_live = parse_json(&routes_a.stdout);
    let routes_snapshot: Value =
        serde_json::from_str(include_str!("../../../data/golden/ported/dev_cli_routes.json"))
            .expect("routes snapshot");
    let live_routes: BTreeSet<String> = routes_live["routes"]
        .as_array()
        .expect("live routes")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let snapshot_routes: BTreeSet<String> = routes_snapshot["routes"]
        .as_array()
        .expect("snapshot routes")
        .iter()
        .map(|row| {
            row["segments"]
                .as_array()
                .expect("segments")
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    assert!(
        snapshot_routes.is_subset(&live_routes),
        "live routes must contain all snapshot routes"
    );
}

#[test]
fn dev_cli_registry_env_parity_crate_health_and_docs_audit_reflect_live_truth() {
    let registry_live =
        parse_json(&run(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]).stdout);
    let registry_snapshot: Value =
        serde_json::from_str(include_str!("../../../data/golden/ported/dev_cli_registry.json"))
            .expect("registry snapshot");
    assert_eq!(registry_live, registry_snapshot);

    let env_live =
        parse_json(&run(&["dev", "cli", "env", "--format", "json", "--no-pretty"]).stdout);
    assert_eq!(
        env_live["source_precedence"],
        serde_json::json!(["flags", "env", "config", "defaults"])
    );
    assert!(env_live["active"]["config_file"].is_string());

    let parity =
        parse_json(&run(&["dev", "cli", "parity", "--format", "json", "--no-pretty"]).stdout);
    assert!(parity["binary_bridge"]["cases"].as_array().is_some_and(|v| !v.is_empty()));

    let crate_health =
        parse_json(&run(&["dev", "cli", "crate-health", "--format", "json", "--no-pretty"]).stdout);
    let crates: BTreeSet<String> = crate_health["crate_metrics"]["crate_decisions"]
        .as_array()
        .expect("crate decisions")
        .iter()
        .filter_map(|row| row["crate"].as_str())
        .map(ToString::to_string)
        .collect();
    for expected in ["bijux-cli", "bijux-cli", "bijux-cli-python"] {
        assert!(crates.contains(expected), "crate-health missing {expected}");
    }

    let docs_audit =
        parse_json(&run(&["dev", "cli", "docs-audit", "--format", "json", "--no-pretty"]).stdout);
    let docs = docs_audit["docs"].as_array().expect("docs list");
    assert!(!docs.is_empty(), "docs audit must list documentation files");
    let docs_paths: BTreeSet<&str> = docs.iter().filter_map(Value::as_str).collect();
    assert!(docs_paths.contains("docs/index.md"));
    assert!(docs_paths.contains("docs/reference/index.md"));
    assert!(docs_audit["docs_audit"]["docs_count"].is_u64());
}

#[test]
fn doctor_plugin_doctor_and_runtime_identity_provide_actionable_diagnostics_for_problem_cases() {
    let root = temp_dir("actionable");
    let config = root.join("broken.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write config");

    let doctor = parse_json(
        &run_env(
            &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
            &[("BIJUXCLI_CONFIG", &config)],
        )
        .stdout,
    );
    assert_eq!(doctor["doctor"]["status"], "degraded");
    let issues = doctor["doctor"]["issues"].as_array().expect("issues");
    assert!(!issues.is_empty());
    assert!(issues[0]["message"].as_str().unwrap_or_default().contains("Malformed line"));
    assert!(issues[0]["path"].is_string());

    let plugin_health = parse_json(
        &run(&["dev", "cli", "plugin-health", "--format", "json", "--no-pretty"]).stdout,
    );
    let text_report = plugin_health["machine_report"]["text_report"].as_str().unwrap_or_default();
    assert!(text_report.contains("Use `bijux dev cli plugin-health --format json`"));

    let runtime = parse_json(
        &run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]).stdout,
    );
    if runtime["active_binary_selection_is_ambiguous"].as_bool().unwrap_or(false) {
        let summary = runtime["text_summary"].as_array().expect("text summary");
        let summary_text = summary.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n");
        assert!(summary_text.contains("path shadowing: detected"));
    }
}

#[test]
fn diagnostics_do_not_invent_unsupported_remediation_steps() {
    let outputs = [
        String::from_utf8(run(&["dev", "cli", "doctor", "--format", "text"]).stdout)
            .expect("doctor text"),
        String::from_utf8(run(&["dev", "cli", "plugin-health", "--format", "text"]).stdout)
            .expect("plugin-health text"),
        String::from_utf8(run(&["dev", "cli", "runtime-identity", "--format", "text"]).stdout)
            .expect("runtime-identity text"),
    ]
    .join("\n");

    for unsupported in ["curl | sh", "sudo rm -rf", "brew install", "pip install -U"] {
        assert!(
            !outputs.to_lowercase().contains(&unsupported.to_lowercase()),
            "diagnostics text must not invent unsupported remediation step: {unsupported}"
        );
    }
}

#[test]
fn diagnostics_text_is_boring_and_json_is_machine_friendly() {
    let text =
        String::from_utf8(run(&["dev", "cli", "doctor", "--format", "text"]).stdout).expect("text");
    let lower = text.to_lowercase();
    for promotional in ["awesome", "amazing", "revolutionary", "best-in-class", "delightful"] {
        assert!(!lower.contains(promotional), "diagnostics text should remain neutral");
    }

    let diagnostics_commands = [
        ["dev", "cli", "doctor"],
        ["dev", "cli", "state-doctor"],
        ["dev", "cli", "state-audit"],
        ["dev", "cli", "plugin-health"],
        ["dev", "cli", "runtime-identity"],
    ];

    for cmd in diagnostics_commands {
        let mut args = cmd.to_vec();
        args.extend(["--format", "json", "--no-pretty"]);
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "json diagnostics failed for {cmd:?}");
        assert!(out.stderr.is_empty(), "json diagnostics should keep stderr empty for {cmd:?}");
        let payload = parse_json(&out.stdout);
        assert!(payload.is_object(), "diagnostics payload should be object for {cmd:?}");
        let stdout_text = String::from_utf8(out.stdout).expect("utf-8");
        assert!(!stdout_text.contains("\u{1b}["), "json output must not contain ANSI escapes");
    }
}

#[test]
fn diagnostics_runs_are_deterministic_for_covered_commands() {
    let cases = [
        ["dev", "cli", "doctor", "--format", "json", "--no-pretty"],
        ["dev", "cli", "contracts", "--format", "json", "--no-pretty"],
        ["dev", "cli", "routes", "--format", "json", "--no-pretty"],
        ["dev", "cli", "registry", "--format", "json", "--no-pretty"],
        ["dev", "cli", "env", "--format", "json", "--no-pretty"],
        ["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"],
    ];

    for args in cases {
        let a = run(&args);
        let b = run(&args);
        assert_eq!(a.status.code(), Some(0));
        assert_eq!(a.stdout, b.stdout, "diagnostics output drift for {args:?}");
        assert_eq!(a.stderr, b.stderr, "diagnostics stderr drift for {args:?}");
    }
}
