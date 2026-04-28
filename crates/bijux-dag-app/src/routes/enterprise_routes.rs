use crate::commands::{DagCli, EnterpriseCommands};
use crate::{emit_json, read_file, ExitCode};
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
    use super::{queue_payload, webhook_payload, QueueSimulation, WebhookSimulation};
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
}
