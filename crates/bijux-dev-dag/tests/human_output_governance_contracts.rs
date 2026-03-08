use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/human_output_governance.json"))
            .expect("read human governance policy"),
    )
    .expect("parse human governance policy")
}

#[test]
fn human_output_inventory_and_gap_reports_are_generated_and_fresh() {
    for rel in [
        "docs/reports/foundation/human_output_snapshot_inventory_report.md",
        "docs/reports/foundation/human_output_surfaces_without_snapshot_report.md",
        "docs/reports/foundation/human_output_without_snapshot_tests_report.md",
        "docs/reports/foundation/wording_drift_equivalent_commands_report.md",
        "docs/reports/foundation/concise_detailed_human_output_coverage_report.md",
        "docs/reference/OPERATOR_UX_REFERENCE_GENERATED.md",
    ] {
        assert!(root().join(rel).exists(), "missing generated human output artifact: {rel}");
    }

    let gap = fs::read_to_string(root().join("docs/reports/foundation/human_output_surfaces_without_snapshot_report.md"))
        .expect("read human output gap report");
    assert!(gap.contains("Missing human snapshot surfaces: 0"));
}

#[test]
fn concise_vs_detailed_examples_exist_for_all_governed_families() {
    let gov = policy();
    for family in gov["families"].as_array().expect("families") {
        let name = family["family"].as_str().expect("family name");
        let concise = root().join(format!("evidence/operator/examples/human_output/{name}/concise.txt"));
        let detailed = root().join(format!("evidence/operator/examples/human_output/{name}/detailed.txt"));
        assert!(concise.exists(), "missing concise example for {name}");
        assert!(detailed.exists(), "missing detailed example for {name}");
    }
}

#[test]
fn human_output_governance_rule_requires_concise_and_detailed_examples() {
    let gov = policy();
    let rule = gov["governance_rule"].as_str().expect("governance rule");
    assert!(rule.contains("concise and detailed"));
    assert!(rule.contains("snapshot"));
}

#[test]
fn human_output_uses_canonical_terminology_without_forbidden_default_terms() {
    let gov = policy();
    let forbidden = gov["forbidden_default_terms"].as_array().expect("forbidden terms");
    let snapshots = [
        "crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt",
        "crates/bijux-dag-app/tests/snapshots/route_detailed_wording.txt",
        "crates/bijux-dag-app/tests/snapshots/inspect_human_output.txt",
        "crates/bijux-dag-app/tests/snapshots/history_human_output.txt",
    ];

    for rel in snapshots {
        let body = fs::read_to_string(root().join(rel)).expect("read snapshot");
        for term in forbidden {
            let t = term.as_str().expect("forbidden term");
            assert!(
                !body.to_lowercase().contains(&t.to_lowercase()),
                "snapshot {rel} contains forbidden default term `{t}`"
            );
        }
    }
}

#[test]
fn human_output_section_ordering_is_stable_for_route_wording() {
    let concise = fs::read_to_string(root().join("crates/bijux-dag-app/tests/snapshots/route_concise_wording.txt"))
        .expect("read concise route snapshot");
    let order = ["run_id", "status", "origin", "integrity_state", "retry_count", "cache_hits", "artifact_count"];
    let mut cursor = 0usize;
    for token in order {
        let pos = concise[cursor..]
            .find(token)
            .unwrap_or_else(|| panic!("missing ordered token {token} in concise route snapshot"));
        cursor += pos;
    }
}

#[test]
fn degraded_data_contract_tests_remain_present_for_human_paths() {
    let source = fs::read_to_string(root().join("crates/bijux-dag-app/tests/operator_malformed_input_no_panic_contracts.rs"))
        .expect("read malformed input contract");
    for token in [
        "malformed",
        "without_panicking",
        "panicked on malformed",
    ] {
        assert!(source.contains(token), "missing degraded-data token {token}");
    }
}
