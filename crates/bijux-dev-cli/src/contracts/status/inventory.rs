//! Status contract inventory registry.

use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

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

fn output_artifact_rows(workspace_root: &Path, outputs: &[String]) -> Vec<Value> {
    outputs
        .iter()
        .map(|output| {
            let artifact_path = workspace_root.join(output);
            json!({
                "path": output,
                "exists": artifact_path.exists(),
            })
        })
        .collect()
}

/// Build status contract inventory payload.
#[must_use]
pub fn build_inventory_report(workspace_root: &Path) -> Value {
    let specs = status_contract_specs();
    let mut kind_counts = BTreeMap::<String, usize>::new();
    let mut workspace_visible_outputs = 0usize;
    let mut workspace_missing_outputs = 0usize;
    let rows: Vec<Value> = specs
        .into_iter()
        .map(|spec| {
            *kind_counts
                .entry(spec.kind.as_str().to_string())
                .or_insert(0) += 1;
            let output_artifacts = output_artifact_rows(workspace_root, &spec.outputs);
            let missing_output_paths: Vec<String> = output_artifacts
                .iter()
                .filter(|item| item.get("exists") != Some(&Value::Bool(true)))
                .filter_map(|item| {
                    item.get("path")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
                .collect();
            let has_missing_outputs = !missing_output_paths.is_empty();
            workspace_missing_outputs += missing_output_paths.len();
            workspace_visible_outputs += output_artifacts
                .iter()
                .filter(|item| item.get("exists") == Some(&Value::Bool(true)))
                .count();
            let mut row = spec.to_row();
            if let Some(obj) = row.as_object_mut() {
                obj.insert(
                    "output_artifacts".to_string(),
                    Value::Array(output_artifacts),
                );
                obj.insert(
                    "missing_output_paths".to_string(),
                    json!(missing_output_paths),
                );
                obj.insert(
                    "workspace_outputs_ready".to_string(),
                    json!(!has_missing_outputs),
                );
            }
            row
        })
        .collect();

    json!({
        "id_policy": "STATUS-CONTRACT-<KIND>-<SLUG>",
        "kinds": kind_counts,
        "count": rows.len(),
        "generated_at_utc": generated_at_utc(),
        "workspace_visibility": {
            "visible_output_count": workspace_visible_outputs,
            "missing_output_count": workspace_missing_outputs,
        },
        "rows": rows,
    })
}
