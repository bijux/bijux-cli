//! Status contract inventory registry.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::contract_engine::maintenance::{generated_at_utc, native_status_contract_rows};

use super::id::{infer_kind, is_status_contract_id};
use super::spec::StatusContractSpec;

fn ci_referenced_ids(workspace_root: &Path) -> BTreeSet<String> {
    let ci_text = fs::read_to_string(workspace_root.join(".github/workflows/ci.yml"))
        .unwrap_or_default();
    ci_text
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                !ch.is_ascii_uppercase() && !ch.is_ascii_digit() && ch != '-'
            })
        })
        .filter(|token| is_status_contract_id(token))
        .map(ToString::to_string)
        .collect()
}

/// Return all known status contract specs.
#[must_use]
pub fn status_contract_specs(workspace_root: &Path) -> Vec<StatusContractSpec> {
    let mut specs: Vec<StatusContractSpec> = native_status_contract_rows()
        .into_iter()
        .filter_map(|row| StatusContractSpec::from_row(&row))
        .collect();

    let known_ids: BTreeSet<String> = specs.iter().map(|spec| spec.contract_id.clone()).collect();
    for id in ci_referenced_ids(workspace_root).difference(&known_ids) {
        let inferred_kind = infer_kind(id);
        specs.push(StatusContractSpec {
            contract_id: id.clone(),
            kind: inferred_kind,
            source_ref: None,
            implementation: "rust-compat".to_string(),
            outputs: Vec::new(),
            command: format!("bijux dev cli maintenance status run --id {id}"),
        });
    }

    specs.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    specs
}

/// Build status contract inventory payload.
#[must_use]
pub fn build_inventory_report(workspace_root: &Path) -> Value {
    let specs = status_contract_specs(workspace_root);
    let mut kind_counts = BTreeMap::<String, usize>::new();
    for spec in &specs {
        *kind_counts.entry(spec.kind.as_str().to_string()).or_insert(0) += 1;
    }

    json!({
        "id_policy": "STATUS-CONTRACT-<KIND>-<SLUG>",
        "kinds": kind_counts,
        "count": specs.len(),
        "generated_at_utc": generated_at_utc(),
        "rows": specs.into_iter().map(|spec| spec.to_row()).collect::<Vec<_>>(),
    })
}
