use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use super::native_status_contracts::{native_status_contract_rows, run_native_status_contract};
use super::support::{extract_artifact_paths, generated_at_utc};

fn build_status_contract_inventory_report(workspace_root: &Path) -> Value {
    let mut rows = Vec::<Value>::new();
    let mut kind_counts = BTreeMap::<String, usize>::new();
    for row in native_status_contract_rows() {
        let Some(kind) = row.get("kind").and_then(Value::as_str) else {
            continue;
        };
        *kind_counts.entry(kind.to_string()).or_insert(0) += 1;
        rows.push(row);
    }
    let known_ids = rows
        .iter()
        .filter_map(|row| row.get("contract_id").and_then(Value::as_str).map(ToString::to_string))
        .collect::<BTreeSet<_>>();
    let ci_text =
        fs::read_to_string(workspace_root.join(".github/workflows/ci.yml")).unwrap_or_default();
    let mut ci_ids = BTreeSet::<String>::new();
    for token in ci_text.split_whitespace() {
        let cleaned = token
            .trim_matches(|ch: char| !ch.is_ascii_uppercase() && !ch.is_ascii_digit() && ch != '-');
        if cleaned.starts_with("STATUS-CONTRACT-") {
            ci_ids.insert(cleaned.to_string());
        }
    }
    for id in ci_ids.difference(&known_ids) {
        let kind = if id.starts_with("STATUS-CONTRACT-GENERATE-") {
            "generate"
        } else if id.starts_with("STATUS-CONTRACT-CHECK-") {
            "check"
        } else if id.starts_with("STATUS-CONTRACT-ENFORCE-") {
            "enforce"
        } else if id.starts_with("STATUS-CONTRACT-WARN-") {
            "warn"
        } else if id.starts_with("STATUS-CONTRACT-RUN-") {
            "run"
        } else {
            "status"
        };
        *kind_counts.entry(kind.to_string()).or_insert(0) += 1;
        rows.push(json!({
            "contract_id": id,
            "kind": kind,
            "source_script": Value::Null,
            "implementation": "rust-compat",
            "outputs": [],
            "command": format!("bijux dev cli scripts status run --id {id}"),
        }));
    }
    rows.sort_by(|left, right| {
        left.get("contract_id")
            .and_then(Value::as_str)
            .cmp(&right.get("contract_id").and_then(Value::as_str))
    });
    json!({
        "id_policy": "STATUS-CONTRACT-<KIND>-<SLUG>",
        "kinds": kind_counts,
        "count": rows.len(),
        "generated_at_utc": generated_at_utc(),
        "rows": rows,
    })
}

fn run_python_status_contract(
    workspace_root: &Path,
    source_script: &str,
    contract_id: &str,
    kind: &str,
    args: &[String],
) -> Value {
    let script = workspace_root.join(source_script);
    let source = fs::read_to_string(&script).unwrap_or_default();
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

fn find_status_contract_row(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_script: Option<&str>,
) -> Option<Value> {
    let rows = build_status_contract_inventory_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(id) = contract_id {
        return rows
            .into_iter()
            .find(|row| row.get("contract_id").and_then(Value::as_str) == Some(id));
    }
    if let Some(source) = source_script {
        return rows
            .into_iter()
            .find(|row| row.get("source_script").and_then(Value::as_str) == Some(source));
    }
    None
}

/// Builds `dev cli scripts status inventory` report payload.
#[must_use]
pub fn build_status_contracts_report(workspace_root: &Path) -> Value {
    build_status_contract_inventory_report(workspace_root)
}

/// Backward-compatible alias for legacy callsites.
#[must_use]
pub fn build_status_scripts_report(workspace_root: &Path) -> Value {
    build_status_contracts_report(workspace_root)
}

/// Runs one status contract by stable id or legacy source alias.
#[must_use]
pub fn run_status_contract(
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
    let Some(row) = find_status_contract_row(workspace_root, contract_id, source_script) else {
        return json!({
            "status": "failed",
            "error": "status contract not found; pass --id with a known STATUS-CONTRACT-* value",
        });
    };
    let contract_id = row.get("contract_id").and_then(Value::as_str).unwrap_or("unknown");
    let source_script = row.get("source_script").and_then(Value::as_str).unwrap_or("");
    let kind = row.get("kind").and_then(Value::as_str).unwrap_or("unknown");
    run_python_status_contract(workspace_root, source_script, contract_id, kind, args)
}

/// Backward-compatible alias for legacy callsites.
#[must_use]
pub fn run_status_script(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_script: Option<&str>,
    args: &[String],
) -> Value {
    run_status_contract(workspace_root, contract_id, source_script, args)
}

/// Runs all status contracts, optionally filtered by kind.
#[must_use]
pub fn run_all_status_contracts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    let mut rows = build_status_contract_inventory_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if let Some(kind) = kind_filter {
        let kind = kind.to_ascii_lowercase();
        rows.retain(|row| row.get("kind").and_then(Value::as_str).is_some_and(|item| item == kind));
    }

    let mut results = Vec::<Value>::new();
    let mut ok = 0usize;
    let mut failed = 0usize;
    for row in rows {
        let contract_id = row.get("contract_id").and_then(Value::as_str);
        let source_script = row.get("source_script").and_then(Value::as_str);
        let result = run_status_contract(workspace_root, contract_id, source_script, args);
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

/// Backward-compatible alias for legacy callsites.
#[must_use]
pub fn run_all_status_scripts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    run_all_status_contracts(workspace_root, kind_filter, args)
}
