use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
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
fn backend_adapter_completion_report_and_gates_are_present() {
    let root = repo_root();
    let required = [
        "docs/reports/foundation/backend_adapter_coverage_completion_report.md",
        "docs/reports/foundation/adapter_conformance_coverage_matrix.json",
        "docs/reports/foundation/backend_capability_query_reference.md",
        "docs/reports/foundation/backend_claims_evidence_links.md",
        "configs/suites/backend_conformance_fast.json",
        "crates/bijux-dev-dag/tests/backend_capability_docs_generation_contracts.rs",
        "crates/bijux-dev-dag/tests/backend_conformance_fast_suite_contracts.rs",
    ];

    for rel in required {
        assert!(
            root.join(rel).exists(),
            "missing backend/adapter completion artifact {rel}"
        );
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/backend_adapter_coverage_completion_report.md"),
    )
    .expect("read backend adapter completion report");

    for required in [
        "(321-340)",
        "runtime/src/adapters/adapter.rs",
        "duplicate adapter rejection",
        "adapter_conformance_coverage_matrix.json",
        "backend_capability_docs_generation_contracts.rs",
        "backend_conformance_fast.json",
    ] {
        assert!(
            report.contains(required),
            "backend/adapter completion report missing {required}"
        );
    }
}
