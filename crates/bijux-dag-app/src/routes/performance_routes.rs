use crate::commands::{DagCli, PerformanceCommands};
use crate::{emit_json, ExitCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct LatencyBudgetSimulation {
    p50_budget_ms: u64,
    p95_budget_ms: u64,
    measurements: Vec<LatencyMeasurement>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct LatencyMeasurement {
    name: String,
    p50_ms: u64,
    p95_ms: u64,
}

#[derive(Debug, Serialize)]
struct LatencyBudgetReport {
    policy_lane: &'static str,
    p50_budget_ms: u64,
    p95_budget_ms: u64,
    within_budget: bool,
    breached_measurements: Vec<String>,
    measurements: Vec<LatencyMeasurement>,
}

fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
}

fn latency_budgets_payload(simulation: &Path) -> Result<LatencyBudgetReport, ExitCode> {
    let simulation: LatencyBudgetSimulation = load_json_file(simulation)?;
    let mut breached_measurements = simulation
        .measurements
        .iter()
        .filter(|measurement| {
            measurement.p50_ms > simulation.p50_budget_ms
                || measurement.p95_ms > simulation.p95_budget_ms
        })
        .map(|measurement| measurement.name.clone())
        .collect::<Vec<_>>();
    breached_measurements.sort();
    let within_budget = breached_measurements.is_empty();
    Ok(LatencyBudgetReport {
        policy_lane: "ENFORCED",
        p50_budget_ms: simulation.p50_budget_ms,
        p95_budget_ms: simulation.p95_budget_ms,
        within_budget,
        breached_measurements,
        measurements: simulation.measurements,
    })
}

pub(crate) fn handle_performance_command(
    cli: &DagCli,
    command: &PerformanceCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PerformanceCommands::LatencyBudgets { simulation } => {
            let payload =
                serde_json::to_value(latency_budgets_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.performance.latency-budgets",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_performance_command;
    use crate::commands::{Commands, DagCli, PerformanceCommands};
    use crate::ExitCode;

    fn quiet_json_cli(command: PerformanceCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Performance { command } }
    }

    #[test]
    fn performance_latency_budgets_accepts_within_budget_measurements() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("latency-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "p50_budget_ms":120,
              "p95_budget_ms":300,
              "measurements":[
                {"name":"route_dispatch","p50_ms":20,"p95_ms":90},
                {"name":"graph_parse","p50_ms":40,"p95_ms":150}
              ]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(PerformanceCommands::LatencyBudgets { simulation: simulation.clone() });
        let code = handle_performance_command(
            &cli,
            &PerformanceCommands::LatencyBudgets { simulation: simulation.clone() },
        )
        .expect("latency budgets");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::latency_budgets_payload(&simulation).expect("report");
        assert!(report.within_budget);
        assert!(report.breached_measurements.is_empty());
        assert_eq!(report.policy_lane, "ENFORCED");
    }

    #[test]
    fn performance_latency_budgets_flags_p50_and_p95_regressions() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("latency-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "p50_budget_ms":120,
              "p95_budget_ms":300,
              "measurements":[
                {"name":"route_dispatch","p50_ms":121,"p95_ms":290},
                {"name":"graph_parse","p50_ms":80,"p95_ms":301}
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::latency_budgets_payload(&simulation).expect("report");
        assert!(!report.within_budget);
        assert_eq!(
            report.breached_measurements,
            vec!["graph_parse".to_string(), "route_dispatch".to_string()]
        );
    }
}
