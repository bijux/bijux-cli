use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::{Path, PathBuf};
use tempfile as _;

#[derive(Debug, serde::Deserialize)]
struct GuardrailsPolicy {
    checks: Vec<GuardrailCheck>,
}

#[derive(Debug, serde::Deserialize)]
struct GuardrailCheck {
    id: String,
    crate_path: String,
    forbidden_tokens: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn crate_responsibility_guardrails_policy_is_well_formed_and_complete() {
    let root = repo_root();
    let raw = std::fs::read_to_string(root.join("configs/policy/crate_responsibility_guardrails.json"))
        .expect("read guardrails policy");
    let policy: GuardrailsPolicy = serde_json::from_str(&raw).expect("parse guardrails policy");
    let mut ids = std::collections::BTreeSet::new();
    for check in &policy.checks {
        assert!(ids.insert(check.id.clone()), "duplicate check id");
        assert!(!check.forbidden_tokens.is_empty(), "empty forbidden token set");
        assert!(
            root.join(&check.crate_path).exists(),
            "crate path missing for check {}",
            check.id
        );
    }
}

#[test]
fn crate_responsibility_guardrails_enforce_forbidden_tokens() {
    let root = repo_root();
    let raw = std::fs::read_to_string(root.join("configs/policy/crate_responsibility_guardrails.json"))
        .expect("read guardrails policy");
    let policy: GuardrailsPolicy = serde_json::from_str(&raw).expect("parse guardrails policy");

    for check in &policy.checks {
        let mut stack = vec![root.join(&check.crate_path)];
        let mut offenders = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read dir") {
                let entry = entry.expect("entry");
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                    continue;
                }
                let content = std::fs::read_to_string(&path).expect("read source");
                for token in &check.forbidden_tokens {
                    if content.contains(token) {
                        let rel = path
                            .strip_prefix(&root)
                            .expect("strip prefix")
                            .to_string_lossy()
                            .replace('\\', "/");
                        offenders.push(format!("{rel} -> {token}"));
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "crate responsibility drift for check {}: {}",
            check.id,
            offenders.join(" | ")
        );
    }
}

#[test]
fn crate_responsibility_reports_exist_and_are_generated() {
    let root = repo_root();
    for rel in [
        "docs/spec/CRATE_RESPONSIBILITY_ALIGNMENT.md",
        "docs/reports/foundation/crate_dependency_graph_overlays.md",
        "docs/reports/foundation/crate_api_size_delta_report.md",
        "docs/reports/foundation/forbidden_dependency_edge_report.md",
        "docs/spec/CRATE_OWNERSHIP_MATRIX.md",
        "docs/reports/foundation/surface_count_by_crate_report.md",
        "docs/reports/foundation/largest_files_by_crate_report.md",
        "docs/reports/foundation/runtime_pub_use_audit.md",
        "docs/reports/foundation/core_pub_use_audit.md",
        "docs/reports/foundation/artifacts_pub_use_audit.md",
        "docs/reports/foundation/module_scope_name_review.md",
    ] {
        assert!(root.join(rel).exists(), "missing crate responsibility report: {rel}");
    }
}
