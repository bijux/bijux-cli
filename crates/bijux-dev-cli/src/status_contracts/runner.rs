//! Status contract execution runner.

use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use crate::contract_engine::maintenance::{
    extract_artifact_paths, generated_at_utc, run_native_status_contract,
};

use super::registry::status_contract_specs;

fn find_spec(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_script: Option<&str>,
) -> Option<super::spec::StatusContractSpec> {
    let rows = status_contract_specs(workspace_root);
    if let Some(id) = contract_id {
        return rows.into_iter().find(|spec| spec.contract_id == id);
    }
    if let Some(source) = source_script {
        return rows
            .into_iter()
            .find(|spec| spec.source_script.as_deref() == Some(source));
    }
    None
}

fn run_python_contract(
    workspace_root: &Path,
    source_script: &str,
    contract_id: &str,
    kind: &str,
    args: &[String],
) -> Value {
    let script = workspace_root.join(source_script);
    let source = std::fs::read_to_string(&script).unwrap_or_default();
    let outputs = extract_artifact_paths(&source);
    let executed =
        Command::new("python3").arg(&script).args(args).current_dir(workspace_root).output();

    match executed {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let status = if output.status.success() { "ok" } else { "failed" };
            json!({
                "status": status,
                "contract_id": contract_id,
                "kind": kind,
                "source_script": source_script,
                "implementation": "python-script",
                "args": args,
                "outputs": outputs,
                "exit_code": exit_code,
                "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
            })
        }
        Err(err) => json!({
            "status": "failed",
            "contract_id": contract_id,
            "kind": kind,
            "source_script": source_script,
            "implementation": "python-script",
            "args": args,
            "outputs": outputs,
            "error": format!("failed to launch python3 for {source_script}: {err}"),
        }),
    }
}

/// Run one status contract by id or source script fallback.
#[must_use]
pub fn run_contract(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_script: Option<&str>,
    args: &[String],
) -> Value {
    if let Some(id) = contract_id {
        if let Some(result) = run_native_status_contract(workspace_root, id) {
            return result;
        }
    }

    let Some(spec) = find_spec(workspace_root, contract_id, source_script) else {
        return json!({
            "status": "failed",
            "error": "status contract not found; pass --id with a known STATUS-CONTRACT-* value",
        });
    };

    let source_script = spec.source_script.unwrap_or_default();
    run_python_contract(
        workspace_root,
        &source_script,
        &spec.contract_id,
        spec.kind.as_str(),
        args,
    )
}

/// Run all status contracts, optionally filtered by kind.
#[must_use]
pub fn run_all_contracts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    let mut specs = status_contract_specs(workspace_root);
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
            spec.source_script.as_deref(),
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
