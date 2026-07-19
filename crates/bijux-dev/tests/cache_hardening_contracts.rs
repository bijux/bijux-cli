use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

#[test]
fn cache_evolution_model_documents_required_sections() {
    let root = workspace_root();
    let model = fs::read_to_string(root.join("docs/spec/CACHE_EVOLUTION_MODEL.md")).expect("model");

    for token in [
        "Intentional cache key inputs",
        "Metadata compatibility",
        "Cache lineage model",
        "Locality decision",
    ] {
        assert!(model.contains(token), "cache evolution model missing token: {token}");
    }
}

#[test]
fn cache_contract_and_prune_policy_track_proof_and_operator_surfaces() {
    let root = workspace_root();
    let contract = fs::read_to_string(root.join("docs/spec/CACHE_CONTRACT.md")).expect("contract");
    let prune =
        fs::read_to_string(root.join("docs/spec/CACHE_PRUNE_POLICY.md")).expect("prune policy");

    for token in [
        "cache_key",
        "cache_metadata_version",
        "execution_contract_fingerprint",
        "evidence/cache/corrupt/missing_outputs_proof.json",
    ] {
        assert!(contract.contains(token), "cache contract missing token: {token}");
    }

    for token in ["prune simulation", "verification", "corrupt entries"] {
        assert!(prune.contains(token), "cache prune policy missing token: {token}");
    }
}

#[test]
fn cache_hardening_report_and_coverage_ledger_link_docs_tests_and_trust_property() {
    let root = workspace_root();
    let report = fs::read_to_string(root.join("docs/reports/foundation/CACHE_HARDENING_REPORT.md"))
        .expect("report");
    let coverage =
        fs::read_to_string(root.join("docs/reports/governance/CACHE_CORRECTNESS_COVERAGE.md"))
            .expect("coverage ledger");

    for token in [
        "docs/spec/CACHE_CONTRACT.md",
        "docs/spec/CACHE_EVOLUTION_MODEL.md",
        "docs/spec/CACHE_PRUNE_POLICY.md",
        "docs/reports/governance/CACHE_CORRECTNESS_COVERAGE.md",
        "crates/bijux-dag-runtime/tests/cache_contracts.rs",
        "crates/bijux-dag-runtime/tests/cache_evolution_contracts.rs",
        "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
        "crates/bijux-dev/tests/cache_hardening_contracts.rs",
        "tp_cache_integrity",
    ] {
        assert!(report.contains(token), "cache hardening report missing token: {token}");
    }

    for token in [
        "cache proof field completeness",
        "cache metadata version acceptance and refusal",
        "warm/cold semantic equivalence",
    ] {
        assert!(coverage.contains(token), "cache coverage ledger missing token: {token}");
    }
}
