use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
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
fn runtime_adapter_scope_catalog_and_reports_exist() {
    let root = repo_root();
    for rel in [
        "configs/policy/runtime_adapter_surface_catalog.json",
        "docs/reports/foundation/runtime_adapter_surface_inventory.md",
        "docs/reports/foundation/backend_capability_matrix.md",
        "docs/reports/foundation/backend_support_matrix.md",
        "docs/reports/foundation/unsupported_capability_approximations_report.md",
        "docs/reports/foundation/implemented_backend_surfaces_report.md",
        "docs/reports/foundation/simulated_backend_surfaces_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing runtime scope report: {rel}");
    }
}

#[test]
fn every_adapter_surface_has_ownership_category() {
    let root = repo_root();
    let raw = fs::read_to_string(root.join("configs/policy/runtime_adapter_surface_catalog.json"))
        .expect("read runtime adapter scope catalog");
    let payload: Value = serde_json::from_str(&raw).expect("parse runtime adapter scope catalog");
    let surfaces = payload["surfaces"].as_array().expect("surfaces array");
    assert!(!surfaces.is_empty());
    for entry in surfaces {
        let category = entry["category"]
            .as_str()
            .expect("category string for each surface");
        assert!(
            matches!(
                category,
                "implemented" | "experimental" | "modeled" | "docs-only"
            ),
            "unsupported category: {category}"
        );
        assert!(
            !entry["owner"]
                .as_str()
                .expect("owner string")
                .trim()
                .is_empty()
        );
    }
}

#[test]
fn backend_support_report_keeps_mode_separation_explicit() {
    let root = repo_root();
    let support = fs::read_to_string(root.join("docs/reports/foundation/backend_support_matrix.md"))
        .expect("read backend support matrix");
    assert!(support.contains("| local | implemented |"));
    assert!(support.contains("| fake-batch-backend | simulated |"));
    assert!(support.contains("| slurm-backend | aspirational |"));
}
