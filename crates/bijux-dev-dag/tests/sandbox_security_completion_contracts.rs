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
fn sandbox_security_specs_and_reports_exist() {
    for rel in [
        "docs/spec/SANDBOX_SECURITY_MODEL_CONTRACT.md",
        "docs/spec/SECURITY_MODEL.md",
        "docs/spec/CONTAINER_EXECUTION_CONTRACT.md",
        "docs/reports/foundation/sandbox_security_benchmarks.md",
        "docs/reports/foundation/sandbox_security_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty sandbox security surface: {rel}"
        );
    }
}

#[test]
fn sandbox_security_corpus_and_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/sandbox_security/regression_corpus.json",
        "configs/suites/sandbox_security_hardening.json",
        "evidence/battle/workflows/adversarial/path_escape_via_declared_outputs_blocked.json",
        "evidence/battle/workflows/adversarial/env_leakage_via_adapters_blocked.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing sandbox security artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/sandbox_security/regression_corpus.json"))
            .expect("parse sandbox corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected sandbox corpus breadth");
    for coverage in [
        "container-isolation",
        "shell-isolation",
        "remote-isolation",
        "env-leakage",
        "filesystem-boundary",
        "path-traversal",
        "symlink-escape",
        "command-injection",
        "argument-sanitization",
        "artifact-write-boundary",
        "artifact-read-boundary",
        "privilege-restriction",
        "policy-enforcement",
        "adversarial",
        "failure-detection",
        "telemetry",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing sandbox coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read("configs/suites/sandbox_security_hardening.json"))
        .expect("parse sandbox suite");
    assert_eq!(suite["id"], "sandbox-security-hardening");
}

#[test]
fn runtime_and_dev_tests_anchor_sandbox_security_contracts() {
    let runtime_security = read("crates/bijux-dag-runtime/tests/security_model_contracts.rs");
    for token in [
        "clean_env_and_allowlist_contract_is_deterministic",
        "input_and_output_authorization_reject_path_traversal_and_symlink_escape",
    ] {
        assert!(
            runtime_security.contains(token),
            "missing runtime security token: {token}"
        );
    }

    let container = read("crates/bijux-dag-runtime/tests/container_execution_contracts.rs");
    assert!(
        container.contains("container_env_isolation_respects_allowlist_and_denylist"),
        "missing container isolation token"
    );

    let env_completion =
        read("crates/bijux-dev-dag/tests/environment_identity_completion_contracts.rs");
    assert!(
        env_completion.contains("environment_identity_behavior_contracts_are_wired_to_runtime_and_replay_surfaces"),
        "missing environment security completion token"
    );
}
