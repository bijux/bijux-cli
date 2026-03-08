use bijux_dag_testkit as _;
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

#[test]
fn top_level_crate_readmes_include_scope_contract_sections() {
    let roots = [
        "../../crates/bijux-dag-core/README.md",
        "../../crates/bijux-dag-runtime/README.md",
        "../../crates/bijux-dag-app/README.md",
        "../../crates/bijux-dag-artifacts/README.md",
        "../../crates/bijux-dag-cli/README.md",
        "../../crates/bijux-dev-dag/README.md",
        "../../crates/bijux-dag-testkit/README.md",
    ];

    for readme in roots {
        let path = Path::new(readme);
        assert!(path.exists(), "missing README: {readme}");
        let body = fs::read_to_string(path).expect("read README");
        assert!(
            body.contains("## Why this crate exists"),
            "README missing why section: {readme}"
        );
        assert!(
            body.contains("## What must never enter this crate"),
            "README missing must-never section: {readme}"
        );
    }
}

#[test]
fn governance_taxonomy_docs_exist() {
    let docs = [
        "../../docs/spec/CRATE_RESPONSIBILITY_ALIGNMENT.md",
        "../../docs/spec/CRATE_OWNERSHIP_MATRIX.md",
        "../../docs/adr/20260308-runtime-contraction-target-architecture.md",
        "../../docs/adr/20260308-dev-dag-governance-scope.md",
        "../../docs/adr/20260308-authoritative-schema-residency.md",
    ];

    for doc in docs {
        assert!(
            Path::new(doc).exists(),
            "required governance doc missing: {doc}"
        );
    }
}
