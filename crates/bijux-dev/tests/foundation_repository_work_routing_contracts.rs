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
struct RepositoryWorkRoutingContract {
    schema_version: String,
    issue_classes: Vec<RepositoryWorkClass>,
}

#[derive(Debug, Deserialize)]
struct RepositoryWorkClass {
    issue_class: String,
    owning_crate: String,
    allowed_workspace_deps: Vec<String>,
    evidence_location: String,
}

#[derive(Debug)]
struct RoutingEvidenceRow {
    issue_class: String,
    owning_crate: String,
    evidence_location: String,
    responsibility: String,
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

fn read_repository_work_routing_contract() -> RepositoryWorkRoutingContract {
    let path = repo_root().join("contracts/foundation/repository_work_routing.v1.json");
    read_json(&path)
}

fn read_routing_evidence_rows() -> Vec<RoutingEvidenceRow> {
    let path = repo_root().join("docs/reports/governance/REPOSITORY_WORK_ROUTING.md");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    raw.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|')
                || trimmed.starts_with("| ---")
                || trimmed.starts_with("| Work class")
            {
                return None;
            }

            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(|cell| cell.trim().trim_matches('`').to_string())
                .collect::<Vec<_>>();
            if cells.len() != 4 {
                return None;
            }

            Some(RoutingEvidenceRow {
                issue_class: cells[0].clone(),
                owning_crate: cells[1].clone(),
                evidence_location: cells[2].clone(),
                responsibility: cells[3].clone(),
            })
        })
        .collect()
}

#[test]
fn repository_work_routing_contract_schema_is_current() {
    let contract = read_repository_work_routing_contract();
    assert_eq!(contract.schema_version, "foundation-repository-work-routing/v1");
}

#[test]
fn repository_work_classes_map_to_known_workspace_crates_and_paths() {
    let product_crates = read_workspace_product_crates();
    let contract = read_repository_work_routing_contract();

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
fn repository_work_routing_evidence_matches_contract() {
    let contract = read_repository_work_routing_contract();
    let by_class = contract
        .issue_classes
        .into_iter()
        .map(|class| (class.issue_class.clone(), class))
        .collect::<BTreeMap<_, _>>();
    let rows = read_routing_evidence_rows();

    assert_eq!(
        rows.len(),
        by_class.len(),
        "routing evidence must cover every governed work class exactly once"
    );

    let mut seen = BTreeSet::new();
    for row in rows {
        assert!(
            row.issue_class != "uncategorized",
            "repository work must not use an uncategorized class"
        );
        assert!(
            seen.insert(row.issue_class.clone()),
            "duplicate routing evidence for {}",
            row.issue_class
        );

        let class = by_class
            .get(&row.issue_class)
            .unwrap_or_else(|| panic!("unknown work class {}", row.issue_class));
        assert_eq!(
            row.owning_crate, class.owning_crate,
            "owning crate drifted for {}",
            row.issue_class
        );
        assert_eq!(
            row.evidence_location, class.evidence_location,
            "evidence location drifted for {}",
            row.issue_class
        );
        assert!(
            !row.responsibility.trim().is_empty(),
            "responsibility is required for {}",
            row.issue_class
        );
    }
}
