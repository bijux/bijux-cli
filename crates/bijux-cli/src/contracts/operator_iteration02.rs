use semver::{Version, VersionReq};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Plugin manifest executable contract for pre-execution validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExecutablePluginManifestContractV1 {
    /// Plugin namespace.
    pub namespace: String,
    /// Plugin version.
    pub version: String,
    /// Entrypoint descriptor.
    pub entrypoint: String,
    /// Declared capabilities.
    pub capabilities: Vec<String>,
    /// Trust class identifier.
    pub trust_class: String,
    /// Declared command list.
    pub commands: Vec<String>,
    /// Host compatibility range.
    pub compatibility_window: String,
}

/// Validate executable plugin manifest contract before subprocess execution.
pub fn validate_executable_plugin_manifest_contract(
    payload: &ExecutablePluginManifestContractV1,
) -> Result<(), String> {
    if payload.namespace.trim().is_empty() {
        return Err("namespace cannot be empty".to_string());
    }
    if payload.entrypoint.trim().is_empty() {
        return Err("entrypoint cannot be empty".to_string());
    }
    if payload.commands.is_empty() {
        return Err("commands cannot be empty".to_string());
    }
    if payload.capabilities.is_empty() {
        return Err("capabilities cannot be empty".to_string());
    }
    if payload.trust_class.trim().is_empty() {
        return Err("trust_class cannot be empty".to_string());
    }
    Version::parse(&payload.version).map_err(|error| format!("invalid version: {error}"))?;
    VersionReq::parse(&payload.compatibility_window)
        .map_err(|error| format!("invalid compatibility_window: {error}"))?;
    Ok(())
}

/// Hardened plugin subprocess execution policy contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginSubprocessExecutionPolicyV1 {
    /// Normalized argv list used for subprocess launch.
    pub argv: Vec<String>,
    /// Timeout in milliseconds.
    pub timeout_ms: u64,
    /// Allowed environment variable keys.
    pub env_allowlist: Vec<String>,
    /// Working directory policy (`workspace-root`, `plugin-root`, `isolated-temp`).
    pub working_directory_policy: String,
    /// Required output envelope schema id.
    pub output_envelope_schema: String,
}

/// Validate hardened subprocess policy before plugin execution.
pub fn validate_plugin_subprocess_execution_policy(
    payload: &PluginSubprocessExecutionPolicyV1,
) -> Result<(), String> {
    if payload.argv.is_empty() {
        return Err("argv cannot be empty".to_string());
    }
    if payload.argv.iter().any(|arg| arg.contains('\n') || arg.contains('\0')) {
        return Err("argv contains invalid control characters".to_string());
    }
    if payload.timeout_ms == 0 {
        return Err("timeout_ms must be greater than zero".to_string());
    }
    if payload.env_allowlist.iter().any(|key| key.trim().is_empty()) {
        return Err("env_allowlist cannot include blank keys".to_string());
    }
    if !matches!(
        payload.working_directory_policy.as_str(),
        "workspace-root" | "plugin-root" | "isolated-temp"
    ) {
        return Err("working_directory_policy is invalid".to_string());
    }
    if payload.output_envelope_schema.trim().is_empty() {
        return Err("output_envelope_schema cannot be empty".to_string());
    }
    Ok(())
}

/// Generated plugin scaffold conformance entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginScaffoldConformanceEntryV1 {
    /// Scaffold language (`rust` or `python`).
    pub language: String,
    /// Whether scaffold compiles or imports successfully.
    pub build_ok: bool,
    /// Whether scaffold route is discovered by root CLI.
    pub discovered_by_root_cli: bool,
    /// Whether scaffold route executes with valid envelope.
    pub executable_with_valid_envelope: bool,
}

/// Plugin scaffold conformance report for generated templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginScaffoldConformanceReportV1 {
    /// Per-language conformance entries.
    pub entries: Vec<PluginScaffoldConformanceEntryV1>,
    /// Overall pass marker.
    pub fully_conformant: bool,
}

/// Build plugin scaffold conformance report from generated scaffold checks.
pub fn build_plugin_scaffold_conformance_report(
    entries: Vec<PluginScaffoldConformanceEntryV1>,
) -> Result<PluginScaffoldConformanceReportV1, String> {
    if entries.is_empty() {
        return Err("entries cannot be empty".to_string());
    }
    let mut ordered = entries;
    ordered.sort_by(|left, right| left.language.cmp(&right.language));
    let fully_conformant = ordered.iter().all(|entry| {
        entry.build_ok && entry.discovered_by_root_cli && entry.executable_with_valid_envelope
    });
    Ok(PluginScaffoldConformanceReportV1 { entries: ordered, fully_conformant })
}

/// Official app descriptor compatibility input contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OfficialAppDescriptorCompatibilityInputV1 {
    /// Host runtime version.
    pub host_version: String,
    /// App descriptor version.
    pub app_version: String,
    /// Host compatibility requirement declared by app.
    pub host_compatibility_window: String,
    /// Lifecycle state (`active`, `deprecated`, `disabled`).
    pub lifecycle_state: String,
    /// Declared command surfaces.
    pub command_surfaces: Vec<String>,
}

/// Official app descriptor compatibility evaluation output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct OfficialAppDescriptorCompatibilityReportV1 {
    /// Compatibility marker.
    pub compatible: bool,
    /// Refusal reason or migration hint.
    pub message: String,
}

/// Evaluate official app descriptor compatibility against host runtime version.
pub fn evaluate_official_app_descriptor_compatibility(
    payload: &OfficialAppDescriptorCompatibilityInputV1,
) -> Result<OfficialAppDescriptorCompatibilityReportV1, String> {
    let host =
        Version::parse(&payload.host_version).map_err(|error| format!("invalid host_version: {error}"))?;
    Version::parse(&payload.app_version).map_err(|error| format!("invalid app_version: {error}"))?;
    let requirement = VersionReq::parse(&payload.host_compatibility_window)
        .map_err(|error| format!("invalid host_compatibility_window: {error}"))?;
    if payload.command_surfaces.is_empty() {
        return Err("command_surfaces cannot be empty".to_string());
    }
    if payload.lifecycle_state == "disabled" {
        return Ok(OfficialAppDescriptorCompatibilityReportV1 {
            compatible: false,
            message: "app is disabled; use migration route".to_string(),
        });
    }
    if requirement.matches(&host) {
        Ok(OfficialAppDescriptorCompatibilityReportV1 {
            compatible: true,
            message: "descriptor compatible with host runtime".to_string(),
        })
    } else {
        Ok(OfficialAppDescriptorCompatibilityReportV1 {
            compatible: false,
            message: "host version outside compatibility window; migrate app descriptor".to_string(),
        })
    }
}

/// Legacy shim support policy decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LegacyShimPolicyDecisionV1 {
    /// Invoked legacy shim command (e.g. `bijux-dag`).
    pub shim_command: String,
    /// Canonical command replacement (e.g. `bijux dag`).
    pub canonical_command: String,
    /// Decision (`supported`, `warned`, `refused`).
    pub decision: String,
    /// Human actionable message.
    pub message: String,
}

/// Evaluate legacy shim policy and provide canonical route mapping.
pub fn evaluate_legacy_shim_policy(
    shim_command: &str,
    canonical_command: &str,
    shim_mode: &str,
) -> Result<LegacyShimPolicyDecisionV1, String> {
    if shim_command.trim().is_empty() || canonical_command.trim().is_empty() {
        return Err("shim_command and canonical_command cannot be empty".to_string());
    }
    let (decision, message) = match shim_mode {
        "supported" => (
            "supported",
            "legacy shim is temporarily supported; prefer canonical command",
        ),
        "warned" => (
            "warned",
            "legacy shim is deprecated; migrate to canonical command",
        ),
        "refused" => ("refused", "legacy shim refused; use canonical command"),
        _ => return Err("shim_mode must be supported, warned, or refused".to_string()),
    };
    Ok(LegacyShimPolicyDecisionV1 {
        shim_command: shim_command.to_string(),
        canonical_command: canonical_command.to_string(),
        decision: decision.to_string(),
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_plugin_scaffold_conformance_report,
        evaluate_legacy_shim_policy,
        evaluate_official_app_descriptor_compatibility,
        validate_executable_plugin_manifest_contract, validate_plugin_subprocess_execution_policy,
        ExecutablePluginManifestContractV1, LegacyShimPolicyDecisionV1,
        PluginScaffoldConformanceEntryV1,
        PluginSubprocessExecutionPolicyV1, OfficialAppDescriptorCompatibilityInputV1,
    };

    #[test]
    fn g011_plugin_manifest_contract_refuses_invalid_compatibility_window() {
        let manifest = ExecutablePluginManifestContractV1 {
            namespace: "community-tools".to_string(),
            version: "1.2.0".to_string(),
            entrypoint: "plugin:main".to_string(),
            capabilities: vec!["inspect".to_string(), "validate".to_string()],
            trust_class: "local".to_string(),
            commands: vec!["community lint".to_string()],
            compatibility_window: "not-semver".to_string(),
        };
        assert!(validate_executable_plugin_manifest_contract(&manifest).is_err());
    }

    #[test]
    fn g012_plugin_subprocess_policy_refuses_invalid_working_directory_policy() {
        let policy = PluginSubprocessExecutionPolicyV1 {
            argv: vec!["plugin-bin".to_string(), "run".to_string()],
            timeout_ms: 30_000,
            env_allowlist: vec!["BIJUX_CONFIG_ROOT".to_string()],
            working_directory_policy: "home-directory".to_string(),
            output_envelope_schema: "command-envelope-v1".to_string(),
        };
        assert!(validate_plugin_subprocess_execution_policy(&policy).is_err());
    }

    #[test]
    fn g013_plugin_scaffold_conformance_requires_discovery_and_envelope_validity() {
        let report = build_plugin_scaffold_conformance_report(vec![
            PluginScaffoldConformanceEntryV1 {
                language: "rust".to_string(),
                build_ok: true,
                discovered_by_root_cli: true,
                executable_with_valid_envelope: true,
            },
            PluginScaffoldConformanceEntryV1 {
                language: "python".to_string(),
                build_ok: true,
                discovered_by_root_cli: false,
                executable_with_valid_envelope: true,
            },
        ])
        .expect("conformance report should build");
        assert!(!report.fully_conformant);
    }

    #[test]
    fn g014_descriptor_compatibility_reports_version_window_mismatch() {
        let report = evaluate_official_app_descriptor_compatibility(
            &OfficialAppDescriptorCompatibilityInputV1 {
                host_version: "0.3.5".to_string(),
                app_version: "1.4.0".to_string(),
                host_compatibility_window: ">=0.4,<0.5".to_string(),
                lifecycle_state: "active".to_string(),
                command_surfaces: vec!["dag run".to_string()],
            },
        )
        .expect("compatibility report should build");
        assert!(!report.compatible);
    }

    #[test]
    fn g015_legacy_shim_policy_warns_with_canonical_route() {
        let decision: LegacyShimPolicyDecisionV1 =
            evaluate_legacy_shim_policy("bijux-dag", "bijux dag", "warned")
                .expect("shim decision should build");
        assert_eq!(decision.decision, "warned");
        assert_eq!(decision.canonical_command, "bijux dag");
    }
}
