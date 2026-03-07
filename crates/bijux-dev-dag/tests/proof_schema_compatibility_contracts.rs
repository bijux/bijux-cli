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
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn proof_schema_compatibility_fixtures_exist() {
    let root = repo_root();
    for rel in [
        "evidence/compat/proof_bundle/v0_1_supported/proof.json",
        "evidence/compat/proof_bundle/unsupported_past/proof.json",
        "docs/spec/PROOF_BUNDLE_SCHEMA_v0.1.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing proof schema fixture: {rel}"
        );
    }
}

#[test]
fn supported_and_unsupported_proof_schema_versions_are_distinct() {
    let root = repo_root();
    let supported: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compat/proof_bundle/v0_1_supported/proof.json"))
            .expect("read supported proof fixture"),
    )
    .expect("parse supported proof fixture");
    let unsupported: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compat/proof_bundle/unsupported_past/proof.json"))
            .expect("read unsupported proof fixture"),
    )
    .expect("parse unsupported proof fixture");

    assert_eq!(supported["schema_version"], "proof-bundle/v0.1");
    assert_ne!(supported["schema_version"], unsupported["schema_version"]);
}
