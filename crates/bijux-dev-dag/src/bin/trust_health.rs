use serde_json::json;
use std::env;
use std::process::ExitCode;

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
    env::var("BIJUX_ACTIVE_IDENTITIES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(3)
}
