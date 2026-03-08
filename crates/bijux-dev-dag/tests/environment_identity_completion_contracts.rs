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
fn environment_identity_spec_and_security_contract_exist() {
    for rel in [
        "docs/spec/ENVIRONMENT_IDENTITY_CONTRACT.md",
        "docs/spec/SECURITY_MODEL.md",
        "docs/spec/RUNTIME_SEMANTICS_CONTRACT.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing environment identity contract surface: {rel}"
        );
    }
}

#[test]
fn environment_identity_behavior_contracts_are_wired_to_runtime_and_replay_surfaces() {
    let security_contracts = read("crates/bijux-dag-runtime/tests/security_model_contracts.rs");
    for token in [
        "clean_env_and_allowlist_contract_is_deterministic",
        "env_pattern_matching_contract_works_for_exact_and_prefix",
    ] {
        assert!(
            security_contracts.contains(token),
            "missing runtime environment contract: {token}"
        );
    }

    let container_contracts =
        read("crates/bijux-dag-runtime/tests/container_execution_contracts.rs");
    assert!(
        container_contracts.contains("container_env_isolation_respects_allowlist_and_denylist"),
        "missing container environment shaping contract"
    );

    let replay_diff = read("crates/bijux-dag-app/src/replay/diff.rs");
    for token in [
        "replay_diff_reports_environment_drift_as_manifest_drift",
        "replay_diff_reports_backend_capability_mismatch_as_manifest_drift",
    ] {
        assert!(
            replay_diff.contains(token),
            "missing replay environment/backed drift contract: {token}"
        );
    }

    let import_export = read("crates/bijux-dag-app/tests/run_dir_import_export_contract.rs");
    assert!(
        import_export
            .contains("import_supports_offline_inspection_path_portability_and_line_endings"),
        "missing imported-run environment portability contract"
    );
}

#[test]
fn regression_corpus_suite_and_benchmark_reports_exist_and_are_parseable() {
    for rel in [
        "evidence/cache/environment/regression_corpus.json",
        "configs/suites/environment_identity_hermeticity_regression.json",
        "docs/reports/foundation/environment_drift_benchmarks.md",
        "docs/reports/foundation/k8s_replay_env_drift_report.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing environment identity governance artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/environment/regression_corpus.json"))
            .expect("parse environment regression corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(
        corpus["cases"].as_array().expect("cases").len() >= 6,
        "expected environment regression corpus breadth"
    );

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/environment_identity_hermeticity_regression.json",
    ))
    .expect("parse environment suite");
    assert_eq!(suite["id"], "environment-identity-hermeticity-regression");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "security_model_contracts",
        "container_execution_contracts",
        "run_dir_import_export_contract",
        "environment_identity_completion_contracts",
    ] {
        assert!(
            commands.contains(token),
            "missing suite command token: {token}"
        );
    }
}
