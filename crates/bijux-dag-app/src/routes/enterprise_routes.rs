use crate::commands::{DagCli, EnterpriseCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{
    execution_mode_status, remote_handoff_valid, validate_remote_identity, ExecutionModeStatus,
    RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
};
use bijux_dag_runtime::simulated_platform::{
    approval_gate_ready,
    authorize, AuthContext, AuthenticationPrincipal, AuthorizationRule, EventSubscription,
    can_renew_credential, credential_is_expired, ApprovalGateNode, CredentialLifecycle,
    CredentialScope, IncidentClassification, IncidentSeverity, QueueResource,
    ServiceArchitectureNote, TenantOwnershipMetadata, TenantScopedDagName, WorkerCredentialBinding,
    WorkflowFamilyImpactAnalysis, WorkflowPortfolio,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WebhookSimulation {
    subscription: EventSubscription,
    auth: AuthContext,
    #[serde(default)]
    authorization_rules: Vec<AuthorizationRule>,
    topic: String,
    dedup_key: String,
    #[serde(default)]
    seen_dedup_keys: Vec<String>,
    signature_valid: bool,
    payload_schema_valid: bool,
}

#[derive(Debug, Serialize)]
struct WebhookReport {
    subscription_id: String,
    topic: String,
    principal: String,
    active: bool,
    authorized: bool,
    duplicate: bool,
    signature_valid: bool,
    payload_schema_valid: bool,
    gaps: Vec<String>,
    webhook_ready: bool,
}

#[derive(Debug, Deserialize)]
struct QueueSimulation {
    queue: QueueResource,
    topic: String,
    consumer_group: String,
    start_offset: u64,
    acked_offset: u64,
    replay_from_offset: u64,
    dedup_key: String,
    #[serde(default)]
    seen_dedup_keys: Vec<String>,
    dead_letter_enabled: bool,
}

#[derive(Debug, Serialize)]
struct QueueReport {
    queue_id: String,
    topic: String,
    consumer_group: String,
    ack_advanced: bool,
    replay_possible: bool,
    duplicate: bool,
    dead_letter_enabled: bool,
    gaps: Vec<String>,
    queue_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ServiceContractSimulation {
    identity: RemoteExecutionIdentity,
    artifact_handoff: RemoteArtifactHandoff,
    observability_handoff: RemoteObservabilityHandoff,
    execution_mode: String,
    retry_budget: u32,
    timeout_seconds: u32,
    idempotent: bool,
    side_effect_class: String,
}

#[derive(Debug, Serialize)]
struct ServiceContractReport {
    run_id: String,
    node_id: String,
    execution_mode: String,
    execution_mode_status: String,
    retry_budget: u32,
    timeout_seconds: u32,
    idempotent: bool,
    side_effect_class: String,
    handoff_valid: bool,
    gaps: Vec<String>,
    service_contract_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IncidentHookSimulation {
    classification: IncidentClassification,
    workflow_id: String,
    #[serde(default)]
    owners: Vec<String>,
    ticket_system: String,
    escalation_target: String,
    #[serde(default)]
    context_fields: Vec<String>,
}

#[derive(Debug, Serialize)]
struct IncidentHookReport {
    workflow_id: String,
    severity: String,
    ticket_system: String,
    escalation_target: String,
    owner_count: usize,
    context_fields: Vec<String>,
    gaps: Vec<String>,
    incident_hook_ready: bool,
}

#[derive(Debug, Deserialize)]
struct AssetLinkSimulation {
    dag: TenantScopedDagName,
    ownership: TenantOwnershipMetadata,
    portfolio: WorkflowPortfolio,
    #[serde(default)]
    business_capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct AssetLinkReport {
    dag_name: String,
    tenant_id: String,
    owner: String,
    portfolio_id: String,
    linked_workflows: usize,
    linked_schedules: usize,
    linked_datasets: usize,
    linked_policies: usize,
    business_capabilities: Vec<String>,
    gaps: Vec<String>,
    asset_link_ready: bool,
}

#[derive(Debug, Deserialize)]
struct CalendarSimulation {
    workflow_id: String,
    action: String,
    now_utc: String,
    #[serde(default)]
    blackout_windows: Vec<String>,
    #[serde(default)]
    maintenance_windows: Vec<String>,
    approval_ticket: Option<String>,
}

#[derive(Debug, Serialize)]
struct CalendarReport {
    workflow_id: String,
    action: String,
    now_utc: String,
    blackout_windows: Vec<String>,
    maintenance_windows: Vec<String>,
    approval_ticket: Option<String>,
    action_allowed: bool,
    gaps: Vec<String>,
    calendar_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ApprovalSimulation {
    gate: ApprovalGateNode,
    external_system: String,
    #[serde(default)]
    approval_ids: Vec<String>,
    callback_received: bool,
    approver_identity: String,
}

#[derive(Debug, Serialize)]
struct ApprovalReport {
    node_id: String,
    policy_ref: String,
    external_system: String,
    approval_count: usize,
    callback_received: bool,
    approver_identity: String,
    gate_contract_ready: bool,
    gaps: Vec<String>,
    approval_ready: bool,
}

#[derive(Debug, Deserialize)]
struct DependencyCatalogSimulation {
    workflow_family: String,
    architecture: ServiceArchitectureNote,
    impact: WorkflowFamilyImpactAnalysis,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DependencyCatalogReport {
    workflow_family: String,
    dependencies: Vec<String>,
    impacted_workflows: Vec<String>,
    api_boundary: String,
    scheduler_boundary: String,
    registry_boundary: String,
    executor_boundary: String,
    gaps: Vec<String>,
    dependency_catalog_ready: bool,
}

#[derive(Debug, Deserialize)]
struct CredentialSimulation {
    now_unix_ms: u128,
    lifecycle: CredentialLifecycle,
    renewal_count: u32,
    scope: CredentialScope,
    binding: WorkerCredentialBinding,
    broker_class: String,
}

#[derive(Debug, Serialize)]
struct CredentialReport {
    worker_id: String,
    lease_id: String,
    expired: bool,
    renewable_now: bool,
    broker_class: String,
    worker_scoped: bool,
    gaps: Vec<String>,
    credential_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn principal_name(principal: &AuthenticationPrincipal) -> String {
    match principal {
        AuthenticationPrincipal::CliUser { subject } => subject.clone(),
        AuthenticationPrincipal::ServiceAccount { service } => service.clone(),
        AuthenticationPrincipal::WorkerIdentity { worker_id } => worker_id.clone(),
    }
}

fn webhook_payload(simulation: WebhookSimulation) -> (serde_json::Value, bool) {
    let WebhookSimulation {
        subscription,
        auth,
        authorization_rules,
        topic,
        dedup_key,
        seen_dedup_keys,
        signature_valid,
        payload_schema_valid,
    } = simulation;
    let authorized = authorize(&auth, "event.receive", &authorization_rules);
    let seen = seen_dedup_keys.into_iter().collect::<BTreeSet<_>>();
    let duplicate = seen.contains(&dedup_key);
    let mut gaps = Vec::new();
    if !subscription.active {
        gaps.push("webhook subscription is inactive".to_string());
    }
    if subscription.topic != topic {
        gaps.push("subscription topic does not match incoming topic".to_string());
    }
    if !authorized {
        gaps.push("incoming webhook principal is not authorized for event.receive".to_string());
    }
    if duplicate {
        gaps.push("incoming webhook event is a duplicate".to_string());
    }
    if !signature_valid {
        gaps.push("webhook signature validation failed".to_string());
    }
    if !payload_schema_valid {
        gaps.push("webhook payload schema validation failed".to_string());
    }
    let report = WebhookReport {
        subscription_id: subscription.subscription_id,
        topic,
        principal: principal_name(&auth.principal),
        active: subscription.active,
        authorized,
        duplicate,
        signature_valid,
        payload_schema_valid,
        webhook_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.webhook_ready;
    (serde_json::to_value(report).expect("webhook report"), ok)
}

fn queue_payload(simulation: QueueSimulation) -> (serde_json::Value, bool) {
    let QueueSimulation {
        queue,
        topic,
        consumer_group,
        start_offset,
        acked_offset,
        replay_from_offset,
        dedup_key,
        seen_dedup_keys,
        dead_letter_enabled,
    } = simulation;
    let duplicate = seen_dedup_keys.into_iter().collect::<BTreeSet<_>>().contains(&dedup_key);
    let ack_advanced = acked_offset >= start_offset;
    let replay_possible = replay_from_offset <= acked_offset;
    let mut gaps = Vec::new();
    if queue.queue_id.trim().is_empty() {
        gaps.push("queue integration requires a queue identifier".to_string());
    }
    if topic.trim().is_empty() {
        gaps.push("queue integration requires a topic or stream name".to_string());
    }
    if consumer_group.trim().is_empty() {
        gaps.push("queue integration requires a consumer group".to_string());
    }
    if !ack_advanced {
        gaps.push("acked offset must not fall behind the consumed offset".to_string());
    }
    if !replay_possible {
        gaps.push("replay offset must not exceed the acknowledged offset".to_string());
    }
    if duplicate {
        gaps.push("queue event deduplication failed".to_string());
    }
    if !dead_letter_enabled {
        gaps.push("queue integration should define a dead-letter path".to_string());
    }
    let report = QueueReport {
        queue_id: queue.queue_id,
        topic,
        consumer_group,
        ack_advanced,
        replay_possible,
        duplicate,
        dead_letter_enabled,
        queue_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.queue_ready;
    (serde_json::to_value(report).expect("queue report"), ok)
}

fn execution_mode_name(mode: ExecutionModeStatus) -> &'static str {
    match mode {
        ExecutionModeStatus::Implemented => "implemented",
        ExecutionModeStatus::Simulated => "simulated",
        ExecutionModeStatus::NotImplemented => "not-implemented",
    }
}

fn service_contract_payload(simulation: ServiceContractSimulation) -> (serde_json::Value, bool) {
    let ServiceContractSimulation {
        identity,
        artifact_handoff,
        observability_handoff,
        execution_mode,
        retry_budget,
        timeout_seconds,
        idempotent,
        side_effect_class,
    } = simulation;
    let mut gaps = Vec::new();
    if let Err(err) = validate_remote_identity(&identity) {
        gaps.push(err);
    }
    let mode_status = execution_mode_status(&execution_mode);
    if matches!(mode_status, ExecutionModeStatus::NotImplemented) {
        gaps.push("external service task targets an unsupported execution mode".to_string());
    }
    let handoff_valid = remote_handoff_valid(&artifact_handoff, &observability_handoff);
    if !handoff_valid {
        gaps.push("artifact or observability handoff is incomplete".to_string());
    }
    if retry_budget == 0 {
        gaps.push("external service task should declare a retry budget".to_string());
    }
    if timeout_seconds == 0 {
        gaps.push("external service task should declare a timeout".to_string());
    }
    if !idempotent && side_effect_class != "append-only" {
        gaps.push("non-idempotent service tasks require stricter side-effect posture".to_string());
    }
    let report = ServiceContractReport {
        run_id: identity.run_id,
        node_id: identity.node_id,
        execution_mode,
        execution_mode_status: execution_mode_name(mode_status).to_string(),
        retry_budget,
        timeout_seconds,
        idempotent,
        side_effect_class,
        handoff_valid,
        service_contract_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.service_contract_ready;
    (serde_json::to_value(report).expect("service contract report"), ok)
}

fn severity_name(severity: &IncidentSeverity) -> &'static str {
    match severity {
        IncidentSeverity::Critical => "critical",
        IncidentSeverity::High => "high",
        IncidentSeverity::Medium => "medium",
        IncidentSeverity::Low => "low",
    }
}

fn incident_hook_payload(simulation: IncidentHookSimulation) -> (serde_json::Value, bool) {
    let IncidentHookSimulation {
        classification,
        workflow_id,
        owners,
        ticket_system,
        escalation_target,
        context_fields,
    } = simulation;
    let mut gaps = Vec::new();
    if workflow_id.trim().is_empty() {
        gaps.push("incident hook requires a workflow identifier".to_string());
    }
    if owners.is_empty() {
        gaps.push("incident hook requires at least one workflow owner".to_string());
    }
    if ticket_system.trim().is_empty() {
        gaps.push("incident hook requires a ticketing or incident system".to_string());
    }
    if escalation_target.trim().is_empty() {
        gaps.push("incident hook requires an escalation target".to_string());
    }
    for required in ["run_id", "tenant_id", "severity", "owner"] {
        if !context_fields.iter().any(|field| field == required) {
            gaps.push(format!("incident hook is missing required context field {required}"));
        }
    }
    let report = IncidentHookReport {
        workflow_id,
        severity: severity_name(&classification.severity).to_string(),
        ticket_system,
        escalation_target,
        owner_count: owners.len(),
        context_fields,
        incident_hook_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.incident_hook_ready;
    (serde_json::to_value(report).expect("incident hook report"), ok)
}

fn asset_link_payload(simulation: AssetLinkSimulation) -> (serde_json::Value, bool) {
    let AssetLinkSimulation { dag, ownership, portfolio, business_capabilities } = simulation;
    let mut gaps = Vec::new();
    if ownership.owner.trim().is_empty() {
        gaps.push("asset link requires an owning team or operator".to_string());
    }
    if portfolio.dag_refs.is_empty() {
        gaps.push("asset link requires at least one workflow reference".to_string());
    }
    if portfolio.dataset_refs.is_empty() {
        gaps.push("asset link should reference at least one dataset".to_string());
    }
    if business_capabilities.is_empty() {
        gaps.push("asset link should name at least one business capability".to_string());
    }
    let report = AssetLinkReport {
        dag_name: format!("{}/{}", dag.namespace, dag.logical_name),
        tenant_id: ownership.tenant_id.0,
        owner: ownership.owner,
        portfolio_id: portfolio.portfolio_id,
        linked_workflows: portfolio.dag_refs.len(),
        linked_schedules: portfolio.schedule_refs.len(),
        linked_datasets: portfolio.dataset_refs.len(),
        linked_policies: portfolio.policy_refs.len(),
        business_capabilities,
        asset_link_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.asset_link_ready;
    (serde_json::to_value(report).expect("asset link report"), ok)
}

fn within_list(now_utc: &str, windows: &[String]) -> bool {
    windows.iter().any(|window| window == now_utc)
}

fn calendar_payload(simulation: CalendarSimulation) -> (serde_json::Value, bool) {
    let CalendarSimulation {
        workflow_id,
        action,
        now_utc,
        blackout_windows,
        maintenance_windows,
        approval_ticket,
    } = simulation;
    let in_blackout = within_list(&now_utc, &blackout_windows);
    let in_maintenance = within_list(&now_utc, &maintenance_windows);
    let action_allowed = if in_blackout {
        false
    } else if in_maintenance {
        approval_ticket.as_deref().is_some_and(|ticket| !ticket.trim().is_empty())
    } else {
        true
    };
    let mut gaps = Vec::new();
    if workflow_id.trim().is_empty() {
        gaps.push("calendar policy requires a workflow identifier".to_string());
    }
    if action.trim().is_empty() {
        gaps.push("calendar policy requires an action name".to_string());
    }
    if blackout_windows.is_empty() && maintenance_windows.is_empty() {
        gaps.push("calendar policy should define blackout or maintenance windows".to_string());
    }
    if in_blackout {
        gaps.push("requested action falls inside a blackout window".to_string());
    }
    if in_maintenance && approval_ticket.as_deref().is_none_or(|ticket| ticket.trim().is_empty()) {
        gaps.push("maintenance window action requires an approval ticket".to_string());
    }
    let report = CalendarReport {
        workflow_id,
        action,
        now_utc,
        blackout_windows,
        maintenance_windows,
        approval_ticket,
        action_allowed,
        calendar_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.calendar_ready;
    (serde_json::to_value(report).expect("calendar report"), ok)
}

fn approval_payload(simulation: ApprovalSimulation) -> (serde_json::Value, bool) {
    let ApprovalSimulation {
        gate,
        external_system,
        approval_ids,
        callback_received,
        approver_identity,
    } = simulation;
    let gate_contract_ready = approval_gate_ready(&gate);
    let mut gaps = Vec::new();
    if !gate_contract_ready {
        gaps.push("approval gate definition is incomplete".to_string());
    }
    if external_system.trim().is_empty() {
        gaps.push("external approval integration requires a system name".to_string());
    }
    if approval_ids.is_empty() {
        gaps.push("external approval integration requires at least one approval id".to_string());
    }
    if !callback_received {
        gaps.push("approval callback has not been received".to_string());
    }
    if approver_identity.trim().is_empty() {
        gaps.push("approval integration requires an approver identity".to_string());
    }
    let report = ApprovalReport {
        node_id: gate.node_id,
        policy_ref: gate.policy_ref,
        external_system,
        approval_count: approval_ids.len(),
        callback_received,
        approver_identity,
        gate_contract_ready,
        approval_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.approval_ready;
    (serde_json::to_value(report).expect("approval report"), ok)
}

fn dependency_catalog_payload(
    simulation: DependencyCatalogSimulation,
) -> (serde_json::Value, bool) {
    let DependencyCatalogSimulation { workflow_family, architecture, impact, dependencies } =
        simulation;
    let mut gaps = Vec::new();
    if dependencies.is_empty() {
        gaps.push("dependency catalog requires declared service dependencies".to_string());
    }
    for (name, value) in [
        ("api", architecture.api_boundary.as_str()),
        ("scheduler", architecture.scheduler_boundary.as_str()),
        ("registry", architecture.registry_boundary.as_str()),
        ("executor", architecture.executor_boundary.as_str()),
    ] {
        if value.trim().is_empty() {
            gaps.push(format!("dependency catalog is missing the {name} boundary"));
        }
    }
    if impact.impacted_workflows.is_empty() {
        gaps.push("dependency catalog should identify impacted workflows for outage analysis".to_string());
    }
    let report = DependencyCatalogReport {
        workflow_family,
        dependencies,
        impacted_workflows: impact.impacted_workflows,
        api_boundary: architecture.api_boundary,
        scheduler_boundary: architecture.scheduler_boundary,
        registry_boundary: architecture.registry_boundary,
        executor_boundary: architecture.executor_boundary,
        dependency_catalog_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.dependency_catalog_ready;
    (serde_json::to_value(report).expect("dependency catalog report"), ok)
}

fn credential_payload(simulation: CredentialSimulation) -> (serde_json::Value, bool) {
    let CredentialSimulation {
        now_unix_ms,
        lifecycle,
        renewal_count,
        scope,
        binding,
        broker_class,
    } = simulation;
    let expired = credential_is_expired(now_unix_ms, &lifecycle);
    let renewable_now = can_renew_credential(renewal_count, &lifecycle);
    let worker_scoped = scope.worker && !scope.cli && !scope.api_client && !scope.scheduler;
    let mut gaps = Vec::new();
    if expired {
        gaps.push("brokered credential is already expired".to_string());
    }
    if !renewable_now {
        gaps.push("brokered credential cannot be renewed within policy".to_string());
    }
    if !worker_scoped {
        gaps.push("credential should be worker-scoped by default".to_string());
    }
    if binding.worker_id.trim().is_empty() || binding.lease_id.trim().is_empty() {
        gaps.push("worker credential binding must name worker and lease".to_string());
    }
    if binding.expires_unix_ms < lifecycle.expires_unix_ms {
        gaps.push("worker binding expires before the credential lifecycle".to_string());
    }
    if broker_class.trim().is_empty() || broker_class == "static-secret" {
        gaps.push("credential path is not using a brokered short-lived class".to_string());
    }
    let report = CredentialReport {
        worker_id: binding.worker_id,
        lease_id: binding.lease_id,
        expired,
        renewable_now,
        broker_class,
        worker_scoped,
        credential_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.credential_ready;
    (serde_json::to_value(report).expect("credential report"), ok)
}

pub(crate) fn handle_enterprise_command(
    cli: &DagCli,
    command: &EnterpriseCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        EnterpriseCommands::Webhook { simulation } => {
            let simulation: WebhookSimulation = parse_json_file(simulation)?;
            let (payload, ok) = webhook_payload(simulation);
            ("dag.enterprise.webhook", payload, ok)
        }
        EnterpriseCommands::Queue { simulation } => {
            let simulation: QueueSimulation = parse_json_file(simulation)?;
            let (payload, ok) = queue_payload(simulation);
            ("dag.enterprise.queue", payload, ok)
        }
        EnterpriseCommands::ServiceContract { simulation } => {
            let simulation: ServiceContractSimulation = parse_json_file(simulation)?;
            let (payload, ok) = service_contract_payload(simulation);
            ("dag.enterprise.service-contract", payload, ok)
        }
        EnterpriseCommands::IncidentHook { simulation } => {
            let simulation: IncidentHookSimulation = parse_json_file(simulation)?;
            let (payload, ok) = incident_hook_payload(simulation);
            ("dag.enterprise.incident-hook", payload, ok)
        }
        EnterpriseCommands::AssetLink { simulation } => {
            let simulation: AssetLinkSimulation = parse_json_file(simulation)?;
            let (payload, ok) = asset_link_payload(simulation);
            ("dag.enterprise.asset-link", payload, ok)
        }
        EnterpriseCommands::Calendar { simulation } => {
            let simulation: CalendarSimulation = parse_json_file(simulation)?;
            let (payload, ok) = calendar_payload(simulation);
            ("dag.enterprise.calendar", payload, ok)
        }
        EnterpriseCommands::Approval { simulation } => {
            let simulation: ApprovalSimulation = parse_json_file(simulation)?;
            let (payload, ok) = approval_payload(simulation);
            ("dag.enterprise.approval", payload, ok)
        }
        EnterpriseCommands::DependencyCatalog { simulation } => {
            let simulation: DependencyCatalogSimulation = parse_json_file(simulation)?;
            let (payload, ok) = dependency_catalog_payload(simulation);
            ("dag.enterprise.dependency-catalog", payload, ok)
        }
        EnterpriseCommands::Credentials { simulation } => {
            let simulation: CredentialSimulation = parse_json_file(simulation)?;
            let (payload, ok) = credential_payload(simulation);
            ("dag.enterprise.credentials", payload, ok)
        }
    };
    emit_json(
        cli,
        surface,
        ok,
        payload,
        if ok {
            Vec::new()
        } else {
            vec![json!({"message":"enterprise integration posture is incomplete","remediation":"fix the reported enterprise boundary gaps before treating this integration as production-ready"})]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        approval_payload, asset_link_payload, calendar_payload, credential_payload,
        dependency_catalog_payload, incident_hook_payload, queue_payload,
        service_contract_payload, webhook_payload, ApprovalSimulation, AssetLinkSimulation,
        CalendarSimulation, CredentialSimulation,
        DependencyCatalogSimulation, IncidentHookSimulation, QueueSimulation,
        ServiceContractSimulation, WebhookSimulation,
    };
    use bijux_dag_runtime::{
        RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
    };
    use bijux_dag_runtime::simulated_platform::{
        ApprovalGateNode, AuthContext, AuthenticationPrincipal, AuthorizationRule,
        CredentialLifecycle, CredentialScope, EventSubscription, IncidentClassification,
        IncidentSeverity, QueueResource, ServiceArchitectureNote, TenantId,
        TenantOwnershipMetadata, TenantScopedDagName, WorkerCredentialBinding,
        WorkflowFamilyImpactAnalysis, WorkflowPortfolio,
    };

    #[test]
    fn webhook_accepts_authorized_signed_unique_event() {
        let simulation = WebhookSimulation {
            subscription: EventSubscription {
                subscription_id: "sub-1".to_string(),
                topic: "workflow.trigger".to_string(),
                endpoint: "https://hooks.example/run".to_string(),
                active: true,
            },
            auth: AuthContext {
                principal: AuthenticationPrincipal::ServiceAccount {
                    service: "catalog-events".to_string(),
                },
                scopes: vec!["events/workflow".to_string()],
            },
            authorization_rules: vec![AuthorizationRule {
                resource_prefix: "events/".to_string(),
                allowed_actions: vec!["event.receive".to_string()],
            }],
            topic: "workflow.trigger".to_string(),
            dedup_key: "evt-1".to_string(),
            seen_dedup_keys: vec!["evt-0".to_string()],
            signature_valid: true,
            payload_schema_valid: true,
        };
        let (payload, ok) = webhook_payload(simulation);
        assert!(ok);
        assert_eq!(payload["webhook_ready"], true);
    }

    #[test]
    fn webhook_flags_duplicate_or_unauthorized_event() {
        let simulation = WebhookSimulation {
            subscription: EventSubscription {
                subscription_id: "sub-2".to_string(),
                topic: "workflow.trigger".to_string(),
                endpoint: "https://hooks.example/run".to_string(),
                active: false,
            },
            auth: AuthContext {
                principal: AuthenticationPrincipal::ServiceAccount {
                    service: "unknown".to_string(),
                },
                scopes: vec!["other/topic".to_string()],
            },
            authorization_rules: vec![AuthorizationRule {
                resource_prefix: "events/".to_string(),
                allowed_actions: vec!["event.receive".to_string()],
            }],
            topic: "workflow.invalid".to_string(),
            dedup_key: "evt-2".to_string(),
            seen_dedup_keys: vec!["evt-2".to_string()],
            signature_valid: false,
            payload_schema_valid: false,
        };
        let (payload, ok) = webhook_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn queue_accepts_monotonic_ack_and_replay_offsets() {
        let simulation = QueueSimulation {
            queue: QueueResource {
                queue_id: "catalog-stream".to_string(),
                tenant: Some("atlas".to_string()),
                priority_policy: "fifo".to_string(),
            },
            topic: "runs.submitted".to_string(),
            consumer_group: "dag-scheduler".to_string(),
            start_offset: 100,
            acked_offset: 104,
            replay_from_offset: 102,
            dedup_key: "msg-1".to_string(),
            seen_dedup_keys: vec!["msg-0".to_string()],
            dead_letter_enabled: true,
        };
        let (payload, ok) = queue_payload(simulation);
        assert!(ok);
        assert_eq!(payload["queue_ready"], true);
    }

    #[test]
    fn queue_flags_replay_or_dedup_regressions() {
        let simulation = QueueSimulation {
            queue: QueueResource {
                queue_id: String::new(),
                tenant: Some("atlas".to_string()),
                priority_policy: "fifo".to_string(),
            },
            topic: String::new(),
            consumer_group: String::new(),
            start_offset: 100,
            acked_offset: 90,
            replay_from_offset: 95,
            dedup_key: "msg-2".to_string(),
            seen_dedup_keys: vec!["msg-2".to_string()],
            dead_letter_enabled: false,
        };
        let (payload, ok) = queue_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn service_contract_accepts_idempotent_remote_task() {
        let simulation = ServiceContractSimulation {
            identity: RemoteExecutionIdentity {
                run_id: "run-1".to_string(),
                node_id: "call-service".to_string(),
                attempt_id: "1".to_string(),
                backend_id: "remote-contract".to_string(),
            },
            artifact_handoff: RemoteArtifactHandoff {
                upload_endpoint: "https://store/upload".to_string(),
                download_endpoint: "https://store/download".to_string(),
                integrity_required: true,
            },
            observability_handoff: RemoteObservabilityHandoff {
                stream_mode: "structured".to_string(),
                trace_forwarding: true,
                retention_days_hint: 14,
            },
            execution_mode: "remote-contract".to_string(),
            retry_budget: 3,
            timeout_seconds: 120,
            idempotent: true,
            side_effect_class: "read-only".to_string(),
        };
        let (payload, ok) = service_contract_payload(simulation);
        assert!(ok);
        assert_eq!(payload["service_contract_ready"], true);
    }

    #[test]
    fn service_contract_flags_unbounded_or_unsupported_service_call() {
        let simulation = ServiceContractSimulation {
            identity: RemoteExecutionIdentity {
                run_id: String::new(),
                node_id: "call-service".to_string(),
                attempt_id: String::new(),
                backend_id: String::new(),
            },
            artifact_handoff: RemoteArtifactHandoff {
                upload_endpoint: String::new(),
                download_endpoint: String::new(),
                integrity_required: false,
            },
            observability_handoff: RemoteObservabilityHandoff {
                stream_mode: String::new(),
                trace_forwarding: false,
                retention_days_hint: 0,
            },
            execution_mode: "unsupported-mode".to_string(),
            retry_budget: 0,
            timeout_seconds: 0,
            idempotent: false,
            side_effect_class: "mutating".to_string(),
        };
        let (payload, ok) = service_contract_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn incident_hook_accepts_owned_context_rich_failure_routing() {
        let simulation = IncidentHookSimulation {
            classification: IncidentClassification {
                incident_type: "workflow-failure".to_string(),
                severity: IncidentSeverity::High,
                routing: "platform-oncall".to_string(),
            },
            workflow_id: "tenant-a/catalog-refresh".to_string(),
            owners: vec!["team-data".to_string()],
            ticket_system: "jira".to_string(),
            escalation_target: "platform-oncall".to_string(),
            context_fields: vec![
                "run_id".to_string(),
                "tenant_id".to_string(),
                "severity".to_string(),
                "owner".to_string(),
            ],
        };
        let (payload, ok) = incident_hook_payload(simulation);
        assert!(ok);
        assert_eq!(payload["incident_hook_ready"], true);
    }

    #[test]
    fn incident_hook_flags_unowned_or_context_thin_routing() {
        let simulation = IncidentHookSimulation {
            classification: IncidentClassification {
                incident_type: "workflow-failure".to_string(),
                severity: IncidentSeverity::Critical,
                routing: "platform-oncall".to_string(),
            },
            workflow_id: String::new(),
            owners: Vec::new(),
            ticket_system: String::new(),
            escalation_target: String::new(),
            context_fields: vec!["run_id".to_string()],
        };
        let (payload, ok) = incident_hook_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn asset_link_accepts_owned_workflow_portfolio_mapping() {
        let simulation = AssetLinkSimulation {
            dag: TenantScopedDagName {
                tenant_id: TenantId("tenant-a".to_string()),
                namespace: "platform".to_string(),
                logical_name: "catalog-refresh".to_string(),
            },
            ownership: TenantOwnershipMetadata {
                tenant_id: TenantId("tenant-a".to_string()),
                owner: "team-data".to_string(),
                labels: vec!["critical".to_string()],
            },
            portfolio: WorkflowPortfolio {
                portfolio_id: "portfolio-1".to_string(),
                dag_refs: vec!["catalog-refresh".to_string()],
                schedule_refs: vec!["daily-catalog".to_string()],
                dataset_refs: vec!["catalog.dataset".to_string()],
                policy_refs: vec!["policy-prod".to_string()],
            },
            business_capabilities: vec!["catalog freshness".to_string()],
        };
        let (payload, ok) = asset_link_payload(simulation);
        assert!(ok);
        assert_eq!(payload["asset_link_ready"], true);
    }

    #[test]
    fn asset_link_flags_missing_owner_or_dataset_context() {
        let simulation = AssetLinkSimulation {
            dag: TenantScopedDagName {
                tenant_id: TenantId("tenant-a".to_string()),
                namespace: "platform".to_string(),
                logical_name: "catalog-refresh".to_string(),
            },
            ownership: TenantOwnershipMetadata {
                tenant_id: TenantId("tenant-a".to_string()),
                owner: String::new(),
                labels: Vec::new(),
            },
            portfolio: WorkflowPortfolio {
                portfolio_id: "portfolio-2".to_string(),
                dag_refs: Vec::new(),
                schedule_refs: Vec::new(),
                dataset_refs: Vec::new(),
                policy_refs: Vec::new(),
            },
            business_capabilities: Vec::new(),
        };
        let (payload, ok) = asset_link_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn calendar_accepts_normal_window_execution() {
        let simulation = CalendarSimulation {
            workflow_id: "tenant-a/catalog-refresh".to_string(),
            action: "submit".to_string(),
            now_utc: "2026-04-29T10:00:00Z".to_string(),
            blackout_windows: vec!["2026-04-28T10:00:00Z".to_string()],
            maintenance_windows: vec!["2026-04-28T22:00:00Z".to_string()],
            approval_ticket: None,
        };
        let (payload, ok) = calendar_payload(simulation);
        assert!(ok);
        assert_eq!(payload["calendar_ready"], true);
    }

    #[test]
    fn calendar_blocks_blackout_and_unapproved_maintenance() {
        let simulation = CalendarSimulation {
            workflow_id: String::new(),
            action: String::new(),
            now_utc: "2026-04-28T22:00:00Z".to_string(),
            blackout_windows: vec!["2026-04-28T22:00:00Z".to_string()],
            maintenance_windows: vec!["2026-04-28T22:00:00Z".to_string()],
            approval_ticket: None,
        };
        let (payload, ok) = calendar_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn approval_accepts_complete_external_gate_signal() {
        let simulation = ApprovalSimulation {
            gate: ApprovalGateNode {
                node_id: "approve-prod".to_string(),
                policy_ref: "policy/prod".to_string(),
                timeout_seconds: 3600,
            },
            external_system: "service-now".to_string(),
            approval_ids: vec!["chg-1001".to_string()],
            callback_received: true,
            approver_identity: "approver@example.com".to_string(),
        };
        let (payload, ok) = approval_payload(simulation);
        assert!(ok);
        assert_eq!(payload["approval_ready"], true);
    }

    #[test]
    fn approval_flags_missing_external_signal_path() {
        let simulation = ApprovalSimulation {
            gate: ApprovalGateNode {
                node_id: String::new(),
                policy_ref: String::new(),
                timeout_seconds: 0,
            },
            external_system: String::new(),
            approval_ids: Vec::new(),
            callback_received: false,
            approver_identity: String::new(),
        };
        let (payload, ok) = approval_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn dependency_catalog_accepts_declared_service_boundaries() {
        let simulation = DependencyCatalogSimulation {
            workflow_family: "catalog-refresh".to_string(),
            architecture: ServiceArchitectureNote {
                api_boundary: "control-plane-api".to_string(),
                scheduler_boundary: "ha-scheduler".to_string(),
                registry_boundary: "artifact-registry".to_string(),
                executor_boundary: "remote-worker".to_string(),
            },
            impact: WorkflowFamilyImpactAnalysis {
                family_id: "catalog-refresh".to_string(),
                impacted_workflows: vec!["catalog-refresh-eu".to_string()],
                impact_reason: "registry outage".to_string(),
            },
            dependencies: vec![
                "control-plane-api".to_string(),
                "artifact-registry".to_string(),
                "remote-worker".to_string(),
            ],
        };
        let (payload, ok) = dependency_catalog_payload(simulation);
        assert!(ok);
        assert_eq!(payload["dependency_catalog_ready"], true);
    }

    #[test]
    fn dependency_catalog_flags_missing_service_mapping() {
        let simulation = DependencyCatalogSimulation {
            workflow_family: "catalog-refresh".to_string(),
            architecture: ServiceArchitectureNote {
                api_boundary: String::new(),
                scheduler_boundary: String::new(),
                registry_boundary: String::new(),
                executor_boundary: String::new(),
            },
            impact: WorkflowFamilyImpactAnalysis {
                family_id: "catalog-refresh".to_string(),
                impacted_workflows: Vec::new(),
                impact_reason: String::new(),
            },
            dependencies: Vec::new(),
        };
        let (payload, ok) = dependency_catalog_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn credentials_accept_short_lived_brokered_worker_scope() {
        let simulation = CredentialSimulation {
            now_unix_ms: 100,
            lifecycle: CredentialLifecycle {
                issued_unix_ms: 10,
                expires_unix_ms: 200,
                renewable: true,
                max_renewals: 3,
            },
            renewal_count: 1,
            scope: CredentialScope { cli: false, api_client: false, scheduler: false, worker: true },
            binding: WorkerCredentialBinding {
                worker_id: "worker-1".to_string(),
                lease_id: "lease-1".to_string(),
                run_scope: Some("run-17".to_string()),
                expires_unix_ms: 220,
            },
            broker_class: "oidc-broker".to_string(),
        };
        let (payload, ok) = credential_payload(simulation);
        assert!(ok);
        assert_eq!(payload["credential_ready"], true);
    }

    #[test]
    fn credentials_flag_static_or_expired_paths() {
        let simulation = CredentialSimulation {
            now_unix_ms: 300,
            lifecycle: CredentialLifecycle {
                issued_unix_ms: 10,
                expires_unix_ms: 200,
                renewable: false,
                max_renewals: 0,
            },
            renewal_count: 1,
            scope: CredentialScope { cli: true, api_client: false, scheduler: false, worker: false },
            binding: WorkerCredentialBinding {
                worker_id: String::new(),
                lease_id: String::new(),
                run_scope: None,
                expires_unix_ms: 100,
            },
            broker_class: "static-secret".to_string(),
        };
        let (payload, ok) = credential_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }
}
