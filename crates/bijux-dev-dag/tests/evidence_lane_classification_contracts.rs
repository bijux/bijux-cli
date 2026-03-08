use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn evidence_command_classification_policy_is_defined() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_command_classification.json"))
            .expect("read evidence command classification policy"),
    )
    .expect("parse evidence command classification policy");

    let release_critical: BTreeSet<String> = policy["release_critical_verify_commands"]
        .as_array()
        .expect("release_critical_verify_commands array")
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .expect("release-critical command")
                .to_string()
        })
        .collect();
    let advisory: BTreeSet<String> = policy["advisory_verify_commands"]
        .as_array()
        .expect("advisory_verify_commands array")
        .iter()
        .map(|entry| entry.as_str().expect("advisory command").to_string())
        .collect();

    assert!(
        !release_critical.is_empty(),
        "release-critical evidence command set cannot be empty"
    );
    assert!(
        !advisory.is_empty(),
        "advisory evidence command set cannot be empty"
    );
    assert!(
        release_critical.is_disjoint(&advisory),
        "release-critical and advisory command sets must not overlap"
    );

    let fast_lane = policy["fast_lane"]["blocking"]
        .as_array()
        .expect("fast_lane.blocking array");
    let fast_lane_commands: BTreeSet<String> = fast_lane
        .iter()
        .map(|entry| entry.as_str().expect("fast lane command").to_string())
        .collect();
    assert_eq!(
        policy["fast_lane"]["advisory_blocking_default"]
            .as_bool()
            .expect("fast_lane.advisory_blocking_default bool"),
        false,
        "advisory evidence must be non-blocking by default in fast lane"
    );
    for command in &fast_lane_commands {
        assert!(
            release_critical.contains(command),
            "fast lane blocking command must be release-critical: {command}"
        );
    }
}

#[test]
fn full_lane_exercises_all_release_critical_evidence_commands() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_command_classification.json"))
            .expect("read evidence command classification policy"),
    )
    .expect("parse evidence command classification policy");

    let release_critical: BTreeSet<String> = policy["release_critical_verify_commands"]
        .as_array()
        .expect("release_critical_verify_commands array")
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .expect("release-critical command")
                .to_string()
        })
        .collect();

    let make_root = fs::read_to_string(root.join("make/root.mk")).expect("read make/root.mk");
    for command in release_critical {
        let target = command.replace("verify evidence-", "evidence-");
        assert!(
            make_root.contains(&format!("@$(MAKE) {}", target)),
            "make test-all must execute release-critical evidence target: {target}"
        );
    }
}

#[test]
fn advisory_evidence_remains_non_blocking_in_fast_lane_docs() {
    let root = repo_root();
    let lanes = fs::read_to_string(root.join("docs/reference/TEST_LANES.md"))
        .expect("read test lanes reference");
    assert!(
        lanes.contains("Advisory evidence checks are non-blocking by default."),
        "test lanes doc must state advisory non-blocking default"
    );
}

#[test]
fn evidence_classification_reports_and_specs_exist() {
    let root = repo_root();
    for rel in [
        "docs/adr/20260308-dev-dag-cleanup-end-state.md",
        "docs/spec/EVIDENCE_GLOSSARY.md",
        "docs/spec/EVIDENCE_TERMS_AND_GOVERNANCE.md",
        "docs/reference/EVIDENCE_GOVERNANCE.md",
        "docs/reports/foundation/release_critical_evidence_matrix.md",
        "docs/reports/foundation/advisory_evidence_matrix.md",
        "docs/reports/foundation/evidence_command_owner_map.md",
        "docs/reports/foundation/evidence_ci_exercise_report.md",
        "docs/reports/foundation/evidence_report_consolidation.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing evidence governance surface: {rel}"
        );
    }
}

#[test]
fn stale_note_only_reports_removed_from_evidence_reports_root() {
    let root = repo_root();
    for rel in [
        "evidence/reports/root_tests_asset_deletions.md",
        "evidence/reports/root_tests_asset_migrations.md",
    ] {
        assert!(
            !root.join(rel).exists(),
            "stale note-only report should be removed: {rel}"
        );
    }
}

#[test]
fn evidence_command_owner_map_covers_governed_verify_commands() {
    let root = repo_root();
    let map =
        fs::read_to_string(root.join("docs/reports/foundation/evidence_command_owner_map.md"))
            .expect("read evidence command owner map");
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_suite_policy.json"))
            .expect("read evidence suite policy"),
    )
    .expect("parse evidence suite policy");

    for suite in policy["suites"].as_array().expect("suites array") {
        let cmd = suite["verify_command"].as_str().expect("verify command");
        assert!(
            map.contains(&format!("`{}`", cmd)),
            "owner map must include verify command: {cmd}"
        );
    }
}
