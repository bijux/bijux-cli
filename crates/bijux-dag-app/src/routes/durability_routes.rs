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

#[derive(Debug, serde::Deserialize)]
struct TypedContractsSimulation {
    typed_contract_counts: std::collections::BTreeMap<String, usize>,
    stringly_contract_counts: std::collections::BTreeMap<String, usize>,
    deny_stringly_surfaces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct TypedContractsReport {
    policy_lane: &'static str,
    typed_contract_counts: std::collections::BTreeMap<String, usize>,
    stringly_contract_counts: std::collections::BTreeMap<String, usize>,
    denied_stringly_hits: Vec<String>,
    typed_coverage_ok: bool,
    gaps: Vec<String>,
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

fn typed_contracts_payload(simulation: &Path) -> Result<TypedContractsReport, ExitCode> {
    let simulation: TypedContractsSimulation = load_json_file(simulation)?;
    let mut denied_stringly_hits = simulation
        .deny_stringly_surfaces
        .iter()
        .filter_map(|surface| {
            simulation
                .stringly_contract_counts
                .get(surface)
                .filter(|count| **count > 0)
                .map(|_| surface.clone())
        })
        .collect::<Vec<_>>();
    denied_stringly_hits.sort();
    let mut gaps = Vec::new();
    if !denied_stringly_hits.is_empty() {
        gaps.push("stringly contract usage remains on denied surfaces".to_string());
    }
    for (surface, count) in &simulation.typed_contract_counts {
        if *count == 0 {
            gaps.push(format!("typed contract coverage missing for surface {surface}"));
        }
    }
    let typed_coverage_ok = gaps.is_empty();
    Ok(TypedContractsReport {
        policy_lane: "ENFORCED",
        typed_contract_counts: simulation.typed_contract_counts,
        stringly_contract_counts: simulation.stringly_contract_counts,
        denied_stringly_hits,
        typed_coverage_ok,
        gaps,
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
        DurabilityCommands::TypedContracts { simulation } => {
            let payload =
                serde_json::to_value(typed_contracts_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.typed-contracts",
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

    #[test]
    fn durability_typed_contracts_accepts_no_denied_stringly_usage() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("typed-contracts-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "typed_contract_counts":{"run_state":12,"failure_class":8,"event_kind":14},
              "stringly_contract_counts":{"run_state":0,"failure_class":0,"event_kind":0},
              "deny_stringly_surfaces":["run_state","failure_class","event_kind"]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(DurabilityCommands::TypedContracts { simulation: simulation.clone() });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::TypedContracts { simulation: simulation.clone() },
        )
        .expect("typed contracts");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::typed_contracts_payload(&simulation).expect("report");
        assert!(report.typed_coverage_ok);
        assert!(report.denied_stringly_hits.is_empty());
        assert!(report.gaps.is_empty());
    }

    #[test]
    fn durability_typed_contracts_flags_stringly_and_missing_typed_coverage() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("typed-contracts-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "typed_contract_counts":{"run_state":0,"failure_class":3},
              "stringly_contract_counts":{"run_state":2,"failure_class":0},
              "deny_stringly_surfaces":["run_state"]
            }"#,
        )
        .expect("write simulation");
        let report = super::typed_contracts_payload(&simulation).expect("report");
        assert!(!report.typed_coverage_ok);
        assert_eq!(report.denied_stringly_hits, vec!["run_state".to_string()]);
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "stringly contract usage remains on denied surfaces"));
        assert!(report
            .gaps
            .iter()
            .any(|gap| gap == "typed contract coverage missing for surface run_state"));
    }
}
