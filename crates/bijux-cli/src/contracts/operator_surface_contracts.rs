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

/// Explain contract with route target and runtime requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandExplainV1 {
    /// Command path being explained.
    pub command: String,
    /// Route target key used by dispatch.
    pub route_target: String,
    /// Handler source (`builtin`, `official-app`, `plugin`).
    pub handler_source: String,
    /// Output schema version identifier.
    pub output_schema: String,
    /// Side-effect class (`read-only`, `writes-config`, `writes-run`, `executes-adapter`, `destructive`).
    pub side_effect_class: String,
    /// Required config keys for successful execution.
    pub required_config_keys: Vec<String>,
}

/// Build command explain record for built-ins, official app routes, and plugins.
pub fn build_command_explain_record(
    command: &str,
    route_target: &str,
    handler_source: &str,
    output_schema: &str,
    side_effect_class: &str,
    required_config_keys: &[&str],
) -> Result<CommandExplainV1, String> {
    for (field, value) in [
        ("command", command),
        ("route_target", route_target),
        ("handler_source", handler_source),
        ("output_schema", output_schema),
        ("side_effect_class", side_effect_class),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{field} cannot be empty"));
        }
    }
    let mut required: Vec<String> = required_config_keys
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect();
    required.sort();
    required.dedup();
    Ok(CommandExplainV1 {
        command: command.to_string(),
        route_target: route_target.to_string(),
        handler_source: handler_source.to_string(),
        output_schema: output_schema.to_string(),
        side_effect_class: side_effect_class.to_string(),
        required_config_keys: required,
    })
}

/// Output mode matrix row for one command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OutputModeParityEntryV1 {
    /// Command path key.
    pub command: String,
    /// Supported output modes.
    pub supported_modes: Vec<String>,
}

/// Output mode parity report across built-in and mounted routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OutputModeParityReportV1 {
    /// Per-command mode support entries.
    pub entries: Vec<OutputModeParityEntryV1>,
    /// Commands missing one or more required modes.
    pub missing_mode_commands: Vec<String>,
    /// Whether parity is complete.
    pub parity_complete: bool,
}

/// Evaluate output mode parity across commands.
pub fn evaluate_output_mode_parity(
    entries: Vec<OutputModeParityEntryV1>,
) -> OutputModeParityReportV1 {
    let required_modes = ["human", "json", "jsonl", "artifact-output"];
    let mut ordered_entries = entries;
    ordered_entries.sort_by(|left, right| left.command.cmp(&right.command));
    let mut missing = Vec::new();
    for entry in &ordered_entries {
        let mut modes = entry.supported_modes.clone();
        modes.sort();
        modes.dedup();
        let has_all_required =
            required_modes.iter().all(|mode| modes.iter().any(|value| value == mode));
        if !has_all_required {
            missing.push(entry.command.clone());
        }
    }
    OutputModeParityReportV1 {
        entries: ordered_entries,
        missing_mode_commands: missing.clone(),
        parity_complete: missing.is_empty(),
    }
}

/// Install diagnostic component status entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InstallDiagnosticComponentV1 {
    /// Component key (`binary_path`, `path_resolution`, `python_bridge`, ...).
    pub component: String,
    /// Component health status.
    pub healthy: bool,
    /// Short detail for operator remediation.
    pub detail: String,
}

/// Root install diagnosis bundle across runtime-critical components.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct InstallDiagnosisBundleV1 {
    /// Evaluated components.
    pub components: Vec<InstallDiagnosticComponentV1>,
    /// Failed components list.
    pub failing_components: Vec<String>,
    /// Healthy summary marker.
    pub healthy_install: bool,
}

/// Build one actionable install diagnosis bundle.
pub fn build_install_diagnosis_bundle(
    components: Vec<InstallDiagnosticComponentV1>,
) -> Result<InstallDiagnosisBundleV1, String> {
    if components.is_empty() {
        return Err("components cannot be empty".to_string());
    }
    let mut ordered_components = components;
    ordered_components.sort_by(|left, right| left.component.cmp(&right.component));
    let failing_components: Vec<String> = ordered_components
        .iter()
        .filter(|entry| !entry.healthy)
        .map(|entry| entry.component.clone())
        .collect();
    Ok(InstallDiagnosisBundleV1 {
        components: ordered_components,
        failing_components: failing_components.clone(),
        healthy_install: failing_components.is_empty(),
    })
}

/// Route registry entry used for completion generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompletionRouteEntryV1 {
    /// Command string.
    pub command: String,
    /// Whether command is hidden from normal listing.
    pub hidden: bool,
    /// Whether command is deprecated.
    pub deprecated: bool,
    /// Whether command is stale and no longer routed.
    pub stale: bool,
}

/// Generated completion snapshot from registry routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompletionSnapshotV1 {
    /// Shell identifier (`bash`, `zsh`, `fish`).
    pub shell: String,
    /// Commands included in completion output.
    pub commands: Vec<String>,
}

/// Build completion snapshot from route registry with stale/deprecated filtering.
pub fn build_completion_snapshot_from_registry(
    shell: &str,
    entries: Vec<CompletionRouteEntryV1>,
    include_deprecated: bool,
) -> Result<CompletionSnapshotV1, String> {
    if shell.trim().is_empty() {
        return Err("shell cannot be empty".to_string());
    }
    let mut commands: Vec<String> = entries
        .into_iter()
        .filter(|entry| !entry.hidden)
        .filter(|entry| !entry.stale)
        .filter(|entry| include_deprecated || !entry.deprecated)
        .map(|entry| entry.command)
        .collect();
    commands.sort();
    commands.dedup();
    Ok(CompletionSnapshotV1 { shell: shell.to_string(), commands })
}

/// Command side-effect classes used for preview and explain surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CommandSideEffectClassV1 {
    ReadOnly,
    WritesConfig,
    WritesRun,
    ExecutesAdapter,
    Destructive,
}

/// Side-effect preview record for a command dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandSideEffectPreviewV1 {
    /// Command being evaluated.
    pub command: String,
    /// Classified side-effect class.
    pub side_effect_class: CommandSideEffectClassV1,
    /// Whether confirmation should be required before dispatch.
    pub requires_confirmation: bool,
}

/// Classify command side-effects for dispatch safety previews.
pub fn classify_command_side_effect(command: &str) -> Result<CommandSideEffectPreviewV1, String> {
    if command.trim().is_empty() {
        return Err("command cannot be empty".to_string());
    }
    let normalized = command.trim().to_ascii_lowercase();
    let class = if normalized.contains("wipe") || normalized.contains("delete") {
        CommandSideEffectClassV1::Destructive
    } else if normalized.contains("plugins install")
        || normalized.contains("plugins uninstall")
        || normalized.contains("dag run")
    {
        CommandSideEffectClassV1::WritesRun
    } else if normalized.contains("config set") || normalized.contains("config unset") {
        CommandSideEffectClassV1::WritesConfig
    } else if normalized.contains("adapter") || normalized.contains("exec") {
        CommandSideEffectClassV1::ExecutesAdapter
    } else {
        CommandSideEffectClassV1::ReadOnly
    };
    let requires_confirmation = matches!(
        class,
        CommandSideEffectClassV1::WritesRun
            | CommandSideEffectClassV1::ExecutesAdapter
            | CommandSideEffectClassV1::Destructive
    );
    Ok(CommandSideEffectPreviewV1 {
        command: command.to_string(),
        side_effect_class: class,
        requires_confirmation,
    })
}

/// Rust/Python bridge parity entry for one command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PythonBridgeParityEntryV1 {
    /// Command under comparison.
    pub command: String,
    /// Rust machine envelope payload.
    pub rust_machine_output: String,
    /// Python bridge machine envelope payload.
    pub python_machine_output: String,
}

/// Parity report for Rust runtime versus Python bridge command envelopes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PythonBridgeParityReportV1 {
    /// Compared command entries.
    pub entries: Vec<PythonBridgeParityEntryV1>,
    /// Commands with output mismatches.
    pub mismatched_commands: Vec<String>,
    /// Whether parity is exact across all compared commands.
    pub parity_exact: bool,
}

/// Build Python bridge parity report from command matrix outputs.
pub fn build_python_bridge_command_parity_report(
    entries: Vec<PythonBridgeParityEntryV1>,
) -> PythonBridgeParityReportV1 {
    let mut ordered_entries = entries;
    ordered_entries.sort_by(|left, right| left.command.cmp(&right.command));
    let mismatched_commands: Vec<String> = ordered_entries
        .iter()
        .filter(|entry| entry.rust_machine_output != entry.python_machine_output)
        .map(|entry| entry.command.clone())
        .collect();
    PythonBridgeParityReportV1 {
        entries: ordered_entries,
        mismatched_commands: mismatched_commands.clone(),
        parity_exact: mismatched_commands.is_empty(),
    }
}

/// Official app route descriptor used during discovery and conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OfficialAppRouteDescriptorV1 {
    /// Route namespace key.
    pub namespace: String,
    /// Descriptor identifier/hash.
    pub descriptor_id: String,
    /// Priority (larger value wins).
    pub priority: i32,
}

/// Official app discovery report with deterministic conflict outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OfficialAppDiscoveryReportV1 {
    /// Winning descriptor per namespace.
    pub winners: Vec<OfficialAppRouteDescriptorV1>,
    /// Refused plugin namespace shadow attempts.
    pub refused_plugin_shadows: Vec<String>,
    /// Refused PATH shim shadow attempts.
    pub refused_path_shims: Vec<String>,
}

/// Resolve official app discovery conflicts deterministically by namespace + priority.
pub fn build_official_app_discovery_report(
    descriptors: Vec<OfficialAppRouteDescriptorV1>,
    plugin_shadow_attempts: Vec<String>,
    path_shim_attempts: Vec<String>,
) -> OfficialAppDiscoveryReportV1 {
    use std::collections::BTreeMap;

    let mut winners_by_namespace: BTreeMap<String, OfficialAppRouteDescriptorV1> = BTreeMap::new();
    for descriptor in descriptors {
        match winners_by_namespace.get(&descriptor.namespace) {
            Some(existing) if existing.priority >= descriptor.priority => {}
            _ => {
                winners_by_namespace.insert(descriptor.namespace.clone(), descriptor);
            }
        }
    }
    OfficialAppDiscoveryReportV1 {
        winners: winners_by_namespace.into_values().collect(),
        refused_plugin_shadows: plugin_shadow_attempts,
        refused_path_shims: path_shim_attempts,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        build_actionable_error_envelope, build_command_explain_record,
        build_compact_operator_help_entrypoint, build_completion_snapshot_from_registry,
        build_install_diagnosis_bundle, build_official_app_discovery_report,
        build_python_bridge_command_parity_report, build_script_stable_command_envelope,
        classify_command_side_effect, evaluate_output_mode_parity, ActionableFailureClassV1,
        CompletionRouteEntryV1, InstallDiagnosticComponentV1, OfficialAppRouteDescriptorV1,
        OutputModeParityEntryV1, PythonBridgeParityEntryV1,
    };

    #[test]
    fn compact_help_entrypoint_requires_core_operator_routes() {
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
    fn script_stable_command_envelope_uses_required_machine_fields() {
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
    fn actionable_error_envelope_includes_remediation_and_evidence_pointer() {
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

    #[test]
    fn command_explain_includes_route_and_side_effect_contract() {
        let record = build_command_explain_record(
            "bijux dag run",
            "dag.runtime.run",
            "official-app",
            "dag-run-envelope-v1",
            "writes-run",
            &["dag.run_root", "dag.cache_root"],
        )
        .expect("explain contract should build");
        assert_eq!(record.route_target, "dag.runtime.run");
        assert_eq!(record.handler_source, "official-app");
        assert_eq!(record.required_config_keys, vec!["dag.cache_root", "dag.run_root"]);
    }

    #[test]
    fn output_mode_parity_reports_commands_missing_required_modes() {
        let report = evaluate_output_mode_parity(vec![
            OutputModeParityEntryV1 {
                command: "bijux dag run".to_string(),
                supported_modes: vec![
                    "human".to_string(),
                    "json".to_string(),
                    "jsonl".to_string(),
                    "artifact-output".to_string(),
                ],
            },
            OutputModeParityEntryV1 {
                command: "bijux doctor".to_string(),
                supported_modes: vec!["human".to_string(), "json".to_string()],
            },
        ]);
        assert!(!report.parity_complete);
        assert_eq!(report.missing_mode_commands, vec!["bijux doctor"]);
    }

    #[test]
    fn install_diagnostics_bundle_identifies_failing_component() {
        let bundle = build_install_diagnosis_bundle(vec![
            InstallDiagnosticComponentV1 {
                component: "binary_path".to_string(),
                healthy: true,
                detail: "resolved /usr/local/bin/bijux".to_string(),
            },
            InstallDiagnosticComponentV1 {
                component: "python_bridge".to_string(),
                healthy: false,
                detail: "missing importable python bridge package".to_string(),
            },
        ])
        .expect("diagnosis bundle should build");
        assert!(!bundle.healthy_install);
        assert_eq!(bundle.failing_components, vec!["python_bridge"]);
    }

    #[test]
    fn completion_snapshot_excludes_hidden_stale_and_deprecated_routes() {
        let snapshot = build_completion_snapshot_from_registry(
            "zsh",
            vec![
                CompletionRouteEntryV1 {
                    command: "bijux dag run".to_string(),
                    hidden: false,
                    deprecated: false,
                    stale: false,
                },
                CompletionRouteEntryV1 {
                    command: "bijux dag old-run".to_string(),
                    hidden: false,
                    deprecated: true,
                    stale: false,
                },
                CompletionRouteEntryV1 {
                    command: "bijux secret route".to_string(),
                    hidden: true,
                    deprecated: false,
                    stale: false,
                },
            ],
            false,
        )
        .expect("completion snapshot should build");
        assert_eq!(snapshot.commands, vec!["bijux dag run"]);
    }

    #[test]
    fn side_effect_classification_marks_risky_dispatches() {
        let preview =
            classify_command_side_effect("bijux dag run --graph sample.json").expect("classify");
        assert!(preview.requires_confirmation);
        assert_eq!(preview.command, "bijux dag run --graph sample.json");
    }

    #[test]
    fn python_bridge_parity_report_detects_machine_output_drift() {
        let report = build_python_bridge_command_parity_report(vec![
            PythonBridgeParityEntryV1 {
                command: "bijux status --format json".to_string(),
                rust_machine_output: "{\"ok\":true}".to_string(),
                python_machine_output: "{\"ok\":true}".to_string(),
            },
            PythonBridgeParityEntryV1 {
                command: "bijux doctor --format json".to_string(),
                rust_machine_output: "{\"ok\":false,\"code\":\"doctor_warn\"}".to_string(),
                python_machine_output: "{\"ok\":false,\"code\":\"doctor_warning\"}".to_string(),
            },
        ]);
        assert!(!report.parity_exact);
        assert_eq!(report.mismatched_commands, vec!["bijux doctor --format json"]);
    }

    #[test]
    fn official_app_discovery_prefers_highest_priority_and_refuses_shadow_attempts() {
        let report = build_official_app_discovery_report(
            vec![
                OfficialAppRouteDescriptorV1 {
                    namespace: "dag".to_string(),
                    descriptor_id: "dag-v1".to_string(),
                    priority: 10,
                },
                OfficialAppRouteDescriptorV1 {
                    namespace: "dag".to_string(),
                    descriptor_id: "dag-v2".to_string(),
                    priority: 20,
                },
            ],
            vec!["plugin:community attempted dag".to_string()],
            vec!["shim:bijux-dag attempted dag".to_string()],
        );
        assert_eq!(report.winners.len(), 1);
        assert_eq!(report.winners[0].descriptor_id, "dag-v2");
        assert_eq!(report.refused_plugin_shadows, vec!["plugin:community attempted dag"]);
    }
}
