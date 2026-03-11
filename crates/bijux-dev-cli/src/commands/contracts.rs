//! Maintainer contracts/schema report assembly.

use std::path::Path;

use serde_json::{json, Value};

use crate::status_contracts::{build_inventory_report, run_all_contracts};

fn contract_rows(schema_ids: &[String]) -> Vec<Value> {
    schema_ids
        .iter()
        .map(|schema| match schema.as_str() {
            "output-envelope-v1" => {
                json!({"name": "output-envelope", "schema": "output-envelope-v1", "version": "1.0.0"})
            }
            "error-envelope-v1" => {
                json!({"name": "error-envelope", "schema": "error-envelope-v1", "version": "1.0.0"})
            }
            "plugin-manifest-v1" => {
                json!({"name": "plugin-manifest", "schema": "plugin-manifest-v1", "version": "1.0.0"})
            }
            other => json!({"name": other, "schema": other, "version": "1.0.0"}),
        })
        .collect()
}

fn nextest_summary_line(total: usize, passed: usize, failed: usize) -> String {
    format!("Summary [contracts]: {total} total, {passed} passed, {failed} failed")
}

fn contracts_all_rows(inventory: &Value, run_results: &Value) -> Vec<Value> {
    let specs = inventory
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let results = run_results
        .get("results")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let row_count = specs.len().max(results.len());

    (0..row_count)
        .map(|idx| {
            let spec = specs.get(idx);
            let run = results.get(idx);

            let contract_id = spec
                .and_then(|row| row.get("contract_id"))
                .and_then(Value::as_str)
                .or_else(|| {
                    run.and_then(|row| row.get("contract_id"))
                        .and_then(Value::as_str)
                })
                .unwrap_or("UNKNOWN-CONTRACT-ID");
            let kind = spec
                .and_then(|row| row.get("kind"))
                .and_then(Value::as_str)
                .unwrap_or("status");
            let implementation = spec
                .and_then(|row| row.get("implementation"))
                .and_then(Value::as_str)
                .unwrap_or("rust");
            let run_status = run
                .and_then(|row| row.get("status"))
                .and_then(Value::as_str)
                .unwrap_or("failed");

            json!({
                "contract_id": contract_id,
                "kind": kind,
                "implementation": implementation,
                "status": if run_status == "ok" { "pass" } else { "fail" },
                "run": run.cloned().unwrap_or_else(|| json!({
                    "status": "failed",
                    "error": "missing execution result"
                })),
            })
        })
        .collect()
}

fn build_all_report_from_payloads(
    runtime_version: &str,
    inventory: &Value,
    run_results: &Value,
    kind_filter: Option<&str>,
) -> Value {
    let total = run_results
        .get("count")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    let passed = run_results
        .get("ok")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    let failed = run_results
        .get("failed")
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);
    let status = if failed == 0 { "pass" } else { "fail" };
    let contracts = contracts_all_rows(inventory, run_results);

    json!({
        "kind": "dev_cli_contracts_all_report_v1",
        "schema_version": "v1",
        "runtime_version": runtime_version,
        "generated_at_utc": run_results.get("generated_at_utc").cloned().unwrap_or(Value::Null),
        "mode": "all",
        "kind_filter": kind_filter.map(str::to_ascii_lowercase),
        "summary": {
            "status": status,
            "total": total,
            "passed": passed,
            "failed": failed,
            "nextest_style": nextest_summary_line(total, passed, failed),
        },
        "inventory": {
            "count": inventory.get("count").cloned().unwrap_or_else(|| Value::from(contracts.len())),
            "kinds": inventory.get("kinds").cloned().unwrap_or_else(|| json!({})),
        },
        "contracts": contracts,
    })
}

/// Builds the maintainer contracts/schema report envelope.
#[must_use]
pub fn build_report(runtime_version: &str) -> Value {
    let schema_ids = vec![
        "output-envelope-v1".to_string(),
        "error-envelope-v1".to_string(),
        "plugin-manifest-v1".to_string(),
    ];
    build_report_from_query(runtime_version, &schema_ids, "v1")
}

/// Builds the maintainer contracts/schema report envelope from routing query data.
#[must_use]
pub fn build_report_from_query(
    runtime_version: &str,
    schema_ids: &[String],
    schema_version: &str,
) -> Value {
    json!({
        "contracts": contract_rows(schema_ids),
        "schema_version": schema_version,
        "runtime_version": runtime_version,
    })
}

/// Builds full status-contract execution report for `dev cli contracts --all`.
#[must_use]
pub fn build_all_report(
    workspace_root: &Path,
    runtime_version: &str,
    kind_filter: Option<&str>,
) -> Value {
    let inventory = build_inventory_report(workspace_root);
    let run_results = run_all_contracts(workspace_root, kind_filter, &[]);
    build_all_report_from_payloads(runtime_version, &inventory, &run_results, kind_filter)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{build_all_report_from_payloads, build_report, build_report_from_query};

    #[test]
    fn contracts_report_shape_is_stable() {
        let report = build_report("0.1.0");
        assert!(report.get("contracts").is_some());
        assert_eq!(
            report
                .get("schema_version")
                .and_then(serde_json::Value::as_str),
            Some("v1")
        );
    }

    #[test]
    fn contracts_report_can_be_built_from_query_inputs() {
        let schema_ids = vec![
            "output-envelope-v1".to_string(),
            "error-envelope-v1".to_string(),
            "plugin-manifest-v1".to_string(),
        ];
        let report = build_report_from_query("0.1.0", &schema_ids, "v1");
        assert_eq!(report["contracts"].as_array().map_or(0, Vec::len), 3);
    }

    #[test]
    fn contracts_all_report_shape_is_stable() {
        let inventory = json!({
            "count": 2,
            "kinds": {"generate": 2},
            "rows": [
                {"contract_id": "STATUS-CONTRACT-GENERATE-ONE", "kind": "generate", "implementation": "rust"},
                {"contract_id": "STATUS-CONTRACT-GENERATE-TWO", "kind": "generate", "implementation": "rust"}
            ],
        });
        let run_results = json!({
            "generated_at_utc": "2026-03-11T00:00:00Z",
            "count": 2,
            "ok": 1,
            "failed": 1,
            "results": [
                {"status": "ok", "contract_id": "STATUS-CONTRACT-GENERATE-ONE"},
                {"status": "failed", "contract_id": "STATUS-CONTRACT-GENERATE-TWO", "error": "boom"}
            ],
        });

        let report =
            build_all_report_from_payloads("0.1.0", &inventory, &run_results, Some("generate"));
        assert_eq!(report["kind"], "dev_cli_contracts_all_report_v1");
        assert_eq!(report["summary"]["total"], 2);
        assert_eq!(report["summary"]["passed"], 1);
        assert_eq!(report["summary"]["failed"], 1);
        assert_eq!(report["summary"]["status"], "fail");
        assert!(report["summary"]["nextest_style"]
            .as_str()
            .unwrap_or_default()
            .contains("Summary [contracts]"));
        assert_eq!(report["contracts"].as_array().map_or(0, Vec::len), 2);
    }
}
