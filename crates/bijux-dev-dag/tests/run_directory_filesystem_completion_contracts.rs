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
fn run_directory_specs_and_reports_exist() {
    for rel in [
        "docs/spec/RUN_DIRECTORY_FILESYSTEM_GUARANTEES.md",
        "docs/spec/RUN_DIR_CONTRACT.md",
        "docs/spec/RUN_DIR_STORAGE_CONTRACT.md",
        "docs/spec/RUN_HISTORY_CORRUPTION_RECOVERY.md",
        "docs/reports/foundation/run_directory_filesystem_benchmarks.md",
        "docs/reports/foundation/run_directory_filesystem_recovery_benchmarks.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty run-directory surface: {rel}");
    }
}

#[test]
fn run_directory_corpus_and_stress_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/run_directory/regression_corpus.json",
        "configs/suites/run_directory_filesystem_stress.json",
        "evidence/compat/run_dir/v0_1_supported/manifest.json",
        "evidence/compat/run_dir/unsupported_future/manifest.json",
        "evidence/fault/corrupt_runs/invalid_outputs_index.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing run-directory artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/run_directory/regression_corpus.json"))
            .expect("parse run-directory corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected run-directory corpus breadth");
    for coverage in [
        "layout-determinism",
        "path-determinism",
        "metadata-ordering",
        "creation-concurrency",
        "crash-recovery",
        "partial-repair",
        "event-log-corruption",
        "node-metadata-corruption",
        "migration",
        "run-dir-compat",
        "portability",
        "filesystem-permissions",
        "atomic-write",
        "corruption-stress",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing run-directory coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/run_directory_filesystem_stress.json"))
            .expect("parse run-directory suite");
    assert_eq!(suite["id"], "run-directory-filesystem-stress");
}

#[test]
fn app_and_artifact_tests_anchor_run_directory_filesystem_contracts() {
    let app_import_export = read("crates/bijux-dag-app/tests/run_dir_import_export_contract.rs");
    for token in [
        "import_supports_offline_inspection_path_portability_and_line_endings",
        "export_without_artifacts_and_import_verify_only_roundtrip_contract",
    ] {
        assert!(
            app_import_export.contains(token),
            "missing app run-directory token: {token}"
        );
    }

    let app_history = read("crates/bijux-dag-app/tests/run_history_ancestry_contracts.rs");
    for token in [
        "damaged_run_directories_return_errors_without_panics",
        "strict_verify_reports_missing_event_traces_referenced_by_manifest",
    ] {
        assert!(
            app_history.contains(token),
            "missing run-history filesystem resilience token: {token}"
        );
    }

    let artifact_resilience =
        read("crates/bijux-dag-artifacts/tests/artifact_storage_resilience_contracts.rs");
    assert!(
        artifact_resilience.contains("half_valid_run_dir_is_never_reported_as_valid"),
        "missing artifact run-dir verification resilience token"
    );
}
