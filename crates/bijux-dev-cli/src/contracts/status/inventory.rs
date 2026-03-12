//! Status contract inventory registry.

use std::collections::BTreeMap;
use std::path::Path;
use serde_json::{json, Value};

use crate::contracts::maintenance::{generated_at_utc, native_status_contract_rows};

use super::model::StatusContractSpec;

/// Return all known status contract specs.
#[must_use]
pub fn status_contract_specs() -> Vec<StatusContractSpec> {
    let mut specs: Vec<StatusContractSpec> = native_status_contract_rows()
        .into_iter()
        .filter_map(|row| StatusContractSpec::from_row(&row))
        .collect();

    specs.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    specs
}

/// Build status contract inventory payload.
#[must_use]
pub fn build_inventory_report(_workspace_root: &Path) -> Value {
    let specs = status_contract_specs();
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
