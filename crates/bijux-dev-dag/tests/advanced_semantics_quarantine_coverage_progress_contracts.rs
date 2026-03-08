use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn advanced_semantics_completion_artifacts_cover_341_360() {
    let root = repo_root();
    let required = [
        "docs/reports/foundation/advanced_semantics_quarantine_completion_report.md",
        "docs/reports/foundation/runtime_internal_surface_inventory_report.md",
        "docs/reports/foundation/runtime_stable_vs_experimental_surface_page.md",
        "docs/reports/foundation/advanced_semantics_no_direct_tests_report.md",
        "docs/reports/foundation/advanced_semantics_no_user_path_report.md",
        "docs/reports/foundation/advanced_semantics_no_examples_report.md",
        "docs/reports/foundation/speculative_surface_budget.md",
        "docs/adr/ADR-advanced-semantics-end-state.md",
    ];

    for rel in required {
        assert!(
            root.join(rel).exists(),
            "missing advanced semantics completion artifact {rel}"
        );
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/advanced_semantics_quarantine_completion_report.md"),
    )
    .expect("read advanced semantics completion report");

    for required in [
        "(341-360)",
        "runtime_internal_surface_inventory_report.md",
        "advanced_semantics_no_direct_tests_report.md",
        "speculative_surface_budget.md",
        "ADR-advanced-semantics-end-state.md",
        "runtime_stable_vs_experimental_surface_page.md",
    ] {
        assert!(
            report.contains(required),
            "advanced semantics completion report missing {required}"
        );
    }
}
