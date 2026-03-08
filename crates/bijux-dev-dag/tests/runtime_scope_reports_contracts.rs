use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn runtime_scope_reports_and_dependency_contract_docs_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/kernel_owned_modules_report.md",
        "docs/reports/foundation/runtime_non_kernel_modules_report.md",
        "docs/reports/foundation/runtime_contract_backing_report.md",
        "docs/reports/foundation/runtime_operator_surface_report.md",
        "docs/reports/foundation/core_public_api_shrink_report.md",
        "docs/reports/foundation/runtime_public_api_shrink_report.md",
        "docs/reports/foundation/kernel_contraction_objectives_report.md",
        "docs/spec/KERNEL_ALLOWED_DEPENDENCIES.md",
        "docs/spec/RUNTIME_ALLOWED_DEPENDENCIES.md",
        "docs/spec/DEV_GOVERNANCE_ALLOWED_DEPENDENCIES.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing runtime scope governance artifact: {rel}"
        );
    }
}

#[test]
fn runtime_scope_reports_have_expected_sections() {
    let root = repo_root();
    let kernel =
        fs::read_to_string(root.join("docs/reports/foundation/kernel_owned_modules_report.md"))
            .expect("read kernel report");
    assert!(kernel.contains("Runtime kernel-owned module set"));

    let non_kernel = fs::read_to_string(
        root.join("docs/reports/foundation/runtime_non_kernel_modules_report.md"),
    )
    .expect("read non-kernel report");
    assert!(non_kernel.contains("Runtime modules outside kernel ownership"));

    let contract_backing =
        fs::read_to_string(root.join("docs/reports/foundation/runtime_contract_backing_report.md"))
            .expect("read contract backing report");
    assert!(contract_backing.contains("Contract-backed modules"));
    assert!(contract_backing.contains("Documented-only modules"));

    let operator =
        fs::read_to_string(root.join("docs/reports/foundation/runtime_operator_surface_report.md"))
            .expect("read operator surface report");
    assert!(operator.contains("Operator-facing modules"));
    assert!(operator.contains("Internal-only modules"));
}
