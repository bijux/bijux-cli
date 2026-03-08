use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde::Deserialize;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct OwnershipPolicy {
    ownership_classes: Vec<OwnershipEntry>,
}

#[derive(Debug, Deserialize)]
struct OwnershipEntry {
    prefix: String,
    class: String,
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn app_module_ownership_policy_covers_all_app_sources() {
    let root = repo_root();
    let policy: OwnershipPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/app_module_ownership_classes.json"))
            .expect("read policy"),
    )
    .expect("parse policy");

    for entry in &policy.ownership_classes {
        assert!(
            matches!(
                entry.class.as_str(),
                "route" | "service" | "renderer" | "support"
            ),
            "unsupported ownership class: {}",
            entry.class
        );
    }

    let mut stack = vec![root.join("crates/bijux-dag-app/src")];
    let mut uncovered = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .expect("prefix")
                .to_string_lossy()
                .replace('\\', "/");
            if !policy
                .ownership_classes
                .iter()
                .any(|p| rel.starts_with(&p.prefix))
            {
                uncovered.push(rel);
            }
        }
    }

    assert!(
        uncovered.is_empty(),
        "new app modules must declare ownership class (route/service/renderer/support): {uncovered:?}"
    );
}

#[test]
fn app_services_completion_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/app_services_command_completion_report.md",
        "docs/reports/foundation/app_services_boundary_bypass_report.md",
        "docs/reports/foundation/app_module_hygiene_coupling_report.md",
        "configs/policy/app_module_ownership_classes.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing app services completion artifact {rel}"
        );
    }
}
