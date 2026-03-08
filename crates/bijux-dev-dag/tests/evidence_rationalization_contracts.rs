use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn evidence_rationalization_policy_covers_all_evidence_verify_commands() {
    let root = repo_root();
    let suite_policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_suite_policy.json"))
            .expect("read evidence suite policy"),
    )
    .expect("parse evidence suite policy");
    let rationalization: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_rationalization_policy.json"))
            .expect("read evidence rationalization policy"),
    )
    .expect("parse evidence rationalization policy");

    let suites: BTreeSet<String> = suite_policy["suites"]
        .as_array()
        .expect("suite policy suites array")
        .iter()
        .map(|entry| {
            entry["verify_command"]
                .as_str()
                .expect("suite verify command")
                .to_string()
        })
        .collect();

    let allowed_classes: BTreeSet<String> = rationalization["severity_classes"]
        .as_array()
        .expect("severity_classes array")
        .iter()
        .map(|entry| entry.as_str().expect("severity class").to_string())
        .collect();
    assert_eq!(
        allowed_classes,
        BTreeSet::from([
            "release-critical".to_string(),
            "release-supporting".to_string(),
            "advisory".to_string(),
        ])
    );

    let mut commands = BTreeSet::new();
    for entry in rationalization["commands"]
        .as_array()
        .expect("commands array")
    {
        let verify_command = entry["verify_command"]
            .as_str()
            .expect("verify_command")
            .to_string();
        let severity_class = entry["severity_class"].as_str().expect("severity_class");
        let audience = entry["audience"].as_str().expect("audience");
        let docs_page = entry["docs_page"].as_str().expect("docs_page");

        assert!(
            allowed_classes.contains(severity_class),
            "unexpected severity class for {verify_command}: {severity_class}"
        );
        assert!(
            !audience.trim().is_empty(),
            "audience must not be empty for {verify_command}"
        );
        assert!(
            root.join(docs_page).exists(),
            "docs mapping is missing for {verify_command}: {docs_page}"
        );
        assert!(
            commands.insert(verify_command.clone()),
            "duplicate command in rationalization policy: {verify_command}"
        );
    }

    assert_eq!(
        suites, commands,
        "evidence rationalization policy must cover exactly the evidence suite verify commands"
    );
}

#[test]
fn release_critical_commands_run_in_green_paths() {
    let root = repo_root();
    let rationalization: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_rationalization_policy.json"))
            .expect("read evidence rationalization policy"),
    )
    .expect("parse evidence rationalization policy");
    let make_root = fs::read_to_string(root.join("make/root.mk")).expect("read make/root.mk");

    for entry in rationalization["commands"]
        .as_array()
        .expect("commands array")
    {
        let command = entry["verify_command"].as_str().expect("verify_command");
        let severity = entry["severity_class"].as_str().expect("severity_class");
        let must_run = entry["must_run_in_green_paths"]
            .as_bool()
            .expect("must_run_in_green_paths bool");

        if severity == "release-critical" {
            assert!(
                must_run,
                "release-critical command must mark green-path requirement: {command}"
            );
            let target = command.replace("verify evidence-", "evidence-");
            assert!(
                make_root.contains(&format!("@$(MAKE) {}", target)),
                "release-critical command missing from make test-release: {command}"
            );
        }
        if severity == "advisory" {
            let target = command.replace("verify evidence-", "evidence-");
            assert!(
                !make_root.contains(&format!("@$(MAKE) {}", target)),
                "advisory command must not block make test-release: {command}"
            );
        }
    }
}

#[test]
fn evidence_reports_have_declared_severity_and_docs_mapping() {
    let root = repo_root();
    let rationalization: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_rationalization_policy.json"))
            .expect("read evidence rationalization policy"),
    )
    .expect("parse evidence rationalization policy");

    let mut report_paths = BTreeSet::new();
    for entry in rationalization["reports"]
        .as_array()
        .expect("reports array")
    {
        let report_path = entry["report_path"].as_str().expect("report_path");
        let severity = entry["severity_class"].as_str().expect("severity_class");
        let audience = entry["audience"].as_str().expect("audience");
        let docs_page = entry["docs_page"].as_str().expect("docs_page");

        assert!(
            ["release-critical", "release-supporting", "advisory"].contains(&severity),
            "invalid report severity for {report_path}: {severity}"
        );
        assert!(
            !audience.trim().is_empty(),
            "report audience must not be empty: {report_path}"
        );
        assert!(
            root.join(report_path).exists(),
            "missing governed report: {report_path}"
        );
        assert!(
            root.join(docs_page).exists(),
            "missing docs mapping target for report {report_path}: {docs_page}"
        );
        assert!(
            report_paths.insert(report_path.to_string()),
            "duplicate report policy entry: {report_path}"
        );
    }
}

#[test]
fn duplicate_signal_consolidation_and_pack_reports_exist() {
    let root = repo_root();
    let rationalization: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_rationalization_policy.json"))
            .expect("read evidence rationalization policy"),
    )
    .expect("parse evidence rationalization policy");

    let duplicate_report =
        root.join("docs/reports/foundation/evidence_outputs_duplicate_signal_report.md");
    assert!(
        duplicate_report.exists(),
        "missing duplicate evidence output report"
    );

    let duplication = rationalization["duplicate_signal_ownership"]
        .as_object()
        .expect("duplicate_signal_ownership object");
    for (signal, mapping) in duplication {
        let owner = mapping["owner_output"].as_str().expect("owner_output");
        assert!(
            root.join(owner).exists(),
            "canonical owner output missing for signal {signal}: {owner}"
        );
        for duplicate in mapping["duplicates_retired"]
            .as_array()
            .expect("duplicates_retired array")
        {
            let duplicate = duplicate.as_str().expect("duplicate output path");
            assert!(
                root.join(duplicate).exists(),
                "duplicate output reference must exist for traceability: {duplicate}"
            );
        }
    }

    for rel in [
        "docs/reports/foundation/compact_release_evidence_pack.md",
        "docs/reports/foundation/compact_release_evidence_pack.json",
        "docs/reports/foundation/compact_advisory_evidence_pack.md",
        "docs/reports/foundation/compact_advisory_evidence_pack.json",
        "docs/reports/foundation/evidence_docs_mapping_report.md",
        "docs/reports/foundation/evidence_suite_exercise_mapping_report.md",
        "docs/reports/foundation/evidence_commands_not_exercised_in_ci_report.md",
        "docs/adr/20260308-evidence-severity-rationalization.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing evidence rationalization artifact: {rel}"
        );
    }

    let release_pack: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("docs/reports/foundation/compact_release_evidence_pack.json"),
        )
        .expect("read compact release pack"),
    )
    .expect("parse compact release pack");
    assert_eq!(release_pack["lane"].as_str(), Some("release-critical"));
    assert!(
        release_pack["commands"]
            .as_array()
            .expect("release commands")
            .iter()
            .any(|entry| entry.as_str() == Some("verify evidence-release-set")),
        "release pack must include release-set verification"
    );

    let advisory_pack: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("docs/reports/foundation/compact_advisory_evidence_pack.json"),
        )
        .expect("read compact advisory pack"),
    )
    .expect("parse compact advisory pack");
    assert_eq!(advisory_pack["lane"].as_str(), Some("advisory"));
    let advisory_commands: BTreeSet<String> = advisory_pack["commands"]
        .as_array()
        .expect("advisory commands")
        .iter()
        .map(|entry| entry.as_str().expect("advisory command").to_string())
        .collect();
    assert_eq!(
        advisory_commands,
        BTreeSet::from(["verify evidence-compare".to_string()]),
        "advisory pack should only include advisory evidence command"
    );
}

#[test]
fn evidence_report_and_command_class_lists_match_policy() {
    let root = repo_root();
    let rationalization: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_rationalization_policy.json"))
            .expect("read evidence rationalization policy"),
    )
    .expect("parse evidence rationalization policy");

    let mut classified: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for entry in rationalization["commands"]
        .as_array()
        .expect("commands array")
    {
        let class = entry["severity_class"]
            .as_str()
            .expect("severity_class")
            .to_string();
        let command = entry["verify_command"]
            .as_str()
            .expect("verify_command")
            .to_string();
        classified.entry(class).or_default().insert(command);
    }

    let release_report = fs::read_to_string(
        root.join("docs/reports/foundation/release_critical_evidence_commands_only_report.md"),
    )
    .expect("read release-critical command report");
    let supporting_report = fs::read_to_string(
        root.join("docs/reports/foundation/release_supporting_evidence_commands_report.md"),
    )
    .expect("read release-supporting command report");
    let advisory_report = fs::read_to_string(
        root.join("docs/reports/foundation/advisory_only_evidence_commands_report.md"),
    )
    .expect("read advisory command report");

    for command in classified
        .get("release-critical")
        .expect("release-critical class")
    {
        assert!(
            release_report.contains(command),
            "release-critical command report missing command: {command}"
        );
    }
    for command in classified
        .get("release-supporting")
        .expect("release-supporting class")
    {
        assert!(
            supporting_report.contains(command),
            "release-supporting command report missing command: {command}"
        );
    }
    for command in classified.get("advisory").expect("advisory class") {
        assert!(
            advisory_report.contains(command),
            "advisory command report missing command: {command}"
        );
    }
}
