use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn vocabulary_421_440_governance_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "configs/policy/vocabulary_registry.json",
        "docs/spec/VOCABULARY_SCOPE_HONESTY_POLICY.md",
        "docs/reports/foundation/platform_scope_module_inventory_report.md",
        "docs/reports/foundation/platform_scope_name_overstatement_report.md",
        "docs/reports/foundation/stale_overreach_names_in_docs_report.md",
        "docs/reports/foundation/stale_overreach_names_in_tests_report.md",
        "docs/reports/foundation/stale_overreach_names_in_examples_report.md",
        "docs/reports/foundation/stale_overreach_names_in_evidence_report.md",
        "docs/reports/foundation/repo_vocabulary_registry_report.md",
        "docs/reports/foundation/repo_vocabulary_drift_report.md",
        "docs/reports/foundation/vocabulary_scope_honesty_421_440_status_report.md",
        "docs/reference/CANONICAL_TERM_GLOSSARY.md",
        "configs/suites/terminology_consistency_verification.json",
        "docs/adr/20260308-vocabulary-and-scope-honesty.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing vocabulary artifact: {rel}"
        );
    }
}

#[test]
fn vocabulary_registry_has_canonical_and_deprecated_mappings() {
    let root = repo_root();
    let payload = fs::read_to_string(root.join("configs/policy/vocabulary_registry.json"))
        .expect("read vocabulary registry");
    let registry: Value = serde_json::from_str(&payload).expect("parse vocabulary registry");

    assert_eq!(registry["format"], "vocabulary-registry/v1");

    let canonical = registry["canonical_terms"]
        .as_array()
        .expect("canonical_terms array");
    assert!(
        canonical.len() >= 6,
        "canonical vocabulary set should be non-trivial"
    );

    let deprecated = registry["deprecated_terms"]
        .as_array()
        .expect("deprecated_terms array");
    assert!(
        !deprecated.is_empty(),
        "deprecated term map must not be empty"
    );
    for entry in deprecated {
        let term = entry["term"].as_str().unwrap_or_default();
        let replacement = entry["replacement"].as_str().unwrap_or_default();
        assert!(!term.trim().is_empty(), "deprecated term is empty");
        assert!(
            !replacement.trim().is_empty(),
            "replacement missing for deprecated term: {term}"
        );
    }

    let aliases = registry["aliases"].as_array().expect("aliases array");
    assert!(!aliases.is_empty(), "alias map should not be empty");
}

#[test]
fn status_report_maps_421_440_requirements() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/vocabulary_scope_honesty_421_440_status_report.md"),
    )
    .expect("read vocabulary scope status report");
    for token in [
        "421-426",
        "427-434",
        "435-439",
        "440",
        "vocabulary_registry.json",
        "terminology_consistency_verification.json",
        "20260308-vocabulary-and-scope-honesty.md",
    ] {
        assert!(
            report.contains(token),
            "vocabulary scope status report missing token: {token}"
        );
    }
}

#[test]
fn help_surface_contract_keeps_overreach_terms_forbidden() {
    let root = repo_root();
    let help_contract =
        fs::read_to_string(root.join("crates/bijux-dag-app/tests/help_surface_contracts.rs"))
            .expect("read help surface contracts");
    for forbidden in [
        "control-plane api",
        "federated scheduler",
        "geo federation",
        "tenant control plane",
    ] {
        assert!(
            help_contract.contains(forbidden),
            "help surface forbidden term guard missing token: {forbidden}"
        );
    }
}
