//! Maintainer contracts/schema report assembly.

use serde_json::{json, Value};

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

#[cfg(test)]
mod tests {
    use super::{build_report, build_report_from_query};

    #[test]
    fn contracts_report_shape_is_stable() {
        let report = build_report("0.1.0");
        assert!(report.get("contracts").is_some());
        assert_eq!(report.get("schema_version").and_then(serde_json::Value::as_str), Some("v1"));
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
}
