use crate::commands::{DagCli, DurabilityCommands};
use crate::routes::simulation_io::load_json_file;
use crate::{emit_json, ExitCode};
use serde::Serialize;
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

#[derive(Debug, serde::Deserialize)]
struct ChangeImpactLabelsSimulation {
    changed_surfaces: Vec<String>,
    pr_labels: Vec<String>,
    required_labels_by_surface: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct ChangeImpactLabelsReport {
    policy_lane: &'static str,
    labels_complete: bool,
    missing_labels: Vec<String>,
    unexpected_labels: Vec<String>,
    changed_surfaces: Vec<String>,
    pr_labels: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ReleaseNotesEvidenceSimulation {
    entries: Vec<ReleaseNoteEvidenceEntry>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct ReleaseNoteEvidenceEntry {
    title: String,
    contracts: Vec<String>,
    fixtures: Vec<String>,
    verifications: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReleaseNotesEvidenceReport {
    policy_lane: &'static str,
    evidence_complete: bool,
    entries_without_evidence: Vec<String>,
    entries: Vec<ReleaseNoteEvidenceEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct MediumAcceptanceGateSimulation {
    nested_graph: bool,
    branch_join: bool,
    cache_reuse: bool,
    forced_rerun: bool,
    bundle_export_import: bool,
    root_cli_mount_parity: bool,
}

#[derive(Debug, Serialize)]
struct MediumAcceptanceGateReport {
    policy_lane: &'static str,
    gate_passed: bool,
    failed_checks: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ProductionCandidateSimulation {
    workflows: Vec<ProductionCandidateWorkflow>,
}

#[derive(Debug, serde::Deserialize, Serialize)]
struct ProductionCandidateWorkflow {
    name: String,
    cache_verified: bool,
    replay_verified: bool,
    diagnostics_bundle_verified: bool,
    retention_policy_verified: bool,
    evidence_verified: bool,
}

#[derive(Debug, Serialize)]
struct ProductionCandidateReport {
    policy_lane: &'static str,
    candidate_passed: bool,
    verified_workflow_count: usize,
    failing_workflows: Vec<String>,
    workflows: Vec<ProductionCandidateWorkflow>,
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
            missing_sections.push(format!("{} missing docs/CONTRACTS.md", contract.crate_name));
        }
        if !contract.has_ownership {
            missing_sections.push(format!("{} missing ownership section", contract.crate_name));
        }
        if !contract.has_non_goals {
            missing_sections.push(format!("{} missing non-goals section", contract.crate_name));
        }
        if !contract.has_stable_outputs {
            missing_sections
                .push(format!("{} missing stable outputs section", contract.crate_name));
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

fn change_impact_labels_payload(simulation: &Path) -> Result<ChangeImpactLabelsReport, ExitCode> {
    let simulation: ChangeImpactLabelsSimulation = load_json_file(simulation)?;
    let mut missing_labels = simulation
        .changed_surfaces
        .iter()
        .filter_map(|surface| {
            simulation
                .required_labels_by_surface
                .get(surface)
                .filter(|required_label| !simulation.pr_labels.contains(*required_label))
                .map(|required_label| format!("{surface}->{required_label}"))
        })
        .collect::<Vec<_>>();
    missing_labels.sort();
    let required_labels = simulation
        .changed_surfaces
        .iter()
        .filter_map(|surface| simulation.required_labels_by_surface.get(surface))
        .collect::<std::collections::BTreeSet<_>>();
    let mut unexpected_labels = simulation
        .pr_labels
        .iter()
        .filter(|label| !required_labels.contains(label))
        .cloned()
        .collect::<Vec<_>>();
    unexpected_labels.sort();
    let labels_complete = missing_labels.is_empty();
    Ok(ChangeImpactLabelsReport {
        policy_lane: "ENFORCED",
        labels_complete,
        missing_labels,
        unexpected_labels,
        changed_surfaces: simulation.changed_surfaces,
        pr_labels: simulation.pr_labels,
    })
}

fn release_notes_evidence_payload(
    simulation: &Path,
) -> Result<ReleaseNotesEvidenceReport, ExitCode> {
    let simulation: ReleaseNotesEvidenceSimulation = load_json_file(simulation)?;
    let mut entries_without_evidence = simulation
        .entries
        .iter()
        .filter(|entry| {
            entry.contracts.is_empty()
                || entry.fixtures.is_empty()
                || entry.verifications.is_empty()
        })
        .map(|entry| entry.title.clone())
        .collect::<Vec<_>>();
    entries_without_evidence.sort();
    let evidence_complete = entries_without_evidence.is_empty();
    Ok(ReleaseNotesEvidenceReport {
        policy_lane: "ENFORCED",
        evidence_complete,
        entries_without_evidence,
        entries: simulation.entries,
    })
}

fn medium_acceptance_gate_payload(
    simulation: &Path,
) -> Result<MediumAcceptanceGateReport, ExitCode> {
    let simulation: MediumAcceptanceGateSimulation = load_json_file(simulation)?;
    let mut failed_checks = Vec::new();
    if !simulation.nested_graph {
        failed_checks.push("nested-graph".to_string());
    }
    if !simulation.branch_join {
        failed_checks.push("branch-join".to_string());
    }
    if !simulation.cache_reuse {
        failed_checks.push("cache-reuse".to_string());
    }
    if !simulation.forced_rerun {
        failed_checks.push("forced-rerun".to_string());
    }
    if !simulation.bundle_export_import {
        failed_checks.push("bundle-export-import".to_string());
    }
    if !simulation.root_cli_mount_parity {
        failed_checks.push("root-cli-mount-parity".to_string());
    }
    let gate_passed = failed_checks.is_empty();
    Ok(MediumAcceptanceGateReport { policy_lane: "ENFORCED", gate_passed, failed_checks })
}

fn production_candidate_payload(simulation: &Path) -> Result<ProductionCandidateReport, ExitCode> {
    let simulation: ProductionCandidateSimulation = load_json_file(simulation)?;
    let mut failing_workflows = simulation
        .workflows
        .iter()
        .filter(|workflow| {
            !(workflow.cache_verified
                && workflow.replay_verified
                && workflow.diagnostics_bundle_verified
                && workflow.retention_policy_verified
                && workflow.evidence_verified)
        })
        .map(|workflow| workflow.name.clone())
        .collect::<Vec<_>>();
    failing_workflows.sort();
    let candidate_passed = !simulation.workflows.is_empty() && failing_workflows.is_empty();
    Ok(ProductionCandidateReport {
        policy_lane: "ENFORCED",
        candidate_passed,
        verified_workflow_count: simulation.workflows.len().saturating_sub(failing_workflows.len()),
        failing_workflows,
        workflows: simulation.workflows,
    })
}

pub(crate) fn handle_durability_command(
    cli: &DagCli,
    command: &DurabilityCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        DurabilityCommands::ModuleSurfaceBudgets { simulation } => {
            let payload = serde_json::to_value(module_surface_budgets_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
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
            let payload = serde_json::to_value(typed_contracts_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
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
            let payload = serde_json::to_value(public_api_review_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
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
            let payload = serde_json::to_value(contract_alignment_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
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
        DurabilityCommands::ChangeImpactLabels { simulation } => {
            let payload = serde_json::to_value(change_impact_labels_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.change-impact-labels",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        DurabilityCommands::ReleaseNotesEvidence { simulation } => {
            let payload = serde_json::to_value(release_notes_evidence_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.release-notes-evidence",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        DurabilityCommands::MediumAcceptanceGate { simulation } => {
            let payload = serde_json::to_value(medium_acceptance_gate_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.medium-acceptance-gate",
                true,
                payload,
                Vec::new(),
                ExitCode::SUCCESS,
            )
        }
        DurabilityCommands::ProductionCandidate { simulation } => {
            let payload = serde_json::to_value(production_candidate_payload(simulation)?)
                .map_err(|_| ExitCode::from(3))?;
            emit_json(
                cli,
                "dag.durability.production-candidate",
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
        let cli = quiet_json_cli(DurabilityCommands::ModuleSurfaceBudgets {
            simulation: simulation.clone(),
        });
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
        let cli =
            quiet_json_cli(DurabilityCommands::TypedContracts { simulation: simulation.clone() });
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
        let cli = quiet_json_cli(DurabilityCommands::ContractAlignment {
            simulation: simulation.clone(),
        });
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
        let cli = quiet_json_cli(DurabilityCommands::CompatibilityFixtures {
            simulation: simulation.clone(),
        });
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

    #[test]
    fn durability_change_impact_labels_accepts_required_surface_labels() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("change-impact-labels-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "changed_surfaces":["cli","runtime","artifact"],
              "pr_labels":["impact:cli","impact:runtime","impact:artifact"],
              "required_labels_by_surface":{
                "cli":"impact:cli",
                "runtime":"impact:runtime",
                "artifact":"impact:artifact"
              }
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(DurabilityCommands::ChangeImpactLabels {
            simulation: simulation.clone(),
        });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::ChangeImpactLabels { simulation: simulation.clone() },
        )
        .expect("change impact labels");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::change_impact_labels_payload(&simulation).expect("report");
        assert!(report.labels_complete);
        assert!(report.missing_labels.is_empty());
    }

    #[test]
    fn durability_change_impact_labels_flags_missing_required_labels() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("change-impact-labels-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "changed_surfaces":["cli","runtime"],
              "pr_labels":["impact:cli","impact:docs"],
              "required_labels_by_surface":{
                "cli":"impact:cli",
                "runtime":"impact:runtime"
              }
            }"#,
        )
        .expect("write simulation");
        let report = super::change_impact_labels_payload(&simulation).expect("report");
        assert!(!report.labels_complete);
        assert_eq!(report.missing_labels, vec!["runtime->impact:runtime".to_string()]);
        assert_eq!(report.unexpected_labels, vec!["impact:docs".to_string()]);
    }

    #[test]
    fn durability_release_notes_evidence_accepts_entries_with_proof() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("release-notes-evidence-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "entries":[
                {
                  "title":"enforce typed contracts on run-state surfaces",
                  "contracts":["contracts/runtime-typed-state.md"],
                  "fixtures":["fixtures/durability/typed-contracts-v2.json"],
                  "verifications":["cargo test -p bijux-dag-app durability_typed_contracts_"]
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(DurabilityCommands::ReleaseNotesEvidence {
            simulation: simulation.clone(),
        });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::ReleaseNotesEvidence { simulation: simulation.clone() },
        )
        .expect("release notes evidence");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::release_notes_evidence_payload(&simulation).expect("report");
        assert!(report.evidence_complete);
        assert!(report.entries_without_evidence.is_empty());
    }

    #[test]
    fn durability_release_notes_evidence_flags_entries_without_proof() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("release-notes-evidence-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "entries":[
                {
                  "title":"add release lane",
                  "contracts":[],
                  "fixtures":["fixtures/release/lane.json"],
                  "verifications":["cargo test -p bijux-dag-app release_lane"]
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::release_notes_evidence_payload(&simulation).expect("report");
        assert!(!report.evidence_complete);
        assert_eq!(report.entries_without_evidence, vec!["add release lane".to_string()]);
    }

    #[test]
    fn durability_medium_acceptance_gate_passes_full_checklist() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("medium-acceptance-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "nested_graph":true,
              "branch_join":true,
              "cache_reuse":true,
              "forced_rerun":true,
              "bundle_export_import":true,
              "root_cli_mount_parity":true
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(DurabilityCommands::MediumAcceptanceGate {
            simulation: simulation.clone(),
        });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::MediumAcceptanceGate { simulation: simulation.clone() },
        )
        .expect("medium acceptance gate");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::medium_acceptance_gate_payload(&simulation).expect("report");
        assert!(report.gate_passed);
        assert!(report.failed_checks.is_empty());
    }

    #[test]
    fn durability_medium_acceptance_gate_flags_missing_scenarios() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("medium-acceptance-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "nested_graph":true,
              "branch_join":false,
              "cache_reuse":false,
              "forced_rerun":true,
              "bundle_export_import":false,
              "root_cli_mount_parity":true
            }"#,
        )
        .expect("write simulation");
        let report = super::medium_acceptance_gate_payload(&simulation).expect("report");
        assert!(!report.gate_passed);
        assert_eq!(
            report.failed_checks,
            vec![
                "branch-join".to_string(),
                "cache-reuse".to_string(),
                "bundle-export-import".to_string()
            ]
        );
    }

    #[test]
    fn durability_production_candidate_passes_when_all_workflows_verified() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("production-candidate-good.json");
        std::fs::write(
            &simulation,
            r#"{
              "workflows":[
                {
                  "name":"workflow-a",
                  "cache_verified":true,
                  "replay_verified":true,
                  "diagnostics_bundle_verified":true,
                  "retention_policy_verified":true,
                  "evidence_verified":true
                },
                {
                  "name":"workflow-b",
                  "cache_verified":true,
                  "replay_verified":true,
                  "diagnostics_bundle_verified":true,
                  "retention_policy_verified":true,
                  "evidence_verified":true
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let cli = quiet_json_cli(DurabilityCommands::ProductionCandidate {
            simulation: simulation.clone(),
        });
        let code = handle_durability_command(
            &cli,
            &DurabilityCommands::ProductionCandidate { simulation: simulation.clone() },
        )
        .expect("production candidate");
        assert_eq!(code, ExitCode::SUCCESS);
        let report = super::production_candidate_payload(&simulation).expect("report");
        assert!(report.candidate_passed);
        assert_eq!(report.verified_workflow_count, 2);
        assert!(report.failing_workflows.is_empty());
    }

    #[test]
    fn durability_production_candidate_flags_incomplete_workflows() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let simulation = dir.path().join("production-candidate-bad.json");
        std::fs::write(
            &simulation,
            r#"{
              "workflows":[
                {
                  "name":"workflow-a",
                  "cache_verified":true,
                  "replay_verified":false,
                  "diagnostics_bundle_verified":true,
                  "retention_policy_verified":true,
                  "evidence_verified":true
                },
                {
                  "name":"workflow-b",
                  "cache_verified":true,
                  "replay_verified":true,
                  "diagnostics_bundle_verified":true,
                  "retention_policy_verified":true,
                  "evidence_verified":true
                }
              ]
            }"#,
        )
        .expect("write simulation");
        let report = super::production_candidate_payload(&simulation).expect("report");
        assert!(!report.candidate_passed);
        assert_eq!(report.verified_workflow_count, 1);
        assert_eq!(report.failing_workflows, vec!["workflow-a".to_string()]);
    }
}
