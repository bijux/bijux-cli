//! Maintainer package health report assembly.

use serde_json::{json, Value};

/// Builds the maintainer package health report payload.
#[must_use]
pub fn build_report(current_rust_state: Value) -> Value {
    let assumptions = vec![
        "config/history/plugins state defaults under HOME/.bijux unless explicit overrides are set",
        "XDG-style HOME locations are treated as regular HOME roots for compatibility paths",
        "PATH order decides active bijux binary and all ambiguity diagnostics derive from that order",
        "completion files are generated under shell-specific directories derived from HOME",
        "state bootstrap must create missing directories and report explicit errors for unwritable roots",
    ];
    json!({
        "package_entrypoints": current_rust_state.get("package_entrypoints").cloned().unwrap_or_else(|| json!([])),
        "runtime_identity_rules": current_rust_state.get("runtime_identity_rules").cloned().unwrap_or_else(|| json!({})),
        "install_state_assumptions": assumptions,
        "install_state_assumption_help": "Use `bijux dev cli package-health --format json` to audit install-state assumptions and entrypoint contracts.",
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::build_report;

    #[test]
    fn package_health_report_shape_is_stable() {
        let report = build_report(json!({"package_entrypoints": [], "runtime_identity_rules": {}}));
        assert!(report.get("package_entrypoints").is_some());
        assert!(report.get("runtime_identity_rules").is_some());
        assert!(report.get("install_state_assumptions").is_some());
    }
}
