use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct WorkspacePackageBoundary {
    schema_version: String,
    release: String,
    owner: String,
    packages: Vec<PackageBoundaryEntry>,
    crates_io_publish_order: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PackageBoundaryEntry {
    #[serde(rename = "crate")]
    crate_name: String,
    product_family: String,
    release_status: String,
    purpose: String,
    private_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    publish: Option<Vec<String>>,
    dependencies: Vec<CargoDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoDependency {
    name: String,
    kind: Option<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn read_boundary() -> WorkspacePackageBoundary {
    let path = repo_root().join("contracts/foundation/workspace_package_boundary.v1.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("invalid {}: {err}", path.display()))
}

fn cargo_metadata() -> CargoMetadata {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root())
        .output()
        .expect("run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    serde_json::from_slice(&output.stdout).expect("parse cargo metadata")
}

fn metadata_map() -> BTreeMap<String, CargoPackage> {
    cargo_metadata().packages.into_iter().map(|package| (package.name.clone(), package)).collect()
}

fn release_status_map(boundary: &WorkspacePackageBoundary) -> BTreeMap<String, &str> {
    boundary
        .packages
        .iter()
        .map(|entry| (entry.crate_name.clone(), entry.release_status.as_str()))
        .collect()
}

fn read_repo_file(path: &str) -> String {
    let absolute = repo_root().join(path);
    fs::read_to_string(&absolute)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", absolute.display()))
}

#[test]
fn workspace_package_boundary_contract_covers_every_workspace_crate() {
    let boundary = read_boundary();
    assert_eq!(boundary.schema_version, "foundation-workspace-package-boundary/v1");
    assert_eq!(boundary.release, "v0.4.0");
    assert_eq!(boundary.owner, "bijux-core");

    let declared =
        boundary.packages.iter().map(|entry| entry.crate_name.clone()).collect::<BTreeSet<_>>();
    let observed = metadata_map().into_keys().collect::<BTreeSet<_>>();

    assert_eq!(
        declared, observed,
        "workspace package boundary must classify every workspace crate exactly once"
    );
}

#[test]
fn workspace_package_boundary_entries_define_release_status_and_stable_purpose() {
    let boundary = read_boundary();

    for entry in &boundary.packages {
        assert!(
            matches!(entry.product_family.as_str(), "bijux-cli" | "bijux-dag" | "maintainer"),
            "crate `{}` has unknown product family `{}`",
            entry.crate_name,
            entry.product_family
        );
        assert!(
            matches!(entry.release_status.as_str(), "public" | "private"),
            "crate `{}` has unknown release status `{}`",
            entry.crate_name,
            entry.release_status
        );
        assert!(
            !entry.purpose.trim().is_empty(),
            "crate `{}` must declare a stable purpose statement",
            entry.crate_name
        );
        if entry.release_status == "private" {
            let reason = entry
                .private_reason
                .as_deref()
                .expect("private entries must explain why they stay private");
            assert!(
                !reason.trim().is_empty(),
                "crate `{}` must keep a non-empty private reason",
                entry.crate_name
            );
        } else {
            assert!(
                entry.private_reason.is_none(),
                "crate `{}` must not define a private reason when it is public",
                entry.crate_name
            );
        }
    }

    let public_crates = boundary
        .packages
        .iter()
        .filter(|entry| entry.release_status == "public")
        .map(|entry| entry.crate_name.clone())
        .collect::<BTreeSet<_>>();
    let publish_order = boundary.crates_io_publish_order.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        publish_order, public_crates,
        "crates.io publish order must list each public crate exactly once"
    );
}

#[test]
fn workspace_package_boundary_matches_publish_flags_and_runtime_dependencies() {
    let boundary = read_boundary();
    let status_by_crate = release_status_map(&boundary);
    let metadata = metadata_map();

    for (crate_name, package) in &metadata {
        match status_by_crate.get(crate_name).copied().expect("boundary must classify crate") {
            "public" => assert!(
                package.publish.is_none(),
                "{crate_name} must stay publishable to crates.io"
            ),
            "private" => assert!(
                matches!(package.publish.as_ref(), Some(allowlist) if allowlist.is_empty()),
                "{crate_name} must keep `publish = false`"
            ),
            _ => unreachable!("validated above"),
        }
    }

    let private_crates = status_by_crate
        .iter()
        .filter(|(_, status)| **status == "private")
        .map(|(crate_name, _)| crate_name.clone())
        .collect::<BTreeSet<_>>();

    let mut violating_edges = Vec::new();
    for entry in boundary.packages.iter().filter(|entry| entry.release_status == "public") {
        let package =
            metadata.get(&entry.crate_name).expect("public crate must exist in cargo metadata");
        for dependency in &package.dependencies {
            if !private_crates.contains(&dependency.name) {
                continue;
            }
            if dependency.kind.as_deref() == Some("dev") {
                continue;
            }
            let kind = dependency.kind.as_deref().unwrap_or("normal");
            violating_edges.push(format!("{} -> {} ({kind})", entry.crate_name, dependency.name));
        }
    }

    assert!(
        violating_edges.is_empty(),
        "public crates must not require private crates at runtime or build time: {violating_edges:?}"
    );
}

#[test]
fn workspace_package_boundary_publish_order_is_dependency_topological() {
    let boundary = read_boundary();
    let metadata = metadata_map();
    let order_index = boundary
        .crates_io_publish_order
        .iter()
        .enumerate()
        .map(|(index, crate_name)| (crate_name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let public_crates = boundary
        .packages
        .iter()
        .filter(|entry| entry.release_status == "public")
        .map(|entry| entry.crate_name.as_str())
        .collect::<BTreeSet<_>>();

    for crate_name in &boundary.crates_io_publish_order {
        let package =
            metadata.get(crate_name).expect("publish order crate must exist in cargo metadata");
        let package_index = order_index.get(crate_name.as_str()).copied().expect("package index");
        for dependency in &package.dependencies {
            if !public_crates.contains(dependency.name.as_str()) {
                continue;
            }
            if dependency.kind.as_deref() == Some("dev") {
                continue;
            }
            let dependency_index =
                order_index.get(dependency.name.as_str()).copied().unwrap_or_else(|| {
                    panic!("public dependency `{}` missing from publish order", dependency.name)
                });
            assert!(
                dependency_index < package_index,
                "publish order must place `{}` before `{}` because cargo metadata reports that dependency edge",
                dependency.name,
                crate_name
            );
        }
    }
}

#[test]
fn workspace_package_boundary_docs_and_release_guides_stay_linked() {
    let readme = read_repo_file("README.md");
    let foundation_index = read_repo_file("docs/bijux-core/foundation/index.md");
    let package_map = read_repo_file("docs/bijux-core/foundation/package-map.md");
    let package_boundary = read_repo_file("docs/bijux-core/foundation/package-boundary.md");
    let packages_index = read_repo_file("docs/bijux-core/packages/index.md");
    let release_operations = read_repo_file("docs/bijux-dev/operations/release-operations.md");
    let release_crates = read_repo_file("docs/bijux-dev/gh-workflows/release-crates.md");

    for content in [readme.as_str(), package_boundary.as_str(), packages_index.as_str()] {
        assert!(
            content.contains("contracts/foundation/workspace_package_boundary.v1.json"),
            "package-boundary-facing docs must point at the canonical contract"
        );
    }

    assert!(
        foundation_index.contains("[Package Boundary](package-boundary.md)"),
        "foundation index must route readers to the package boundary page"
    );
    assert!(
        package_map.contains("[Package Boundary](package-boundary.md)"),
        "package map must point readers to the package boundary page"
    );
    assert!(
        release_operations.contains("../../bijux-core/foundation/package-boundary.md"),
        "release operations must point to the package boundary handbook page"
    );
    assert!(
        release_crates.contains("contracts/foundation/workspace_package_boundary.v1.json"),
        "release-crates workflow doc must point to the package boundary contract"
    );
}
