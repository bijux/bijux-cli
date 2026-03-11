#![forbid(unsafe_code)]
//! Closure suites for dev-cli control-plane surfaces.

use std::collections::BTreeMap;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn dev_cli_commands() -> Vec<Vec<String>> {
    include_str!("../../../data/fixtures/routing/dev_cli_subcommands.txt")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().map(ToString::to_string).collect())
        .collect()
}

fn to_refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

#[test]
fn integration_suite_runs_all_dev_cli_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--format".to_string());
        args.push("json".to_string());
        args.push("--no-pretty".to_string());
        let out = run(&to_refs(&args));
        assert!(
            out.status.success(),
            "command failed: {:?}\nstdout={}\nstderr={}",
            args,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn json_shape_suite_is_stable_for_all_dev_cli_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--format".to_string());
        args.push("json".to_string());
        args.push("--no-pretty".to_string());
        let first = run(&to_refs(&args));
        let second = run(&to_refs(&args));
        assert!(first.status.success(), "first run failed for {:?}", args);
        assert!(second.status.success(), "second run failed for {:?}", args);
        let first_json: Value = serde_json::from_slice(&first.stdout).expect("json");
        let second_json: Value = serde_json::from_slice(&second.stdout).expect("json");
        assert!(first_json.is_object(), "payload must be object for {:?}", args);
        assert_eq!(
            first_json.as_object().map(|obj| obj.keys().cloned().collect::<Vec<_>>()),
            second_json.as_object().map(|obj| obj.keys().cloned().collect::<Vec<_>>()),
            "top-level key drift for {:?}",
            args
        );
    }
}

#[test]
fn text_snapshot_suite_keeps_head_lines_stable_for_primary_dashboards() {
    let commands = [
        ("dev cli status --format text", "dev cli status"),
        ("dev cli parity --format text", "dev cli parity"),
        ("dev cli state-audit --format text", "dev cli state-audit"),
    ];
    let mut heads = BTreeMap::<String, String>::new();
    for (argv, name) in commands {
        let args: Vec<String> = argv.split_whitespace().map(ToString::to_string).collect();
        let out = run(&to_refs(&args));
        assert!(out.status.success(), "text run failed for {argv}");
        let text = String::from_utf8(out.stdout).expect("utf8");
        let head = text.lines().take(3).collect::<Vec<_>>().join("\n");
        heads.insert(name.to_string(), head);
    }
    let snapshot = serde_json::to_string_pretty(&heads).expect("serialize");
    assert_eq!(
        snapshot + "\n",
        include_str!("../../../data/golden/cli_surface/dev_cli_control_plane_text_heads.json"),
        "text head snapshot drift"
    );
}

#[test]
fn failure_path_suite_rejects_unknown_flags_for_all_dev_cli_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--unknown-flag".to_string());
        let out = run(&to_refs(&args));
        assert!(!out.status.success(), "command unexpectedly accepted unknown flag: {:?}", args);
    }
}

#[test]
fn repeated_run_determinism_suite_for_all_dev_cli_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--format".to_string());
        args.push("json".to_string());
        args.push("--no-pretty".to_string());
        let first = run(&to_refs(&args));
        let second = run(&to_refs(&args));
        assert!(first.status.success(), "first run failed for {:?}", args);
        assert!(second.status.success(), "second run failed for {:?}", args);
        assert_eq!(first.stdout, second.stdout, "stdout drift for {:?}", args);
        assert_eq!(first.stderr, second.stderr, "stderr drift for {:?}", args);
    }
}

#[test]
fn runtime_data_consistency_suite_matches_query_truth() {
    let routes = run(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let inspect = run(&["inspect", "--format", "json", "--no-pretty"]);
    let env = run(&["dev", "cli", "env", "--format", "json", "--no-pretty"]);
    assert!(routes.status.success());
    assert!(inspect.status.success());
    assert!(env.status.success());

    let routes_json: Value = serde_json::from_slice(&routes.stdout).expect("routes json");
    let inspect_json: Value = serde_json::from_slice(&inspect.stdout).expect("inspect json");
    let route_roots: std::collections::BTreeSet<String> = routes_json["routes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    let inspect_roots: std::collections::BTreeSet<String> = inspect_json["route_sources"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    assert!(route_roots.iter().all(|root| inspect_roots.contains(root)));
}

#[test]
fn script_replacement_suite_tracks_migrated_workflows() {
    let script = run(&["dev", "cli", "script-audit", "--format", "json", "--no-pretty"]);
    assert!(script.status.success());
    let payload: Value = serde_json::from_slice(&script.stdout).expect("script json");
    let scripts = payload["scripts"].as_array().expect("scripts rows");
    assert_eq!(
        payload["replacement_rule"],
        "new maintainer automation defaults to bijux dev cli commands"
    );
    let required = [
        "scripts/status/generate_status_reports.py",
        "scripts/parity/generate_command_law_reports.py",
        "scripts/status/generate_state_audit_reports.py",
        "scripts/status/generate_install_truth_reports.py",
    ];
    for path in required {
        let found =
            scripts.iter().any(|row| row.get("path") == Some(&Value::String(path.to_string())));
        assert!(found, "missing script inventory row for replacement candidate: {path}");
    }
}

#[test]
fn package_runtime_state_parity_suite_has_consistent_truth_commands() {
    let identity = run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]);
    let package = run(&["dev", "cli", "package-health", "--format", "json", "--no-pretty"]);
    let state_audit = run(&["dev", "cli", "state-audit", "--format", "json", "--no-pretty"]);
    let state_doctor = run(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"]);
    assert!(identity.status.success());
    assert!(package.status.success());
    assert!(state_audit.status.success());
    assert!(state_doctor.status.success());

    let identity_json: Value = serde_json::from_slice(&identity.stdout).expect("identity json");
    let package_json: Value = serde_json::from_slice(&package.stdout).expect("package json");
    let audit_json: Value = serde_json::from_slice(&state_audit.stdout).expect("audit json");
    let doctor_json: Value = serde_json::from_slice(&state_doctor.stdout).expect("doctor json");

    assert!(identity_json.get("entrypoints").is_some());
    assert!(package_json.get("install_state_assumptions").is_some());
    assert!(audit_json.get("paths").is_some());
    assert!(doctor_json.get("doctor").is_some());
}

#[test]
fn audit_integration_suite_links_script_docs_and_crate_health() {
    let script = run(&["dev", "cli", "script-audit", "--format", "json", "--no-pretty"]);
    let docs = run(&["dev", "cli", "docs-audit", "--format", "json", "--no-pretty"]);
    let crate_health = run(&["dev", "cli", "crate-health", "--format", "json", "--no-pretty"]);
    assert!(script.status.success());
    assert!(docs.status.success());
    assert!(crate_health.status.success());

    let script_json: Value = serde_json::from_slice(&script.stdout).expect("script json");
    let docs_json: Value = serde_json::from_slice(&docs.stdout).expect("docs json");
    let crate_json: Value = serde_json::from_slice(&crate_health.stdout).expect("crate json");

    assert!(script_json["scripts"].as_array().is_some_and(|v| !v.is_empty()));
    assert!(docs_json["docs_count"].as_u64().is_some_and(|v| v > 0));
    assert!(crate_json.get("crate_metrics").is_some());
}

#[test]
fn default_dashboard_and_truth_commands_are_explicit_in_payloads() {
    let status = run(&["dev", "cli", "status", "--format", "json", "--no-pretty"]);
    let parity = run(&["dev", "cli", "parity", "--format", "json", "--no-pretty"]);
    let runtime_identity =
        run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"]);
    let state_audit = run(&["dev", "cli", "state-audit", "--format", "json", "--no-pretty"]);
    assert!(status.status.success());
    assert!(parity.status.success());
    assert!(runtime_identity.status.success());
    assert!(state_audit.status.success());

    let status_json: Value = serde_json::from_slice(&status.stdout).expect("status json");
    let parity_json: Value = serde_json::from_slice(&parity.stdout).expect("parity json");
    let runtime_json: Value =
        serde_json::from_slice(&runtime_identity.stdout).expect("runtime identity json");
    let state_json: Value = serde_json::from_slice(&state_audit.stdout).expect("state audit json");

    assert_eq!(status_json["maintainer_dashboard_default"], "bijux dev cli status");
    assert_eq!(parity_json["migration_dashboard_default"], "bijux dev cli parity");
    assert_eq!(runtime_json["runtime_truth_default"], "bijux dev cli runtime-identity");
    assert_eq!(state_json["state_truth_default"], "bijux dev cli state-audit");
}
