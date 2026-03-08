use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn speculative_surfaces_have_lifecycle_budget_and_owner_rules() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/advanced_semantics_governance.json"))
            .expect("read advanced semantics policy"),
    )
    .expect("parse advanced semantics policy");

    let budget = &policy["speculative_surface_budget"];
    let max = budget["max_speculative_modules"]
        .as_u64()
        .expect("budget max");
    let current = budget["current_speculative_modules"]
        .as_u64()
        .expect("budget current");
    assert!(current <= max, "speculative surface budget exceeded");

    for entry in policy["advanced_semantics_modules"]
        .as_array()
        .expect("advanced semantics array")
    {
        let module = entry["module"].as_str().expect("module");
        let category = entry["category"].as_str().expect("category");
        let owner_repo = entry["owner_repo"].as_str().expect("owner_repo");
        let lifecycle = entry["lifecycle"].as_str().expect("lifecycle");
        assert!(
            !owner_repo.trim().is_empty(),
            "owner_repo missing for {module}"
        );
        if category == "speculative" {
            assert_eq!(
                lifecycle, "expire-or-graduate",
                "speculative module lifecycle rule broken for {module}"
            );
            assert!(
                entry["target_date"].as_str().is_some(),
                "speculative module missing target_date: {module}"
            );
        }
    }
}

#[test]
fn advanced_semantics_fast_lane_smoke_exclusion_is_documented() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/advanced_semantics_fast_lane_exclusion.md"),
    )
    .expect("read advanced semantics fast-lane exclusion report");
    assert!(report.contains("excluded from default fast-lane smoke expectations"));
}

#[test]
fn retained_advanced_semantics_families_have_example_fixtures() {
    let root = repo_root();
    for rel in [
        "crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/kernel_relevant_example.json",
        "crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/runtime_relevant_example.json",
        "crates/bijux-dag-runtime/tests/fixtures/advanced_semantics/adapter_relevant_example.json",
        "docs/reports/foundation/advanced_semantics_retained_examples.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing retained advanced semantics example: {rel}"
        );
    }
}

#[test]
fn advisory_review_report_covers_non_core_advanced_semantics_domains() {
    let root = repo_root();
    let review = fs::read_to_string(
        root.join("docs/reports/foundation/advanced_semantics_advisory_reviews.md"),
    )
    .expect("read advisory reviews report");
    for section in [
        "AI-Assisted Diagnostics",
        "Workflow Product Abstractions",
        "Dataset and Catalog Semantics",
        "Cost-Aware Scheduling",
        "Federated, Geo, and HA Scheduler Contracts",
    ] {
        assert!(
            review.contains(section),
            "missing advanced semantics advisory review section: {section}"
        );
    }
}

#[test]
fn quarantine_and_budget_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/advanced_semantics_quarantine_review.md",
        "docs/reports/foundation/speculative_surface_budget.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing advanced semantics governance report: {rel}"
        );
    }
}
