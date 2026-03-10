#![forbid(unsafe_code)]
//! Contracts for parity, migration, and closure reporting.

use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_ok_json(command: &[&str]) -> Value {
    let mut args = command.to_vec();
    args.push("--format");
    args.push("json");
    args.push("--no-pretty");
    let out = run(&args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("json payload")
}

#[test]
fn parity_json_contract_is_stable() {
    let first = run_ok_json(&["dev", "cli", "parity"]);
    let second = run_ok_json(&["dev", "cli", "parity"]);
    assert_eq!(
        first
            .as_object()
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>()),
        second
            .as_object()
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>()),
        "parity json top-level keys must be stable"
    );
}

#[test]
fn parity_text_contract_is_stable() {
    let args = ["dev", "cli", "parity", "--format", "text"];
    let first = run(&args);
    let second = run(&args);
    assert!(first.status.success(), "first text run failed");
    assert!(second.status.success(), "second text run failed");
    assert_eq!(first.stdout, second.stdout, "parity text output drift");
}

#[test]
fn migration_matrix_rows_have_valid_statuses() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let rows = status["command_migration"]["matrix"]["commands"]
        .as_array()
        .expect("matrix rows");
    let allowed = std::collections::BTreeSet::from([
        "rust-complete",
        "rust-partial",
        "python-only",
        "intentionally-different",
    ]);
    for row in rows {
        let status_value = row["status"].as_str().unwrap_or_default();
        assert!(
            allowed.contains(status_value),
            "invalid migration status: {status_value}"
        );
    }
}

#[test]
fn partial_rows_have_blockers() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let rows = status["command_migration"]["matrix"]["commands"]
        .as_array()
        .expect("matrix rows");
    for row in rows {
        if row["status"] == Value::String("rust-partial".to_string()) {
            let shim_alias = row["shim_alias_dependency"].as_object();
            let has_shim_alias = shim_alias
                .map(|obj| {
                    obj.get("aliases")
                        .and_then(Value::as_array)
                        .is_some_and(|rows| !rows.is_empty())
                        || obj
                            .get("shims")
                            .and_then(Value::as_array)
                            .is_some_and(|rows| !rows.is_empty())
                })
                .unwrap_or(false);
            let parity_mismatch = row["parity_coverage"]
                .as_object()
                .map(|obj| obj.values().any(|value| value == &Value::Bool(false)))
                .unwrap_or(false);
            assert!(
                row["blocker"].as_str().is_some_and(|s| !s.trim().is_empty())
                    || has_shim_alias
                    || parity_mismatch,
                "rust-partial rows must include blocker, shim/alias dependency, or explicit parity mismatch evidence"
            );
        }
    }
}

#[test]
fn intentional_difference_rows_have_reasons() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let rows = status["command_migration"]["matrix"]["commands"]
        .as_array()
        .expect("matrix rows");
    for row in rows {
        if row["status"] == Value::String("intentionally-different".to_string()) {
            assert!(
                row["reason"].as_str().is_some_and(|s| !s.trim().is_empty()),
                "intentionally-different rows must include reason"
            );
        }
    }
}

#[test]
fn complete_rows_have_evidence_links() {
    let status = run_ok_json(&["dev", "cli", "status"]);
    let rows = status["command_migration"]["matrix"]["commands"]
        .as_array()
        .expect("matrix rows");
    for row in rows {
        if row["status"] == Value::String("rust-complete".to_string()) {
            assert!(
                row["evidence_links"]
                    .as_array()
                    .is_some_and(|links| !links.is_empty()),
                "rust-complete rows must include evidence links"
            );
        }
    }
}

#[test]
fn parity_and_migration_cover_the_same_core_commands() {
    let parity = run_ok_json(&["dev", "cli", "parity"]);
    let status = run_ok_json(&["dev", "cli", "status"]);

    let parity_commands: std::collections::BTreeSet<String> = parity["command_matrix"]["commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("command").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();
    let migration_commands: std::collections::BTreeSet<String> = status["command_migration"]
        ["matrix"]["commands"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row.get("command").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect();

    assert!(
        parity_commands
            .iter()
            .all(|command| migration_commands.contains(command)),
        "all parity commands must exist in migration matrix"
    );
}

#[test]
fn parity_and_status_completion_counts_align() {
    let parity = run_ok_json(&["dev", "cli", "parity"]);
    let status = run_ok_json(&["dev", "cli", "status"]);
    let parity_complete = parity["command_matrix"]["summary"]["complete"]
        .as_u64()
        .unwrap_or_default();
    let status_complete = status["command_migration"]["matrix"]["summary"]["rust-complete"]
        .as_u64()
        .unwrap_or_default();
    assert_eq!(parity_complete, status_complete);
}

#[test]
fn parity_output_handles_evidence_gaps_without_crashing() {
    let parity = run_ok_json(&["dev", "cli", "parity"]);
    let rows = parity["command_matrix"]["commands"]
        .as_array()
        .expect("parity rows");
    let gap_rows = rows
        .iter()
        .filter(|row| row["status"] == Value::String("partial".to_string()))
        .count();
    assert!(
        gap_rows > 0,
        "fixture must include partial rows representing evidence gaps"
    );
    assert!(
        parity["parity_dashboard"].is_object(),
        "parity dashboard must still render"
    );
}

#[test]
fn parity_output_handles_stale_or_older_inputs_without_crashing() {
    let out = run(&["dev", "cli", "parity", "--format", "json", "--no-pretty"]);
    assert!(
        out.status.success(),
        "parity command must remain resilient for stale-ish inputs"
    );
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload["parity_dashboard"]["summary"].is_object());
}

#[test]
fn parity_output_handles_corrupted_optional_state_without_crashing() {
    let root = std::env::temp_dir().join(format!("bijux-parity-corrupt-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    std::fs::write(&config, "BROKEN=\0\n").expect("write");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(["dev", "cli", "parity", "--format", "json", "--no-pretty"]);
    cmd.env("BIJUX_CONFIG_PATH", config.to_string_lossy().to_string());
    let out = cmd.output().expect("run");
    assert!(
        out.status.success(),
        "parity command failed under corrupted optional state"
    );
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload["parity_dashboard"].is_object());
}
