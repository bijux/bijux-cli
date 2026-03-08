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
fn evidence_481_500_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/evidence_core_claim_inventory_report.md",
        "docs/reports/foundation/evidence_internal_support_inventory_report.md",
        "docs/reports/foundation/evidence_weak_decision_value_report.md",
        "docs/reports/foundation/evidence_staleness_report.md",
        "docs/reports/foundation/evidence_duplication_report.md",
        "docs/reports/foundation/evidence_compact_index_report.md",
        "docs/reports/foundation/evidence_claim_to_family_map_report.md",
        "docs/reports/foundation/evidence_maintenance_checklist.md",
        "docs/reports/foundation/evidence_signal_sharpening_481_500_status_report.md",
        "docs/reports/foundation/evidence_signal_health_dashboard.md",
        "configs/suites/evidence_signal_sharpening_verification.json",
        "docs/adr/20260308-evidence-minimalism.md",
    ] {
        assert!(root.join(rel).exists(), "missing evidence artifact: {rel}");
    }
}

#[test]
fn evidence_family_governance_has_required_metadata_tags() {
    let root = repo_root();
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_family_governance.json"))
            .expect("read evidence governance policy"),
    )
    .expect("parse evidence governance policy");

    for family in policy["families"].as_array().expect("families array") {
        let name = family["name"].as_str().unwrap_or("<unknown>");
        for key in [
            "severity_tag",
            "audience_tag",
            "source_of_truth_tag",
            "used_by_release_review",
        ] {
            assert!(
                family.get(key).is_some(),
                "missing `{key}` for evidence family {name}"
            );
        }
    }
}

#[test]
fn evidence_status_report_maps_481_500_requirements() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/evidence_signal_sharpening_481_500_status_report.md"),
    )
    .expect("read evidence sharpening status report");

    for token in [
        "481-489",
        "490-493",
        "494-500",
        "evidence_signal_sharpening_verification.json",
        "20260308-evidence-minimalism.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn evidence_sharpening_suite_contains_expected_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/evidence_signal_sharpening_verification.json"))
            .expect("read evidence sharpening suite"),
    )
    .expect("parse evidence sharpening suite");

    assert_eq!(suite["id"], "evidence-signal-sharpening-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "evidence_signal_sharpening_481_500_contracts",
        "evidence_rationalization_141_160_contracts",
        "evidence_lane_classification_contracts",
        "release_evidence_linkage_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}
