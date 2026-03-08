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
fn bundle_specs_roundtrip_and_integrity_contracts_are_linked() {
    for rel in [
        "docs/spec/GRAPH_BUNDLE_FORMAT_v1.md",
        "docs/spec/RUN_BUNDLE_FORMAT_v1.md",
        "docs/spec/ARTIFACT_BUNDLE_FORMAT_v1.md",
    ] {
        assert!(repo_root().join(rel).exists(), "missing bundle spec: {rel}");
    }

    let import_export_tests = read("crates/bijux-dag-app/tests/run_dir_import_export_contract.rs");
    for test_name in [
        "graph_snapshot_only_bundle_roundtrip_is_stable",
        "export_without_artifacts_and_import_verify_only_roundtrip_contract",
        "artifact_heavy_bundle_roundtrip_verify_only_is_stable",
        "imported_run_replay_and_diff_against_original_are_stable",
        "import_rejects_corrupted_file_payload_before_acceptance",
        "import_rejects_truncated_bundle_with_clear_failure",
        "import_rejects_unsupported_bundle_version_fixture",
        "import_supports_offline_inspection_path_portability_and_line_endings",
    ] {
        assert!(
            import_export_tests.contains(test_name),
            "missing bundle integrity contract test: {test_name}"
        );
    }
}

#[test]
fn imported_bundle_identity_lineage_proof_and_portability_surfaces_exist() {
    let replay_lineage = read("crates/bijux-dag-app/tests/replay_lineage_planning_contract.rs");
    assert!(
        replay_lineage.contains("replay_accepts_imported_run_as_source"),
        "missing imported-run replay lineage test"
    );

    let replay_proof = read("crates/bijux-dag-app/tests/replay_proof_contract.rs");
    assert!(
        replay_proof.contains("replay_prove_reports_strict_equivalent_on_exact_pair")
            && replay_proof.contains("replay_prove_reports_diverged_on_corrupt_source_pair"),
        "missing replay proof preservation contract tests"
    );

    for rel in [
        "docs/reports/foundation/portability_scorecard.md",
        "docs/reports/foundation/bundle_import_fidelity_explain.md",
        "docs/reports/foundation/import_export_git_mapping.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing portability evidence report: {rel}"
        );
    }
}

#[test]
fn bundle_benchmark_and_regression_artifacts_are_present() {
    for rel in [
        "docs/reports/foundation/bundle_export_import_benchmarks.md",
        "docs/reports/foundation/bundle_export_import_latency_report.md",
        "docs/reports/foundation/bundle_import_export_verify_fsck_benchmarks.md",
        "evidence/cache/import_export/bundle_regression_corpus.json",
        "configs/suites/bundle_portability_integrity_regression.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing bundle benchmark or regression artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/import_export/bundle_regression_corpus.json",
    ))
    .expect("parse bundle regression corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(
        corpus["cases"].as_array().expect("cases").len() >= 6,
        "bundle regression corpus must hold at least six cases"
    );

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/bundle_portability_integrity_regression.json",
    ))
    .expect("parse suite");
    assert_eq!(suite["id"], "bundle-portability-integrity-regression");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "run_dir_import_export_contract",
        "replay_lineage_planning_contract",
        "replay_proof_contract",
        "bundle_portability_completion_contracts",
    ] {
        assert!(commands.contains(token), "missing suite command token: {token}");
    }
}
