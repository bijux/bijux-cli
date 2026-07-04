use std::path::Path;

use serde_json::{json, Value};

use super::compliance::{build_flaky_tests_report, build_ignored_dag_tests_report};
use super::inventory::{generated_at_utc, write_json};

fn status_generator_rows() -> Vec<Value> {
    vec![
        json!({
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
            "command": "bijux-dev-cli maintenance generate --id GEN-STATUS-FLAKY-TEST-LABELS",
        }),
        json!({
            "generator_id": "GEN-STATUS-IGNORED-DAG-TESTS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/ignored_dag_tests.json"],
            "command": "bijux-dev-cli maintenance generate --id GEN-STATUS-IGNORED-DAG-TESTS",
        }),
    ]
}

fn build_status_generators_report() -> Value {
    let rows = status_generator_rows();
    json!({
        "id_policy": "GEN-STATUS-<GENERATOR-SLUG>",
        "generated_at_utc": generated_at_utc(),
        "count": rows.len(),
        "rows": rows,
    })
}

fn run_flaky_tests_generator(workspace_root: &Path) -> Value {
    let report = build_flaky_tests_report(workspace_root);
    let output_path = workspace_root.join("artifacts/status/flaky_tests.json");
    match write_json(&output_path, &report) {
        Ok(()) => json!({
            "status": "ok",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
        }),
        Err(err) => json!({
            "status": "failed",
            "generator_id": "GEN-STATUS-FLAKY-TEST-LABELS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/flaky_tests.json"],
            "error": err,
        }),
    }
}

fn run_ignored_dag_tests_generator(workspace_root: &Path) -> Value {
    let report = build_ignored_dag_tests_report(workspace_root);
    let output_path = workspace_root.join("artifacts/status/ignored_dag_tests.json");
    match write_json(&output_path, &report) {
        Ok(()) => json!({
            "status": "ok",
            "generator_id": "GEN-STATUS-IGNORED-DAG-TESTS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/ignored_dag_tests.json"],
        }),
        Err(err) => json!({
            "status": "failed",
            "generator_id": "GEN-STATUS-IGNORED-DAG-TESTS",
            "source_ref": Value::Null,
            "implementation": "rust",
            "outputs": ["artifacts/status/ignored_dag_tests.json"],
            "error": err,
        }),
    }
}

fn run_status_generator_entry(workspace_root: &Path, row: &Value) -> Value {
    let Some(generator_id) = row.get("generator_id").and_then(Value::as_str) else {
        return json!({"status": "failed", "error": "missing generator_id"});
    };
    if generator_id == "GEN-STATUS-FLAKY-TEST-LABELS" {
        return run_flaky_tests_generator(workspace_root);
    }
    if generator_id == "GEN-STATUS-IGNORED-DAG-TESTS" {
        return run_ignored_dag_tests_generator(workspace_root);
    }
    json!({
        "status": "failed",
        "generator_id": generator_id,
        "error": "generator is not rust-native",
    })
}

/// Builds `bijux-dev-cli maintenance generators` report payload.
#[must_use]
pub fn build_generators_report(_workspace_root: &Path) -> Value {
    build_status_generators_report()
}

/// Runs one status generator by stable id.
#[must_use]
pub fn run_generator(
    workspace_root: &Path,
    generator_id: Option<&str>,
    source_ref: Option<&str>,
) -> Value {
    if source_ref.is_some() {
        return json!({
            "status": "failed",
            "error": "source_ref lookup is unsupported for rust-native generators; pass --id",
        });
    }

    let rows = status_generator_rows();

    let Some(id) = generator_id else {
        return json!({
            "status": "failed",
            "error": "generator not found; pass --id with a known status generator",
        });
    };

    if let Some(row) =
        rows.into_iter().find(|row| row.get("generator_id").and_then(Value::as_str) == Some(id))
    {
        return run_status_generator_entry(workspace_root, &row);
    }

    json!({
        "status": "failed",
        "error": "generator not found; pass --id with a known status generator",
    })
}

/// Runs all status generators.
#[must_use]
pub fn run_all_generators(workspace_root: &Path) -> Value {
    let rows = status_generator_rows();
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
