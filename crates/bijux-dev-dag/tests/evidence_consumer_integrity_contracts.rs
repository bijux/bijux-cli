use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
}

#[test]
fn crate_tests_do_not_own_scenario_json_outside_allowlist() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files);

    let allowed_prefixes = [
        "crates/bijux-dag-core/tests/fixtures/",
        "crates/bijux-dag-core/tests/compat/",
        "crates/bijux-dag-core/tests/snapshots/",
        "crates/bijux-dag-runtime/tests/fixtures/",
        "crates/bijux-dag-artifacts/tests/fixtures/",
        "crates/bijux-dag-runtime/tests/bin/",
        "crates/bijux-dag-app/tests/snapshots/",
    ];

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.contains("/tests/") {
            continue;
        }
        if !(rel.ends_with(".json") || rel.ends_with(".dag.json") || rel.ends_with(".sh")) {
            continue;
        }
        let allowed = allowed_prefixes
            .iter()
            .any(|prefix| rel.starts_with(prefix));
        assert!(
            allowed,
            "crate test scenario/helper file is outside explicit allowlist: {}",
            rel
        );
    }
}

#[test]
fn example_dag_assets_exist_only_in_evidence_authoring_examples() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root, &mut files);
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with(".dag.json") {
            continue;
        }
        if rel.contains("/examples/") {
            assert!(
                rel.starts_with("evidence/authoring/examples/"),
                "example DAG exists outside evidence/authoring/examples: {}",
                rel
            );
        }
    }
}

#[test]
fn legacy_comparison_and_benchmark_scenario_roots_are_empty() {
    let root = repo_root();
    for rel in ["comparisons/scenarios", "benchmarks/scenarios"] {
        let dir = root.join(rel);
        if !dir.exists() {
            continue;
        }
        let mut files = Vec::new();
        collect_files(&dir, &mut files);
        let scenario_files: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let rel = path
                    .strip_prefix(&root)
                    .ok()?
                    .to_string_lossy()
                    .replace('\\', "/");
                if rel.ends_with(".json") || rel.ends_with(".dag.json") {
                    Some(rel)
                } else {
                    None
                }
            })
            .collect();
        assert!(
            scenario_files.is_empty(),
            "legacy scenario root still contains scenario assets: {}",
            scenario_files.join(", ")
        );
    }
}

#[test]
fn top_level_tests_do_not_keep_migrated_scenario_assets() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("tests"), &mut files);

    let allowed = BTreeSet::from([
        "tests/e2e/matrix.json".to_string(),
        "tests/integration_fixtures/minimal_consumer/dag.json".to_string(),
    ]);

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if !(rel.ends_with(".json") || rel.ends_with(".dag.json")) {
            continue;
        }
        assert!(
            allowed.contains(&rel),
            "top-level tests contains migrated scenario asset: {}",
            rel
        );
    }
}

#[test]
fn battle_perf_and_compare_consumers_reference_evidence_roots_only() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files);

    let forbidden_refs = [
        "tests/e2e/replay/fixtures/",
        "tests/e2e/fixtures/e2e_minimal.json",
        "benchmarks/scenarios/",
        "comparisons/scenarios/",
    ];

    let mut violations = Vec::new();
    let ignore_paths = [
        "crates/bijux-dev-dag/src/commands/mod.rs",
        "crates/bijux-dev-dag/tests/evidence_consumer_integrity_contracts.rs",
        "crates/bijux-dev-dag/tests/evidence_access_contracts.rs",
    ];

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if ignore_paths.iter().any(|ignore| rel == *ignore) {
            continue;
        }
        if !(rel.ends_with(".rs") || rel.ends_with(".md") || rel.ends_with(".json")) {
            continue;
        }
        let text = fs::read_to_string(&file).expect("read file");
        for forbidden in forbidden_refs {
            if text.contains(forbidden) {
                violations.push(format!("{} -> {}", rel, forbidden));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "consumer surfaces still reference legacy roots: {}",
        violations.join(" | ")
    );
}

#[test]
fn evidence_consumer_report_exists_and_lists_key_surfaces() {
    let root = repo_root();
    let report = root.join("evidence/reports/test_evidence_consumers.md");
    assert!(report.exists(), "missing evidence consumer mapping report");
    let text = fs::read_to_string(report).expect("read report");
    for required in [
        "crates/bijux-dag-core/tests/examples_contract.rs",
        "crates/bijux-dag-app/tests/comparison_harness_contract.rs",
        "crates/bijux-dag-app/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/infrastructure_fixture_contract.rs",
    ] {
        assert!(
            text.contains(required),
            "consumer report missing required surface: {}",
            required
        );
    }
}
