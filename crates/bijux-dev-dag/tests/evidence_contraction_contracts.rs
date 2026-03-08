use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn evidence_family_governance_policy_requires_purpose_and_ownership_docs() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/evidence_family_governance.json"))
            .expect("read evidence family governance policy"),
    )
    .expect("parse evidence family governance policy");

    assert_eq!(
        policy["family_addition_rule"]["requires_purpose_classification"]
            .as_bool()
            .expect("requires_purpose_classification bool"),
        true
    );
    assert_eq!(
        policy["family_addition_rule"]["requires_ownership_docs"]
            .as_bool()
            .expect("requires_ownership_docs bool"),
        true
    );

    for family in policy["families"].as_array().expect("families array") {
        let name = family["name"].as_str().expect("family name");
        assert!(
            !family["purpose"]
                .as_str()
                .expect("family purpose")
                .trim()
                .is_empty(),
            "family purpose must be non-empty: {name}"
        );
        let class = family["release_class"].as_str().expect("release class");
        assert!(
            matches!(class, "blocking" | "advisory"),
            "invalid release class for {name}: {class}"
        );
        for doc in family["ownership_docs"]
            .as_array()
            .expect("ownership docs array")
        {
            let doc = doc.as_str().expect("ownership doc path");
            assert!(
                root.join(doc).exists(),
                "ownership doc must exist for family {name}: {doc}"
            );
        }
    }
}

#[test]
fn evidence_command_families_stay_separated_by_dedicated_modules() {
    let root = repo_root();
    let authoring =
        fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/authoring_evidence.rs"))
            .expect("read authoring evidence command module");
    let battle =
        fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/battle_evidence.rs"))
            .expect("read battle evidence command module");
    let compare =
        fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/compare_evidence.rs"))
            .expect("read compare evidence command module");
    let perf = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/perf_evidence.rs"))
        .expect("read perf evidence command module");

    assert!(!authoring.contains("run_battle_"));
    assert!(!authoring.contains("run_perf_"));
    assert!(!authoring.contains("run_compare_"));

    assert!(!battle.contains("run_perf_"));
    assert!(!battle.contains("run_compare_"));

    assert!(!perf.contains("run_battle_"));
    assert!(!perf.contains("run_compare_"));

    assert!(!compare.contains("run_battle_"));
    assert!(!compare.contains("run_perf_"));
}

#[test]
fn evidence_dashboards_exist_in_human_and_machine_readable_forms() {
    let root = repo_root();
    assert!(root
        .join("docs/reports/foundation/evidence_dashboard.md")
        .exists());
    assert!(root
        .join("docs/reports/foundation/evidence_dashboard.json")
        .exists());

    let dashboard_json: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("docs/reports/foundation/evidence_dashboard.json"))
            .expect("read evidence dashboard json"),
    )
    .expect("parse evidence dashboard json");

    let release_critical: BTreeSet<String> = dashboard_json["release_critical_families"]
        .as_array()
        .expect("release critical families")
        .iter()
        .map(|entry| entry.as_str().expect("release critical family").to_string())
        .collect();
    assert!(release_critical.contains("battle"));
    assert!(release_critical.contains("perf"));

    let advisory: BTreeSet<String> = dashboard_json["advisory_families"]
        .as_array()
        .expect("advisory families")
        .iter()
        .map(|entry| entry.as_str().expect("advisory family").to_string())
        .collect();
    assert!(advisory.contains("compare"));
}

#[test]
fn evidence_release_note_and_internal_only_docs_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/EVIDENCE_RELEASE_NOTE_TRUST.md",
        "docs/spec/EVIDENCE_INTERNAL_ONLY_SURFACES.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing evidence documentation: {rel}"
        );
    }
}

#[test]
fn evidence_docs_and_checks_linkage_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/evidence_docs_without_checks_report.md",
        "docs/reports/foundation/evidence_checks_without_docs_report.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing evidence linkage report: {rel}"
        );
    }
}

#[test]
fn release_review_checklist_includes_evidence_pruning_review_item() {
    let root = repo_root();
    let checklist = fs::read_to_string(root.join("docs/spec/RELEASE_REVIEW_CHECKLIST.md"))
        .expect("read release review checklist");
    assert!(
        checklist.contains("Evidence pruning review complete"),
        "release review checklist must include evidence pruning review"
    );
}
