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
                let name = path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                if matches!(name, "target" | "artifacts" | ".git") {
                    continue;
                }
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
        "crates/bijux-dag-app/tests/fixtures/",
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

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if !(rel.ends_with(".json") || rel.ends_with(".dag.json")) {
            continue;
        }
        panic!("top-level tests contains canonical scenario asset: {}", rel);
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

#[test]
fn evidence_consumption_by_crate_report_exists_and_covers_core_crates() {
    let root = repo_root();
    let report = root.join("evidence/reports/evidence_consumption_by_crate.md");
    assert!(
        report.exists(),
        "missing crate-level evidence consumption report"
    );
    let text = fs::read_to_string(report).expect("read report");
    for required in [
        "bijux-dag-core",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-artifacts",
        "bijux-dev-dag",
    ] {
        assert!(
            text.contains(required),
            "crate-level evidence report missing crate: {required}"
        );
    }
}

#[test]
fn evidence_access_helper_contract_doc_exists() {
    let root = repo_root();
    let doc = root.join("docs/spec/TESTKIT_EVIDENCE_ACCESS_CONTRACT.md");
    assert!(doc.exists(), "missing testkit evidence access contract doc");
    let text = fs::read_to_string(doc).expect("read contract doc");
    for token in [
        "load_evidence_registry_checked",
        "resolve_evidence_asset_by_id_checked",
        "evidence_asset_ids",
        "read-only",
    ] {
        assert!(
            text.contains(token),
            "testkit evidence access contract doc missing token: {token}"
        );
    }
}

#[test]
fn test_surfaces_do_not_mutate_evidence_assets() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_files(&root.join("crates"), &mut files);

    let mut violations = Vec::new();
    let forbidden_inline_patterns = [
        "fs::write(\"evidence/",
        "fs::write(root.join(\"evidence/",
        "fs::write(repo_root().join(\"evidence/",
        "fs::remove_file(\"evidence/",
        "fs::remove_file(root.join(\"evidence/",
        "fs::remove_file(repo_root().join(\"evidence/",
        "fs::rename(\"evidence/",
        "fs::rename(root.join(\"evidence/",
        "fs::rename(repo_root().join(\"evidence/",
        "fs::create_dir_all(\"evidence/",
        "fs::create_dir_all(root.join(\"evidence/",
        "fs::create_dir_all(repo_root().join(\"evidence/",
    ];

    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("strip")
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.contains("/tests/") || !rel.ends_with(".rs") {
            continue;
        }
        let text = fs::read_to_string(&file).expect("read file");
        if forbidden_inline_patterns
            .iter()
            .any(|pattern| text.contains(pattern))
        {
            violations.push(rel);
        }
    }

    assert!(
        violations.is_empty(),
        "test surfaces must treat evidence as read-only; mutating patterns found: {}",
        violations.join(", ")
    );
}
