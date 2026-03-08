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
struct VerifyInput {
    run_id: String,
    has_binary_provenance: bool,
    has_plugin_provenance: bool,
    has_environment_attestation: bool,
    has_signed_artifacts: bool,
}

fn main() -> ExitCode {
    let input_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "attestation-input.json".to_string());

    match verify_file(&input_path) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
            if report["passed"].as_bool().unwrap_or(false) {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            let report = json!({
                "passed": false,
                "errors": [error],
            });
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
            ExitCode::from(1)
        }
    }
}

fn verify_file(path: &str) -> Result<serde_json::Value, String> {
    if !Path::new(path).exists() {
        return Err(format!("input file not found: {path}"));
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let input: VerifyInput =
        serde_json::from_str(&raw).map_err(|e| format!("invalid json payload: {e}"))?;

    let mut errors = Vec::new();
    if !input.has_binary_provenance {
        errors.push("missing binary provenance".to_string());
    }
    if !input.has_plugin_provenance {
        errors.push("missing plugin provenance".to_string());
    }
    if !input.has_environment_attestation {
        errors.push("missing environment attestation".to_string());
    }
    if !input.has_signed_artifacts {
        errors.push("missing signed artifacts".to_string());
    }

    Ok(json!({
        "run_id": input.run_id,
        "passed": errors.is_empty(),
        "errors": errors,
        "required": {
            "binary_provenance": true,
            "plugin_provenance": true,
            "environment_attestation": true,
            "signed_artifacts": true
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::verify_file;
    use std::fs;

    #[test]
    fn verify_file_reports_missing_required_flags() {
        let temp = tempfile::tempdir().expect("tempdir");
        let input = temp.path().join("attestation-input.json");
        fs::write(
            &input,
            r#"{
              "run_id": "run-1",
              "has_binary_provenance": true,
              "has_plugin_provenance": false,
              "has_environment_attestation": true,
              "has_signed_artifacts": false
            }"#,
        )
        .expect("write input");

        let report = verify_file(input.to_str().expect("utf8 path")).expect("verify");
        assert_eq!(report["passed"], false);
        assert_eq!(report["run_id"], "run-1");
    }
}
