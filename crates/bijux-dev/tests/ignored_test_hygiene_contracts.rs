use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ReleaseTestLaneGovernance {
    portfolios: Vec<IgnoredTestPortfolio>,
}

#[derive(Debug, Deserialize)]
struct IgnoredTestPortfolio {
    path: String,
    ignore_reason: String,
    surface_class: String,
    tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IgnoredTestCase {
    path: String,
    reason: String,
    name: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn read_governance() -> ReleaseTestLaneGovernance {
    let path = repo_root().join("configs/dag/policy/release_test_lane_governance.json");
    let raw =
        fs::read_to_string(&path).unwrap_or_else(|err| panic!("read governance failed: {err}"));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse governance failed: {err}"))
}

fn collect_ignored_tests(root: &Path) -> BTreeSet<IgnoredTestCase> {
    let mut ignored = BTreeSet::new();
    let mut stack = vec![root.join("crates")];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("read_dir {} failed: {err}", dir.display()))
        {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }

            let source = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {} failed: {err}", path.display()));
            let rel =
                path.strip_prefix(root).expect("relative path").to_string_lossy().into_owned();
            let mut pending_reason: Option<String> = None;

            for line in source.lines() {
                let trimmed = line.trim();
                if let Some(reason) = trimmed.strip_prefix("#[ignore = \"") {
                    let reason = reason.strip_suffix("\"]").expect("ignore attribute suffix");
                    pending_reason = Some(reason.to_string());
                    continue;
                }

                if let Some(reason) = pending_reason.take() {
                    if let Some(name) = trimmed.strip_prefix("fn ") {
                        let name =
                            name.split('(').next().expect("function name").trim().to_string();
                        ignored.insert(IgnoredTestCase { path: rel.clone(), reason, name });
                    } else {
                        panic!("ignore attribute in {rel} was not followed by a test function");
                    }
                }
            }
        }
    }

    ignored
}

fn governed_ignored_tests(governance: ReleaseTestLaneGovernance) -> BTreeSet<IgnoredTestCase> {
    governance
        .portfolios
        .into_iter()
        .flat_map(|portfolio| {
            assert_eq!(
                portfolio.ignore_reason, portfolio.surface_class,
                "governed ignored portfolio {} must keep matching ignore reason and surface class",
                portfolio.path
            );
            portfolio.tests.into_iter().map(move |name| IgnoredTestCase {
                path: portfolio.path.clone(),
                reason: portfolio.ignore_reason.clone(),
                name,
            })
        })
        .collect()
}

fn is_nonstable_reason(reason: &str) -> bool {
    matches!(reason, "experimental" | "internal")
}

#[test]
fn workspace_ignored_tests_match_governed_dag_portfolios() {
    let root = repo_root();
    let actual = collect_ignored_tests(&root);
    let governed = governed_ignored_tests(read_governance());

    assert_eq!(
        actual, governed,
        "ignored Rust tests must be limited to the governed DAG nonstable portfolios"
    );
}

#[test]
fn governed_dag_ignored_tests_keep_nonstable_reasons() {
    let governed = governed_ignored_tests(read_governance());
    assert!(
        governed.iter().all(|test| is_nonstable_reason(&test.reason)),
        "ignored Rust tests must keep explicit experimental or internal quarantine reasons: {governed:#?}"
    );
}
