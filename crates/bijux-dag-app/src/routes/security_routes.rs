use crate::commands::{DagCli, SecurityCommands};
use crate::{emit_json, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    can_renew_credential, credential_is_expired, credential_scopes_matrix,
    builtin_role_definitions, evaluate_authorization_acceptance, evaluate_dry_run,
    is_action_allowed_in_environment, validate_custom_role,
    local_dev_bypass_allowed, readiness_for_federation, revoked_principals_set, trust_health_report,
    Action, ActionKind, AuthenticationEvent, CredentialLifecycle, CredentialRevocation,
    CredentialScope, CustomRoleDefinition, DecisionType, EnvironmentAuthorizationRule,
    IdentityPrincipal, LocalDevAuthBypassRule, PolicyEvaluationRequest, ResourceKind, ResourceRef,
    ResourceScope, SubjectIdentity, SubjectKind, TrustHealthReport,
    check_scheduler_admission, enforce_tenant_plugin_allowlist, resolve_tenant_overlay,
    scope_lineage_query, tenant_provisioning_bootstrap, validate_tenant_isolation,
    RuntimeSecretContract,
    TenantConfigOverlay, TenantId, TenantLineageScope, TenantPluginAllowlist,
    TenantPolicyBundleRef, TenantProvisioningSpec, TenantQueueIsolationPolicy,
    TenantRegistryPartition, TenantSchedulerAdmission,
};
use bijux_dag_runtime::{
    leak_conformance_check, secret_readiness, secret_scope_allows, secure_cleanup_required,
    secure_mode_effective, select_secret_version, summarize_sensitive_classes,
    validate_secret_delivery_mode, SecretDeliveryPolicy, SecretInjectionMode,
    SecretMaskingPolicy, SecretRotationRule, SecretScopeRule, SecretSource, SecretUsageAuditEvent,
    SecureExecutionMode, SecureTeardownPolicy, SecureWorkspaceRule, SensitiveArtifactRestriction,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::Path;

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

fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
}

fn auth_payload(simulation: &Path) -> Result<AuthReport, ExitCode> {
    let simulation: AuthSimulation = load_json_file(simulation)?;
    let revoked = revoked_principals_set(&simulation.revocations);
    let local_bypass_blocked =
        simulation.bypass_rules.iter().all(|rule| !local_dev_bypass_allowed(&simulation.environment, rule));
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
        action: Action { name: simulation.action_name.clone(), kind: simulation.action_kind },
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
    let dry_run = evaluate_dry_run(&request, &simulation.granted_permissions, &simulation.policy_bundle_version);
    let environment_allows =
        is_action_allowed_in_environment(&simulation.action_name, &simulation.environment, &simulation.environment_rules);
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
    let scheduler_admitted =
        check_scheduler_admission(simulation.queued_runs, simulation.pending_dispatches, &admission);
    let allowlist =
        TenantPluginAllowlist { tenant_id: requested_tenant.clone(), allowed_plugins: simulation.allowed_plugins };
    let plugin_allowed = enforce_tenant_plugin_allowlist(&simulation.plugin_name, &allowlist);
    let lineage_scope = TenantLineageScope {
        tenant_id: requested_tenant.clone(),
        allowed_artifact_ids: simulation.allowed_artifact_ids,
    };
    let visible_artifact_ids = scope_lineage_query(&simulation.requested_artifact_ids, &lineage_scope);
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
    let delivery_allowed = validate_secret_delivery_mode(&simulation.delivery_mode, &simulation.delivery_policy);
    let scope_allowed = secret_scope_allows(&simulation.allowed_scope, &simulation.requested_scope);
    let readiness = secret_readiness(
        &simulation.sources,
        &simulation.masking_policy,
        &simulation.audit_events,
        simulation.secure_mode.enabled,
    );
    let strict_mode_effective = secure_mode_effective(&simulation.secure_mode.environment, &simulation.secure_mode);
    let cleanup_required = secure_cleanup_required(&simulation.workspace_rule, &simulation.teardown_policy);
    let leak_clean = leak_conformance_check(&simulation.observed_outputs);
    let brokered_credentials = !simulation.uses_static_secret
        && simulation
            .sources
            .iter()
            .any(|source| matches!(source, SecretSource::ExternalManager { .. }));
    let region_allowed = simulation.allowed_regions.iter().any(|region| region == &simulation.requested_region);
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
    if simulation.secret_contract.secret_refs.is_empty() || !simulation.secret_contract.redaction_required {
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

pub(crate) fn handle_security_command(
    cli: &DagCli,
    command: &SecurityCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        SecurityCommands::Auth { simulation } => {
            let payload = serde_json::to_value(auth_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.auth", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Authz { simulation } => {
            let payload = serde_json::to_value(authz_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.authz", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Tenant { simulation } => {
            let payload = serde_json::to_value(tenant_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.tenant", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        SecurityCommands::Secrets { simulation } => {
            let payload = serde_json::to_value(secrets_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.secrets", true, payload, Vec::new(), ExitCode::SUCCESS)
        }
        _ => emit_json(
            cli,
            "dag.security",
            false,
            json!({"status":"not-yet-implemented"}),
            vec![json!({"message":"security surface not yet implemented for this command in the current commit boundary"})],
            ExitCode::from(2),
        ),
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
        let code = handle_security_command(&cli, &SecurityCommands::Auth { simulation: simulation.clone() })
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
        let code =
            handle_security_command(&cli, &SecurityCommands::Authz { simulation: simulation.clone() })
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
        let code = handle_security_command(&cli, &SecurityCommands::Tenant { simulation: simulation.clone() })
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
        let code =
            handle_security_command(&cli, &SecurityCommands::Secrets { simulation: simulation.clone() })
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
}
