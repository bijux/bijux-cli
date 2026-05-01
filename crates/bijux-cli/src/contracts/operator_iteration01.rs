use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Compact root help entrypoint contract for operator-first discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompactHelpEntryPointV1 {
    /// Core built-in root routes shown to operators.
    pub built_in_routes: Vec<String>,
    /// Mounted official app root routes.
    pub mounted_app_routes: Vec<String>,
    /// Plugin namespace routes.
    pub plugin_routes: Vec<String>,
    /// Suggested next commands from the root screen.
    pub next_commands: Vec<String>,
}

/// Build compact root help with required operator routes and deterministic ordering.
pub fn build_compact_operator_help_entrypoint(
    built_in_routes: &[&str],
    mounted_app_routes: &[&str],
    plugin_routes: &[&str],
    next_commands: &[&str],
) -> Result<CompactHelpEntryPointV1, String> {
    let normalized_built_ins = unique_sorted_non_empty(built_in_routes)?;
    for required in ["dag", "config", "doctor", "plugins"] {
        if !normalized_built_ins.iter().any(|entry| entry == required) {
            return Err(format!("built_in_routes missing required root command `{required}`"));
        }
    }
    Ok(CompactHelpEntryPointV1 {
        built_in_routes: normalized_built_ins,
        mounted_app_routes: unique_sorted_non_empty(mounted_app_routes)?,
        plugin_routes: unique_sorted_non_empty(plugin_routes)?,
        next_commands: unique_sorted_non_empty(next_commands)?,
    })
}

fn unique_sorted_non_empty(values: &[&str]) -> Result<Vec<String>, String> {
    let mut normalized: Vec<String> = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();
    normalized.sort();
    normalized.dedup();
    if normalized.is_empty() {
        return Err("route/help collections cannot be empty".to_string());
    }
    Ok(normalized)
}

/// Script-stable command envelope contract shared across runtime and bridge entrypoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScriptStableCommandEnvelopeV1 {
    /// Schema version for stable machine parsing.
    pub schema_version: String,
    /// Canonical command path string.
    pub command: String,
    /// Success marker for script decisions.
    pub ok: bool,
    /// Stable command result code.
    pub code: String,
    /// Command payload.
    pub data: Value,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Fatal errors (empty on success).
    pub errors: Vec<String>,
}

/// Build script-stable command envelope with exact required fields.
pub fn build_script_stable_command_envelope(
    schema_version: &str,
    command: &str,
    ok: bool,
    code: &str,
    data: Value,
    warnings: Vec<String>,
    errors: Vec<String>,
) -> Result<ScriptStableCommandEnvelopeV1, String> {
    if schema_version.trim().is_empty() {
        return Err("schema_version cannot be empty".to_string());
    }
    if command.trim().is_empty() {
        return Err("command cannot be empty".to_string());
    }
    if code.trim().is_empty() {
        return Err("code cannot be empty".to_string());
    }
    if ok && !errors.is_empty() {
        return Err("success envelope cannot contain errors".to_string());
    }
    if !ok && errors.is_empty() {
        return Err("failure envelope must contain at least one error".to_string());
    }
    Ok(ScriptStableCommandEnvelopeV1 {
        schema_version: schema_version.to_string(),
        command: command.to_string(),
        ok,
        code: code.to_string(),
        data,
        warnings,
        errors,
    })
}

/// Stable actionable failure classes for CLI/runtime failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ActionableFailureClassV1 {
    Parse,
    Config,
    Plugin,
    Dag,
    Io,
    Runtime,
}

/// Useful error envelope with explicit remediation and evidence pointers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ActionableErrorEnvelopeV1 {
    /// Failure class for coarse machine triage.
    pub failure_class: ActionableFailureClassV1,
    /// Stable error code.
    pub code: String,
    /// Human-readable failure message.
    pub message: String,
    /// Concrete remediation action for operators.
    pub remediation: String,
    /// Optional evidence pointer for logs or run artifacts.
    pub evidence_pointer: Option<String>,
}

/// Build actionable error envelope with required diagnostic fields.
pub fn build_actionable_error_envelope(
    failure_class: ActionableFailureClassV1,
    code: &str,
    message: &str,
    remediation: &str,
    evidence_pointer: Option<&str>,
) -> Result<ActionableErrorEnvelopeV1, String> {
    if code.trim().is_empty() {
        return Err("code cannot be empty".to_string());
    }
    if message.trim().is_empty() {
        return Err("message cannot be empty".to_string());
    }
    if remediation.trim().is_empty() {
        return Err("remediation cannot be empty".to_string());
    }
    if evidence_pointer.is_some_and(|value| value.trim().is_empty()) {
        return Err("evidence_pointer cannot be blank when present".to_string());
    }
    Ok(ActionableErrorEnvelopeV1 {
        failure_class,
        code: code.to_string(),
        message: message.to_string(),
        remediation: remediation.to_string(),
        evidence_pointer: evidence_pointer.map(ToString::to_string),
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        build_actionable_error_envelope, build_compact_operator_help_entrypoint,
        build_script_stable_command_envelope, ActionableFailureClassV1,
    };

    #[test]
    fn g001_compact_help_entrypoint_requires_core_operator_routes() {
        let report = build_compact_operator_help_entrypoint(
            &["doctor", "plugins", "dag", "config"],
            &["atlas", "genomics"],
            &["community-tools"],
            &["bijux dag --help", "bijux doctor"],
        )
        .expect("compact help should build");
        assert_eq!(report.built_in_routes, vec!["config", "dag", "doctor", "plugins"]);
        assert_eq!(report.mounted_app_routes, vec!["atlas", "genomics"]);
    }

    #[test]
    fn g002_script_stable_command_envelope_uses_required_machine_fields() {
        let envelope = build_script_stable_command_envelope(
            "command-envelope-v1",
            "bijux dag plan",
            true,
            "ok",
            json!({"plan_id":"plan-001"}),
            vec!["using cached descriptor".to_string()],
            Vec::new(),
        )
        .expect("script-stable envelope should build");
        assert_eq!(envelope.schema_version, "command-envelope-v1");
        assert!(envelope.ok);
        assert_eq!(envelope.errors.len(), 0);
        assert_eq!(envelope.warnings.len(), 1);
    }

    #[test]
    fn g003_actionable_error_envelope_includes_remediation_and_evidence_pointer() {
        let error = build_actionable_error_envelope(
            ActionableFailureClassV1::Plugin,
            "plugin_manifest_invalid",
            "plugin manifest rejected",
            "run `bijux plugins inspect <name>` and correct manifest fields",
            Some("artifacts/cli/errors/plugin-manifest-invalid.log"),
        )
        .expect("actionable error should build");
        assert_eq!(error.code, "plugin_manifest_invalid");
        assert_eq!(
            error.evidence_pointer.as_deref(),
            Some("artifacts/cli/errors/plugin-manifest-invalid.log")
        );
    }
}
