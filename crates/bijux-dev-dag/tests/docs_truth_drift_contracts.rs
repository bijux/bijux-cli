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

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> serde_json::Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/docs_truth_governance.json"))
            .expect("read docs truth governance policy"),
    )
    .expect("parse docs truth governance policy")
}

#[test]
fn docs_truth_policy_declares_required_rules_and_sources() {
    let policy = policy();
    assert_eq!(
        policy["governance_rules"]["docs_for_speculative_surfaces_must_be_explicit"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["docs_for_shipped_surfaces_must_link_concrete_evidence"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["documentation_truth_drift_is_release_gate"],
        true
    );

    for (_, path) in policy["canonical_sources"]
        .as_object()
        .expect("canonical_sources object")
    {
        let rel = path.as_str().expect("source path");
        assert!(root().join(rel).exists(), "missing canonical source: {rel}");
    }
}

#[test]
fn docs_page_inventory_and_gap_reports_exist() {
    for rel in [
        "docs/reports/foundation/docs_pages_by_owner_and_codepaths_report.md",
        "docs/reports/foundation/docs_pages_without_linked_code_tests_report.md",
        "docs/reports/foundation/code_surfaces_without_linked_docs_report.md",
        "docs/reports/foundation/docs_pages_speculative_modeled_report.md",
        "docs/reports/foundation/docs_pages_stable_shipped_report.md",
        "docs/reports/foundation/docs_oversell_vs_generated_data_report.md",
        "docs/reports/foundation/docs_stale_after_contraction_report.md",
        "docs/reports/foundation/docs_merge_candidates_report.md",
        "docs/reports/foundation/docs_cleanup_app_router_report.md",
        "docs/reports/foundation/docs_cleanup_runtime_platform_report.md",
        "docs/reports/foundation/docs_cleanup_evidence_governance_report.md",
        "docs/reports/foundation/documentation_truth_drift_gate_report.md",
        "docs/adr/20260308-documentation-truth-policy.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing docs-truth artifact: {rel}"
        );
    }
}

#[test]
fn docs_pages_have_owners_links_and_surface_class() {
    let policy = policy();
    for page in policy["docs_pages"].as_array().expect("docs_pages array") {
        let path = page["page"].as_str().expect("docs page path");
        let owner = page["owner"].as_str().expect("owner");
        let class = page["surface_class"].as_str().expect("surface class");
        let linked_code = page["linked_code_paths"]
            .as_array()
            .expect("linked code paths");
        let linked_tests = page["linked_tests"].as_array().expect("linked tests");

        assert!(
            root().join(path).exists(),
            "missing governed docs page: {path}"
        );
        assert!(
            !owner.trim().is_empty(),
            "docs owner must not be empty: {path}"
        );
        assert!(
            ["stable", "speculative"].contains(&class),
            "invalid docs surface class for {path}: {class}"
        );
        assert!(
            !linked_code.is_empty(),
            "linked_code_paths cannot be empty: {path}"
        );
        assert!(
            !linked_tests.is_empty(),
            "linked_tests cannot be empty: {path}"
        );
    }
}

#[test]
fn mission_positioning_support_backend_evidence_benchmark_surfaces_stay_aligned() {
    let mission = fs::read_to_string(root().join("docs/spec/MISSION_STATEMENT.md"))
        .expect("read mission statement");
    let positioning = fs::read_to_string(root().join("docs/reference/POSITIONING_NOTE.md"))
        .expect("read positioning note");
    assert!(
        mission.contains("Git for computation graphs."),
        "mission canonical one-liner drifted"
    );
    assert!(
        positioning.contains("deterministic computation-graph truth engine"),
        "positioning canonical scope wording drifted"
    );

    let support = fs::read_to_string(root().join("docs/reference/SUPPORT_MATRIX.md"))
        .expect("read support matrix");
    let backend_matrix: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            root().join("evidence/reports/backend_capability_matrix_generated.json"),
        )
        .expect("read generated backend capability matrix"),
    )
    .expect("parse backend capability matrix");
    let simulated_backends: BTreeSet<String> = backend_matrix["backends"]
        .as_array()
        .expect("backends")
        .iter()
        .filter(|b| b["status"].as_str() == Some("simulated"))
        .map(|b| b["backend"].as_str().expect("backend").to_string())
        .collect();
    for backend in simulated_backends {
        assert!(
            support.to_lowercase().contains(&backend),
            "support matrix missing generated backend surface: {backend}"
        );
    }

    let evidence_doc = fs::read_to_string(root().join("docs/reference/EVIDENCE_GOVERNANCE.md"))
        .expect("read evidence governance doc");
    assert!(
        evidence_doc.contains("configs/policy/evidence_rationalization_policy.json"),
        "evidence governance doc must cite rationalization policy"
    );

    let benchmark_doc = fs::read_to_string(root().join("docs/reference/RUN_BENCHMARKS.md"))
        .expect("read benchmark run doc");
    assert!(
        benchmark_doc.contains("configs/policy/benchmark_signal_governance.json"),
        "benchmark doc must cite benchmark governance policy"
    );
}

#[test]
fn docs_truth_drift_gate_target_exists_in_makefile() {
    let make_root = fs::read_to_string(root().join("make/root.mk")).expect("read make/root.mk");
    assert!(
        make_root.contains("docs-truth-drift:"),
        "missing docs-truth-drift target"
    );
    assert!(
        make_root.contains("docs_truth_drift_contracts"),
        "docs-truth-drift target must run docs truth drift contracts"
    );
}
