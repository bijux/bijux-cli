use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn command_family_modules_and_binaries_have_direct_tests() {
    let root = repo_root();
    let required = [
        "crates/bijux-dev-dag/src/commands/authoring_evidence.rs",
        "crates/bijux-dev-dag/src/commands/battle_evidence.rs",
        "crates/bijux-dev-dag/src/commands/benchmark_harness.rs",
        "crates/bijux-dev-dag/src/commands/compare_evidence.rs",
        "crates/bijux-dev-dag/src/commands/evidence_access.rs",
        "crates/bijux-dev-dag/src/commands/evidence_control_plane.rs",
        "crates/bijux-dev-dag/src/commands/evidence_registry.rs",
        "crates/bijux-dev-dag/src/commands/model.rs",
        "crates/bijux-dev-dag/src/commands/perf_evidence.rs",
        "crates/bijux-dev-dag/src/commands/suite_catalog.rs",
        "crates/bijux-dev-dag/src/bin/attestation_verify.rs",
        "crates/bijux-dev-dag/src/bin/integrated_verify.rs",
        "crates/bijux-dev-dag/src/bin/migration_simulate.rs",
        "crates/bijux-dev-dag/src/bin/trust_health.rs",
    ];

    for rel in required {
        let src = fs::read_to_string(root.join(rel)).expect("read source");
        assert!(
            src.contains("#[cfg(test)]"),
            "missing #[cfg(test)] in {rel}"
        );
        assert!(src.contains("#[test]"), "missing #[test] in {rel}");
    }
}

#[test]
fn command_router_extracts_file_catalog_helpers() {
    let root = repo_root();
    let mod_src = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read mod source");

    assert!(
        mod_src.contains("mod file_catalog;"),
        "commands/mod.rs must declare file_catalog module"
    );
    for removed in [
        "fn newest_run(",
        "fn two_latest_runs(",
        "fn wildcard_match(",
        "fn collect_all_files(",
        "fn collect_files_with_extension(",
    ] {
        assert!(
            !mod_src.contains(removed),
            "commands/mod.rs still owns extracted helper: {removed}"
        );
    }

    assert!(
        root.join("crates/bijux-dev-dag/src/commands/file_catalog.rs")
            .exists(),
        "extracted file catalog helper module missing"
    );
}

#[test]
fn decomposition_reports_and_adr_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/dev_dag_zero_coverage_report_by_command_family.md",
        "docs/reports/foundation/dev_dag_command_size_report_by_family.md",
        "docs/reports/foundation/dev_dag_command_decomposition_completion_report.md",
        "docs/adr/20260308-dev-dag-command-decomposition-shape.md",
    ] {
        assert!(root.join(rel).exists(), "missing artifact {rel}");
    }
}
