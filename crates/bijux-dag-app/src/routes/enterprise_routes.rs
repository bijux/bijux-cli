use crate::commands::{DagCli, EnterpriseCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{
    execution_mode_status, remote_handoff_valid, validate_remote_identity, ExecutionModeStatus,
    RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
};
use bijux_dag_runtime::simulated_platform::{
    authorize, AuthContext, AuthenticationPrincipal, AuthorizationRule, EventSubscription,
    QueueResource,
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
        queue_payload, service_contract_payload, webhook_payload, QueueSimulation,
        ServiceContractSimulation, WebhookSimulation,
    };
    use bijux_dag_runtime::{
        RemoteArtifactHandoff, RemoteExecutionIdentity, RemoteObservabilityHandoff,
    };
    use bijux_dag_runtime::simulated_platform::{
        AuthContext, AuthenticationPrincipal, AuthorizationRule, EventSubscription,
        QueueResource,
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
}
