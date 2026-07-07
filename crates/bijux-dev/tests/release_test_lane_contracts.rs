use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ReleaseTestLaneGovernance {
    format: String,
    required_release_lane: ReleaseLane,
    full_verification_lane: FullVerificationLane,
    rules: GovernanceRules,
    portfolios: Vec<IgnoredTestPortfolio>,
}

#[derive(Debug, Deserialize)]
struct ReleaseLane {
    make_target: String,
    ci_entrypoint: String,
    nextest_profile: String,
    ignored_tests_allowed: bool,
}

#[derive(Debug, Deserialize)]
struct FullVerificationLane {
    make_target: String,
}

#[derive(Debug, Deserialize)]
struct GovernanceRules {
    forbid_flaky_ignored_tests: bool,
    release_lane_may_not_depend_on_ignored_tests: bool,
    ignored_tests_require_quarantine_record: bool,
    ignored_tests_must_be_nonstable: bool,
}

#[derive(Debug, Deserialize)]
struct IgnoredTestPortfolio {
    path: String,
    execution_lane: String,
    ignore_reason: String,
    surface_class: String,
    outside_required_release_lane: bool,
    full_lane_command: String,
    tests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct IgnoredTestCase {
    path: String,
    reason: String,
    name: String,
}

fn is_nonstable_reason(reason: &str) -> bool {
    matches!(reason, "experimental" | "internal")
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

fn collect_ignored_tests(root: &Path) -> Vec<IgnoredTestCase> {
    let targets =
        [root.join("crates/bijux-dag-app/tests"), root.join("crates/bijux-dag-cli/tests")];
    let mut ignored = Vec::new();

    for dir in targets {
        for entry in fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("read_dir {} failed: {err}", dir.display()))
        {
            let entry = entry.expect("dir entry");
            let path = entry.path();
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
                        ignored.push(IgnoredTestCase { path: rel.clone(), reason, name });
                    } else {
                        panic!("ignore attribute in {rel} was not followed by a test function");
                    }
                }
            }
        }
    }

    ignored.sort();
    ignored
}

#[test]
fn release_test_lane_governance_is_current() {
    let governance = read_governance();
    assert_eq!(governance.format, "release-test-lane-governance/v2");
    assert_eq!(governance.required_release_lane.make_target, "test-release-rs");
    assert_eq!(governance.required_release_lane.ci_entrypoint, "make gh-test");
    assert_eq!(governance.required_release_lane.nextest_profile, "ci");
    assert!(!governance.required_release_lane.ignored_tests_allowed);
    assert_eq!(governance.full_verification_lane.make_target, "test-all-rs");
    assert!(governance.rules.forbid_flaky_ignored_tests);
    assert!(governance.rules.release_lane_may_not_depend_on_ignored_tests);
    assert!(governance.rules.ignored_tests_require_quarantine_record);
    assert!(governance.rules.ignored_tests_must_be_nonstable);
}

#[test]
fn every_ignored_dag_test_is_declared_and_non_flaky() {
    let root = repo_root();
    let governance = read_governance();
    let actual = collect_ignored_tests(&root);

    assert!(
        actual.iter().all(|test| !test.reason.contains("flaky")),
        "flaky ignored tests remain in DAG release surfaces: {actual:#?}"
    );

    let declared: BTreeSet<IgnoredTestCase> = governance
        .portfolios
        .iter()
        .flat_map(|portfolio| {
            portfolio.tests.iter().map(|name| IgnoredTestCase {
                path: portfolio.path.clone(),
                reason: portfolio.ignore_reason.clone(),
                name: name.clone(),
            })
        })
        .collect();
    let actual: BTreeSet<IgnoredTestCase> = actual.into_iter().collect();

    assert_eq!(
        declared, actual,
        "release test lane governance must exactly match live ignored DAG tests"
    );
}

#[test]
fn ignored_dag_portfolios_stay_outside_required_release_lane() {
    let governance = read_governance();
    assert!(!governance.portfolios.is_empty(), "expected governed ignored DAG portfolios");

    for portfolio in governance.portfolios {
        assert_eq!(
            portfolio.execution_lane, "full",
            "ignored portfolio {} must stay in the full verification lane",
            portfolio.path
        );
        assert!(
            portfolio.outside_required_release_lane,
            "ignored portfolio {} must remain outside the required release lane",
            portfolio.path
        );
        assert!(
            is_nonstable_reason(&portfolio.ignore_reason),
            "ignored portfolio {} must keep an explicit nonstable quarantine reason",
            portfolio.path
        );
        assert_eq!(
            portfolio.ignore_reason, portfolio.surface_class,
            "ignored portfolio {} must keep matching ignore reason and surface class",
            portfolio.path
        );
        assert!(
            portfolio.full_lane_command.contains("--run-ignored only"),
            "ignored portfolio {} must document an explicit full-lane command",
            portfolio.path
        );
        assert!(
            !portfolio.tests.is_empty(),
            "ignored portfolio {} must declare at least one test",
            portfolio.path
        );
    }
}
