//! Status contract execution runner.

use std::path::Path;

use serde_json::{json, Value};

use crate::contracts::maintenance::{generated_at_utc, run_native_status_contract};

use super::inventory::status_contract_specs;

fn find_spec(
    contract_id: Option<&str>,
    source_ref: Option<&str>,
) -> Option<super::model::StatusContractSpec> {
    let rows = status_contract_specs();
    if let Some(id) = contract_id {
        return rows.into_iter().find(|spec| spec.contract_id == id);
    }
    if let Some(source) = source_ref {
        return rows.into_iter().find(|spec| spec.source_ref.as_deref() == Some(source));
    }
    None
}

/// Run one status contract by id.
#[must_use]
pub fn run_contract(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_ref: Option<&str>,
    args: &[String],
) -> Value {
    if !args.is_empty() {
        return json!({
            "status": "failed",
            "contract_id": contract_id,
            "source_ref": source_ref,
            "error": "status contracts do not support passthrough arguments after '--'",
            "passthrough_args": args,
        });
    }

    if let Some(id) = contract_id {
        if let Some(result) = run_native_status_contract(workspace_root, id) {
            return result;
        }
    }

    let Some(spec) = find_spec(contract_id, source_ref) else {
        return json!({
            "status": "failed",
            "error": "status contract not found; pass --id with a known STATUS-CONTRACT-* value",
        });
    };

    if spec.implementation == "rust" {
        if let Some(result) = run_native_status_contract(workspace_root, &spec.contract_id) {
            return result;
        }
    }

    json!({
        "status": "failed",
        "contract_id": spec.contract_id,
        "implementation": spec.implementation,
        "error": "only rust-native status contract execution is supported",
    })
}

/// Run all status contracts, optionally filtered by kind.
#[must_use]
pub fn run_all_contracts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    if !args.is_empty() {
        return json!({
            "generated_at_utc": generated_at_utc(),
            "kind_filter": kind_filter.map(|kind| kind.to_ascii_lowercase()),
            "count": 0,
            "ok": 0,
            "failed": 1,
            "error": "status contracts do not support passthrough arguments after '--'",
            "passthrough_args": args,
            "results": [json!({
                "status": "failed",
                "error": "status contracts do not support passthrough arguments after '--'",
                "passthrough_args": args,
            })],
        });
    }

    let mut specs = status_contract_specs();
    if let Some(kind) = kind_filter {
        let kind = kind.to_ascii_lowercase();
        specs.retain(|spec| spec.kind.as_str() == kind);
    }

    let mut results = Vec::<Value>::new();
    let mut ok = 0usize;
    let mut failed = 0usize;

    for spec in specs {
        let result = run_contract(
            workspace_root,
            Some(spec.contract_id.as_str()),
            spec.source_ref.as_deref(),
            args,
        );
        if result.get("status").and_then(Value::as_str) == Some("ok") {
            ok += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }

    json!({
        "generated_at_utc": generated_at_utc(),
        "kind_filter": kind_filter.map(|kind| kind.to_ascii_lowercase()),
        "count": results.len(),
        "ok": ok,
        "failed": failed,
        "results": results,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{run_all_contracts, run_contract};

    #[test]
    fn run_contract_rejects_passthrough_args() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let payload = run_contract(&root, None, None, &["--unexpected".to_string()]);
        assert_eq!(payload["status"], "failed");
        assert!(payload["error"]
            .as_str()
            .is_some_and(|value| value.contains("passthrough arguments")));
    }

    #[test]
    fn run_all_contracts_rejects_passthrough_args() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let payload = run_all_contracts(&root, None, &["--unexpected".to_string()]);
        assert_eq!(payload["failed"], 1);
        assert!(payload["error"]
            .as_str()
            .is_some_and(|value| value.contains("passthrough arguments")));
    }
}
