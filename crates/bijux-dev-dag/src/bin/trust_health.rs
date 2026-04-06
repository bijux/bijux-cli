use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
#[cfg(test)]
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::env;
use std::process::ExitCode;
use tempfile as _;

fn main() -> ExitCode {
    let credential_classes = vec!["local-token", "service-token", "worker-lease-token"];
    let policy_baselines = vec!["baseline-2026-03", "tenant-alpha-policy-2026-03"];

    let report = json!({
        "active_identities": infer_active_identity_count(),
        "credential_classes": credential_classes,
        "policy_baselines": policy_baselines,
        "status": "ok"
    });

    println!("{}", serde_json::to_string_pretty(&report).expect("json"));
    ExitCode::SUCCESS
}

fn infer_active_identity_count() -> usize {
    parse_active_identity_count(env::var("BIJUX_ACTIVE_IDENTITIES").ok())
}

fn parse_active_identity_count(raw: Option<String>) -> usize {
    raw.and_then(|v| v.parse::<usize>().ok()).unwrap_or(3)
}

#[cfg(test)]
mod tests {
    use super::parse_active_identity_count;

    #[test]
    fn defaults_when_variable_not_set() {
        assert_eq!(parse_active_identity_count(None), 3);
    }

    #[test]
    fn parses_identity_count_from_environment() {
        assert_eq!(parse_active_identity_count(Some("9".to_string())), 9);
    }
}
