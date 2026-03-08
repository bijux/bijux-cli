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
fn engineering_simplification_artifacts_exist() {
    let root = repo_root();
    let required = [
        "docs/reports/foundation/awkward_command_families_simplification_report.md",
        "docs/reports/foundation/awkward_runtime_surfaces_contraction_report.md",
        "docs/reports/foundation/awkward_evidence_families_rationalization_report.md",
        "docs/reports/foundation/awkward_benchmark_families_rationalization_report.md",
        "docs/reports/foundation/awkward_fixture_families_cleanup_report.md",
        "docs/reports/foundation/awkward_docs_clusters_cleanup_targets_report.md",
        "docs/reports/foundation/top_20_product_awkwardness_sources_report.md",
        "docs/reports/foundation/top_20_repo_awkwardness_sources_report.md",
        "docs/reports/foundation/top_20_operator_confusion_sources_report.md",
        "docs/reports/foundation/top_20_maintainer_friction_sources_report.md",
        "docs/reports/foundation/high_impact_shortlist_1_600_report.md",
        "docs/reports/foundation/low_risk_high_signal_shortlist_1_600_report.md",
        "docs/reports/foundation/reference_grade_correctness_shortlist_1_600_report.md",
        "docs/reports/foundation/deawkwarding_only_shortlist_1_600_report.md",
        "docs/reports/foundation/runtime_contraction_only_shortlist_1_600_report.md",
        "docs/reports/foundation/operator_surface_sharpening_only_shortlist_1_600_report.md",
        "docs/reports/foundation/workstream_dependency_map_1_600_report.md",
        "docs/reports/foundation/execution_board_1_600_report.md",
        "docs/reports/foundation/post_600_remaining_work_report.md",
        "docs/reports/foundation/engineering_simplification_581_600_status_report.md",
        "configs/suites/engineering_simplification_verification.json",
        "docs/adr/20260308-product-surface-end-state-before-next-expansion.md",
    ];

    for path in required {
        assert!(root.join(path).exists(), "missing required artifact: {path}");
    }
}

#[test]
fn status_report_covers_581_through_600() {
    let root = repo_root();
    let content = fs::read_to_string(
        root.join("docs/reports/foundation/engineering_simplification_581_600_status_report.md"),
    )
    .expect("status report must be readable");

    for marker in [
        "581-586",
        "587-590",
        "591-596",
        "597",
        "598",
        "599",
        "600",
    ] {
        assert!(
            content.contains(marker),
            "status report missing marker: {marker}"
        );
    }
}

#[test]
fn verification_suite_includes_contract() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/engineering_simplification_verification.json"))
            .expect("suite file must be readable"),
    )
    .expect("suite json must parse");
    let suite_text = suite.to_string();

    for expected in [
        "runtime_scope_contraction_401_420_contracts",
        "vocabulary_scope_honesty_421_440_contracts",
        "operator_surface_441_460_contracts",
        "dev_dag_contraction_461_480_contracts",
        "evidence_signal_sharpening_481_500_contracts",
        "benchmark_minimalism_501_520_contracts",
        "fixture_contraction_521_540_contracts",
        "repo_tree_simplification_541_560_contracts",
        "internal_contract_discipline_561_580_contracts",
        "final_engineering_simplification_581_600_contracts",
    ] {
        assert!(suite_text.contains(expected), "suite missing: {expected}");
    }
}

#[test]
fn adr_defines_pre_expansion_end_state() {
    let root = repo_root();
    let adr = fs::read_to_string(
        root.join("docs/adr/20260308-product-surface-end-state-before-next-expansion.md"),
    )
    .expect("ADR must be readable");

    for phrase in [
        "Before any next expansion",
        "deterministic local semantics",
        "canonical, concise by default",
        "speculative and experimental scopes remain quarantined",
        "direct tests",
    ] {
        assert!(adr.contains(phrase), "ADR missing phrase: {phrase}");
    }
}
