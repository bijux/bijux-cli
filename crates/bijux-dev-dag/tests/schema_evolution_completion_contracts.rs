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
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("read file")
}

#[test]
fn schema_evolution_policy_and_registry_surfaces_exist() {
    for rel in [
        "docs/spec/SCHEMA_EVOLUTION_POLICY.md",
        "docs/spec/SCHEMA_EVOLUTION_RULEBOOK.md",
        "docs/spec/SCHEMA_BACKWARD_COMPATIBILITY_GUARANTEES.md",
        "docs/spec/SCHEMA_FORWARD_COMPATIBILITY_LIMITATIONS.md",
        "docs/reference/COMPATIBILITY_MATRIX_GENERATED.md",
        "docs/reports/foundation/schema_changelog.md",
        "docs/reports/foundation/schema_migration_benchmarks.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty schema evolution surface: {rel}");
    }
}

#[test]
fn compatibility_and_migration_fixtures_cover_graph_run_artifact_proof_diff_explain() {
    for rel in [
        "evidence/compat/graph_schema/v0_1_supported/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_future/minimal.dag.json",
        "evidence/compat/run_schema/v0_1_supported/minimal.manifest.json",
        "evidence/compat/run_schema/unsupported_future/minimal.manifest.json",
        "evidence/compat/artifact_schema/v0_1_supported/minimal.outputs.json",
        "evidence/compat/artifact_schema/unsupported_past/minimal.outputs.json",
        "evidence/compat/proof_schema/v0_1_supported/minimal.proof.json",
        "evidence/compat/proof_schema/unsupported_past/minimal.proof.json",
        "evidence/compat/diff_schema/v0_1_supported/minimal.diff.json",
        "evidence/compat/diff_schema/unsupported_future/minimal.diff.json",
        "evidence/compat/explain_schema/v0_1_supported/minimal.explain.json",
        "evidence/compat/explain_schema/unsupported_future/minimal.explain.json",
        "evidence/compat/migrations/graph/v0_1_supported/source.dag.json",
        "evidence/compat/migrations/run/v0_1_supported/source.manifest.json",
        "evidence/compat/migrations/artifact/v0_1_supported/source.outputs.json",
        "evidence/compat/migrations/proof/v0_1_supported/source.proof.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing schema compatibility or migration fixture: {rel}"
        );
    }
}

#[test]
fn diff_and_explain_supported_and_future_versions_are_distinct() {
    let diff_supported: Value = serde_json::from_str(&read(
        "evidence/compat/diff_schema/v0_1_supported/minimal.diff.json",
    ))
    .expect("parse supported diff fixture");
    let diff_future: Value = serde_json::from_str(&read(
        "evidence/compat/diff_schema/unsupported_future/minimal.diff.json",
    ))
    .expect("parse future diff fixture");
    assert_eq!(diff_supported["schema_version"], "run-diff/v0.1");
    assert_ne!(diff_supported["schema_version"], diff_future["schema_version"]);

    let explain_supported: Value = serde_json::from_str(&read(
        "evidence/compat/explain_schema/v0_1_supported/minimal.explain.json",
    ))
    .expect("parse supported explain fixture");
    let explain_future: Value = serde_json::from_str(&read(
        "evidence/compat/explain_schema/unsupported_future/minimal.explain.json",
    ))
    .expect("parse future explain fixture");
    assert_eq!(
        explain_supported["schema_version"],
        "run-explain-failure/v0.1"
    );
    assert_ne!(
        explain_supported["schema_version"],
        explain_future["schema_version"]
    );
}

#[test]
fn schema_regression_suite_declares_required_contract_commands() {
    let suite: Value = serde_json::from_str(&read(
        "configs/suites/schema_compatibility_regression.json",
    ))
    .expect("parse schema suite");
    assert_eq!(suite["id"], "schema-compatibility-regression");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "schema_governance_contracts",
        "proof_schema_compatibility_contracts",
        "schema_evolution_completion_contracts",
    ] {
        assert!(commands.contains(token), "missing schema suite token: {token}");
    }
}
