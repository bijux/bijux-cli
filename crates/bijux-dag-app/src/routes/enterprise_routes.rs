use crate::commands::{DagCli, EnterpriseCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    authorize, AuthContext, AuthenticationPrincipal, AuthorizationRule, EventSubscription,
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
    use super::{webhook_payload, WebhookSimulation};
    use bijux_dag_runtime::simulated_platform::{
        AuthContext, AuthenticationPrincipal, AuthorizationRule, EventSubscription,
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
}
