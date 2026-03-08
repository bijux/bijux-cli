use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn advanced_semantics_scope_docs_and_adr_are_present() {
    let root = repo_root();
    for rel in [
        "docs/spec/ADVANCED_SEMANTICS_SCOPE.md",
        "docs/spec/ADVANCED_SEMANTICS_RETAINED_SURFACES.md",
        "docs/spec/ADVANCED_SEMANTICS_QUARANTINED_SURFACES.md",
        "docs/adr/20260308-advanced-semantics-runtime-boundary.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing advanced semantics end-state doc: {rel}"
        );
    }
}

#[test]
fn quarantined_surface_policy_requires_expire_or_graduate() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/advanced_semantics_governance.json"))
            .expect("read governance"),
    )
    .expect("parse governance");

    for entry in policy["advanced_semantics_modules"]
        .as_array()
        .expect("advanced_semantics_modules")
    {
        if entry["category"] == "speculative" {
            assert_eq!(entry["lifecycle"], "expire-or-graduate");
            assert!(entry["target_date"].as_str().is_some());
        }
    }
}
