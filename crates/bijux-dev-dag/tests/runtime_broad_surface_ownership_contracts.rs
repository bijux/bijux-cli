use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tempfile as _;

#[derive(Debug, serde::Deserialize)]
struct OwnershipPolicy {
    entries: Vec<OwnershipEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct OwnershipEntry {
    module: String,
    surface_status: String,
    ownership_category: String,
    owner_repo: String,
    decision: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn every_broad_runtime_module_has_ownership_classification() {
    let root = repo_root();
    let raw = std::fs::read_to_string(root.join("configs/policy/runtime_broad_surface_ownership.json"))
        .expect("read ownership policy");
    let policy: OwnershipPolicy = serde_json::from_str(&raw).expect("parse ownership policy");

    let mut seen = BTreeSet::new();
    for entry in policy.entries {
        assert!(seen.insert(entry.module.clone()), "duplicate module policy entry");
        assert!(
            matches!(
                entry.surface_status.as_str(),
                "implemented" | "experimental" | "modeled" | "docs-only"
            ),
            "invalid surface status for {}",
            entry.module
        );
        assert!(
            !entry.ownership_category.trim().is_empty(),
            "missing ownership_category for {}",
            entry.module
        );
        assert!(!entry.owner_repo.trim().is_empty(), "missing owner_repo for {}", entry.module);
        assert!(
            matches!(entry.decision.as_str(), "keep" | "quarantine" | "delete"),
            "invalid decision for {}",
            entry.module
        );
        let module_path = root.join("crates/bijux-dag-runtime/src").join(&entry.module);
        assert!(module_path.exists(), "missing runtime module: {}", module_path.display());
    }
}

#[test]
fn generated_runtime_surface_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/runtime_broad_surface_inventory.md",
        "docs/reports/foundation/runtime_broad_surface_zero_coverage_report.md",
        "docs/reports/foundation/runtime_experimental_surfaces.md",
        "docs/reports/foundation/runtime_stable_surfaces.md",
        "docs/reports/foundation/runtime_modeled_only_surfaces.md",
        "docs/reports/foundation/runtime_quarantined_owner_repo_map.md",
        "docs/reports/foundation/runtime_keep_quarantine_delete_review.md",
        "docs/architecture/runtime_quarantine_rationale.md",
    ] {
        assert!(root.join(rel).exists(), "missing runtime surface report: {rel}");
    }
}
