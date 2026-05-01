use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct WorkspaceProductMapContract {
    products: Vec<WorkspaceProduct>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceProduct {
    #[serde(rename = "crate")]
    crate_name: String,
}

#[derive(Debug, Deserialize)]
struct BacklogIssueClassRoutingContract {
    schema_version: String,
    issue_classes: Vec<BacklogIssueClass>,
}

#[derive(Debug, Deserialize)]
struct BacklogIssueClass {
    issue_class: String,
    owning_crate: String,
    allowed_workspace_deps: Vec<String>,
    evidence_location: String,
}

#[derive(Debug)]
struct LedgerRow {
    goal: String,
    issue_class: String,
    owning_crate: String,
    evidence_location: String,
    status: String,
    note: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> T {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("invalid json {}: {err}", path.display()))
}

fn read_workspace_product_crates() -> BTreeSet<String> {
    let path = repo_root().join("contracts/foundation/workspace_product_map.v1.json");
    let contract: WorkspaceProductMapContract = read_json(&path);
    contract.products.into_iter().map(|product| product.crate_name).collect()
}

fn read_backlog_issue_class_contract() -> BacklogIssueClassRoutingContract {
    let path = repo_root().join("contracts/foundation/backlog_issue_class_routing.v1.json");
    read_json(&path)
}

fn read_backlog_routing_ledger_rows() -> Vec<LedgerRow> {
    let path = repo_root().join("docs/bijux-core/foundation/backlog-routing-ledger.md");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') {
                return None;
            }
            if trimmed.starts_with("| ---") || trimmed.starts_with("| Goal") {
                return None;
            }

            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().to_string())
                .collect::<Vec<_>>();
            if cells.len() != 6 {
                return None;
            }

            Some(LedgerRow {
                goal: cells[0].clone(),
                issue_class: cells[1].clone(),
                owning_crate: cells[2].clone(),
                evidence_location: cells[3].clone(),
                status: cells[4].clone(),
                note: cells[5].clone(),
            })
        })
        .collect()
}

#[test]
fn backlog_issue_class_contract_schema_is_current() {
    let contract = read_backlog_issue_class_contract();
    assert_eq!(contract.schema_version, "foundation-backlog-issue-class-routing/v1");
}

#[test]
fn backlog_issue_classes_map_to_known_workspace_crates_and_paths() {
    let product_crates = read_workspace_product_crates();
    let contract = read_backlog_issue_class_contract();

    let mut seen = BTreeSet::new();
    for class in contract.issue_classes {
        assert!(
            seen.insert(class.issue_class.clone()),
            "duplicate issue_class entry: {}",
            class.issue_class
        );
        assert!(
            product_crates.contains(&class.owning_crate),
            "owning crate is not a known workspace product: {}",
            class.owning_crate
        );
        assert!(
            !class.evidence_location.trim().is_empty(),
            "evidence location is required for {}",
            class.issue_class
        );

        let evidence_path = repo_root().join(&class.evidence_location);
        assert!(
            evidence_path.exists(),
            "evidence location does not exist for {}: {}",
            class.issue_class,
            evidence_path.display()
        );

        for dep in class.allowed_workspace_deps {
            assert!(
                product_crates.contains(&dep),
                "allowed dependency for {} is not a known product crate: {}",
                class.issue_class,
                dep
            );
        }
    }
}

#[test]
fn backlog_ledger_rows_are_categorized_and_never_uncategorized() {
    let contract = read_backlog_issue_class_contract();
    let mut by_class = BTreeMap::new();
    for class in contract.issue_classes {
        by_class.insert(class.issue_class, class.owning_crate);
    }

    let rows = read_backlog_routing_ledger_rows();
    assert!(!rows.is_empty(), "backlog routing ledger must list tracked goals");

    for row in rows {
        assert!(
            row.issue_class != "uncategorized",
            "goal {} must not use uncategorized issue class",
            row.goal
        );

        let Some(expected_owner) = by_class.get(&row.issue_class) else {
            panic!("goal {} references unknown issue class {}", row.goal, row.issue_class);
        };

        assert_eq!(
            &row.owning_crate, expected_owner,
            "goal {} owning crate drifted from issue class contract",
            row.goal
        );

        assert!(
            !row.evidence_location.trim().is_empty(),
            "goal {} must include an evidence location",
            row.goal
        );

        assert!(
            matches!(
                row.status.as_str(),
                "not-started" | "in-progress" | "done" | "deferred" | "blocked"
            ),
            "goal {} has unknown status: {}",
            row.goal,
            row.status
        );

        assert!(!row.note.trim().is_empty(), "goal {} must include a note", row.goal);
    }
}
