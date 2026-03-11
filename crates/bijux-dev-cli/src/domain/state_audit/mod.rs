//! Maintainer state audit report assembly.

use serde_json::{json, Value};

/// State path status inputs used by state audit assembly.
#[derive(Debug, Clone)]
pub struct StatePathStatusInput {
    /// Config file path status payload.
    pub config: Value,
    /// History file path status payload.
    pub history: Value,
    /// Plugin registry path status payload.
    pub plugins_registry: Value,
    /// Memory file path status payload.
    pub memory: Value,
}

/// Builds the maintainer state audit report payload.
#[must_use]
pub fn build_report(paths: StatePathStatusInput, corruption_health: Value) -> Value {
    json!({
        "state_truth_default": "bijux dev cli state-audit",
        "evidence_ids": [
            "EVIDENCE-1104-CONFIG-CORRUPTION",
            "EVIDENCE-1105-HISTORY-RESILIENCE",
            "EVIDENCE-1106-MEMORY-RESILIENCE"
        ],
        "paths": {
            "config": paths.config,
            "history": paths.history,
            "plugins_registry": paths.plugins_registry,
            "memory": paths.memory,
        },
        "corruption_health": corruption_health,
        "runtime": "dev-cli",
    })
}

/// Builds the maintainer state doctor report payload.
#[must_use]
pub fn build_doctor_report(diagnosis: Value) -> Value {
    json!({
        "runtime": "dev-cli",
        "state_truth_default": "bijux dev cli state-audit",
        "doctor": diagnosis,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{build_doctor_report, build_report, StatePathStatusInput};

    #[test]
    fn state_audit_report_shape_is_stable() {
        let report = build_report(
            StatePathStatusInput {
                config: json!({"path": "config"}),
                history: json!({"path": "history"}),
                plugins_registry: json!({"path": "plugins"}),
                memory: json!({"path": "memory"}),
            },
            json!({"status": "healthy"}),
        );
        assert!(report.get("paths").is_some());
        assert!(report.get("corruption_health").is_some());
    }

    #[test]
    fn state_doctor_report_shape_is_stable() {
        let report = build_doctor_report(json!({"status": "healthy"}));
        assert!(report.get("doctor").is_some());
    }
}
