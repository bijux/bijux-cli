use crate::commands::{DagCli, SecurityCommands};
use crate::{emit_json, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    can_renew_credential, credential_is_expired, credential_scopes_matrix,
    local_dev_bypass_allowed, readiness_for_federation, revoked_principals_set, trust_health_report,
    AuthenticationEvent, CredentialLifecycle, CredentialRevocation, CredentialScope,
    IdentityPrincipal, LocalDevAuthBypassRule, TrustHealthReport,
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

pub(crate) fn handle_security_command(
    cli: &DagCli,
    command: &SecurityCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        SecurityCommands::Auth { simulation } => {
            let payload = serde_json::to_value(auth_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(cli, "dag.security.auth", true, payload, Vec::new(), ExitCode::SUCCESS)
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
}
