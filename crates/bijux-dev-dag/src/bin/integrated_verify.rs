use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde::Deserialize;
use serde_json::json;
use sha2 as _;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use tempfile as _;

#[derive(Debug, Deserialize)]
struct VerificationInput {
    multi_tenant_passed: bool,
    ha_scheduler_passed: bool,
    policy_passed: bool,
    backend_passed: bool,
    artifact_passed: bool,
    compatibility_passed: bool,
}

fn main() -> ExitCode {
    let input_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "integrated-verify-input.json".to_string());

    match run_integrated_verification(&input_path) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
            if report["passed"].as_bool().unwrap_or(false) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"passed": false, "error": error}))
                    .expect("json")
            );
            ExitCode::from(1)
        }
    }
}

fn run_integrated_verification(path: &str) -> Result<serde_json::Value, String> {
    if !Path::new(path).exists() {
        return Err(format!("verification input not found: {path}"));
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let input: VerificationInput =
        serde_json::from_str(&raw).map_err(|e| format!("invalid input payload: {e}"))?;

    let checks = vec![
        ("multi-tenant", input.multi_tenant_passed),
        ("ha-scheduler", input.ha_scheduler_passed),
        ("policy", input.policy_passed),
        ("backend", input.backend_passed),
        ("artifact", input.artifact_passed),
        ("compatibility", input.compatibility_passed),
    ];

    let failed: Vec<String> = checks
        .iter()
        .filter_map(|(name, passed)| {
            if *passed {
                None
            } else {
                Some((*name).to_string())
            }
        })
        .collect();

    Ok(json!({
        "lane": "platform-integrated-verification",
        "passed": failed.is_empty(),
        "failed_domains": failed
    }))
}

#[cfg(test)]
mod tests {
    use super::run_integrated_verification;
    use std::fs;

    #[test]
    fn integrated_verification_collects_failed_domains() {
        let temp = tempfile::tempdir().expect("tempdir");
        let input = temp.path().join("integrated-verify-input.json");
        fs::write(
            &input,
            r#"{
              "multi_tenant_passed": true,
              "ha_scheduler_passed": false,
              "policy_passed": true,
              "backend_passed": true,
              "artifact_passed": false,
              "compatibility_passed": true
            }"#,
        )
        .expect("write input");

        let report =
            run_integrated_verification(input.to_str().expect("utf8 path")).expect("verify");
        assert_eq!(report["passed"], false);
        assert_eq!(report["failed_domains"][0], "ha-scheduler");
        assert_eq!(report["failed_domains"][1], "artifact");
    }
}
