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

#[test]
fn backend_capability_pages_remain_generated_from_command_aligned_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    let capability_reference = fs::read_to_string(
        root.join("docs/reports/foundation/backend_capability_query_reference.md"),
    )
    .expect("capability reference");
    assert!(capability_reference.contains("generated_from:"));
    assert!(capability_reference.contains("bijux dag capabilities --backend local --json"));
    assert!(capability_reference.contains("bijux dag capabilities --backend kubernetes --json"));
    assert!(capability_reference.contains("bijux dag capabilities --backend hpc --json"));
    assert!(capability_reference.contains("bijux dag capabilities --backend remote --json"));

    let claims =
        fs::read_to_string(root.join("docs/reports/foundation/backend_claims_evidence_links.md"))
            .expect("claims");
    assert!(claims.contains("generated_from:"));
    assert!(claims.contains("adapter_conformance_coverage_matrix.json"));
}
