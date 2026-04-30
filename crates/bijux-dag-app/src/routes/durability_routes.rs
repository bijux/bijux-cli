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

#[derive(Debug, serde::Deserialize)]
struct PublicApiReviewSimulation {
    crates: Vec<PublicApiCrateSurface>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct PublicApiCrateSurface {
    crate_name: String,
    stable: usize,
    experimental: usize,
    test_only: usize,
    accidental: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PublicApiReviewReport {
    policy_lane: &'static str,
    total_public_items: usize,
    accidental_items: Vec<String>,
    review_passed: bool,
    crates: Vec<PublicApiCrateSurface>,
}

#[derive(Debug, serde::Deserialize)]
struct ContractAlignmentSimulation {
    contracts: Vec<ContractCoverageEntry>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct ContractCoverageEntry {
    crate_name: String,
    has_contract: bool,
    has_ownership: bool,
    has_non_goals: bool,
    has_stable_outputs: bool,
    has_forbidden_dependencies: bool,
}

#[derive(Debug, Serialize)]
struct ContractAlignmentReport {
    policy_lane: &'static str,
    aligned: bool,
    missing_sections: Vec<String>,
    contracts: Vec<ContractCoverageEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct CompatibilityFixturesSimulation {
    fixtures: std::collections::BTreeMap<String, Vec<String>>,
    required_categories: Vec<String>,
    min_versions_per_category: usize,
}

#[derive(Debug, Serialize)]
struct CompatibilityFixturesReport {
    policy_lane: &'static str,
    has_required_categories: bool,
    version_depth_ok: bool,
    missing_categories: Vec<String>,
    underfilled_categories: Vec<String>,
    fixture_count_by_category: std::collections::BTreeMap<String, usize>,
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

fn public_api_review_payload(simulation: &Path) -> Result<PublicApiReviewReport, ExitCode> {
    let simulation: PublicApiReviewSimulation = load_json_file(simulation)?;
    let mut accidental_items = simulation
        .crates
        .iter()
        .flat_map(|crate_surface| {
            crate_surface
                .accidental
                .iter()
                .map(|item| format!("{}::{item}", crate_surface.crate_name))
        })
        .collect::<Vec<_>>();
    accidental_items.sort();
    let total_public_items = simulation
        .crates
        .iter()
        .map(|crate_surface| {
            crate_surface.stable
                + crate_surface.experimental
                + crate_surface.test_only
                + crate_surface.accidental.len()
        })
        .sum::<usize>();
    let review_passed = accidental_items.is_empty();
    Ok(PublicApiReviewReport {
        policy_lane: "ENFORCED",
        total_public_items,
        accidental_items,
        review_passed,
        crates: simulation.crates,
    })
}

fn contract_alignment_payload(simulation: &Path) -> Result<ContractAlignmentReport, ExitCode> {
    let simulation: ContractAlignmentSimulation = load_json_file(simulation)?;
    let mut missing_sections = Vec::new();
    for contract in &simulation.contracts {
        if !contract.has_contract {
            missing_sections.push(format!("{} missing CONTRACT.md", contract.crate_name));
        }
        if !contract.has_ownership {
            missing_sections.push(format!("{} missing ownership section", contract.crate_name));
        }
        if !contract.has_non_goals {
            missing_sections.push(format!("{} missing non-goals section", contract.crate_name));
        }
        if !contract.has_stable_outputs {
            missing_sections.push(format!("{} missing stable outputs section", contract.crate_name));
        }
        if !contract.has_forbidden_dependencies {
            missing_sections
                .push(format!("{} missing forbidden dependencies section", contract.crate_name));
        }
    }
    missing_sections.sort();
    let aligned = missing_sections.is_empty();
    Ok(ContractAlignmentReport {
        policy_lane: "ENFORCED",
        aligned,
        missing_sections,
        contracts: simulation.contracts,
    })
}

fn compatibility_fixtures_payload(
    simulation: &Path,
) -> Result<CompatibilityFixturesReport, ExitCode> {
    let simulation: CompatibilityFixturesSimulation = load_json_file(simulation)?;
    let fixture_count_by_category = simulation
        .fixtures
        .iter()
        .map(|(category, fixtures)| (category.clone(), fixtures.len()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut missing_categories = simulation
        .required_categories
        .iter()
        .filter(|category| !simulation.fixtures.contains_key(*category))
        .cloned()
        .collect::<Vec<_>>();
    missing_categories.sort();
    let mut underfilled_categories = simulation
        .fixtures
        .iter()
        .filter(|(_, fixtures)| fixtures.len() < simulation.min_versions_per_category)
        .map(|(category, _)| category.clone())
        .collect::<Vec<_>>();
    underfilled_categories.sort();

    let has_required_categories = missing_categories.is_empty();
    let version_depth_ok = underfilled_categories.is_empty();
    Ok(CompatibilityFixturesReport {
        policy_lane: "ENFORCED",
        has_required_categories,
        version_depth_ok,
        missing_categories,
        underfilled_categories,
        fixture_count_by_category,
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
        DurabilityCommands::PublicApiReview { simulation } => {
            let payload =
                serde_json::to_value(public_api_review_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.public-api-review",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        DurabilityCommands::ContractAlignment { simulation } => {
            let payload =
                serde_json::to_value(contract_alignment_payload(simulation)?).map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.contract-alignment",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        DurabilityCommands::CompatibilityFixtures { simulation } => {
            let payload = serde_json::to_value(compatibility_fixtures_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.compatibility-fixtures",
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

    #[test]
    fn durability_public_api_review_accepts_no_accidental_items() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("public-api-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "crates":[
                {
                  "crate_name":"bijux-dag-core",
                  "stable":42,
                  "experimental":4,
                  "test_only":2,
                  "accidental":[]
                },
                {
                  "crate_name":"bijux-dag-runtime",
                  "stable":28,
                  "experimental":3,
                  "test_only":1,
                  "accidental":[]
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(DurabilityCommands::PublicApiReview { simulation: simulation.clone() });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::PublicApiReview { simulation: simulation.clone() },
        )
        .expect("public api review");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::public_api_review_payload(&simulation).expect("report");
        assert!(report.review_passed);
        assert!(report.accidental_items.is_empty());
        assert_eq!(report.total_public_items, 80);
    }

    #[test]
    fn durability_public_api_review_flags_accidental_items() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("public-api-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "crates":[
                {
                  "crate_name":"bijux-dag-core",
                  "stable":40,
                  "experimental":3,
                  "test_only":2,
                  "accidental":["internal::legacy_hash","routes::debug_surface"]
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::public_api_review_payload(&simulation).expect("report");
        assert!(!report.review_passed);
        assert_eq!(
            report.accidental_items,
            vec![
                "bijux-dag-core::internal::legacy_hash".to_string(),
                "bijux-dag-core::routes::debug_surface".to_string()
            ]
        );
    }

    #[test]
    fn durability_contract_alignment_accepts_complete_contract_sections() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("contract-alignment-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "contracts":[
                {
                  "crate_name":"bijux-dag-core",
                  "has_contract":true,
                  "has_ownership":true,
                  "has_non_goals":true,
                  "has_stable_outputs":true,
                  "has_forbidden_dependencies":true
                },
                {
                  "crate_name":"bijux-dag-runtime",
                  "has_contract":true,
                  "has_ownership":true,
                  "has_non_goals":true,
                  "has_stable_outputs":true,
                  "has_forbidden_dependencies":true
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(DurabilityCommands::ContractAlignment { simulation: simulation.clone() });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::ContractAlignment { simulation: simulation.clone() },
        )
        .expect("contract alignment");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::contract_alignment_payload(&simulation).expect("report");
        assert!(report.aligned);
        assert!(report.missing_sections.is_empty());
    }

    #[test]
    fn durability_contract_alignment_flags_missing_contract_sections() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("contract-alignment-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "contracts":[
                {
                  "crate_name":"bijux-dag-core",
                  "has_contract":true,
                  "has_ownership":false,
                  "has_non_goals":true,
                  "has_stable_outputs":false,
                  "has_forbidden_dependencies":true
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::contract_alignment_payload(&simulation).expect("report");
        assert!(!report.aligned);
        assert_eq!(
            report.missing_sections,
            vec![
                "bijux-dag-core missing ownership section".to_string(),
                "bijux-dag-core missing stable outputs section".to_string()
            ]
        );
    }

    #[test]
    fn durability_compatibility_fixtures_accepts_required_coverage() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("compatibility-fixtures-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "fixtures":{
                "graph_spec":["v1.json","v2.json"],
                "plan":["v1.json","v2.json"],
                "run_manifest":["v1.json","v2.json"],
                "artifact_index":["v1.json","v2.json"],
                "cache_key":["v1.json","v2.json"],
                "cli_envelope":["v1.json","v2.json"]
              },
              "required_categories":[
                "graph_spec",
                "plan",
                "run_manifest",
                "artifact_index",
                "cache_key",
                "cli_envelope"
              ],
              "min_versions_per_category":2
            }"#,
        )
        .expect("write simulation");
        let cli =
            quiet_json_cli(DurabilityCommands::CompatibilityFixtures { simulation: simulation.clone() });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::CompatibilityFixtures { simulation: simulation.clone() },
        )
        .expect("compatibility fixtures");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::compatibility_fixtures_payload(&simulation).expect("report");
        assert!(report.has_required_categories);
        assert!(report.version_depth_ok);
        assert!(report.missing_categories.is_empty());
        assert!(report.underfilled_categories.is_empty());
    }

    #[test]
    fn durability_compatibility_fixtures_flags_missing_and_shallow_categories() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("compatibility-fixtures-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "fixtures":{
                "graph_spec":["v1.json"],
                "plan":["v1.json","v2.json"]
              },
              "required_categories":["graph_spec","plan","run_manifest"],
              "min_versions_per_category":2
            }"#,
        )
        .expect("write simulation");
        let report = super::compatibility_fixtures_payload(&simulation).expect("report");
        assert!(!report.has_required_categories);
        assert!(!report.version_depth_ok);
        assert_eq!(report.missing_categories, vec!["run_manifest".to_string()]);
        assert_eq!(report.underfilled_categories, vec!["graph_spec".to_string()]);
    }
}
