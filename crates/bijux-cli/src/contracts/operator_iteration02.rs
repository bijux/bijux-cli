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

#[cfg(test)]
mod tests {
    use super::{
        build_plugin_scaffold_conformance_report,
        validate_executable_plugin_manifest_contract, validate_plugin_subprocess_execution_policy,
        ExecutablePluginManifestContractV1, PluginScaffoldConformanceEntryV1,
        PluginSubprocessExecutionPolicyV1,
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
}
