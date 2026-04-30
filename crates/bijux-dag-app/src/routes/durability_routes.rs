use crate::commands::{DagCli, DurabilityCommands};
use crate::{emit_json, ExitCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
struct ModuleSurfaceBudgetsSimulation {
    max_module_lines: usize,
    modules: Vec<ModuleLineCount>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct ModuleLineCount {
    module: String,
    lines: usize,
}

#[derive(Debug, Serialize)]
struct ModuleSurfaceBudgetsReport {
    policy_lane: &'static str,
    max_module_lines: usize,
    oversized_modules: Vec<String>,
    within_budget: bool,
    modules: Vec<ModuleLineCount>,
}

fn load_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = fs::read_to_string(path).map_err(|_| ExitCode::from(3))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(2))
}

fn module_surface_budgets_payload(
    simulation: &Path,
) -> Result<ModuleSurfaceBudgetsReport, ExitCode> {
    let simulation: ModuleSurfaceBudgetsSimulation = load_json_file(simulation)?;
    let mut oversized_modules = simulation
        .modules
        .iter()
        .filter(|module| module.lines > simulation.max_module_lines)
        .map(|module| module.module.clone())
        .collect::<Vec<_>>();
    oversized_modules.sort();
    let within_budget = oversized_modules.is_empty();
    Ok(ModuleSurfaceBudgetsReport {
        policy_lane: "ENFORCED",
        max_module_lines: simulation.max_module_lines,
        oversized_modules,
        within_budget,
        modules: simulation.modules,
    })
}

pub(crate) fn handle_durability_command(
    cli: &DagCli,
    command: &DurabilityCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        DurabilityCommands::ModuleSurfaceBudgets { simulation } => {
            let payload =
                serde_json::to_value(module_surface_budgets_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.module-surface-budgets",
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
    use super::handle_durability_command;
    use crate::commands::{Commands, DagCli, DurabilityCommands};
    use crate::ExitCode;

    fn quiet_json_cli(command: DurabilityCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Durability { command } }
    }

    #[test]
    fn durability_module_surface_budgets_accepts_modules_within_line_budget() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("module-budgets-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "max_module_lines":1200,
              "modules":[
                {"module":"routes/security_routes.rs","lines":1100},
                {"module":"routes/performance_routes.rs","lines":900}
              ]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(DurabilityCommands::ModuleSurfaceBudgets { simulation: simulation.clone() });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::ModuleSurfaceBudgets { simulation: simulation.clone() },
        )
        .expect("module budgets");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::module_surface_budgets_payload(&simulation).expect("report");
        assert!(report.within_budget);
        assert!(report.oversized_modules.is_empty());
    }

    #[test]
    fn durability_module_surface_budgets_flags_oversized_modules() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("module-budgets-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "max_module_lines":1000,
              "modules":[
                {"module":"routes/security_routes.rs","lines":1300},
                {"module":"routes/performance_routes.rs","lines":1323}
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::module_surface_budgets_payload(&simulation).expect("report");
        assert!(!report.within_budget);
        assert_eq!(
            report.oversized_modules,
            vec![
                "routes/performance_routes.rs".to_string(),
                "routes/security_routes.rs".to_string()
            ]
        );
    }
}
