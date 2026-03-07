use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn foundation_hardening_registry_and_reports_exist() {
    let root = repo_root();
    for required in [
        "configs/suites/foundation_hardening.json",
        "docs/reports/foundation/release_evidence_report.md",
        "docs/reports/foundation/repository_proof_statement.md",
        "docs/reports/foundation/foundation_final_report.md",
    ] {
        assert!(
            root.join(required).exists(),
            "missing foundation hardening surface: {required}"
        );
    }
}

#[test]
fn foundation_hardening_registry_references_known_suite_ids() {
    let root = repo_root();
    let payload = fs::read_to_string(root.join("configs/suites/foundation_hardening.json"))
        .expect("foundation hardening suite file should exist");
    let registry: serde_json::Value =
        serde_json::from_str(&payload).expect("suite registry should parse");
    let ids = registry
        .get("suite_ids")
        .and_then(serde_json::Value::as_array)
        .expect("suite_ids should exist");
    assert!(!ids.is_empty(), "suite_ids should not be empty");

    let source = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read command source");
    for id in ids {
        let id = id.as_str().expect("suite id should be string");
        assert!(
            source.contains(&format!("id: \"{id}\"")),
            "foundation hardening suite id must map to a defined suite: {id}"
        );
    }
}

#[test]
fn release_evidence_and_proof_statement_cover_required_claims() {
    let root = repo_root();
    let release_report =
        fs::read_to_string(root.join("docs/reports/foundation/release_evidence_report.md"))
            .expect("release evidence report should exist");
    for token in [
        "battle-suite-mandatory",
        "replay-contract",
        "cache-evolution",
        "artifact-hardening",
        "config-policy-determinism",
        "raw test totals are insufficient",
    ] {
        assert!(
            release_report.contains(token),
            "release evidence report missing required token `{token}`"
        );
    }

    let proof = fs::read_to_string(root.join("docs/reports/foundation/repository_proof_statement.md"))
        .expect("repository proof statement should exist");
    for token in [
        "What this repository can prove today",
        "scheduler readiness and determinism invariants",
        "release readiness requires evidence surfaces",
        "foundation-hardening suites",
    ] {
        assert!(
            proof.contains(token),
            "repository proof statement missing required token `{token}`"
        );
    }
}
