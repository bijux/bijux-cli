use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_runtime_modules(root: &Path) -> Vec<String> {
    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut out = Vec::new();
    let mut stack = vec![runtime_src.clone()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read runtime dir") {
            let path = entry.expect("runtime entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
                let rel = path
                    .strip_prefix(&runtime_src)
                    .expect("strip runtime prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                if rel != "lib.rs" {
                    out.push(rel);
                }
            }
        }
    }
    out.sort();
    out
}

#[test]
fn runtime_scope_401_420_governance_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/runtime_scope_401_420_status_report.md",
        "docs/reports/foundation/runtime_scope_classification_report.md",
        "docs/reports/foundation/runtime_public_surface_size_report.md",
        "docs/reports/foundation/runtime_public_surface_shrink_trend_report.md",
        "docs/spec/RUNTIME_SCOPE_GOVERNANCE_POLICY.md",
        "configs/policy/runtime_module_lifecycle_status.json",
        "configs/suites/runtime_scope_contraction_verification.json",
        "docs/adr/20260308-runtime-scope-end-state.md",
    ] {
        assert!(root.join(rel).exists(), "missing runtime scope artifact: {rel}");
    }
}

#[test]
fn runtime_lifecycle_policy_covers_all_runtime_modules() {
    let root = repo_root();
    let payload = fs::read_to_string(root.join("configs/policy/runtime_module_lifecycle_status.json"))
        .expect("read runtime lifecycle policy");
    let policy: Value = serde_json::from_str(&payload).expect("parse runtime lifecycle policy");

    let allowed: BTreeSet<String> = policy["allowed_lifecycle_status"]
        .as_array()
        .expect("allowed_lifecycle_status array")
        .iter()
        .map(|value| value.as_str().expect("status string").to_string())
        .collect();

    let prefixes = policy["status_by_prefix"]
        .as_array()
        .expect("status_by_prefix array");
    let quarantine_prefixes: Vec<String> = policy["quarantine_prefixes"]
        .as_array()
        .expect("quarantine_prefixes")
        .iter()
        .map(|value| value.as_str().expect("quarantine prefix").to_string())
        .collect();

    let mut seen_statuses = BTreeSet::new();
    for module in collect_runtime_modules(&root) {
        let mut matched_status = None;
        for entry in prefixes {
            let prefix = entry["prefix"].as_str().expect("prefix string");
            if module.starts_with(prefix) {
                let status = entry["status"].as_str().expect("status string");
                assert!(allowed.contains(status), "unsupported lifecycle status `{status}`");
                matched_status = Some(status.to_string());
                seen_statuses.insert(status.to_string());
                if status == "experimental" || status == "speculative" {
                    assert!(
                        quarantine_prefixes.iter().any(|prefix| module.starts_with(prefix)),
                        "experimental/speculative module must stay quarantined: {module}"
                    );
                }
                break;
            }
        }
        assert!(
            matched_status.is_some(),
            "runtime module missing lifecycle status mapping: {module}"
        );
    }

    for required in ["core", "adapter", "operator-support", "experimental", "speculative"] {
        assert!(
            seen_statuses.contains(required),
            "lifecycle status not represented by runtime module mapping: {required}"
        );
    }

    let expiration = policy["expiration_criteria"]
        .as_object()
        .expect("expiration_criteria object");
    for key in ["experimental", "speculative"] {
        let criteria = expiration
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(
            !criteria.trim().is_empty(),
            "missing expiration criteria for lifecycle status: {key}"
        );
    }
}

#[test]
fn runtime_scope_401_420_status_report_references_required_outputs() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/runtime_scope_401_420_status_report.md"),
    )
    .expect("read runtime scope status report");

    for token in [
        "401-404",
        "405-413",
        "414-416",
        "417-420",
        "runtime_module_lifecycle_status.json",
        "runtime_scope_contraction_verification.json",
        "20260308-runtime-scope-end-state.md",
    ] {
        assert!(report.contains(token), "missing runtime scope mapping token: {token}");
    }
}
