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
struct SimulationInput {
    run_count: usize,
    artifact_count: usize,
    migration_steps: usize,
    supported_paths: Vec<String>,
    from_version: String,
    to_version: String,
}

fn main() -> ExitCode {
    let input_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "migration-simulate-input.json".to_string());

    match run_simulation(&input_path) {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).expect("json"));
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({"error": error})).expect("json")
            );
            ExitCode::from(1)
        }
    }
}

fn run_simulation(path: &str) -> Result<serde_json::Value, String> {
    if !Path::new(path).exists() {
        return Err(format!("simulation input not found: {path}"));
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let input: SimulationInput =
        serde_json::from_str(&raw).map_err(|e| format!("invalid simulation input: {e}"))?;

    let path_key = format!("{}->{}", input.from_version, input.to_version);
    let supported = input.supported_paths.iter().any(|item| item == &path_key);

    let requires_downtime = input.migration_steps > 3;
    let estimated_minutes = (input.migration_steps as u32) * 15;

    Ok(json!({
        "path": path_key,
        "supported": supported,
        "impact": {
            "affected_runs": input.run_count,
            "affected_artifacts": input.artifact_count,
            "requires_downtime": requires_downtime,
            "estimated_minutes": estimated_minutes
        },
        "recommendation": if supported {
            "canary rollout with verification evidence"
        } else {
            "block rollout until migration path is approved"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::run_simulation;
    use std::fs;

    #[test]
    fn simulation_marks_unsupported_paths_and_downtime() {
        let temp = tempfile::tempdir().expect("tempdir");
        let input = temp.path().join("migration-simulate-input.json");
        fs::write(
            &input,
            r#"{
              "run_count": 7,
              "artifact_count": 13,
              "migration_steps": 4,
              "supported_paths": ["1.0->1.1"],
              "from_version": "1.0",
              "to_version": "2.0"
            }"#,
        )
        .expect("write input");

        let report = run_simulation(input.to_str().expect("utf8 path")).expect("simulate");
        assert_eq!(report["supported"], false);
        assert_eq!(report["impact"]["requires_downtime"], true);
    }
}
