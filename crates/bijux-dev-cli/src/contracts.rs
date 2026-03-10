//! Maintainer contracts/schema report assembly.

use serde_json::{json, Value};

/// Builds the maintainer contracts/schema report envelope.
#[must_use]
pub fn build_report(runtime_version: &str) -> Value {
    json!({
        "contracts": [
            {
                "name": "output-envelope",
                "schema": "output-envelope-v1",
                "version": "1.0.0",
            },
            {
                "name": "error-envelope",
                "schema": "error-envelope-v1",
                "version": "1.0.0",
            },
            {
                "name": "plugin-manifest",
                "schema": "plugin-manifest-v1",
                "version": "1.0.0",
            }
        ],
        "schema_version": "v1",
        "runtime_version": runtime_version,
    })
}

#[cfg(test)]
mod tests {
    use super::build_report;

    #[test]
    fn contracts_report_shape_is_stable() {
        let report = build_report("0.1.0");
        assert!(report.get("contracts").is_some());
        assert_eq!(report.get("schema_version").and_then(serde_json::Value::as_str), Some("v1"));
    }
}
