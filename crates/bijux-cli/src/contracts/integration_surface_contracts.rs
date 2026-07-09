use schemars::JsonSchema;
use semver::{Version, VersionReq};
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
    let host = Version::parse(&payload.host_version)
        .map_err(|error| format!("invalid host_version: {error}"))?;
    Version::parse(&payload.app_version)
        .map_err(|error| format!("invalid app_version: {error}"))?;
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
            message: "host version outside compatibility window; migrate app descriptor"
                .to_string(),
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
        "supported" => {
            ("supported", "legacy shim is temporarily supported; prefer canonical command")
        }
        "warned" => ("warned", "legacy shim is deprecated; migrate to canonical command"),
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

/// Route conflict contender metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RouteConflictContenderV1 {
    /// Contender route source (`built-in`, `official-app`, `plugin`, `alias`, `shim`).
    pub source: String,
    /// Resolved command target key.
    pub target: String,
    /// Priority where larger values win.
    pub priority: i32,
}

/// Deterministic route conflict resolution output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RouteConflictResolutionV1 {
    /// Canonical route key.
    pub route_key: String,
    /// Winning contender if resolution succeeded.
    pub winner: Option<RouteConflictContenderV1>,
    /// Refusal reason when contenders tie irreconcilably.
    pub refusal_reason: Option<String>,
}

/// Resolve route conflicts deterministically by priority then stable source ordering.
pub fn resolve_route_conflict_deterministically(
    route_key: &str,
    contenders: Vec<RouteConflictContenderV1>,
) -> Result<RouteConflictResolutionV1, String> {
    if route_key.trim().is_empty() {
        return Err("route_key cannot be empty".to_string());
    }
    if contenders.is_empty() {
        return Ok(RouteConflictResolutionV1 {
            route_key: route_key.to_string(),
            winner: None,
            refusal_reason: Some("no contenders registered".to_string()),
        });
    }
    let mut ordered = contenders;
    ordered.sort_by(|left, right| {
        right
            .priority
            .cmp(&left.priority)
            .then_with(|| left.source.cmp(&right.source))
            .then_with(|| left.target.cmp(&right.target))
    });
    let winner = ordered.first().cloned().expect("contenders not empty");
    let same_rank: Vec<&RouteConflictContenderV1> = ordered
        .iter()
        .filter(|candidate| {
            candidate.priority == winner.priority && candidate.target != winner.target
        })
        .collect();
    if same_rank.is_empty() {
        Ok(RouteConflictResolutionV1 {
            route_key: route_key.to_string(),
            winner: Some(winner),
            refusal_reason: None,
        })
    } else {
        Ok(RouteConflictResolutionV1 {
            route_key: route_key.to_string(),
            winner: None,
            refusal_reason: Some("priority tie with conflicting targets".to_string()),
        })
    }
}

/// Provenance record for dispatched app route handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AppRouteProvenanceRecordV1 {
    /// Command route path.
    pub route_path: String,
    /// App descriptor hash.
    pub descriptor_hash: String,
    /// Handler binary or module identity.
    pub handler_identity: String,
    /// Output schema identifier.
    pub output_schema: String,
}

/// Build app route provenance record for support and evidence bundles.
pub fn build_app_route_provenance_record(
    route_path: &str,
    descriptor_hash: &str,
    handler_identity: &str,
    output_schema: &str,
) -> Result<AppRouteProvenanceRecordV1, String> {
    for (field, value) in [
        ("route_path", route_path),
        ("descriptor_hash", descriptor_hash),
        ("handler_identity", handler_identity),
        ("output_schema", output_schema),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{field} cannot be empty"));
        }
    }
    Ok(AppRouteProvenanceRecordV1 {
        route_path: route_path.to_string(),
        descriptor_hash: descriptor_hash.to_string(),
        handler_identity: handler_identity.to_string(),
        output_schema: output_schema.to_string(),
    })
}

/// SDK example conformance entry for one language implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkExampleConformanceEntryV1 {
    /// Example language (`rust` or `python`).
    pub language: String,
    /// Whether example exposes inspectable commands.
    pub exposes_inspectable_commands: bool,
    /// Whether example includes config handling.
    pub supports_config_contract: bool,
    /// Whether example includes explicit error pathway.
    pub supports_error_contract: bool,
    /// Whether machine output follows envelope contract.
    pub emits_machine_output_envelope: bool,
}

/// SDK example conformance report for mounted example apps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SdkExampleConformanceReportV1 {
    /// Per-language entries.
    pub entries: Vec<SdkExampleConformanceEntryV1>,
    /// Whether all required conformance checks pass.
    pub fully_conformant: bool,
}

/// Build SDK example conformance report.
pub fn build_sdk_example_conformance_report(
    entries: Vec<SdkExampleConformanceEntryV1>,
) -> Result<SdkExampleConformanceReportV1, String> {
    if entries.is_empty() {
        return Err("entries cannot be empty".to_string());
    }
    let mut ordered = entries;
    ordered.sort_by(|left, right| left.language.cmp(&right.language));
    let fully_conformant = ordered.iter().all(|entry| {
        entry.exposes_inspectable_commands
            && entry.supports_config_contract
            && entry.supports_error_contract
            && entry.emits_machine_output_envelope
    });
    Ok(SdkExampleConformanceReportV1 { entries: ordered, fully_conformant })
}

/// Plugin trust-class enforcement decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PluginTrustEnforcementDecisionV1 {
    /// Trust class (`official`, `local`, `experimental`, `disabled`).
    pub trust_class: String,
    /// Command classification (`read-only`, `destructive`).
    pub command_risk: String,
    /// Whether execution is allowed.
    pub allowed: bool,
    /// Decision rationale.
    pub rationale: String,
}

/// Enforce plugin trust classes for command execution behavior.
pub fn enforce_plugin_trust_class_behavior(
    trust_class: &str,
    command_risk: &str,
    experimental_destructive_enabled: bool,
) -> Result<PluginTrustEnforcementDecisionV1, String> {
    if trust_class.trim().is_empty() || command_risk.trim().is_empty() {
        return Err("trust_class and command_risk cannot be empty".to_string());
    }
    let decision = match (trust_class, command_risk) {
        ("disabled", _) => (false, "plugin trust class is disabled"),
        ("experimental", "destructive") if !experimental_destructive_enabled => {
            (false, "experimental destructive command requires explicit enable flag")
        }
        ("experimental", "destructive") => (true, "experimental destructive override enabled"),
        (_, _) => (true, "trust policy allows command"),
    };
    Ok(PluginTrustEnforcementDecisionV1 {
        trust_class: trust_class.to_string(),
        command_risk: command_risk.to_string(),
        allowed: decision.0,
        rationale: decision.1.to_string(),
    })
}

/// Side-effect-free app capability discovery report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AppCapabilityDiscoveryReportV1 {
    /// App namespace.
    pub app_namespace: String,
    /// Command groups exposed by descriptor metadata.
    pub command_groups: Vec<String>,
    /// Feature flags declared by app descriptor.
    pub feature_flags: Vec<String>,
    /// Required config keys.
    pub required_config_keys: Vec<String>,
    /// Output schema versions.
    pub schema_versions: Vec<String>,
    /// Optional runtime prerequisites missing at discovery time.
    pub missing_prerequisites: Vec<String>,
}

/// Build side-effect-free app capability discovery report.
pub fn build_app_capability_discovery_report(
    app_namespace: &str,
    command_groups: Vec<String>,
    feature_flags: Vec<String>,
    required_config_keys: Vec<String>,
    schema_versions: Vec<String>,
    missing_prerequisites: Vec<String>,
) -> Result<AppCapabilityDiscoveryReportV1, String> {
    if app_namespace.trim().is_empty() {
        return Err("app_namespace cannot be empty".to_string());
    }
    if command_groups.is_empty() {
        return Err("command_groups cannot be empty".to_string());
    }
    if schema_versions.is_empty() {
        return Err("schema_versions cannot be empty".to_string());
    }
    Ok(AppCapabilityDiscoveryReportV1 {
        app_namespace: app_namespace.to_string(),
        command_groups,
        feature_flags,
        required_config_keys,
        schema_versions,
        missing_prerequisites,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        build_app_capability_discovery_report, build_app_route_provenance_record,
        build_plugin_scaffold_conformance_report, build_sdk_example_conformance_report,
        enforce_plugin_trust_class_behavior, evaluate_legacy_shim_policy,
        evaluate_official_app_descriptor_compatibility, resolve_route_conflict_deterministically,
        validate_executable_plugin_manifest_contract, validate_plugin_subprocess_execution_policy,
        ExecutablePluginManifestContractV1, LegacyShimPolicyDecisionV1,
        OfficialAppDescriptorCompatibilityInputV1, PluginScaffoldConformanceEntryV1,
        PluginSubprocessExecutionPolicyV1, PluginTrustEnforcementDecisionV1,
        RouteConflictContenderV1, SdkExampleConformanceEntryV1,
    };

    #[test]
    fn plugin_manifest_contract_refuses_invalid_compatibility_window() {
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
    fn plugin_subprocess_policy_refuses_invalid_working_directory_policy() {
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
    fn plugin_scaffold_conformance_requires_discovery_and_envelope_validity() {
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
    fn descriptor_compatibility_reports_version_window_mismatch() {
        let report = evaluate_official_app_descriptor_compatibility(
            &OfficialAppDescriptorCompatibilityInputV1 {
                host_version: "0.4.0".to_string(),
                app_version: "1.4.0".to_string(),
                host_compatibility_window: ">=0.5,<0.6".to_string(),
                lifecycle_state: "active".to_string(),
                command_surfaces: vec!["dag run".to_string()],
            },
        )
        .expect("compatibility report should build");
        assert!(!report.compatible);
    }

    #[test]
    fn legacy_shim_policy_warns_with_canonical_route() {
        let decision: LegacyShimPolicyDecisionV1 =
            evaluate_legacy_shim_policy("bijux-dag", "bijux dag", "warned")
                .expect("shim decision should build");
        assert_eq!(decision.decision, "warned");
        assert_eq!(decision.canonical_command, "bijux dag");
    }

    #[test]
    fn route_conflict_resolution_is_deterministic_by_priority() {
        let resolution = resolve_route_conflict_deterministically(
            "dag run",
            vec![
                RouteConflictContenderV1 {
                    source: "plugin".to_string(),
                    target: "plugin.dag.run".to_string(),
                    priority: 10,
                },
                RouteConflictContenderV1 {
                    source: "official-app".to_string(),
                    target: "official.dag.run".to_string(),
                    priority: 100,
                },
            ],
        )
        .expect("conflict resolution should succeed");
        assert_eq!(resolution.winner.expect("winner").target, "official.dag.run");
        assert!(resolution.refusal_reason.is_none());
    }

    #[test]
    fn route_provenance_record_captures_handler_and_descriptor_hash() {
        let record = build_app_route_provenance_record(
            "dag run",
            "sha256:abc123",
            "bijux-dag-cli::dag::run",
            "dag-run-envelope-v1",
        )
        .expect("provenance record should build");
        assert_eq!(record.descriptor_hash, "sha256:abc123");
        assert_eq!(record.handler_identity, "bijux-dag-cli::dag::run");
    }

    #[test]
    fn sdk_example_conformance_requires_config_and_error_contracts() {
        let report = build_sdk_example_conformance_report(vec![
            SdkExampleConformanceEntryV1 {
                language: "rust".to_string(),
                exposes_inspectable_commands: true,
                supports_config_contract: true,
                supports_error_contract: true,
                emits_machine_output_envelope: true,
            },
            SdkExampleConformanceEntryV1 {
                language: "python".to_string(),
                exposes_inspectable_commands: true,
                supports_config_contract: false,
                supports_error_contract: true,
                emits_machine_output_envelope: true,
            },
        ])
        .expect("sdk conformance should build");
        assert!(!report.fully_conformant);
    }

    #[test]
    fn experimental_destructive_plugin_command_is_blocked_without_override() {
        let decision: PluginTrustEnforcementDecisionV1 =
            enforce_plugin_trust_class_behavior("experimental", "destructive", false)
                .expect("trust decision should build");
        assert!(!decision.allowed);
        assert!(decision.rationale.contains("requires explicit enable flag"));
    }

    #[test]
    fn app_capability_discovery_reports_missing_optional_prerequisites() {
        let report = build_app_capability_discovery_report(
            "dag",
            vec!["run".to_string(), "plan".to_string()],
            vec!["cache".to_string()],
            vec!["dag.run_root".to_string()],
            vec!["dag-run-envelope-v1".to_string()],
            vec!["apptainer-not-installed".to_string()],
        )
        .expect("capability report should build");
        assert_eq!(report.app_namespace, "dag");
        assert_eq!(report.missing_prerequisites, vec!["apptainer-not-installed"]);
    }
}
