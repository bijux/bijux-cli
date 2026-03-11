use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::{json, Value};

use super::compliance::build_flaky_tests_report;
use super::shared::{
    extract_artifact_paths, generated_at_utc, status_generator_id, status_generator_sources,
    write_json,
};

fn build_status_generators_report(workspace_root: &Path) -> Value {
    let mut rows: Vec<Value> = status_generator_sources(workspace_root)
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(workspace_root.join(&path)).unwrap_or_default();
            let outputs = extract_artifact_paths(&source);
            let id = status_generator_id(&path);
            json!({
                "generator_id": id,
                "source_script": path,
                "implementation": "python-script",
                "outputs": outputs,
                "command": format!("bijux dev cli scripts generate --id {id}"),
            })
        })
        .collect();
    rows.push(json!({
        "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
        "source_script": Value::Null,
        "implementation": "rust",
        "outputs": ["artifacts/status/flaky_tests.json"],
        "command": "bijux dev cli scripts generate --id GEN-STATUS-FLAKY-TEST-LABELS",
    }));
    rows.sort_by(|left, right| {
        left.get("generator_id")
            .and_then(Value::as_str)
            .cmp(&right.get("generator_id").and_then(Value::as_str))
    });
    json!({
        "id_policy": "GEN-STATUS-<GENERATOR-SLUG>",
        "generated_at_utc": generated_at_utc(),
        "count": rows.len(),
        "rows": rows,
    })
}

fn run_python_generator(workspace_root: &Path, source_script: &str, outputs: &[String]) -> Value {
    let script = workspace_root.join(source_script);
    let executed = Command::new("python3").arg(&script).current_dir(workspace_root).output();
    match executed {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(1);
            let status = if output.status.success() { "ok" } else { "failed" };
            json!({
                "status": status,
                "generator_id": status_generator_id(source_script),
                "source_script": source_script,
                "implementation": "python-script",
                "exit_code": exit_code,
                "stdout": String::from_utf8_lossy(&output.stdout).trim(),
                "stderr": String::from_utf8_lossy(&output.stderr).trim(),
                "outputs": outputs,
            })
        }
        Err(err) => json!({
            "status": "failed",
            "generator_id": status_generator_id(source_script),
            "source_script": source_script,
            "implementation": "python-script",
            "error": format!("failed to launch python3 for {source_script}: {err}"),
            "outputs": outputs,
        }),
    }
}

fn run_flaky_tests_generator(workspace_root: &Path) -> Value {
    let report = build_flaky_tests_report(workspace_root);
    let output_path = workspace_root.join("artifacts/status/flaky_tests.json");
    match write_json(&output_path, &report) {
        Ok(()) => json!({
            "status": "ok",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
        }),
        Err(err) => json!({
            "status": "failed",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_script": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
            "error": err,
        }),
    }
}

fn run_status_generator_entry(workspace_root: &Path, row: &Value) -> Value {
    let Some(generator_id) = row.get("generator_id").and_then(Value::as_str) else {
        return json!({"status": "failed", "error": "missing generator_id"});
    };
    let outputs: Vec<String> = row
        .get("outputs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(ToString::to_string))
        .collect();
    if generator_id == "GEN-STATUS-FLAKY-TEST-LABELS" {
        return run_flaky_tests_generator(workspace_root);
    }
    let Some(source_script) = row.get("source_script").and_then(Value::as_str) else {
        return json!({
            "status": "failed",
            "generator_id": generator_id,
            "error": "missing source_script for python generator",
        });
    };
    run_python_generator(workspace_root, source_script, &outputs)
}

/// Builds `dev cli scripts generators` report payload.
#[must_use]
pub fn build_generators_report(workspace_root: &Path) -> Value {
    build_status_generators_report(workspace_root)
}

/// Runs one status generator by stable id or source path.
#[must_use]
pub fn run_generator(
    workspace_root: &Path,
    generator_id: Option<&str>,
    source_script: Option<&str>,
) -> Value {
    let rows = build_status_generators_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let selection = if let Some(id) = generator_id {
        rows.into_iter().find(|row| row.get("generator_id").and_then(Value::as_str) == Some(id))
    } else if let Some(source) = source_script {
        rows.into_iter()
            .find(|row| row.get("source_script").and_then(Value::as_str) == Some(source))
    } else {
        None
    };

    if let Some(row) = selection {
        return run_status_generator_entry(workspace_root, &row);
    }
    json!({
        "status": "failed",
        "error": "generator not found; pass --id or --source with a known status generator",
    })
}

/// Runs all status generators.
#[must_use]
pub fn run_all_generators(workspace_root: &Path) -> Value {
    let rows = build_status_generators_report(workspace_root)
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut results = Vec::<Value>::new();
    let mut ok = 0usize;
    let mut failed = 0usize;
    for row in rows {
        let result = run_status_generator_entry(workspace_root, &row);
        if result.get("status").and_then(Value::as_str) == Some("ok") {
            ok += 1;
        } else {
            failed += 1;
        }
        results.push(result);
    }
    json!({
        "generated_at_utc": generated_at_utc(),
        "count": results.len(),
        "ok": ok,
        "failed": failed,
        "results": results,
    })
}
