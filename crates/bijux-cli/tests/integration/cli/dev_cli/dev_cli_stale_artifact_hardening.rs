#![forbid(unsafe_code)]
//! Stale-artifact hardening contracts for dev-cli control-plane credibility.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

const GENERATOR_ID: &str = "STATUS-CONTRACT-GENERATE-DEV-CLI-STALE-ARTIFACT-REPORTS";
const GATE_ID: &str = "STATUS-CONTRACT-ENFORCE-DEV-CLI-STALE-ARTIFACT-GATE";

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

fn write_json(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, "{}\n").expect("write");
}

fn seeded_artifact_root(prefix: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()));
    let status = root.join("artifacts").join("status");
    fs::create_dir_all(&status).expect("mkdir");
    for name in [
        "evidence_integrity_artifact.json",
        "parity_drift_artifact.json",
        "migration_truth_artifact.json",
        "package_health_diagnostics_artifact.json",
        "state_audit_truth_artifact.json",
        "docs_audit.json",
        "maintenance_gap_behaviors.json",
        "duplication_hotspots.json",
        "dev_cli_next_report.json",
    ] {
        write_json(&status.join(name));
    }
    root
}

fn run_generator(root: &Path, force_stale: &[&str], inject_mode: bool) -> Value {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(["dev", "cli", "maintenance", "status", "run", "--id", GENERATOR_ID])
        .current_dir(workspace_root())
        .env("DEV_CLI_STALE_ARTIFACT_ROOT", root)
        .env("DEV_CLI_STALE_MAX_SECONDS", "999999999");
    if !force_stale.is_empty() {
        cmd.env("DEV_CLI_FORCE_STALE_FILES", force_stale.join(","));
    }
    if inject_mode {
        cmd.env("DEV_CLI_INJECT_STALE_ARTIFACT", "1");
    }
    let out = cmd.output().expect("generator should run");
    assert!(
        out.status.success(),
        "generator failed\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = fs::read_to_string(root.join("artifacts/status/stale_artifact_artifact.json"))
        .expect("read stale artifact");
    serde_json::from_str(&payload).expect("valid json")
}

fn run_gate(root: &Path, allow_injection_drift: bool) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(["dev", "cli", "maintenance", "status", "run", "--id", GATE_ID])
        .current_dir(workspace_root())
        .env("DEV_CLI_STALE_ARTIFACT_ROOT", root);
    if allow_injection_drift {
        cmd.env("DEV_CLI_ALLOW_INJECTION_DRIFT", "1");
    }
    cmd.output().expect("gate should run")
}

#[test]
fn stale_scenarios_are_detected_for_all_required_commands() {
    let root = seeded_artifact_root("bijux-stale-scenarios");
    let payload = run_generator(
        &root,
        &[
            "artifacts/status/evidence_integrity_artifact.json",
            "artifacts/status/parity_drift_artifact.json",
            "artifacts/status/migration_truth_artifact.json",
            "artifacts/status/package_health_diagnostics_artifact.json",
            "artifacts/status/state_audit_truth_artifact.json",
            "artifacts/status/docs_audit.json",
            "artifacts/status/maintenance_gap_behaviors.json",
            "artifacts/status/duplication_hotspots.json",
        ],
        false,
    );
    let checks = payload["checks"].as_array().cloned().unwrap_or_default();
    assert!(!checks.is_empty(), "stale checks should exist");
    for scenario_id in [
        "evidence_stale_before_evidence_stale",
        "parity_stale_before_status",
        "migration_stale_before_truth",
        "package_health_stale_before_dashboard",
        "state_audit_stale_before_blockers",
        "docs_audit_stale_before_repo_health",
        "maintenance_audit_stale_before_repo_health",
        "crate_health_stale_before_crate_health",
    ] {
        let matched = checks.iter().any(|row| {
            row["scenario_id"] == scenario_id
                && (row["state"] == "stale" || row["state"] == "missing")
        });
        assert!(matched, "scenario {scenario_id} should be stale/missing");
    }
}

#[test]
fn deleted_evidence_artifact_is_reported_as_missing() {
    let root = seeded_artifact_root("bijux-stale-missing-evidence");
    fs::remove_file(root.join("artifacts/status/evidence_integrity_artifact.json"))
        .expect("remove evidence");
    let payload = run_generator(&root, &[], false);
    let checks = payload["checks"].as_array().cloned().unwrap_or_default();
    let evidence_row = checks
        .iter()
        .find(|row| row["scenario_id"] == "evidence_deleted_before_evidence_audit")
        .expect("evidence scenario exists");
    assert_eq!(evidence_row["state"], Value::String("missing".to_string()));
}

#[test]
fn mixed_stale_and_fresh_inputs_remain_honest() {
    let root = seeded_artifact_root("bijux-stale-mixed");
    let payload = run_generator(&root, &["artifacts/status/migration_truth_artifact.json"], false);
    let summary = &payload["summary"];
    let stale_or_missing = summary["stale_or_missing_count"].as_u64().unwrap_or(0);
    let fresh = summary["fresh_count"].as_u64().unwrap_or(0);
    assert!(stale_or_missing > 0, "must report stale or missing inputs");
    assert!(fresh > 0, "must still report fresh inputs");
}

#[test]
fn critical_stale_inputs_fail_gate() {
    let root = seeded_artifact_root("bijux-stale-critical-gate");
    run_generator(&root, &["artifacts/status/parity_drift_artifact.json"], false);
    let out = run_gate(&root, false);
    assert!(!out.status.success(), "critical stale input should fail gate");
}

#[test]
fn warning_only_stale_inputs_are_tolerated_with_warning() {
    let root = seeded_artifact_root("bijux-stale-warning-gate");
    let payload = run_generator(&root, &["artifacts/status/dev_cli_next_report.json"], false);
    assert_eq!(payload["summary"]["critical_stale_count"].as_u64().unwrap_or(99), 0);
    assert!(payload["summary"]["warning_stale_count"].as_u64().unwrap_or(0) > 0);
    let out = run_gate(&root, false);
    assert!(
        out.status.success(),
        "warning-only stale inputs should be tolerated by gate\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn ci_injection_mode_detects_stale_and_is_verifiable() {
    let root = seeded_artifact_root("bijux-stale-injection");
    let payload = run_generator(&root, &[], true);
    assert!(payload["summary"]["injection_mode"].as_bool().unwrap_or(false));
    assert!(payload["summary"]["critical_stale_count"].as_u64().unwrap_or(0) > 0);
    let out = run_gate(&root, true);
    assert!(
        out.status.success(),
        "injection verification mode should pass\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
