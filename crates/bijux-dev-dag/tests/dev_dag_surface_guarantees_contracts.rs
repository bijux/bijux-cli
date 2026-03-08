use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn dev_dag_461_480_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/dev_dag_command_purpose_map_report.md",
        "docs/reports/foundation/dev_dag_command_redundancy_461_480_report.md",
        "docs/reports/foundation/dev_dag_legacy_command_namespace_report.md",
        "docs/reports/foundation/dev_dag_advisory_surface_policy_report.md",
        "docs/reports/foundation/dev_dag_release_critical_command_pack_report.md",
        "docs/reports/foundation/dev_dag_maintenance_command_pack_report.md",
        "docs/reports/foundation/dev_dag_surface_shrink_report.md",
        "docs/reports/foundation/dev_dag_command_size_trend_report.md",
        "docs/reports/foundation/dev_dag_health_dashboard.md",
        "docs/reports/foundation/dev_dag_contraction_461_480_status_report.md",
        "configs/suites/dev_dag_release_critical_pack.json",
        "configs/suites/dev_dag_maintenance_pack.json",
        "configs/suites/dev_dag_contraction_verification.json",
        "docs/adr/20260308-dev-dag-long-term-role.md",
    ] {
        assert!(root.join(rel).exists(), "missing dev-dag artifact: {rel}");
    }
}

#[test]
fn dev_dag_contraction_status_report_maps_461_480() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/dev_dag_contraction_461_480_status_report.md"),
    )
    .expect("read dev-dag contraction status report");

    for token in [
        "461-469",
        "470-474",
        "475-480",
        "dev_dag_contraction_verification.json",
        "20260308-dev-dag-long-term-role.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn dev_dag_command_packs_and_verification_suite_are_machine_stable() {
    let root = repo_root();

    let release_pack: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/dev_dag_release_critical_pack.json"))
            .expect("read release-critical pack"),
    )
    .expect("parse release-critical pack");
    assert_eq!(release_pack["id"], "dev-dag-release-critical-pack");

    let maintenance_pack: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/dev_dag_maintenance_pack.json"))
            .expect("read maintenance pack"),
    )
    .expect("parse maintenance pack");
    assert_eq!(maintenance_pack["id"], "dev-dag-maintenance-pack");

    let verification_suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/dev_dag_contraction_verification.json"))
            .expect("read dev-dag verification suite"),
    )
    .expect("parse dev-dag verification suite");
    assert_eq!(verification_suite["id"], "dev-dag-contraction-verification");

    let verification_cmds = verification_suite["commands"]
        .as_array()
        .expect("verification commands")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "dev_dag_surface_guarantees_contracts",
        "dev_dag_surface_contraction_contracts",
        "dev_dag_contraction_coverage_progress_contracts",
        "dev_dag_command_safety_contracts",
    ] {
        assert!(verification_cmds.contains(token), "missing suite token: {token}");
    }
}

#[test]
fn advisory_and_legacy_surfaces_are_not_primary_operator_help_paths() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/dev_dag_legacy_command_namespace_report.md"),
    )
    .expect("read legacy namespace report");
    assert!(report.contains("excluded from primary help narratives"));

    let advisory = fs::read_to_string(
        root.join("docs/reports/foundation/dev_dag_advisory_surface_policy_report.md"),
    )
    .expect("read advisory policy report");
    assert!(advisory.contains("must not affect blocker paths"));
}
