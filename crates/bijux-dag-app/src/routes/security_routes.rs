use crate::commands::{DagCli, SecurityCommands};
use crate::routes::simulation_io::load_json_file;
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    builtin_role_definitions, can_promote_artifact, can_renew_credential,
    check_scheduler_admission, credential_is_expired, credential_scopes_matrix,
    enforce_tenant_plugin_allowlist, evaluate_authorization_acceptance, evaluate_dry_run,
    is_action_allowed_in_environment, local_dev_bypass_allowed, readiness_for_federation,
    require_provenance_completeness, resolve_tenant_overlay, revoked_principals_set,
    scope_lineage_query, tenant_provisioning_bootstrap, trust_health_report, validate_custom_role,
    validate_tenant_isolation, verify_attestation_or_fail, Action, ActionKind, ArtifactTrustLabel,
    AuthenticationEvent, CredentialLifecycle, CredentialRevocation, CredentialScope,
    CustomRoleDefinition, DecisionType, EnvironmentAuthorizationRule, IdentityPrincipal,
    LocalDevAuthBypassRule, PolicyEvaluationRequest, PromotionPolicy, ProvenanceCompletenessPolicy,
    ResourceKind, ResourceRef, ResourceScope, RunProvenanceAttestation, RuntimeSecretContract,
    SignedArtifactManifest, SubjectIdentity, SubjectKind, TenantConfigOverlay, TenantId,
    TenantLineageScope, TenantPluginAllowlist, TenantPolicyBundleRef, TenantProvisioningSpec,
    TenantQueueIsolationPolicy, TenantRegistryPartition, TenantSchedulerAdmission,
    TrustHealthReport,
};
use bijux_dag_runtime::{
    authorize_input_path, authorize_output_path, is_allowed_env_key, is_denied_env_key,
    leak_conformance_check, secret_readiness, secret_scope_allows, secure_cleanup_required,
    secure_mode_effective, select_secret_version, shape_environment, summarize_sensitive_classes,
    validate_secret_delivery_mode, SecretDeliveryPolicy, SecretInjectionMode, SecretMaskingPolicy,
    SecretRotationRule, SecretScopeRule, SecretSource, SecretUsageAuditEvent, SecureExecutionMode,
    SecureTeardownPolicy, SecureWorkspaceRule, SensitiveArtifactRestriction,
};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct FilesystemAllowlistSimulation {
    input_root: String,
    output_root: String,
    read_candidates: Vec<String>,
    write_candidates: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FilesystemCandidateReport {
    path: String,
    allowed: bool,
    reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct FilesystemAllowlistReport {
    policy_lane: &'static str,
    all_reads_allowed: bool,
    all_writes_allowed: bool,
    read_results: Vec<FilesystemCandidateReport>,
    write_results: Vec<FilesystemCandidateReport>,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct EnvAllowlistSimulation {
    clean_env: bool,
    allowlist: Vec<String>,
    denylist: Vec<String>,
    #[serde(default)]
    ambient_env: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    explicit_env: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct EnvAllowlistReport {
    policy_lane: &'static str,
    passed_keys: Vec<String>,
    blocked_keys: Vec<String>,
    leaked_keys: Vec<String>,
    secret_like_keys_blocked: bool,
    gaps: Vec<String>,
}

#[derive(Debug, Serialize)]
struct NetworkPolicyNodeReport {
    node_id: String,
    approved_for_network: bool,
    cache_trust_impact: &'static str,
    replay_trust_impact: &'static str,
    policy_lane: &'static str,
}

#[derive(Debug, Serialize)]
struct NetworkPolicyReport {
    policy_lane: &'static str,
    network_nodes_present: bool,
    network_nodes_approved: bool,
    cache_trust_impact: &'static str,
    replay_trust_impact: &'static str,
    node_reports: Vec<NetworkPolicyNodeReport>,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct CommandInjectionSimulation {
    command_argv: Vec<String>,
    explicit_shell: bool,
    allow_metacharacters: bool,
    working_directory: Option<String>,
}

#[derive(Debug, Serialize)]
struct CommandInjectionReport {
    policy_lane: &'static str,
    shell_interpretation_requested: bool,
    implicit_shell_detected: bool,
    risky_tokens: Vec<String>,
    injection_hardened: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ArtifactSecretsSimulation {
    #[serde(default)]
    durable_fields: std::collections::BTreeMap<String, String>,
    redaction_enabled: bool,
    refuse_on_secret: bool,
}

#[derive(Debug, Serialize)]
struct ArtifactSecretsReport {
    policy_lane: &'static str,
    flagged_fields: Vec<String>,
    redacted_fields: std::collections::BTreeMap<String, String>,
    durable_write_allowed: bool,
    action: &'static str,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AuthSimulation {
    environment: String,
    now_unix_ms: u128,
    renewal_count: u32,
    short_lived_worker_creds_supported: bool,
    revocation_propagation_supported: bool,
    lifecycle: CredentialLifecycle,
    scope: CredentialScope,
    #[serde(default)]
    principals: Vec<IdentityPrincipal>,
    #[serde(default)]
    credential_classes: Vec<String>,
    #[serde(default)]
    policy_baselines: Vec<String>,
    #[serde(default)]
    bypass_rules: Vec<LocalDevAuthBypassRule>,
    #[serde(default)]
    auth_events: Vec<AuthenticationEvent>,
    #[serde(default)]
    revocations: Vec<CredentialRevocation>,
}

#[derive(Debug, Serialize)]
struct AuthReport {
    environment: String,
    anonymous_access_blocked: bool,
    local_bypass_blocked: bool,
    credential_expired: bool,
    renewable: bool,
    revoked_principals: Vec<String>,
    scope_matrix: std::collections::BTreeMap<String, bool>,
    federation_ready: bool,
    trust_health: TrustHealthReport,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct AuthzSimulation {
    subject_id: String,
    subject_kind: SubjectKind,
    tenant_id: Option<String>,
    action_name: String,
    action_kind: ActionKind,
    resource_kind: ResourceKind,
    resource_id: String,
    resource_tenant_id: Option<String>,
    environment: String,
    policy_bundle_version: String,
    #[serde(default)]
    granted_permissions: Vec<String>,
    #[serde(default)]
    environment_rules: Vec<EnvironmentAuthorizationRule>,
    #[serde(default)]
    decisions: Vec<(String, DecisionType)>,
    #[serde(default)]
    cross_tenant_denials: Vec<bool>,
    #[serde(default)]
    custom_role: Option<CustomRoleDefinition>,
}

#[derive(Debug, Serialize)]
struct AuthzReport {
    built_in_roles: Vec<String>,
    dry_run_allow: bool,
    environment_allows: bool,
    least_privilege_holds: bool,
    cross_tenant_escalation_blocked: bool,
    custom_role_valid: bool,
    failures: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TenantSimulation {
    requested_tenant: String,
    api_tenant: String,
    scheduler_tenant: String,
    artifact_tenant: String,
    metrics_tenant: String,
    lineage_tenant: String,
    queued_runs: usize,
    pending_dispatches: usize,
    plugin_name: String,
    requested_artifact_ids: Vec<String>,
    #[serde(default)]
    global_defaults: std::collections::BTreeMap<String, String>,
    overlay_values: std::collections::BTreeMap<String, String>,
    overlay_overrides: std::collections::BTreeMap<String, String>,
    queue_names: Vec<String>,
    allowed_plugins: Vec<String>,
    allowed_artifact_ids: Vec<String>,
    namespace: String,
    storage_partition: String,
    index_prefix: String,
    policy_bundle_id: String,
    policy_bundle_version: String,
    max_enqueued_runs: usize,
    max_dispatches_per_tick: usize,
}

#[derive(Debug, Serialize)]
struct TenantReport {
    isolated: bool,
    scheduler_admitted: bool,
    plugin_allowed: bool,
    visible_artifact_ids: Vec<String>,
    merged_config: std::collections::BTreeMap<String, String>,
    bootstrap_steps: Vec<String>,
    violations: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SecretsSimulation {
    available_versions: Vec<String>,
    pinned_version: Option<String>,
    is_backfill: bool,
    allowed_regions: Vec<String>,
    requested_region: String,
    uses_static_secret: bool,
    secret_contract: RuntimeSecretContract,
    rotation: SecretRotationRule,
    delivery_policy: SecretDeliveryPolicy,
    delivery_mode: SecretInjectionMode,
    allowed_scope: SecretScopeRule,
    requested_scope: SecretScopeRule,
    masking_policy: SecretMaskingPolicy,
    sources: Vec<SecretSource>,
    audit_events: Vec<SecretUsageAuditEvent>,
    secure_mode: SecureExecutionMode,
    workspace_rule: SecureWorkspaceRule,
    teardown_policy: SecureTeardownPolicy,
    restrictions: Vec<SensitiveArtifactRestriction>,
    observed_outputs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SecretsReport {
    selected_version: Option<String>,
    pinned: bool,
    delivery_allowed: bool,
    scope_allowed: bool,
    readiness_ok: bool,
    strict_mode_effective: bool,
    cleanup_required: bool,
    leak_clean: bool,
    brokered_credentials: bool,
    region_allowed: bool,
    sensitive_classes: std::collections::BTreeMap<String, (u32, bool)>,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SupplyChainSimulation {
    trust_label: ArtifactTrustLabel,
    completeness_policy: ProvenanceCompletenessPolicy,
    promotion_policy: PromotionPolicy,
    attestation: RunProvenanceAttestation,
    signed_manifests: Vec<SignedArtifactManifest>,
}

#[derive(Debug, Serialize)]
struct SupplyChainReport {
    completeness_passed: bool,
    promotion_allowed: bool,
    pinned_inputs: bool,
    trust_domain_bound: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SupplyInventorySimulation {
    run_id: String,
    #[serde(default)]
    binaries: Vec<InventoryComponent>,
    #[serde(default)]
    containers: Vec<InventoryComponent>,
    #[serde(default)]
    plugins: Vec<InventoryComponent>,
    #[serde(default)]
    adapters: Vec<InventoryComponent>,
}

#[derive(Debug, serde::Deserialize)]
struct InventoryComponent {
    id: String,
    version: String,
    checksum: Option<String>,
}

#[derive(Debug, Serialize)]
struct SupplyInventoryReport {
    policy_lane: &'static str,
    run_id: String,
    binaries: Vec<InventoryComponentReport>,
    containers: Vec<InventoryComponentReport>,
    plugins: Vec<InventoryComponentReport>,
    adapters: Vec<InventoryComponentReport>,
    inventory_complete: bool,
    gaps: Vec<String>,
}

#[derive(Debug, Serialize)]
struct InventoryComponentReport {
    id: String,
    version: String,
    checksum_present: bool,
}

#[derive(Debug, serde::Deserialize)]
struct TrustClassesSimulation {
    run_id: String,
    evidence_complete: bool,
    advisory_only_controls: bool,
    simulated_backend: bool,
    release_controls_met: bool,
    audit_controls_met: bool,
    policy_violations: usize,
}

#[derive(Debug, Serialize)]
struct TrustClassesReport {
    policy_lane: &'static str,
    run_id: String,
    run_trust_class: &'static str,
    artifact_trust_class: &'static str,
    evidence_complete: bool,
    classification_reasons: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct MalformedInputFuzzSimulation {
    #[serde(default)]
    graph_payloads: Vec<String>,
    #[serde(default)]
    config_payloads: Vec<String>,
    #[serde(default)]
    route_argv_sets: Vec<Vec<String>>,
    #[serde(default)]
    plugin_manifest_payloads: Vec<String>,
    #[serde(default)]
    bundle_payloads: Vec<String>,
    #[serde(default)]
    run_manifest_payloads: Vec<String>,
}

#[derive(Debug, Serialize)]
struct MalformedInputFuzzReport {
    policy_lane: &'static str,
    cases_total: usize,
    panics_observed: usize,
    graph_rejections: usize,
    config_rejections: usize,
    route_rejections: usize,
    plugin_manifest_rejections: usize,
    bundle_rejections: usize,
    run_manifest_rejections: usize,
    crash_free: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct PluginManifestProbe {
    namespace: String,
    version: String,
    entrypoint: String,
    #[serde(default)]
    capabilities: Vec<String>,
    trust_class: String,
}

#[derive(Debug, serde::Deserialize)]
struct RunManifestProbe {
    manifest_version: String,
    run_id: String,
    status: String,
}

#[derive(Debug, serde::Deserialize)]
struct DependencyRiskSimulation {
    #[serde(default)]
    dependencies: Vec<DependencyRiskEntry>,
    core_runtime_threshold: f64,
    tooling_threshold: f64,
}

#[derive(Debug, serde::Deserialize)]
struct DependencyRiskEntry {
    name: String,
    surface: DependencySurface,
    risk_score: f64,
    known_vulnerabilities: u32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DependencySurface {
    CoreRuntime,
    DocsTooling,
    DevTooling,
    ReleaseTooling,
}

#[derive(Debug, Serialize)]
struct DependencyRiskReport {
    policy_lane: &'static str,
    core_runtime_risk_score: f64,
    tooling_risk_score: f64,
    core_runtime_exceeds_threshold: bool,
    tooling_exceeds_threshold: bool,
    core_runtime_dependencies: Vec<String>,
    tooling_dependencies: Vec<String>,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct DataAccessSimulation {
    input_root: String,
    candidate_input: String,
    output_root: String,
    candidate_output: String,
    allowed_dataset_ids: Vec<String>,
    requested_dataset_ids: Vec<String>,
    allowed_artifact_ids: Vec<String>,
    requested_artifact_ids: Vec<String>,
    tenant_id: String,
}

#[derive(Debug, Serialize)]
struct DataAccessReport {
    input_path_allowed: bool,
    output_path_allowed: bool,
    dataset_entitlements_ok: bool,
    visible_artifact_ids: Vec<String>,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OverrideSimulation {
    actor: String,
    tenant_id: Option<String>,
    action_name: String,
    action_kind: ActionKind,
    resource_kind: ResourceKind,
    resource_id: String,
    resource_tenant_id: Option<String>,
    environment: String,
    policy_bundle_version: String,
    granted_permissions: Vec<String>,
    environment_rules: Vec<EnvironmentAuthorizationRule>,
    reason: String,
    audit_event_id: Option<String>,
    #[serde(default)]
    approvers: Vec<String>,
    #[serde(default)]
    required_approvals: usize,
    break_glass: bool,
}

#[derive(Debug, Serialize)]
struct OverrideReport {
    override_allowed: bool,
    reason_recorded: bool,
    audit_recorded: bool,
    environment_allows: bool,
    break_glass_policy_valid: bool,
    approval_quorum_met: bool,
    scoped_to_tenant: bool,
    gaps: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OverrideAuditSimulation {
    actor: String,
    run_id: String,
    forced_rerun: bool,
    bypassed_policy: bool,
    cache_disabled: bool,
    accepted_degraded_evidence: bool,
    retention_change: Option<String>,
    reason: String,
    evidence_pointer: Option<String>,
    audit_event_id: Option<String>,
    timestamp_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct OverrideAuditReport {
    policy_lane: &'static str,
    audit_complete: bool,
    recorded_actions: Vec<String>,
    missing_records: Vec<String>,
    actor: String,
    run_id: String,
    audit_event_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct SafeDefaultsNodeReport {
    node_id: String,
    risky_defaults: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SafeDefaultsReport {
    workflow_name: String,
    safe_by_default: bool,
    nodes: Vec<SafeDefaultsNodeReport>,
    gaps: Vec<String>,
}

fn filesystem_allowlist_payload(simulation: &Path) -> Result<FilesystemAllowlistReport, ExitCode> {
    let simulation: FilesystemAllowlistSimulation = load_json_file(simulation)?;
    let input_root = Path::new(&simulation.input_root);
    let output_root = Path::new(&simulation.output_root);

    let mut read_results = Vec::new();
    let mut write_results = Vec::new();
    let mut gaps = Vec::new();

    for candidate in &simulation.read_candidates {
        match authorize_input_path(input_root, Path::new(candidate)) {
            Ok(_) => read_results.push(FilesystemCandidateReport {
                path: candidate.clone(),
                allowed: true,
                reason: None,
            }),
            Err(reason) => {
                gaps.push(format!("read candidate denied: {candidate} ({reason})"));
                read_results.push(FilesystemCandidateReport {
                    path: candidate.clone(),
                    allowed: false,
                    reason: Some(reason),
                });
            }
        }
    }

    for candidate in &simulation.write_candidates {
        match authorize_output_path(output_root, Path::new(candidate)) {
            Ok(_) => write_results.push(FilesystemCandidateReport {
                path: candidate.clone(),
                allowed: true,
                reason: None,
            }),
            Err(reason) => {
                gaps.push(format!("write candidate denied: {candidate} ({reason})"));
                write_results.push(FilesystemCandidateReport {
                    path: candidate.clone(),
                    allowed: false,
                    reason: Some(reason),
                });
            }
        }
    }

    let all_reads_allowed = read_results.iter().all(|item| item.allowed);
    let all_writes_allowed = write_results.iter().all(|item| item.allowed);
    Ok(FilesystemAllowlistReport {
        policy_lane: "ENFORCED",
        all_reads_allowed,
        all_writes_allowed,
        read_results,
        write_results,
        gaps,
    })
}

fn is_secret_like_env_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("SECRET") || upper.contains("TOKEN") || upper.contains("PASSWORD")
}

fn env_allowlist_payload(simulation: &Path) -> Result<EnvAllowlistReport, ExitCode> {
    let simulation: EnvAllowlistSimulation = load_json_file(simulation)?;
    let shaped = shape_environment(
        &simulation.ambient_env,
        simulation.clean_env,
        &simulation.allowlist,
        &simulation.denylist,
        &simulation.explicit_env,
    );
    let mut passed_keys = shaped.keys().cloned().collect::<Vec<_>>();
    passed_keys.sort();

    let all_input_keys = simulation
        .ambient_env
        .keys()
        .chain(simulation.explicit_env.keys())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let mut blocked_keys =
        all_input_keys.iter().filter(|key| !shaped.contains_key(*key)).cloned().collect::<Vec<_>>();
    blocked_keys.sort();

    let mut leaked_keys = shaped
        .keys()
        .filter(|key| {
            is_secret_like_env_key(key)
                || (!simulation.allowlist.is_empty()
                    && !is_allowed_env_key(key, &simulation.allowlist))
                || is_denied_env_key(key, &simulation.denylist)
        })
        .cloned()
        .collect::<Vec<_>>();
    leaked_keys.sort();

    let secret_like_keys_blocked = shaped.keys().all(|key| !is_secret_like_env_key(key));
    let mut gaps = Vec::new();
    if !leaked_keys.is_empty() {
        gaps.push(
            "effective environment still exposes sensitive or non-compliant keys".to_string(),
        );
    }
    if !secret_like_keys_blocked {
        gaps.push("secret-like environment keys are not fully blocked".to_string());
    }
    if simulation.allowlist.is_empty() {
        gaps.push("allowlist is empty; environment filtering is policy-weak".to_string());
    }
    Ok(EnvAllowlistReport {
        policy_lane: "ENFORCED",
        passed_keys,
        blocked_keys,
        leaked_keys,
        secret_like_keys_blocked,
        gaps,
    })
}

fn network_policy_payload(dag: &Path) -> Result<NetworkPolicyReport, ExitCode> {
    let graph = parse_graph(&read_file(dag)?)?;
    let mut node_reports = Vec::new();
    let mut gaps = Vec::new();
    let mut network_nodes_present = false;
    let mut network_nodes_approved = true;

    for node in &graph.nodes {
        let has_network_effect =
            node.effects.iter().any(|effect| matches!(effect, bijux_dag_core::Effect::Network));
        if !has_network_effect {
            continue;
        }
        network_nodes_present = true;
        let approved_for_network =
            node.tags.iter().any(|tag| tag == "network-approved" || tag == "egress-reviewed");
        if !approved_for_network {
            network_nodes_approved = false;
            gaps.push(format!(
                "node {} declares network effect without network-approved tag",
                node.id
            ));
        }
        node_reports.push(NetworkPolicyNodeReport {
            node_id: node.id.clone(),
            approved_for_network,
            cache_trust_impact: "reduced",
            replay_trust_impact: "reduced",
            policy_lane: "ADVISORY",
        });
    }

    if network_nodes_present {
        gaps.push(
            "network effects reduce cache/replay trust unless non-cacheable enforcement is active"
                .to_string(),
        );
    }
    let policy_lane = if network_nodes_present { "ADVISORY" } else { "ENFORCED" };
    Ok(NetworkPolicyReport {
        policy_lane,
        network_nodes_present,
        network_nodes_approved,
        cache_trust_impact: if network_nodes_present { "reduced" } else { "none" },
        replay_trust_impact: if network_nodes_present { "reduced" } else { "none" },
        node_reports,
        gaps,
    })
}

fn token_looks_shell_interpreted(token: &str) -> bool {
    [';', '|', '&', '`', '$', '>', '<'].iter().any(|ch| token.contains(*ch))
}

fn command_injection_payload(simulation: &Path) -> Result<CommandInjectionReport, ExitCode> {
    let simulation: CommandInjectionSimulation = load_json_file(simulation)?;
    let argv = simulation.command_argv;
    let implicit_shell_detected = argv.len() >= 2
        && (argv[0].ends_with("sh") || argv[0].ends_with("bash") || argv[0].ends_with("zsh"))
        && argv[1] == "-c";
    let shell_interpretation_requested = simulation.explicit_shell || implicit_shell_detected;
    let mut risky_tokens = argv
        .iter()
        .filter(|token| token_looks_shell_interpreted(token))
        .cloned()
        .collect::<Vec<_>>();
    risky_tokens.sort();
    risky_tokens.dedup();

    let mut gaps = Vec::new();
    if implicit_shell_detected && !simulation.explicit_shell {
        gaps.push(
            "implicit shell interpretation is forbidden; require explicit_shell=true".to_string(),
        );
    }
    if !risky_tokens.is_empty() && !simulation.allow_metacharacters {
        gaps.push(
            "metacharacter-bearing argv tokens require explicit allow_metacharacters".to_string(),
        );
    }
    if let Some(cwd) = simulation.working_directory.as_deref() {
        if cwd.contains("..") {
            gaps.push("working_directory contains parent traversal segments".to_string());
        }
    }

    let injection_hardened = gaps.is_empty();
    Ok(CommandInjectionReport {
        policy_lane: "ENFORCED",
        shell_interpretation_requested,
        implicit_shell_detected,
        risky_tokens,
        injection_hardened,
        gaps,
    })
}

fn artifact_field_looks_secret(path: &str, value: &str) -> bool {
    let field = path.to_ascii_lowercase();
    let body = value.to_ascii_lowercase();
    field.contains("secret")
        || field.contains("password")
        || field.contains("token")
        || field.contains("apikey")
        || field.contains("api_key")
        || body.contains("bearer ")
        || body.contains("token=")
        || body.contains("password=")
        || body.contains("api_key=")
        || body.contains("secret=")
}

fn artifact_secrets_payload(simulation: &Path) -> Result<ArtifactSecretsReport, ExitCode> {
    let simulation: ArtifactSecretsSimulation = load_json_file(simulation)?;
    let mut flagged_fields = simulation
        .durable_fields
        .iter()
        .filter(|(path, value)| artifact_field_looks_secret(path, value))
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    flagged_fields.sort();

    let mut redacted_fields = simulation.durable_fields.clone();
    if simulation.redaction_enabled {
        for field in &flagged_fields {
            if let Some(value) = redacted_fields.get_mut(field) {
                *value = "[REDACTED]".to_string();
            }
        }
    }

    let durable_write_allowed =
        flagged_fields.is_empty() || (simulation.redaction_enabled && !simulation.refuse_on_secret);
    let action = if flagged_fields.is_empty() {
        "clean"
    } else if simulation.refuse_on_secret {
        "refused"
    } else if simulation.redaction_enabled {
        "redacted"
    } else {
        "violating"
    };
    let mut gaps = Vec::new();
    if !flagged_fields.is_empty() && !simulation.redaction_enabled && !simulation.refuse_on_secret {
        gaps.push("secret-bearing durable fields were neither redacted nor refused".to_string());
    }
    if !flagged_fields.is_empty() && simulation.refuse_on_secret {
        gaps.push("durable artifact write refused due to secret-bearing fields".to_string());
    }

    Ok(ArtifactSecretsReport {
        policy_lane: "ENFORCED",
        flagged_fields,
        redacted_fields,
        durable_write_allowed,
        action,
        gaps,
    })
}

fn auth_payload(simulation: &Path) -> Result<AuthReport, ExitCode> {
    let simulation: AuthSimulation = load_json_file(simulation)?;
    let revoked = revoked_principals_set(&simulation.revocations);
    let local_bypass_blocked = simulation
        .bypass_rules
        .iter()
        .all(|rule| !local_dev_bypass_allowed(&simulation.environment, rule));
    let federation = readiness_for_federation(
        &simulation.bypass_rules,
        simulation.revocation_propagation_supported,
        simulation.short_lived_worker_creds_supported,
        &simulation.auth_events,
    );
    let trust_health = trust_health_report(
        &simulation.principals,
        &simulation.credential_classes,
        &simulation.policy_baselines,
    );
    let scope_matrix = credential_scopes_matrix(&simulation.scope);
    let credential_expired = credential_is_expired(simulation.now_unix_ms, &simulation.lifecycle);
    let renewable = can_renew_credential(simulation.renewal_count, &simulation.lifecycle);
    let anonymous_access_blocked = !simulation.principals.is_empty();

    let mut gaps = Vec::new();
    if !anonymous_access_blocked {
        gaps.push("no authenticated principals are configured".to_string());
    }
    if !local_bypass_blocked {
        gaps.push("local bypass is enabled outside the local environment".to_string());
    }
    if credential_expired {
        gaps.push("credential lifecycle is already expired".to_string());
    }
    if !federation.audit_events_complete {
        gaps.push("authentication audit chain is incomplete".to_string());
    }

    Ok(AuthReport {
        environment: simulation.environment,
        anonymous_access_blocked,
        local_bypass_blocked,
        credential_expired,
        renewable,
        revoked_principals: revoked.into_iter().collect(),
        scope_matrix,
        federation_ready: federation.local_auth_isolated
            && federation.revocation_propagation_supported
            && federation.short_lived_worker_creds_supported
            && federation.audit_events_complete,
        trust_health,
        gaps,
    })
}

fn authz_payload(simulation: &Path) -> Result<AuthzReport, ExitCode> {
    let simulation: AuthzSimulation = load_json_file(simulation)?;
    let request = PolicyEvaluationRequest {
        request_id: "security-authz".to_string(),
        subject: SubjectIdentity {
            subject_id: simulation.subject_id,
            kind: simulation.subject_kind,
            tenant_id: simulation.tenant_id.clone(),
        },
        action: Action {
            name: simulation.action_name.clone(),
            kind: simulation.action_kind.clone(),
        },
        resource: ResourceRef {
            kind: simulation.resource_kind,
            id: simulation.resource_id,
            tenant_id: simulation.resource_tenant_id.clone(),
        },
        scope: match simulation.tenant_id.clone() {
            Some(tenant_id) => ResourceScope::Tenant { tenant_id },
            None => ResourceScope::Global,
        },
        environment: simulation.environment.clone(),
    };
    let dry_run = evaluate_dry_run(
        &request,
        &simulation.granted_permissions,
        &simulation.policy_bundle_version,
    );
    let environment_allows = is_action_allowed_in_environment(
        &simulation.action_name,
        &simulation.environment,
        &simulation.environment_rules,
    );
    let acceptance =
        evaluate_authorization_acceptance(&simulation.decisions, &simulation.cross_tenant_denials);
    let custom_role_valid = simulation
        .custom_role
        .as_ref()
        .map(|role| validate_custom_role(role).is_ok())
        .unwrap_or(true);
    let mut failures = acceptance.failures.clone();
    if !environment_allows {
        failures.push("environment rule denies this action".to_string());
    }
    if !custom_role_valid {
        failures.push("custom role definition is invalid".to_string());
    }
    Ok(AuthzReport {
        built_in_roles: builtin_role_definitions()
            .into_iter()
            .map(|role| format!("{:?}", role.role))
            .collect(),
        dry_run_allow: dry_run.would_allow,
        environment_allows,
        least_privilege_holds: acceptance.least_privilege_holds,
        cross_tenant_escalation_blocked: acceptance.no_cross_tenant_escalation,
        custom_role_valid,
        failures,
    })
}

fn parse_tenant_id(value: &str) -> Result<TenantId, ExitCode> {
    TenantId::parse(value).map_err(|_| ExitCode::from(2))
}

fn tenant_payload(simulation: &Path) -> Result<TenantReport, ExitCode> {
    let simulation: TenantSimulation = load_json_file(simulation)?;
    let requested_tenant = parse_tenant_id(&simulation.requested_tenant)?;
    let api_tenant = parse_tenant_id(&simulation.api_tenant)?;
    let scheduler_tenant = parse_tenant_id(&simulation.scheduler_tenant)?;
    let artifact_tenant = parse_tenant_id(&simulation.artifact_tenant)?;
    let metrics_tenant = parse_tenant_id(&simulation.metrics_tenant)?;
    let lineage_tenant = parse_tenant_id(&simulation.lineage_tenant)?;

    let isolation = validate_tenant_isolation(
        &requested_tenant,
        &api_tenant,
        &scheduler_tenant,
        &artifact_tenant,
        &metrics_tenant,
        &lineage_tenant,
    );
    let admission = TenantSchedulerAdmission {
        tenant_id: requested_tenant.clone(),
        max_enqueued_runs: simulation.max_enqueued_runs,
        max_dispatches_per_tick: simulation.max_dispatches_per_tick,
    };
    let scheduler_admitted = check_scheduler_admission(
        simulation.queued_runs,
        simulation.pending_dispatches,
        &admission,
    );
    let allowlist = TenantPluginAllowlist {
        tenant_id: requested_tenant.clone(),
        allowed_plugins: simulation.allowed_plugins,
    };
    let plugin_allowed = enforce_tenant_plugin_allowlist(&simulation.plugin_name, &allowlist);
    let lineage_scope = TenantLineageScope {
        tenant_id: requested_tenant.clone(),
        allowed_artifact_ids: simulation.allowed_artifact_ids,
    };
    let visible_artifact_ids =
        scope_lineage_query(&simulation.requested_artifact_ids, &lineage_scope);
    let overlay = TenantConfigOverlay {
        tenant_id: requested_tenant.clone(),
        values: simulation.overlay_values,
        overrides: simulation.overlay_overrides,
    };
    let merged_config = resolve_tenant_overlay(&simulation.global_defaults, &overlay);
    let provisioning = TenantProvisioningSpec {
        tenant_id: requested_tenant,
        namespace: simulation.namespace,
        registry_partition: TenantRegistryPartition {
            tenant_id: parse_tenant_id(&simulation.requested_tenant)?,
            storage_partition: simulation.storage_partition,
            index_prefix: simulation.index_prefix,
        },
        default_queue_isolation: TenantQueueIsolationPolicy {
            tenant_id: parse_tenant_id(&simulation.requested_tenant)?,
            queue_names: simulation.queue_names,
            hard_isolation: true,
        },
        default_policy_bundle: TenantPolicyBundleRef {
            tenant_id: parse_tenant_id(&simulation.requested_tenant)?,
            policy_bundle_id: simulation.policy_bundle_id,
            policy_bundle_version: simulation.policy_bundle_version,
        },
    };
    let bootstrap_steps = tenant_provisioning_bootstrap(&provisioning);
    let mut violations = isolation.violations;
    if !scheduler_admitted {
        violations.push("scheduler admission quota exceeded".to_string());
    }
    if !plugin_allowed {
        violations.push("plugin allowlist rejected the requested plugin".to_string());
    }

    Ok(TenantReport {
        isolated: violations.iter().all(|v| !v.contains("tenant scope mismatch")),
        scheduler_admitted,
        plugin_allowed,
        visible_artifact_ids,
        merged_config,
        bootstrap_steps,
        violations,
    })
}

fn secrets_payload(simulation: &Path) -> Result<SecretsReport, ExitCode> {
    let simulation: SecretsSimulation = load_json_file(simulation)?;
    let selection = select_secret_version(
        &simulation.available_versions,
        simulation.pinned_version.as_deref(),
        &simulation.rotation,
        simulation.is_backfill,
    );
    let delivery_allowed =
        validate_secret_delivery_mode(&simulation.delivery_mode, &simulation.delivery_policy);
    let scope_allowed = secret_scope_allows(&simulation.allowed_scope, &simulation.requested_scope);
    let readiness = secret_readiness(
        &simulation.sources,
        &simulation.masking_policy,
        &simulation.audit_events,
        simulation.secure_mode.enabled,
    );
    let strict_mode_effective =
        secure_mode_effective(&simulation.secure_mode.environment, &simulation.secure_mode);
    let cleanup_required =
        secure_cleanup_required(&simulation.workspace_rule, &simulation.teardown_policy);
    let leak_clean = leak_conformance_check(&simulation.observed_outputs);
    let brokered_credentials = !simulation.uses_static_secret
        && simulation
            .sources
            .iter()
            .any(|source| matches!(source, SecretSource::ExternalManager { .. }));
    let region_allowed =
        simulation.allowed_regions.iter().any(|region| region == &simulation.requested_region);
    let sensitive_classes = summarize_sensitive_classes(&simulation.restrictions);
    let mut gaps = Vec::new();
    if selection.is_none() {
        gaps.push("no valid secret version could be selected".to_string());
    }
    if !delivery_allowed {
        gaps.push("secret delivery mode is not allowed".to_string());
    }
    if !scope_allowed {
        gaps.push("secret scope does not allow the requested access".to_string());
    }
    if !readiness.source_connected || !readiness.masking_enabled || !readiness.audit_enabled {
        gaps.push("secret integration readiness is incomplete".to_string());
    }
    if !strict_mode_effective {
        gaps.push("strict secret execution mode is not effective".to_string());
    }
    if !cleanup_required {
        gaps.push("secure cleanup policy is incomplete".to_string());
    }
    if !leak_clean {
        gaps.push("observed outputs contain secret-looking material".to_string());
    }
    if !brokered_credentials {
        gaps.push("workload still depends on static or non-brokered credentials".to_string());
    }
    if !region_allowed {
        gaps.push("requested region is outside the approved secret region set".to_string());
    }
    if simulation.secret_contract.secret_refs.is_empty()
        || !simulation.secret_contract.redaction_required
    {
        gaps.push("runtime secret contract is incomplete".to_string());
    }

    Ok(SecretsReport {
        selected_version: selection.as_ref().map(|item| item.selected_version.clone()),
        pinned: selection.as_ref().map(|item| item.pinned).unwrap_or(false),
        delivery_allowed,
        scope_allowed,
        readiness_ok: readiness.source_connected
            && readiness.masking_enabled
            && readiness.audit_enabled
            && readiness.strict_mode_supported,
        strict_mode_effective,
        cleanup_required,
        leak_clean,
        brokered_credentials,
        region_allowed,
        sensitive_classes,
        gaps,
    })
}

fn supply_chain_payload(simulation: &Path) -> Result<SupplyChainReport, ExitCode> {
    let simulation: SupplyChainSimulation = load_json_file(simulation)?;
    let verification = require_provenance_completeness(
        &simulation.attestation,
        &simulation.signed_manifests,
        &simulation.completeness_policy,
    );
    let completeness_passed = verify_attestation_or_fail(verification.clone()).is_ok();
    let promotion_allowed = can_promote_artifact(
        &simulation.trust_label,
        completeness_passed,
        &simulation.promotion_policy,
    );
    let pinned_inputs = !simulation.attestation.output_digests.is_empty()
        && simulation.signed_manifests.iter().all(|manifest| !manifest.digest.trim().is_empty());
    let trust_domain_bound = !simulation.attestation.environment.trust_domain.trim().is_empty();
    let mut gaps = verification.errors;
    if !promotion_allowed {
        gaps.push("artifact promotion policy rejected this trust posture".to_string());
    }
    if !pinned_inputs {
        gaps.push("artifact digests are not fully pinned".to_string());
    }
    if !trust_domain_bound {
        gaps.push("environment trust domain is missing".to_string());
    }

    Ok(SupplyChainReport {
        completeness_passed,
        promotion_allowed,
        pinned_inputs,
        trust_domain_bound,
        gaps,
    })
}

fn map_inventory_components(
    kind: &str,
    components: &[InventoryComponent],
    gaps: &mut Vec<String>,
) -> Vec<InventoryComponentReport> {
    components
        .iter()
        .map(|component| {
            let checksum_present =
                component.checksum.as_ref().is_some_and(|value| !value.trim().is_empty());
            if !checksum_present {
                gaps.push(format!(
                    "{kind} component {}@{} is missing a checksum",
                    component.id, component.version
                ));
            }
            if component.id.trim().is_empty() || component.version.trim().is_empty() {
                gaps.push(format!("{kind} component has missing id/version fields"));
            }
            InventoryComponentReport {
                id: component.id.clone(),
                version: component.version.clone(),
                checksum_present,
            }
        })
        .collect()
}

fn supply_inventory_payload(simulation: &Path) -> Result<SupplyInventoryReport, ExitCode> {
    let simulation: SupplyInventorySimulation = load_json_file(simulation)?;
    let mut gaps = Vec::new();
    let binaries = map_inventory_components("binary", &simulation.binaries, &mut gaps);
    let containers = map_inventory_components("container", &simulation.containers, &mut gaps);
    let plugins = map_inventory_components("plugin", &simulation.plugins, &mut gaps);
    let adapters = map_inventory_components("adapter", &simulation.adapters, &mut gaps);
    if binaries.is_empty() && containers.is_empty() && plugins.is_empty() && adapters.is_empty() {
        gaps.push("supply inventory is empty".to_string());
    }
    let inventory_complete = gaps.is_empty();
    Ok(SupplyInventoryReport {
        policy_lane: "ENFORCED",
        run_id: simulation.run_id,
        binaries,
        containers,
        plugins,
        adapters,
        inventory_complete,
        gaps,
    })
}

fn trust_classes_payload(simulation: &Path) -> Result<TrustClassesReport, ExitCode> {
    let simulation: TrustClassesSimulation = load_json_file(simulation)?;
    let mut classification_reasons = Vec::new();
    let (run_trust_class, artifact_trust_class) = if simulation.simulated_backend {
        classification_reasons.push("simulated backend execution".to_string());
        ("simulated", "simulated")
    } else if simulation.policy_violations > 0 {
        classification_reasons
            .push(format!("policy violations present: {}", simulation.policy_violations));
        ("draft", "draft")
    } else if !simulation.evidence_complete {
        classification_reasons.push("required evidence is incomplete".to_string());
        ("operational", "operational")
    } else if simulation.advisory_only_controls {
        classification_reasons.push("control coverage is advisory-only".to_string());
        ("advisory", "advisory")
    } else if simulation.release_controls_met {
        classification_reasons.push("release controls and evidence are complete".to_string());
        ("release", "release")
    } else if simulation.audit_controls_met {
        classification_reasons.push("audit controls and evidence are complete".to_string());
        ("audit", "audit")
    } else {
        classification_reasons.push("baseline enforced controls passed".to_string());
        ("operational", "operational")
    };
    Ok(TrustClassesReport {
        policy_lane: "ENFORCED",
        run_id: simulation.run_id,
        run_trust_class,
        artifact_trust_class,
        evidence_complete: simulation.evidence_complete,
        classification_reasons,
    })
}

fn malformed_input_fuzz_payload(simulation: &Path) -> Result<MalformedInputFuzzReport, ExitCode> {
    let simulation: MalformedInputFuzzSimulation = load_json_file(simulation)?;
    let mut graph_rejections = 0usize;
    let mut config_rejections = 0usize;
    let mut route_rejections = 0usize;
    let mut plugin_manifest_rejections = 0usize;
    let mut bundle_rejections = 0usize;
    let mut run_manifest_rejections = 0usize;
    let mut panics_observed = 0usize;

    for payload in &simulation.graph_payloads {
        let outcome = std::panic::catch_unwind(|| parse_graph(payload));
        match outcome {
            Ok(Ok(_)) => {}
            Ok(Err(_)) => graph_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }
    for payload in &simulation.config_payloads {
        let outcome = std::panic::catch_unwind(|| {
            serde_json::from_str::<crate::PartialRuntimeSurfaceConfig>(payload)
        });
        match outcome {
            Ok(Ok(_)) => {}
            Ok(Err(_)) => config_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }
    for argv in &simulation.route_argv_sets {
        let outcome = std::panic::catch_unwind(|| {
            let mut args = Vec::with_capacity(argv.len() + 1);
            args.push("bijux-dag".to_string());
            args.extend(argv.iter().cloned());
            crate::dag_command().try_get_matches_from(args)
        });
        match outcome {
            Ok(Ok(_)) => {}
            Ok(Err(_)) => route_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }
    for payload in &simulation.plugin_manifest_payloads {
        let outcome =
            std::panic::catch_unwind(|| serde_json::from_str::<PluginManifestProbe>(payload));
        match outcome {
            Ok(Ok(manifest)) => {
                if manifest.namespace.trim().is_empty()
                    || manifest.version.trim().is_empty()
                    || manifest.entrypoint.trim().is_empty()
                    || manifest.trust_class.trim().is_empty()
                    || manifest.capabilities.is_empty()
                {
                    plugin_manifest_rejections += 1;
                }
            }
            Ok(Err(_)) => plugin_manifest_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }
    for payload in &simulation.bundle_payloads {
        let outcome =
            std::panic::catch_unwind(|| serde_json::from_str::<serde_json::Value>(payload));
        match outcome {
            Ok(Ok(bundle)) => {
                if !crate::verify_bundle_invariants(&bundle).is_empty() {
                    bundle_rejections += 1;
                }
            }
            Ok(Err(_)) => bundle_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }
    for payload in &simulation.run_manifest_payloads {
        let outcome =
            std::panic::catch_unwind(|| serde_json::from_str::<RunManifestProbe>(payload));
        match outcome {
            Ok(Ok(manifest)) => {
                if manifest.manifest_version.trim().is_empty()
                    || manifest.run_id.trim().is_empty()
                    || manifest.status.trim().is_empty()
                {
                    run_manifest_rejections += 1;
                }
            }
            Ok(Err(_)) => run_manifest_rejections += 1,
            Err(_) => panics_observed += 1,
        }
    }

    let cases_total = simulation.graph_payloads.len()
        + simulation.config_payloads.len()
        + simulation.route_argv_sets.len()
        + simulation.plugin_manifest_payloads.len()
        + simulation.bundle_payloads.len()
        + simulation.run_manifest_payloads.len();
    let mut gaps = Vec::new();
    if cases_total == 0 {
        gaps.push("fuzz corpus is empty".to_string());
    }
    if panics_observed > 0 {
        gaps.push("one or more malformed inputs triggered panic behavior".to_string());
    }
    let crash_free = panics_observed == 0;

    Ok(MalformedInputFuzzReport {
        policy_lane: "ENFORCED",
        cases_total,
        panics_observed,
        graph_rejections,
        config_rejections,
        route_rejections,
        plugin_manifest_rejections,
        bundle_rejections,
        run_manifest_rejections,
        crash_free,
        gaps,
    })
}

fn dependency_risk_payload(simulation: &Path) -> Result<DependencyRiskReport, ExitCode> {
    let simulation: DependencyRiskSimulation = load_json_file(simulation)?;
    let mut core_runtime_risk_score = 0.0f64;
    let mut tooling_risk_score = 0.0f64;
    let mut core_runtime_dependencies = Vec::new();
    let mut tooling_dependencies = Vec::new();
    let mut gaps = Vec::new();

    for dependency in &simulation.dependencies {
        let weighted_score =
            dependency.risk_score + (dependency.known_vulnerabilities as f64 * 0.5);
        match dependency.surface {
            DependencySurface::CoreRuntime => {
                core_runtime_risk_score += weighted_score;
                core_runtime_dependencies.push(dependency.name.clone());
            }
            DependencySurface::DocsTooling
            | DependencySurface::DevTooling
            | DependencySurface::ReleaseTooling => {
                tooling_risk_score += weighted_score;
                tooling_dependencies.push(dependency.name.clone());
            }
        }
    }

    core_runtime_dependencies.sort();
    tooling_dependencies.sort();
    let core_runtime_exceeds_threshold =
        core_runtime_risk_score > simulation.core_runtime_threshold;
    let tooling_exceeds_threshold = tooling_risk_score > simulation.tooling_threshold;

    if core_runtime_dependencies.is_empty() {
        gaps.push("core-runtime dependency inventory is empty".to_string());
    }
    if tooling_dependencies.is_empty() {
        gaps.push("tooling dependency inventory is empty".to_string());
    }
    if core_runtime_exceeds_threshold {
        gaps.push("core-runtime dependency risk exceeds threshold".to_string());
    }
    if tooling_exceeds_threshold {
        gaps.push("tooling dependency risk exceeds threshold".to_string());
    }

    Ok(DependencyRiskReport {
        policy_lane: "ENFORCED",
        core_runtime_risk_score,
        tooling_risk_score,
        core_runtime_exceeds_threshold,
        tooling_exceeds_threshold,
        core_runtime_dependencies,
        tooling_dependencies,
        gaps,
    })
}

fn data_access_payload(simulation: &Path) -> Result<DataAccessReport, ExitCode> {
    let simulation: DataAccessSimulation = load_json_file(simulation)?;
    let input_path_allowed = authorize_input_path(
        Path::new(&simulation.input_root),
        Path::new(&simulation.candidate_input),
    )
    .is_ok();
    let output_path_allowed = authorize_output_path(
        Path::new(&simulation.output_root),
        Path::new(&simulation.candidate_output),
    )
    .is_ok();
    let allowed_datasets =
        simulation.allowed_dataset_ids.into_iter().collect::<std::collections::BTreeSet<_>>();
    let dataset_entitlements_ok =
        simulation.requested_dataset_ids.iter().all(|dataset| allowed_datasets.contains(dataset));
    let lineage_scope = TenantLineageScope {
        tenant_id: parse_tenant_id(&simulation.tenant_id)?,
        allowed_artifact_ids: simulation.allowed_artifact_ids,
    };
    let visible_artifact_ids =
        scope_lineage_query(&simulation.requested_artifact_ids, &lineage_scope);

    let mut gaps = Vec::new();
    if !input_path_allowed {
        gaps.push("requested input path escapes the authorized input root".to_string());
    }
    if !output_path_allowed {
        gaps.push("requested output path escapes the authorized output root".to_string());
    }
    if !dataset_entitlements_ok {
        gaps.push("requested datasets exceed declared entitlements".to_string());
    }
    if visible_artifact_ids.len() != simulation.requested_artifact_ids.len() {
        gaps.push("artifact lineage query was narrowed by tenant scope".to_string());
    }

    Ok(DataAccessReport {
        input_path_allowed,
        output_path_allowed,
        dataset_entitlements_ok,
        visible_artifact_ids,
        gaps,
    })
}

fn override_payload(simulation: &Path) -> Result<OverrideReport, ExitCode> {
    let simulation: OverrideSimulation = load_json_file(simulation)?;
    let request = PolicyEvaluationRequest {
        request_id: "security-override".to_string(),
        subject: SubjectIdentity {
            subject_id: simulation.actor,
            kind: SubjectKind::User,
            tenant_id: simulation.tenant_id.clone(),
        },
        action: Action {
            name: simulation.action_name.clone(),
            kind: simulation.action_kind.clone(),
        },
        resource: ResourceRef {
            kind: simulation.resource_kind,
            id: simulation.resource_id,
            tenant_id: simulation.resource_tenant_id.clone(),
        },
        scope: match simulation.resource_tenant_id.clone().or(simulation.tenant_id.clone()) {
            Some(tenant_id) => ResourceScope::Tenant { tenant_id },
            None => ResourceScope::Global,
        },
        environment: simulation.environment.clone(),
    };
    let dry_run = evaluate_dry_run(
        &request,
        &simulation.granted_permissions,
        &simulation.policy_bundle_version,
    );
    let environment_allows = is_action_allowed_in_environment(
        &simulation.action_name,
        &simulation.environment,
        &simulation.environment_rules,
    );
    let reason_recorded = !simulation.reason.trim().is_empty();
    let audit_recorded =
        simulation.audit_event_id.as_ref().is_some_and(|value| !value.trim().is_empty());
    let approval_quorum_met = simulation.approvers.len() >= simulation.required_approvals;
    let scoped_to_tenant = simulation.tenant_id.is_none()
        || simulation.resource_tenant_id.is_none()
        || simulation.tenant_id == simulation.resource_tenant_id;
    let dual_control_required =
        simulation.break_glass || matches!(simulation.action_kind, ActionKind::Administer);
    let break_glass_policy_valid =
        !simulation.break_glass || (reason_recorded && audit_recorded && approval_quorum_met);
    let mut gaps = Vec::new();
    if !dry_run.would_allow {
        gaps.push("override actor lacks the required permission".to_string());
    }
    if !environment_allows {
        gaps.push("override is denied in this environment".to_string());
    }
    if !reason_recorded {
        gaps.push("override reason is missing".to_string());
    }
    if !audit_recorded {
        gaps.push("override audit record is missing".to_string());
    }
    if !scoped_to_tenant {
        gaps.push("override crosses tenant scope boundaries".to_string());
    }
    if dual_control_required && !approval_quorum_met {
        gaps.push("override lacks the required approval quorum".to_string());
    }
    if !break_glass_policy_valid {
        gaps.push("break-glass override is not fully justified".to_string());
    }
    Ok(OverrideReport {
        override_allowed: dry_run.would_allow
            && environment_allows
            && break_glass_policy_valid
            && scoped_to_tenant,
        reason_recorded,
        audit_recorded,
        environment_allows,
        break_glass_policy_valid,
        approval_quorum_met,
        scoped_to_tenant,
        gaps,
    })
}

fn override_audit_payload(simulation: &Path) -> Result<OverrideAuditReport, ExitCode> {
    let simulation: OverrideAuditSimulation = load_json_file(simulation)?;
    let mut recorded_actions = Vec::new();
    if simulation.forced_rerun {
        recorded_actions.push("forced-rerun".to_string());
    }
    if simulation.bypassed_policy {
        recorded_actions.push("bypassed-policy".to_string());
    }
    if simulation.cache_disabled {
        recorded_actions.push("cache-disabled".to_string());
    }
    if simulation.accepted_degraded_evidence {
        recorded_actions.push("accepted-degraded-evidence".to_string());
    }
    if simulation.retention_change.is_some() {
        recorded_actions.push("retention-changed".to_string());
    }

    let reason_recorded = !simulation.reason.trim().is_empty();
    let evidence_recorded =
        simulation.evidence_pointer.as_ref().is_some_and(|pointer| !pointer.trim().is_empty());
    let event_recorded =
        simulation.audit_event_id.as_ref().is_some_and(|event_id| !event_id.trim().is_empty());
    let timestamp_recorded = simulation.timestamp_unix_ms > 0;

    let mut missing_records = Vec::new();
    if !recorded_actions.is_empty() && !reason_recorded {
        missing_records.push("override reason is missing".to_string());
    }
    if !recorded_actions.is_empty() && !evidence_recorded {
        missing_records.push("evidence pointer is missing".to_string());
    }
    if !recorded_actions.is_empty() && !event_recorded {
        missing_records.push("audit event id is missing".to_string());
    }
    if !recorded_actions.is_empty() && !timestamp_recorded {
        missing_records.push("audit timestamp is missing".to_string());
    }

    Ok(OverrideAuditReport {
        policy_lane: "ENFORCED",
        audit_complete: missing_records.is_empty(),
        recorded_actions,
        missing_records,
        actor: simulation.actor,
        run_id: simulation.run_id,
        audit_event_id: simulation.audit_event_id,
    })
}

fn safe_defaults_payload(dag: &Path) -> Result<SafeDefaultsReport, ExitCode> {
    let graph = parse_graph(&read_file(dag)?)?;
    let workflow_name =
        graph.meta.as_ref().map(|meta| meta.name.clone()).unwrap_or_else(|| "unnamed".to_string());
    let mut nodes = Vec::new();
    let mut gaps = Vec::new();
    if graph.meta.as_ref().is_none_or(|meta| meta.owners.is_empty()) {
        gaps.push("workflow has no owners".to_string());
    }
    if graph.meta.as_ref().is_none_or(|meta| meta.tags.is_empty()) {
        gaps.push("workflow has no taxonomy tags".to_string());
    }
    if graph.nondeterminism_allowed {
        gaps.push("workflow permits nondeterministic execution by default".to_string());
    }
    for node in &graph.nodes {
        let mut risky_defaults = Vec::new();
        let container_env_allowlist =
            node.container.as_ref().map(|spec| spec.env_allowlist.as_slice()).unwrap_or(&[]);
        if node.timeout_ms.is_none() {
            risky_defaults.push("missing-timeout".to_string());
        }
        if node.resources.is_none() {
            risky_defaults.push("missing-resource-bounds".to_string());
        }
        if !node.effects.is_empty() && node.tags.is_empty() {
            risky_defaults.push("effectful-node-without-tags".to_string());
        }
        if node.effects.iter().any(|effect| matches!(effect, bijux_dag_core::Effect::Network)) {
            risky_defaults.push("network-effect-enabled".to_string());
        }
        if node.effects.iter().any(|effect| matches!(effect, bijux_dag_core::Effect::Clock)) {
            risky_defaults.push("clock-effect-enabled".to_string());
        }
        if node.effects.iter().any(|effect| matches!(effect, bijux_dag_core::Effect::Env))
            && node.env_allowlist.is_empty()
            && container_env_allowlist.is_empty()
        {
            risky_defaults.push("env-effect-without-allowlist".to_string());
        }
        if !risky_defaults.is_empty() {
            gaps.push(format!(
                "node {} has unsafe defaults: {}",
                node.id,
                risky_defaults.join(",")
            ));
        }
        nodes.push(SafeDefaultsNodeReport { node_id: node.id.clone(), risky_defaults });
    }
    Ok(SafeDefaultsReport { workflow_name, safe_by_default: gaps.is_empty(), nodes, gaps })
}

pub(crate) fn handle_security_command(
    cli: &DagCli,
    command: &SecurityCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        SecurityCommands::FilesystemAllowlist { simulation } => {
            let payload = serde_json::to_value(filesystem_allowlist_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.filesystem-allowlist",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::EnvAllowlist { simulation } => {
            let payload = serde_json::to_value(env_allowlist_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.env-allowlist",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::NetworkPolicy { dag } => {
            let payload = serde_json::to_value(network_policy_payload(dag)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.network-policy",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::CommandInjection { simulation } => {
            let payload = serde_json::to_value(command_injection_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.command-injection",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::ArtifactSecrets { simulation } => {
            let payload = serde_json::to_value(artifact_secrets_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.artifact-secrets",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::Auth { simulation } => {
            let payload =
                serde_json::to_value(auth_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.auth", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Authz { simulation } => {
            let payload =
                serde_json::to_value(authz_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.authz", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Tenant { simulation } => {
            let payload =
                serde_json::to_value(tenant_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.tenant", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Secrets { simulation } => {
            let payload = serde_json::to_value(secrets_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.secrets", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::SupplyChain { simulation } => {
            let payload = serde_json::to_value(supply_chain_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.supply-chain",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::SupplyInventory { simulation } => {
            let payload = serde_json::to_value(supply_inventory_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.supply-inventory",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::TrustClasses { simulation } => {
            let payload = serde_json::to_value(trust_classes_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.trust-classes",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::MalformedInputFuzz { simulation } => {
            let payload = serde_json::to_value(malformed_input_fuzz_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.malformed-input-fuzz",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::DependencyRisk { simulation } => {
            let payload = serde_json::to_value(dependency_risk_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.dependency-risk",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::DataAccess { simulation } => {
            let payload = serde_json::to_value(data_access_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.data-access", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Override { simulation } => {
            let payload = serde_json::to_value(override_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.override", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::OverrideAudit { simulation } => {
            let payload = serde_json::to_value(override_audit_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.override-audit",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        SecurityCommands::SafeDefaults { dag } => {
            let payload =
                serde_json::to_value(safe_defaults_payload(dag)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.security.safe-defaults",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_security_command;
    use crate::commands::{Commands, DagCli, SecurityCommands};
    use crate::ExitCode;

    fn quiet_json_cli(command: SecurityCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Security { command } }
    }

    #[test]
    fn security_filesystem_allowlist_accepts_scoped_candidates() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let input_root = dir.path().join("inputs");
        let output_root = dir.path().join("outputs");
        std::fs::create_dir_all(&input_root).expect("input root");
        std::fs::create_dir_all(&output_root).expect("output root");
        let read_file = input_root.join("a").join("in.txt");
        let write_file = output_root.join("b").join("out.txt");
        std::fs::create_dir_all(read_file.parent().expect("read parent")).expect("read parent");
        std::fs::create_dir_all(write_file.parent().expect("write parent")).expect("write parent");
        std::fs::write(&read_file, "ok").expect("write read file");
        std::fs::write(&write_file, "ok").expect("write write file");

        let simulation = dir.path().join("filesystem.json");
        std::fs::write(
            &simulation,
            format!(
                r#"{{
                  "input_root":"{}",
                  "output_root":"{}",
                  "read_candidates":["{}"],
                  "write_candidates":["{}"]
                }}"#,
                input_root.display(),
                output_root.display(),
                read_file.display(),
                write_file.display()
            ),
        )
        .expect("write simulation");

        let cli = quiet_json_cli(SecurityCommands::FilesystemAllowlist {
            simulation: simulation.clone(),
        });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::FilesystemAllowlist { simulation: simulation.clone() },
        )
        .expect("filesystem allowlist");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::filesystem_allowlist_payload(&simulation).expect("report");
        assert_eq!(report.policy_lane, "ENFORCED");
        assert!(report.all_reads_allowed);
        assert!(report.all_writes_allowed);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_filesystem_allowlist_rejects_traversal_and_symlink_escape() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let input_root = dir.path().join("inputs");
        let output_root = dir.path().join("outputs");
        let outside_root = dir.path().join("outside");
        std::fs::create_dir_all(&input_root).expect("input root");
        std::fs::create_dir_all(&output_root).expect("output root");
        std::fs::create_dir_all(&outside_root).expect("outside root");
        let outside_read = outside_root.join("outside.txt");
        let outside_write = outside_root.join("outside-out.txt");
        std::fs::write(&outside_read, "outside").expect("outside read");
        std::fs::write(&outside_write, "outside").expect("outside write");

        let symlink_in = input_root.join("escape-link");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside_read, &symlink_in).expect("create symlink");
        #[cfg(not(unix))]
        std::fs::copy(&outside_read, &symlink_in).expect("copy fallback");

        let escaped_read = input_root.join("..").join("outside").join("outside.txt");
        let escaped_write = output_root.join("..").join("outside").join("outside-out.txt");
        let simulation = dir.path().join("filesystem-bad.json");
        std::fs::write(
            &simulation,
            format!(
                r#"{{
                  "input_root":"{}",
                  "output_root":"{}",
                  "read_candidates":["{}","{}"],
                  "write_candidates":["{}"]
                }}"#,
                input_root.display(),
                output_root.display(),
                escaped_read.display(),
                symlink_in.display(),
                escaped_write.display()
            ),
        )
        .expect("write simulation");

        let report = super::filesystem_allowlist_payload(&simulation).expect("report");
        assert!(!report.all_reads_allowed);
        assert!(!report.all_writes_allowed);
        assert!(report.gaps.iter().any(|gap| gap.contains("escapes authorized root")));
    }

    #[test]
    fn security_env_allowlist_accepts_clean_filtered_environment() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("env.json");
        std::fs::write(
            &simulation,
            r#"{
              "clean_env":false,
              "allowlist":["BIJUX_*"],
              "denylist":["BIJUX_DENY_*"],
              "ambient_env":{"BIJUX_ALLOWED_A":"1","BIJUX_DENY_A":"2","SECRET_TOKEN":"raw"},
              "explicit_env":{"BIJUX_ALLOWED_B":"3"}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::EnvAllowlist { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::EnvAllowlist { simulation: simulation.clone() },
        )
        .expect("env allowlist");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::env_allowlist_payload(&simulation).expect("report");
        assert_eq!(report.policy_lane, "ENFORCED");
        assert_eq!(
            report.passed_keys,
            vec!["BIJUX_ALLOWED_A".to_string(), "BIJUX_ALLOWED_B".to_string()]
        );
        assert!(report.secret_like_keys_blocked);
        assert!(report.leaked_keys.is_empty());
    }

    #[test]
    fn security_env_allowlist_flags_secret_like_environment_leaks() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("env-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "clean_env":false,
              "allowlist":["*"],
              "denylist":[],
              "ambient_env":{"SECRET_TOKEN":"raw","BIJUX_ALLOWED_A":"1"},
              "explicit_env":{"PASSWORD":"raw"}
            }"#,
        )
        .expect("write simulation");
        let report = super::env_allowlist_payload(&simulation).expect("report");
        assert!(!report.secret_like_keys_blocked);
        assert!(report.leaked_keys.iter().any(|key| key == "SECRET_TOKEN"));
        assert!(report.leaked_keys.iter().any(|key| key == "PASSWORD"));
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "secret-like environment keys are not fully blocked"));
    }

    #[test]
    fn security_network_policy_accepts_graph_without_network_effects() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"no-network","owners":["team-core"],"tags":["prod"]},
              "nodes":[{"id":"n1","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"ok"}}],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let cli = quiet_json_cli(SecurityCommands::NetworkPolicy { dag: dag.clone() });
        let code =
            handle_security_command(&cli, &SecurityCommands::NetworkPolicy { dag: dag.clone() })
                .expect("network policy");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::network_policy_payload(&dag).expect("report");
        assert_eq!(report.policy_lane, "ENFORCED");
        assert!(!report.network_nodes_present);
        assert_eq!(report.cache_trust_impact, "none");
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_network_policy_flags_unapproved_network_effects_as_advisory() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"with-network","owners":["team-core"],"tags":["prod"]},
              "nodes":[
                {
                  "id":"n1",
                  "kind":"shell",
                  "inputs":[],
                  "outputs":[{"name":"out","path":"out"}],
                  "params":{"argv":["/bin/true"]},
                  "effects":["filesystem","network"],
                  "tags":["filesystem-reviewed"]
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let report = super::network_policy_payload(&dag).expect("report");
        assert_eq!(report.policy_lane, "ADVISORY");
        assert!(report.network_nodes_present);
        assert!(!report.network_nodes_approved);
        assert_eq!(report.cache_trust_impact, "reduced");
        assert!(report.gaps.iter().any(|gap| gap.contains("network-approved tag")));
    }

    #[test]
    fn security_command_injection_accepts_safe_argv_contract() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("inj.json");
        std::fs::write(
            &simulation,
            r#"{
              "command_argv":["/usr/bin/python3","script.py","--input","file.txt"],
              "explicit_shell":false,
              "allow_metacharacters":false,
              "working_directory":"./work"
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::CommandInjection { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::CommandInjection { simulation: simulation.clone() },
        )
        .expect("command injection");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::command_injection_payload(&simulation).expect("report");
        assert!(report.injection_hardened);
        assert!(!report.shell_interpretation_requested);
        assert!(report.risky_tokens.is_empty());
    }

    #[test]
    fn security_command_injection_flags_implicit_shell_and_metacharacters() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("inj-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "command_argv":["/bin/sh","-c","cat input.txt | grep token=foo"],
              "explicit_shell":false,
              "allow_metacharacters":false,
              "working_directory":"../escape"
            }"#,
        )
        .expect("write simulation");
        let report = super::command_injection_payload(&simulation).expect("report");
        assert!(!report.injection_hardened);
        assert!(report.implicit_shell_detected);
        assert!(report.risky_tokens.iter().any(|token| token.contains("|")));
        for expected in [
            "implicit shell interpretation is forbidden; require explicit_shell=true",
            "metacharacter-bearing argv tokens require explicit allow_metacharacters",
            "working_directory contains parent traversal segments",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_artifact_secrets_accepts_clean_durable_fields() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("artifact-secrets-clean.json");
        std::fs::write(
            &simulation,
            r#"{
              "durable_fields":{
                "logs.stdout":"run completed",
                "config.profile":"prod",
                "outputs.summary":"ok"
              },
              "redaction_enabled":true,
              "refuse_on_secret":true
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::ArtifactSecrets { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::ArtifactSecrets { simulation: simulation.clone() },
        )
        .expect("artifact secrets");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::artifact_secrets_payload(&simulation).expect("report");
        assert!(report.flagged_fields.is_empty());
        assert!(report.durable_write_allowed);
        assert_eq!(report.action, "clean");
    }

    #[test]
    fn security_artifact_secrets_redacts_or_refuses_seeded_secret_fields() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("artifact-secrets-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "durable_fields":{
                "logs.stderr":"Bearer token=abc123",
                "config.db_password":"super-secret",
                "outputs.api_key":"AKIAEXAMPLE"
              },
              "redaction_enabled":true,
              "refuse_on_secret":true
            }"#,
        )
        .expect("write simulation");
        let report = super::artifact_secrets_payload(&simulation).expect("report");
        assert!(!report.flagged_fields.is_empty());
        assert!(report.flagged_fields.iter().any(|field| field == "logs.stderr"));
        assert!(report.flagged_fields.iter().any(|field| field == "config.db_password"));
        assert!(report.flagged_fields.iter().any(|field| field == "outputs.api_key"));
        assert!(!report.durable_write_allowed);
        assert_eq!(report.action, "refused");
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "durable artifact write refused due to secret-bearing fields"));
    }

    #[test]
    fn security_auth_accepts_isolated_federation_ready_identity() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("auth.json");
        std::fs::write(
            &simulation,
            r#"{
              "environment":"prod",
              "now_unix_ms":100,
              "renewal_count":0,
              "short_lived_worker_creds_supported":true,
              "revocation_propagation_supported":true,
              "lifecycle":{"issued_unix_ms":1,"expires_unix_ms":200,"renewable":true,"max_renewals":3},
              "scope":{"cli":true,"api_client":true,"scheduler":true,"worker":true},
              "principals":[{"principal_id":"svc-prod","kind":"Service","tenant_id":"atlas"}],
              "credential_classes":["oidc","worker-lease"],
              "policy_baselines":["least-privilege","short-lived-creds"],
              "bypass_rules":[{"enabled":false,"environment":"local","marker":"dev-only"}],
              "auth_events":[
                {"kind":"Login","principal_id":"svc-prod","unix_ms":1,"reason":null},
                {"kind":"Refresh","principal_id":"svc-prod","unix_ms":2,"reason":null},
                {"kind":"Revoke","principal_id":"svc-prod","unix_ms":3,"reason":"rotation"},
                {"kind":"Failure","principal_id":"svc-prod","unix_ms":4,"reason":"denied"}
              ],
              "revocations":[]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::Auth { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::Auth { simulation: simulation.clone() },
        )
        .expect("auth");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::auth_payload(&simulation).expect("report");
        assert!(report.anonymous_access_blocked);
        assert!(report.local_bypass_blocked);
        assert!(report.federation_ready);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_auth_flags_expired_and_bypassable_identity() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("auth.json");
        std::fs::write(
            &simulation,
            r#"{
              "environment":"prod",
              "now_unix_ms":500,
              "renewal_count":4,
              "short_lived_worker_creds_supported":false,
              "revocation_propagation_supported":false,
              "lifecycle":{"issued_unix_ms":1,"expires_unix_ms":100,"renewable":true,"max_renewals":1},
              "scope":{"cli":true,"api_client":false,"scheduler":false,"worker":false},
              "principals":[],
              "credential_classes":[],
              "policy_baselines":[],
              "bypass_rules":[{"enabled":true,"environment":"prod","marker":"bad"}],
              "auth_events":[{"kind":"Login","principal_id":"svc-prod","unix_ms":1,"reason":null}],
              "revocations":[{"principal_id":"svc-prod","reason":"compromised","revoked_unix_ms":10,"propagate_to_running_operations":true}]
            }"#,
        )
        .expect("write simulation");
        let report = super::auth_payload(&simulation).expect("report");
        assert!(!report.anonymous_access_blocked);
        assert!(!report.local_bypass_blocked);
        assert!(report.credential_expired);
        assert!(!report.federation_ready);
        for expected in [
            "no authenticated principals are configured",
            "local bypass is enabled outside the local environment",
            "credential lifecycle is already expired",
            "authentication audit chain is incomplete",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_authz_accepts_least_privilege_policy_path() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("authz.json");
        std::fs::write(
            &simulation,
            r#"{
              "subject_id":"operator-a",
              "subject_kind":"User",
              "tenant_id":"atlas",
              "action_name":"run.cancel",
              "action_kind":"Execute",
              "resource_kind":"Run",
              "resource_id":"run-01",
              "resource_tenant_id":"atlas",
              "environment":"prod",
              "policy_bundle_version":"2026-04-28",
              "granted_permissions":["run.cancel","run.read"],
              "environment_rules":[{"environment":"prod","denied_actions":["platform.administer"]}],
              "decisions":[["run.cancel","Allow"],["platform.administer","Deny"]],
              "cross_tenant_denials":[true,true],
              "custom_role":{"role_name":"run-operator","permissions":["run.cancel","run.read"]}
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::Authz { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::Authz { simulation: simulation.clone() },
        )
        .expect("authz");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::authz_payload(&simulation).expect("report");
        assert!(report.dry_run_allow);
        assert!(report.environment_allows);
        assert!(report.least_privilege_holds);
        assert!(report.cross_tenant_escalation_blocked);
        assert!(report.custom_role_valid);
        assert!(report.failures.is_empty());
    }

    #[test]
    fn security_authz_flags_environment_and_role_violations() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("authz.json");
        std::fs::write(
            &simulation,
            r#"{
              "subject_id":"admin-a",
              "subject_kind":"User",
              "tenant_id":"atlas",
              "action_name":"platform.administer",
              "action_kind":"Administer",
              "resource_kind":"Tenant",
              "resource_id":"atlas",
              "resource_tenant_id":"atlas",
              "environment":"prod",
              "policy_bundle_version":"2026-04-28",
              "granted_permissions":["platform.administer","tenant.manage"],
              "environment_rules":[{"environment":"prod","denied_actions":["platform.administer"]}],
              "decisions":[["platform.administer","Allow"]],
              "cross_tenant_denials":[false],
              "custom_role":{"role_name":"bad-admin","permissions":["platform.administer","tenant.manage"]}
            }"#,
        )
        .expect("write simulation");
        let report = super::authz_payload(&simulation).expect("report");
        assert!(!report.environment_allows);
        assert!(!report.least_privilege_holds);
        assert!(!report.cross_tenant_escalation_blocked);
        assert!(!report.custom_role_valid);
        for expected in [
            "least-privilege boundary violated",
            "cross-tenant escalation was allowed",
            "environment rule denies this action",
            "custom role definition is invalid",
        ] {
            assert!(report.failures.iter().any(|failure| failure == expected));
        }
    }

    #[test]
    fn security_tenant_accepts_fully_isolated_request() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("tenant.json");
        std::fs::write(
            &simulation,
            r#"{
              "requested_tenant":"atlas",
              "api_tenant":"atlas",
              "scheduler_tenant":"atlas",
              "artifact_tenant":"atlas",
              "metrics_tenant":"atlas",
              "lineage_tenant":"atlas",
              "queued_runs":2,
              "pending_dispatches":1,
              "plugin_name":"builtin-python",
              "requested_artifact_ids":["a1","a2"],
              "global_defaults":{"retry":"3"},
              "overlay_values":{"region":"eu"},
              "overlay_overrides":{"retry":"5"},
              "queue_names":["atlas-main"],
              "allowed_plugins":["builtin-python","builtin-shell"],
              "allowed_artifact_ids":["a1","a2","a3"],
              "namespace":"atlas-prod",
              "storage_partition":"s3://atlas",
              "index_prefix":"atlas/",
              "policy_bundle_id":"bundle-1",
              "policy_bundle_version":"v1",
              "max_enqueued_runs":5,
              "max_dispatches_per_tick":3
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::Tenant { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::Tenant { simulation: simulation.clone() },
        )
        .expect("tenant");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::tenant_payload(&simulation).expect("report");
        assert!(report.isolated);
        assert!(report.scheduler_admitted);
        assert!(report.plugin_allowed);
        assert_eq!(report.visible_artifact_ids, vec!["a1".to_string(), "a2".to_string()]);
        assert_eq!(report.merged_config.get("retry").map(String::as_str), Some("5"));
        assert!(report.violations.is_empty());
    }

    #[test]
    fn security_tenant_flags_scope_admission_and_plugin_violations() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("tenant.json");
        std::fs::write(
            &simulation,
            r#"{
              "requested_tenant":"atlas",
              "api_tenant":"other",
              "scheduler_tenant":"atlas",
              "artifact_tenant":"other",
              "metrics_tenant":"atlas",
              "lineage_tenant":"other",
              "queued_runs":20,
              "pending_dispatches":10,
              "plugin_name":"unapproved",
              "requested_artifact_ids":["a1","a2"],
              "global_defaults":{},
              "overlay_values":{},
              "overlay_overrides":{},
              "queue_names":["atlas-main"],
              "allowed_plugins":["builtin-python"],
              "allowed_artifact_ids":["a1"],
              "namespace":"atlas-prod",
              "storage_partition":"s3://atlas",
              "index_prefix":"atlas/",
              "policy_bundle_id":"bundle-1",
              "policy_bundle_version":"v1",
              "max_enqueued_runs":5,
              "max_dispatches_per_tick":3
            }"#,
        )
        .expect("write simulation");
        let report = super::tenant_payload(&simulation).expect("report");
        assert!(!report.isolated);
        assert!(!report.scheduler_admitted);
        assert!(!report.plugin_allowed);
        assert_eq!(report.visible_artifact_ids, vec!["a1".to_string()]);
        for expected in [
            "api tenant scope mismatch",
            "artifact tenant scope mismatch",
            "lineage tenant scope mismatch",
            "scheduler admission quota exceeded",
            "plugin allowlist rejected the requested plugin",
        ] {
            assert!(report.violations.iter().any(|violation| violation == expected));
        }
    }

    #[test]
    fn security_secrets_accepts_brokered_region_scoped_delivery() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("secrets.json");
        std::fs::write(
            &simulation,
            r#"{
              "available_versions":["v1","v2"],
              "pinned_version":"v2",
              "is_backfill":false,
              "allowed_regions":["eu","us"],
              "requested_region":"eu",
              "uses_static_secret":false,
              "secret_contract":{"secret_refs":["vault:db"],"injection_mode":"backend-native","redaction_required":true},
              "rotation":{"allow_latest":true,"require_pin_for_backfill":true},
              "delivery_policy":{"allowed_modes":["BackendNative"],"deny_process_args":true},
              "delivery_mode":"BackendNative",
              "allowed_scope":{"tenant_id":"atlas","dag_id":"wf","run_id":null,"node_id":null,"worker_id":null},
              "requested_scope":{"tenant_id":"atlas","dag_id":"wf","run_id":"run-1","node_id":"n1","worker_id":"w1"},
              "masking_policy":{"redact_logs":true,"redact_diagnostics":true,"redact_manifests":true,"redact_exports":true},
              "sources":[{"ExternalManager":{"provider":"vault","path":"secret/data/db"}}],
              "audit_events":[{"secret_id":"db","node_id":"n1","run_id":"run-1","unix_ms":1,"access_mode":"read"}],
              "secure_mode":{"enabled":true,"environment":"prod","strict_policy_bundle":"strict-v1"},
              "workspace_rule":{"secure_temp_cleanup":true,"remove_secret_mounts_on_exit":true},
              "teardown_policy":{"wipe_env_on_cancel":true,"wipe_files_on_cancel":true,"teardown_timeout_ms":1000},
              "restrictions":[{"class":"Regulated","min_retention_days":30,"export_requires_approval":true}],
              "observed_outputs":["all good"]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::Secrets { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::Secrets { simulation: simulation.clone() },
        )
        .expect("secrets");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::secrets_payload(&simulation).expect("report");
        assert_eq!(report.selected_version.as_deref(), Some("v2"));
        assert!(report.delivery_allowed);
        assert!(report.scope_allowed);
        assert!(report.readiness_ok);
        assert!(report.strict_mode_effective);
        assert!(report.cleanup_required);
        assert!(report.leak_clean);
        assert!(report.brokered_credentials);
        assert!(report.region_allowed);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_secrets_flags_static_leaky_cross_region_posture() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("secrets.json");
        std::fs::write(
            &simulation,
            r#"{
              "available_versions":["v1"],
              "pinned_version":null,
              "is_backfill":true,
              "allowed_regions":["eu"],
              "requested_region":"us",
              "uses_static_secret":true,
              "secret_contract":{"secret_refs":[],"injection_mode":"env","redaction_required":false},
              "rotation":{"allow_latest":false,"require_pin_for_backfill":true},
              "delivery_policy":{"allowed_modes":["FileMount"],"deny_process_args":true},
              "delivery_mode":"Env",
              "allowed_scope":{"tenant_id":"atlas","dag_id":"wf","run_id":null,"node_id":null,"worker_id":null},
              "requested_scope":{"tenant_id":"other","dag_id":"wf","run_id":"run-1","node_id":"n1","worker_id":"w1"},
              "masking_policy":{"redact_logs":false,"redact_diagnostics":true,"redact_manifests":true,"redact_exports":true},
              "sources":[],
              "audit_events":[],
              "secure_mode":{"enabled":true,"environment":"prod","strict_policy_bundle":"strict-v1"},
              "workspace_rule":{"secure_temp_cleanup":false,"remove_secret_mounts_on_exit":true},
              "teardown_policy":{"wipe_env_on_cancel":false,"wipe_files_on_cancel":false,"teardown_timeout_ms":1000},
              "restrictions":[],
              "observed_outputs":["password=plaintext"]
            }"#,
        )
        .expect("write simulation");
        let report = super::secrets_payload(&simulation).expect("report");
        assert!(report.selected_version.is_none());
        assert!(!report.delivery_allowed);
        assert!(!report.scope_allowed);
        assert!(!report.leak_clean);
        assert!(!report.brokered_credentials);
        assert!(!report.region_allowed);
        for expected in [
            "no valid secret version could be selected",
            "secret delivery mode is not allowed",
            "secret scope does not allow the requested access",
            "secret integration readiness is incomplete",
            "secure cleanup policy is incomplete",
            "observed outputs contain secret-looking material",
            "workload still depends on static or non-brokered credentials",
            "requested region is outside the approved secret region set",
            "runtime secret contract is incomplete",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_supply_chain_accepts_complete_attested_artifact() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("supply.json");
        std::fs::write(
            &simulation,
            r#"{
              "trust_label":"Approved",
              "completeness_policy":{
                "require_binary_provenance":true,
                "require_plugin_provenance":true,
                "require_environment_attestation":true,
                "require_signed_artifacts":true
              },
              "promotion_policy":{"allowed_labels":["Approved","Attested"],"require_completeness":true},
              "attestation":{
                "run_id":"run-1",
                "dag_snapshot_id":"graph-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"v1",
                "output_digests":["sha256:1"],
                "binaries":[{"component":"Scheduler","version":"1","build_id":"b1","source_revision":"r1","build_timestamp_utc":"2026-04-28T00:00:00Z"}],
                "plugins":[{"plugin_name":"builtin-python","version":"1","source":"bijux","trust_tier":"Official","approved":true}],
                "environment":{"backend":"kubernetes","capability_class":"standard","trust_domain":"prod-eu"}
              },
              "signed_manifests":[{"artifact_id":"a1","digest":"sha256:1","signer_identity":"svc","signature":"sig"}]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::SupplyChain { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::SupplyChain { simulation: simulation.clone() },
        )
        .expect("supply");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::supply_chain_payload(&simulation).expect("report");
        assert!(report.completeness_passed);
        assert!(report.promotion_allowed);
        assert!(report.pinned_inputs);
        assert!(report.trust_domain_bound);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_supply_chain_rejects_incomplete_untrusted_artifact() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("supply.json");
        std::fs::write(
            &simulation,
            r#"{
              "trust_label":"Verified",
              "completeness_policy":{
                "require_binary_provenance":true,
                "require_plugin_provenance":true,
                "require_environment_attestation":true,
                "require_signed_artifacts":true
              },
              "promotion_policy":{"allowed_labels":["Approved"],"require_completeness":true},
              "attestation":{
                "run_id":"run-1",
                "dag_snapshot_id":"graph-1",
                "plan_fingerprint":"plan-1",
                "policy_bundle_version":"v1",
                "output_digests":[],
                "binaries":[],
                "plugins":[],
                "environment":{"backend":"","capability_class":"","trust_domain":""}
              },
              "signed_manifests":[]
            }"#,
        )
        .expect("write simulation");
        let report = super::supply_chain_payload(&simulation).expect("report");
        assert!(!report.completeness_passed);
        assert!(!report.promotion_allowed);
        assert!(!report.pinned_inputs);
        assert!(!report.trust_domain_bound);
        for expected in [
            "binary provenance is required",
            "plugin provenance is required",
            "environment attestation is incomplete",
            "signed artifacts are required",
            "artifact promotion policy rejected this trust posture",
            "artifact digests are not fully pinned",
            "environment trust domain is missing",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_supply_inventory_accepts_checksum_complete_inventory() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("supply-inventory.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-77",
              "binaries":[{"id":"bijux-dag","version":"0.4.0","checksum":"sha256:111"}],
              "containers":[{"id":"ghcr.io/bijux/runtime","version":"2026.04","checksum":"sha256:222"}],
              "plugins":[{"id":"builtin-python","version":"1.2.0","checksum":"sha256:333"}],
              "adapters":[{"id":"shell","version":"1","checksum":"sha256:444"}]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::SupplyInventory { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::SupplyInventory { simulation: simulation.clone() },
        )
        .expect("supply inventory");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::supply_inventory_payload(&simulation).expect("report");
        assert!(report.inventory_complete);
        assert!(report.gaps.is_empty());
        assert_eq!(report.run_id, "run-77");
    }

    #[test]
    fn security_supply_inventory_flags_missing_component_checksums() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("supply-inventory-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-88",
              "binaries":[{"id":"bijux-dag","version":"0.4.0","checksum":null}],
              "containers":[{"id":"ghcr.io/bijux/runtime","version":"2026.04","checksum":" "}],
              "plugins":[{"id":"builtin-python","version":"1.2.0","checksum":"sha256:333"}],
              "adapters":[{"id":"shell","version":"","checksum":"sha256:444"}]
            }"#,
        )
        .expect("write simulation");
        let report = super::supply_inventory_payload(&simulation).expect("report");
        assert!(!report.inventory_complete);
        for expected in [
            "binary component bijux-dag@0.4.0 is missing a checksum",
            "container component ghcr.io/bijux/runtime@2026.04 is missing a checksum",
            "adapter component has missing id/version fields",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_trust_classes_classifies_release_grade_evidence() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("trust-classes.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-900",
              "evidence_complete":true,
              "advisory_only_controls":false,
              "simulated_backend":false,
              "release_controls_met":true,
              "audit_controls_met":true,
              "policy_violations":0
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::TrustClasses { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::TrustClasses { simulation: simulation.clone() },
        )
        .expect("trust classes");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::trust_classes_payload(&simulation).expect("report");
        assert_eq!(report.run_trust_class, "release");
        assert_eq!(report.artifact_trust_class, "release");
        assert!(report
            .classification_reasons
            .iter()
            .any(|reason| { reason == "release controls and evidence are complete" }));
    }

    #[test]
    fn security_trust_classes_downgrades_simulated_or_incomplete_runs() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("trust-classes-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "run_id":"run-901",
              "evidence_complete":false,
              "advisory_only_controls":true,
              "simulated_backend":true,
              "release_controls_met":false,
              "audit_controls_met":false,
              "policy_violations":2
            }"#,
        )
        .expect("write simulation");
        let report = super::trust_classes_payload(&simulation).expect("report");
        assert_eq!(report.run_trust_class, "simulated");
        assert_eq!(report.artifact_trust_class, "simulated");
        assert!(report
            .classification_reasons
            .iter()
            .any(|reason| reason == "simulated backend execution"));
    }

    #[test]
    fn security_malformed_input_fuzz_reports_crash_free_corpus() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("malformed-fuzz.json");
        std::fs::write(
            &simulation,
            r#"{
              "graph_payloads":["{bad","{\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]}"],
              "config_payloads":["{\"cache\":\"unknown\"}","{\"jobs\":-1}"],
              "route_argv_sets":[["validate"],["run","--out"]],
              "plugin_manifest_payloads":["{bad-json","{\"namespace\":\"\",\"version\":\"1\",\"entrypoint\":\"\",\"trust_class\":\"\",\"capabilities\":[]}"],
              "bundle_payloads":["{bad-json","{\"kind\":\"bundle\"}"],
              "run_manifest_payloads":["{bad-json","{\"manifest_version\":\"\",\"run_id\":\"\",\"status\":\"\"}"]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::MalformedInputFuzz { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::MalformedInputFuzz { simulation: simulation.clone() },
        )
        .expect("malformed-input-fuzz");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::malformed_input_fuzz_payload(&simulation).expect("report");
        assert!(report.crash_free);
        assert_eq!(report.panics_observed, 0);
        assert!(report.graph_rejections > 0);
        assert!(report.config_rejections > 0);
        assert!(report.route_rejections > 0);
        assert!(report.plugin_manifest_rejections > 0);
        assert!(report.bundle_rejections > 0);
        assert!(report.run_manifest_rejections > 0);
    }

    #[test]
    fn security_malformed_input_fuzz_flags_empty_corpus() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("malformed-empty.json");
        std::fs::write(
            &simulation,
            r#"{
              "graph_payloads":[],
              "config_payloads":[],
              "route_argv_sets":[],
              "plugin_manifest_payloads":[],
              "bundle_payloads":[],
              "run_manifest_payloads":[]
            }"#,
        )
        .expect("write simulation");
        let report = super::malformed_input_fuzz_payload(&simulation).expect("report");
        assert_eq!(report.cases_total, 0);
        assert!(report.gaps.iter().any(|gap| gap == "fuzz corpus is empty"));
    }

    #[test]
    fn security_dependency_risk_distinguishes_core_and_tooling_surfaces() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("dependency-risk.json");
        std::fs::write(
            &simulation,
            r#"{
              "dependencies":[
                {"name":"tokio","surface":"core-runtime","risk_score":1.5,"known_vulnerabilities":0},
                {"name":"serde","surface":"core-runtime","risk_score":0.5,"known_vulnerabilities":0},
                {"name":"mkdocs","surface":"docs-tooling","risk_score":1.0,"known_vulnerabilities":1},
                {"name":"cargo-audit","surface":"dev-tooling","risk_score":0.8,"known_vulnerabilities":0},
                {"name":"release-please","surface":"release-tooling","risk_score":0.7,"known_vulnerabilities":0}
              ],
              "core_runtime_threshold":3.0,
              "tooling_threshold":4.0
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::DependencyRisk { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::DependencyRisk { simulation: simulation.clone() },
        )
        .expect("dependency risk");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::dependency_risk_payload(&simulation).expect("report");
        assert_eq!(report.core_runtime_risk_score, 2.0);
        assert_eq!(report.tooling_risk_score, 3.0);
        assert!(!report.core_runtime_exceeds_threshold);
        assert!(!report.tooling_exceeds_threshold);
        assert_eq!(
            report.core_runtime_dependencies,
            vec!["serde".to_string(), "tokio".to_string()]
        );
        assert_eq!(
            report.tooling_dependencies,
            vec!["cargo-audit".to_string(), "mkdocs".to_string(), "release-please".to_string()]
        );
    }

    #[test]
    fn security_dependency_risk_flags_threshold_breaches() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("dependency-risk-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "dependencies":[
                {"name":"tokio","surface":"core-runtime","risk_score":3.0,"known_vulnerabilities":2},
                {"name":"mkdocs","surface":"docs-tooling","risk_score":2.0,"known_vulnerabilities":3}
              ],
              "core_runtime_threshold":2.0,
              "tooling_threshold":2.5
            }"#,
        )
        .expect("write simulation");
        let report = super::dependency_risk_payload(&simulation).expect("report");
        assert!(report.core_runtime_exceeds_threshold);
        assert!(report.tooling_exceeds_threshold);
        for expected in [
            "core-runtime dependency risk exceeds threshold",
            "tooling dependency risk exceeds threshold",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_data_access_accepts_scoped_dataset_and_path_usage() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let input_root = dir.path().join("inputs");
        let output_root = dir.path().join("outputs");
        std::fs::create_dir_all(&input_root).expect("input root");
        std::fs::create_dir_all(&output_root).expect("output root");
        let candidate_input = input_root.join("a/file.txt");
        let candidate_output = output_root.join("b/file.txt");
        std::fs::create_dir_all(candidate_input.parent().expect("input parent"))
            .expect("input parent");
        std::fs::create_dir_all(candidate_output.parent().expect("output parent"))
            .expect("output parent");
        std::fs::write(&candidate_input, "ok").expect("write input");
        std::fs::write(&candidate_output, "ok").expect("write output");
        let simulation = dir.path().join("data-access.json");
        std::fs::write(
            &simulation,
            format!(
                r#"{{
                  "input_root":"{}",
                  "candidate_input":"{}",
                  "output_root":"{}",
                  "candidate_output":"{}",
                  "allowed_dataset_ids":["ds1","ds2"],
                  "requested_dataset_ids":["ds1"],
                  "allowed_artifact_ids":["a1","a2"],
                  "requested_artifact_ids":["a1","a2"],
                  "tenant_id":"atlas"
                }}"#,
                input_root.display(),
                candidate_input.display(),
                output_root.display(),
                candidate_output.display()
            ),
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::DataAccess { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::DataAccess { simulation: simulation.clone() },
        )
        .expect("data access");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::data_access_payload(&simulation).expect("report");
        assert!(report.input_path_allowed);
        assert!(report.output_path_allowed);
        assert!(report.dataset_entitlements_ok);
        assert_eq!(report.visible_artifact_ids, vec!["a1".to_string(), "a2".to_string()]);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_data_access_flags_escaped_paths_and_unapproved_datasets() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let input_root = dir.path().join("inputs");
        let output_root = dir.path().join("outputs");
        std::fs::create_dir_all(&input_root).expect("input root");
        std::fs::create_dir_all(&output_root).expect("output root");
        let simulation = dir.path().join("data-access.json");
        std::fs::write(
            &simulation,
            format!(
                r#"{{
                  "input_root":"{}",
                  "candidate_input":"{}",
                  "output_root":"{}",
                  "candidate_output":"{}",
                  "allowed_dataset_ids":["ds1"],
                  "requested_dataset_ids":["ds1","ds2"],
                  "allowed_artifact_ids":["a1"],
                  "requested_artifact_ids":["a1","a2"],
                  "tenant_id":"atlas"
                }}"#,
                input_root.display(),
                dir.path().join("../outside.txt").display(),
                output_root.display(),
                dir.path().join("../outside-out.txt").display()
            ),
        )
        .expect("write simulation");
        let report = super::data_access_payload(&simulation).expect("report");
        assert!(!report.input_path_allowed);
        assert!(!report.output_path_allowed);
        assert!(!report.dataset_entitlements_ok);
        assert_eq!(report.visible_artifact_ids, vec!["a1".to_string()]);
        for expected in [
            "requested input path escapes the authorized input root",
            "requested output path escapes the authorized output root",
            "requested datasets exceed declared entitlements",
            "artifact lineage query was narrowed by tenant scope",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_override_accepts_audited_break_glass_action() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("override.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"operator-a",
              "tenant_id":"atlas",
              "action_name":"run.repair",
              "action_kind":"Manage",
              "resource_kind":"Run",
              "resource_id":"run-1",
              "resource_tenant_id":"atlas",
              "environment":"prod",
              "policy_bundle_version":"2026-04-28",
              "granted_permissions":["run.repair"],
              "environment_rules":[{"environment":"prod","denied_actions":["platform.administer"]}],
              "reason":"recover corrupted manifest",
              "audit_event_id":"audit-1",
              "approvers":["owner-a","owner-b"],
              "required_approvals":2,
              "break_glass":true
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(SecurityCommands::Override { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::Override { simulation: simulation.clone() },
        )
        .expect("override");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::override_payload(&simulation).expect("report");
        assert!(report.override_allowed);
        assert!(report.reason_recorded);
        assert!(report.audit_recorded);
        assert!(report.environment_allows);
        assert!(report.break_glass_policy_valid);
        assert!(report.approval_quorum_met);
        assert!(report.scoped_to_tenant);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_override_rejects_unaudited_override() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("override.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"operator-a",
              "tenant_id":"atlas",
              "action_name":"platform.administer",
              "action_kind":"Administer",
              "resource_kind":"Run",
              "resource_id":"run-1",
              "resource_tenant_id":"other",
              "environment":"prod",
              "policy_bundle_version":"2026-04-28",
              "granted_permissions":[],
              "environment_rules":[{"environment":"prod","denied_actions":["platform.administer"]}],
              "reason":"",
              "audit_event_id":null,
              "approvers":[],
              "required_approvals":2,
              "break_glass":true
            }"#,
        )
        .expect("write simulation");
        let report = super::override_payload(&simulation).expect("report");
        assert!(!report.override_allowed);
        for expected in [
            "override actor lacks the required permission",
            "override is denied in this environment",
            "override reason is missing",
            "override audit record is missing",
            "override crosses tenant scope boundaries",
            "override lacks the required approval quorum",
            "break-glass override is not fully justified",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_override_audit_accepts_complete_override_audit_record() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("override-audit.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"operator-a",
              "run_id":"run-42",
              "forced_rerun":true,
              "bypassed_policy":true,
              "cache_disabled":true,
              "accepted_degraded_evidence":true,
              "retention_change":"audit",
              "reason":"incident containment",
              "evidence_pointer":"artifacts/overrides/run-42.json",
              "audit_event_id":"audit-override-42",
              "timestamp_unix_ms":1714471234000
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(SecurityCommands::OverrideAudit { simulation: simulation.clone() });
        let code = handle_security_command(
            &cli,
            &SecurityCommands::OverrideAudit { simulation: simulation.clone() },
        )
        .expect("override audit");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::override_audit_payload(&simulation).expect("report");
        assert!(report.audit_complete);
        assert_eq!(
            report.recorded_actions,
            vec![
                "forced-rerun".to_string(),
                "bypassed-policy".to_string(),
                "cache-disabled".to_string(),
                "accepted-degraded-evidence".to_string(),
                "retention-changed".to_string(),
            ]
        );
        assert!(report.missing_records.is_empty());
        assert_eq!(report.actor, "operator-a");
        assert_eq!(report.run_id, "run-42");
    }

    #[test]
    fn security_override_audit_flags_missing_audit_fields() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("override-audit-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "actor":"operator-a",
              "run_id":"run-42",
              "forced_rerun":true,
              "bypassed_policy":false,
              "cache_disabled":true,
              "accepted_degraded_evidence":false,
              "retention_change":null,
              "reason":"",
              "evidence_pointer":null,
              "audit_event_id":null,
              "timestamp_unix_ms":0
            }"#,
        )
        .expect("write simulation");
        let report = super::override_audit_payload(&simulation).expect("report");
        assert!(!report.audit_complete);
        for expected in [
            "override reason is missing",
            "evidence pointer is missing",
            "audit event id is missing",
            "audit timestamp is missing",
        ] {
            assert!(report.missing_records.iter().any(|gap| gap == expected));
        }
    }

    #[test]
    fn security_safe_defaults_accepts_bounded_workflow() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"safe","owners":["team-core"],"tags":["prod","reviewed"]},
              "nodes":[
                {
                  "id":"n1",
                  "kind":"shell",
                  "inputs":[],
                  "outputs":[{"name":"out","path":"out"}],
                  "params":{"argv":["/bin/true"]},
                  "timeout_ms":1000,
                  "resources":{"cpu":1,"mem_mb":128},
                  "tags":["filesystem-reviewed"],
                  "retry":{"max_attempts":1,"backoff_ms":100},
                  "effects":["filesystem"]
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let cli = quiet_json_cli(SecurityCommands::SafeDefaults { dag: dag.clone() });
        let code =
            handle_security_command(&cli, &SecurityCommands::SafeDefaults { dag: dag.clone() })
                .expect("safe defaults");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::safe_defaults_payload(&dag).expect("report");
        assert!(report.safe_by_default);
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn security_safe_defaults_flags_unsafe_workflow_defaults() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let dag = dir.path().join("dag.json");
        std::fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"unsafe","owners":[],"tags":[]},
              "nondeterminism_allowed":true,
              "nodes":[
                {
                  "id":"n1",
                  "kind":"shell",
                  "inputs":[],
                  "outputs":[{"name":"out","path":"out"}],
                  "params":{"argv":["/bin/true"]},
                  "retry":{"max_attempts":0,"backoff_ms":0},
                  "effects":["network","env","clock"]
                }
              ],
              "edges":[]
            }"#,
        )
        .expect("write dag");
        let report = super::safe_defaults_payload(&dag).expect("report");
        assert!(!report.safe_by_default);
        for expected in [
            "workflow has no owners",
            "workflow has no taxonomy tags",
            "workflow permits nondeterministic execution by default",
        ] {
            assert!(report.gaps.iter().any(|gap| gap == expected));
        }
        let node_gap = report
            .gaps
            .iter()
            .find(|gap| gap.starts_with("node n1 has unsafe defaults:"))
            .expect("node gap");
        for expected in [
            "missing-timeout",
            "missing-resource-bounds",
            "effectful-node-without-tags",
            "network-effect-enabled",
            "clock-effect-enabled",
            "env-effect-without-allowlist",
        ] {
            assert!(node_gap.contains(expected));
        }
    }
}
