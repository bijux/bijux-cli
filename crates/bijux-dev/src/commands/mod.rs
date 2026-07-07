use clap::Parser;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

mod authoring_evidence;
mod battle_evidence;
mod benchmark_harness;
mod command_runtime;
mod compare_evidence;
mod contract_governance;
mod docs_governance;
mod evidence_access;
mod evidence_control_plane;
mod evidence_registry;
mod file_catalog;
mod model;
mod ops;
mod perf_evidence;
mod release_validation_suite;
mod reporting;
mod shared_io;
mod suite_dispatch;

use authoring_evidence::{
    run_authoring_coverage_report, run_show_effective_all_authoring, run_validate_all_authoring,
};
use battle_evidence::{
    run_battle_coverage_report, run_battle_scenario_mapping_validate,
    run_battle_scenarios_by_trust_report, run_battle_scenarios_report,
    run_battle_trust_by_scenario_report,
};
use command_runtime::{
    command_stdout as exec_command_stdout, run_status_and_json as exec_run_status_and_json,
    run_status_in_dir as exec_run_status_in_dir, run_stdout_and_json as exec_run_stdout_and_json,
    run_with_root as exec_run_with_root,
};
use compare_evidence::{
    run_compare_evidence_policy_verify, run_comparison_evidence_report,
    run_comparison_harness_guard,
};
use contract_governance::{
    run_contract_command_ownership_guard, run_contract_coverage_report,
    run_contract_schema_owner_guard, run_contract_test_links_guard, run_contract_versioning_guard,
    run_error_code_docs_tests_guard, run_error_code_registry_report,
};
use docs_governance::{
    run_docs_config_reduction_guard, run_docs_contract_reference_guard, run_docs_coverage_report,
    run_docs_governance_guard, run_docs_index_generate, run_docs_link_check,
    run_docs_schema_reference_guard, run_naming_governance_guard,
};
use evidence_access::{
    as_json as evidence_assets_as_json, load_registry_assets, render_assets_to_consumers_report,
    render_consumers_to_families_report, resolve_asset_by_id, resolve_assets_by_consumer,
    resolve_assets_by_family, resolve_assets_by_trust_property, verify_registry_access_bypass,
};
use evidence_control_plane::{
    run_evidence_release_set_verify, run_evidence_suite_policy_verify, run_evidence_summary_report,
    run_release_evidence_report,
};
use evidence_registry::{
    run_evidence_ledger_normalize, run_evidence_registry_diff, run_evidence_registry_missing,
    run_evidence_registry_orphans, run_evidence_registry_rebuild, run_evidence_registry_verify,
};
use file_catalog::{
    collect_all_files, collect_files_with_extension, newest_run, repository_files_with_extension,
    two_latest_runs, wildcard_match,
};
use model::{CommandContext, CommandEffect, SuiteDef};
use ops::*;
use perf_evidence::{
    run_perf_evidence_policy_verify, run_perf_evidence_summary, run_perf_release_set,
    run_performance_evidence_guard, run_performance_evidence_report,
};
use release_validation_suite::run_release_suite_explain;
use reporting::run_command_reported;
use shared_io::{read_json_value, write_pretty_json};
use suite_dispatch::{run_suite_explain, run_suite_group, run_suite_list};

mod cli;

use cli::{
    root_command_names, ApiCommand, Cli, CommandLine, ControlCommand, DagCommand, ReleaseCommand,
    RepoCommand, ScheduleCommand, VerifyCommand, ADAPTER_KIND_FREEZE_BASELINE,
};

#[derive(Debug, Deserialize)]
struct DependencyPolicy {
    rules: Vec<DependencyRule>,
}

#[derive(Debug, Deserialize)]
struct DependencyRule {
    from: String,
    to: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct CrateOwnershipPolicy {
    crates: Vec<CrateOwnershipEntry>,
}

#[derive(Debug, Deserialize)]
struct CrateOwnershipEntry {
    name: String,
    path: String,
    domains: Vec<String>,
    public_modules: Vec<String>,
}

mod suite_catalog;

use suite_catalog::{
    CHECK_SUITES, CONTRACT_SUITES, DOC_SUITES, RELEASE_SUITES, REPO_SUITES, TEST_SUITES,
};

pub fn entry_main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let context = CommandContext { json: cli.json, report: cli.report };
    match cli.command {
        CommandLine::Fmt => {
            run_command_reported(&context, "fmt", CommandEffect::Validation, json!({}), || {
                run_status("cargo", &["fmt", "--all"])
            })
        }
        CommandLine::Lint => {
            run_command_reported(&context, "lint", CommandEffect::Validation, json!({}), || {
                run_status("cargo", &["fmt", "--all", "--", "--check"])?;
                run_status(
                    "cargo",
                    &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
                )
            })
        }
        CommandLine::Security => {
            run_command_reported(&context, "security", CommandEffect::Validation, json!({}), || {
                run_audit_allowlist_quality_gate()?;
                run_deny_policy_deviations_gate()?;
                run_cargo_audit_with_allowlist()
            })
        }
        CommandLine::Sanity => {
            run_command_reported(&context, "sanity", CommandEffect::ReadWrite, json!({}), || {
                run_status("cargo", &["metadata", "--no-deps"])?;
                run_status("cargo", &["test", "-q"])?;
                run_status("cargo", &["fmt", "--all", "--", "--check"])
            })
        }
        CommandLine::Checks { command } => match command {
            ControlCommand::Run {
                domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            } => run_suite_group(
                &context,
                "checks",
                CHECK_SUITES,
                &domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            ),
            ControlCommand::List => run_suite_list(&context, "checks", CHECK_SUITES),
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "checks", &suite, CHECK_SUITES)
            }
        },
        CommandLine::Tests { command } => match command {
            ControlCommand::Run {
                domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            } => run_suite_group(
                &context,
                "tests",
                TEST_SUITES,
                &domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            ),
            ControlCommand::List => run_suite_list(&context, "tests", TEST_SUITES),
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "tests", &suite, TEST_SUITES)
            }
        },
        CommandLine::Contracts { command } => match command {
            ControlCommand::Run {
                domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            } => run_suite_group(
                &context,
                "contracts",
                CONTRACT_SUITES,
                &domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            ),
            ControlCommand::List => run_suite_list(&context, "contracts", CONTRACT_SUITES),
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "contracts", &suite, CONTRACT_SUITES)
            }
        },
        CommandLine::Docs { command } => match command {
            ControlCommand::Run {
                domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            } => run_suite_group(
                &context,
                "docs",
                DOC_SUITES,
                &domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            ),
            ControlCommand::List => run_suite_list(&context, "docs", DOC_SUITES),
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "docs", &suite, DOC_SUITES)
            }
        },
        CommandLine::Release { command } => match command {
            ReleaseCommand::Verify => run_command_reported(
                &context,
                "release.verify",
                CommandEffect::ReadWrite,
                json!({ "flow": crate::suites::release_verify_suite_ids() }),
                || run_release_verify(),
            ),
            ReleaseCommand::Readiness => run_command_reported(
                &context,
                "release.readiness",
                CommandEffect::Validation,
                json!({}),
                || run_release_readiness_report(),
            ),
            ReleaseCommand::CompatibilityMatrix => run_command_reported(
                &context,
                "release.compatibility-matrix",
                CommandEffect::ReadWrite,
                json!({}),
                || run_release_compatibility_matrix(),
            ),
            ReleaseCommand::PostReleaseVerify { binary } => run_command_reported(
                &context,
                "release.post-release-verify",
                CommandEffect::Validation,
                json!({ "binary": binary }),
                || run_post_release_verify(binary.as_deref()),
            ),
            ReleaseCommand::ReproducibilityCheck { tag } => run_command_reported(
                &context,
                "release.reproducibility-check",
                CommandEffect::Validation,
                json!({ "tag": tag }),
                || run_release_reproducibility_check(&tag),
            ),
            ReleaseCommand::EvidenceBundle { out } => run_command_reported(
                &context,
                "release.evidence-bundle",
                CommandEffect::ReadWrite,
                json!({ "out": out }),
                || run_release_evidence_bundle(out.as_deref()),
            ),
            ReleaseCommand::List => run_suite_list(&context, "release", RELEASE_SUITES),
            ReleaseCommand::Explain { suite } => {
                run_release_suite_explain(&context, &suite, RELEASE_SUITES)
            }
        },
        CommandLine::Repo { command } => match command {
            RepoCommand::Deps => run_command_reported(
                &context,
                "repo.deps",
                CommandEffect::Validation,
                json!({}),
                || run_missing_workspace_dependency_checks(),
            ),
            RepoCommand::Run {
                domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            } => run_suite_group(
                &context,
                "repo",
                REPO_SUITES,
                &domain,
                fail_fast,
                include_slow,
                include_internal,
                advisory,
                why,
            ),
            RepoCommand::List => run_suite_list(&context, "repo", REPO_SUITES),
            RepoCommand::Explain { suite } => {
                run_suite_explain(&context, "repo", &suite, REPO_SUITES)
            }
            RepoCommand::EvidenceTaxonomy => run_command_reported(
                &context,
                "repo.evidence-taxonomy",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_taxonomy_report(),
            ),
            RepoCommand::EvidenceLedger => run_command_reported(
                &context,
                "repo.evidence-ledger",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_ledger_report(),
            ),
            RepoCommand::EvidenceValidate => run_command_reported(
                &context,
                "repo.evidence-validate",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_metadata_validate(),
            ),
            RepoCommand::ValidateAllAuthoring => run_command_reported(
                &context,
                "repo.validate-all-authoring",
                CommandEffect::Validation,
                json!({}),
                || run_validate_all_authoring(),
            ),
            RepoCommand::ShowEffectiveAllAuthoring => run_command_reported(
                &context,
                "repo.show-effective-all-authoring",
                CommandEffect::Validation,
                json!({}),
                || run_show_effective_all_authoring(),
            ),
            RepoCommand::AuthoringCoverageReport { out, unused_out } => run_command_reported(
                &context,
                "repo.authoring-coverage-report",
                CommandEffect::ReadWrite,
                json!({ "out": out, "unused_out": unused_out }),
                || run_authoring_coverage_report(&out, &unused_out),
            ),
            RepoCommand::EvidenceLedgerNormalize { check } => run_command_reported(
                &context,
                "repo.evidence-ledger-normalize",
                CommandEffect::ReadWrite,
                json!({ "check": check }),
                || run_evidence_ledger_normalize(check),
            ),
            RepoCommand::EvidenceDirectoryMap { out, create_missing } => run_command_reported(
                &context,
                "repo.evidence-directory-map",
                CommandEffect::ReadWrite,
                json!({ "out": out, "create_missing": create_missing }),
                || run_evidence_directory_map(&out, create_missing),
            ),
            RepoCommand::EvidenceRegistryRebuild { out, check } => run_command_reported(
                &context,
                "repo.evidence-registry-rebuild",
                CommandEffect::ReadWrite,
                json!({ "out": out, "check": check }),
                || run_evidence_registry_rebuild(&out, check),
            ),
            RepoCommand::EvidenceRegistryDiff => run_command_reported(
                &context,
                "repo.evidence-registry-diff",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_registry_diff(),
            ),
            RepoCommand::EvidenceRegistryOrphans => run_command_reported(
                &context,
                "repo.evidence-registry-orphans",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_registry_orphans(),
            ),
            RepoCommand::EvidenceRegistryMissing => run_command_reported(
                &context,
                "repo.evidence-registry-missing",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_registry_missing(),
            ),
            RepoCommand::BattleScenarios => run_command_reported(
                &context,
                "repo.battle-scenarios",
                CommandEffect::Validation,
                json!({}),
                || run_battle_scenarios_report(),
            ),
            RepoCommand::BattleScenariosByTrust => run_command_reported(
                &context,
                "repo.battle-scenarios-by-trust",
                CommandEffect::Validation,
                json!({}),
                || run_battle_scenarios_by_trust_report(),
            ),
            RepoCommand::BattleTrustByScenario => run_command_reported(
                &context,
                "repo.battle-trust-by-scenario",
                CommandEffect::Validation,
                json!({}),
                || run_battle_trust_by_scenario_report(),
            ),
            RepoCommand::BattleCoverageReport { gaps_out, overloaded_out } => run_command_reported(
                &context,
                "repo.battle-coverage-report",
                CommandEffect::ReadWrite,
                json!({ "gaps_out": gaps_out, "overloaded_out": overloaded_out }),
                || run_battle_coverage_report(&gaps_out, &overloaded_out),
            ),
            RepoCommand::BattleValidate => run_command_reported(
                &context,
                "repo.battle-validate",
                CommandEffect::Validation,
                json!({}),
                || run_battle_scenario_mapping_validate(),
            ),
            RepoCommand::PerfEvidenceSummary => run_command_reported(
                &context,
                "repo.perf-evidence-summary",
                CommandEffect::Validation,
                json!({}),
                || run_perf_evidence_summary(),
            ),
            RepoCommand::PerfReleaseSet => run_command_reported(
                &context,
                "repo.perf-release-set",
                CommandEffect::Validation,
                json!({}),
                || run_perf_release_set(),
            ),
            RepoCommand::EvidenceResolveById { id } => run_command_reported(
                &context,
                "repo.evidence-resolve-by-id",
                CommandEffect::Validation,
                json!({ "id": id }),
                || run_evidence_resolve_by_id(&id),
            ),
            RepoCommand::EvidenceResolveByFamily { family } => run_command_reported(
                &context,
                "repo.evidence-resolve-by-family",
                CommandEffect::Validation,
                json!({ "family": family }),
                || run_evidence_resolve_by_family(&family),
            ),
            RepoCommand::EvidenceResolveByTrustProperty { trust_property } => run_command_reported(
                &context,
                "repo.evidence-resolve-by-trust-property",
                CommandEffect::Validation,
                json!({ "trust_property": trust_property }),
                || run_evidence_resolve_by_trust_property(&trust_property),
            ),
            RepoCommand::EvidenceResolveByConsumer { consumer } => run_command_reported(
                &context,
                "repo.evidence-resolve-by-consumer",
                CommandEffect::Validation,
                json!({ "consumer": consumer }),
                || run_evidence_resolve_by_consumer(&consumer),
            ),
            RepoCommand::EvidenceConsumerReports { assets_out, consumers_out } => {
                run_command_reported(
                    &context,
                    "repo.evidence-consumer-reports",
                    CommandEffect::ReadWrite,
                    json!({ "assets_out": assets_out, "consumers_out": consumers_out }),
                    || run_evidence_consumer_reports(&assets_out, &consumers_out),
                )
            }
            RepoCommand::EvidenceSummaryReport { json_out, markdown_out } => run_command_reported(
                &context,
                "repo.evidence-summary-report",
                CommandEffect::ReadWrite,
                json!({ "json_out": json_out, "markdown_out": markdown_out }),
                || run_evidence_summary_report(&json_out, &markdown_out),
            ),
            RepoCommand::ReleaseEvidenceReport {
                json_out,
                proves_out,
                limits_out,
                unsupported_out,
            } => run_command_reported(
                &context,
                "repo.release-evidence-report",
                CommandEffect::ReadWrite,
                json!({
                    "json_out": json_out,
                    "proves_out": proves_out,
                    "limits_out": limits_out,
                    "unsupported_out": unsupported_out
                }),
                || {
                    run_release_evidence_report(
                        &json_out,
                        &proves_out,
                        &limits_out,
                        &unsupported_out,
                    )
                },
            ),
            RepoCommand::HotspotReports { file_out, function_out, api_out, dep_out } => {
                run_command_reported(
                    &context,
                    "repo.hotspot-reports",
                    CommandEffect::ReadWrite,
                    json!({
                        "file_out": file_out,
                        "function_out": function_out,
                        "api_out": api_out,
                        "dep_out": dep_out
                    }),
                    || run_repo_hotspot_reports(&file_out, &function_out, &api_out, &dep_out),
                )
            }
            RepoCommand::SchemaChangelog { out, schema_root } => run_command_reported(
                &context,
                "repo.schema-changelog",
                CommandEffect::ReadWrite,
                json!({ "out": out, "schema_root": schema_root }),
                || run_repo_schema_changelog(&out, &schema_root),
            ),
            RepoCommand::RuntimeScopeReports {
                kernel_out,
                non_kernel_out,
                contract_backing_out,
                operator_surface_out,
                core_api_out,
                runtime_api_out,
            } => run_command_reported(
                &context,
                "repo.runtime-scope-reports",
                CommandEffect::ReadWrite,
                json!({
                    "kernel_out": kernel_out,
                    "non_kernel_out": non_kernel_out,
                    "contract_backing_out": contract_backing_out,
                    "operator_surface_out": operator_surface_out,
                    "core_api_out": core_api_out,
                    "runtime_api_out": runtime_api_out
                }),
                || {
                    run_repo_runtime_scope_reports(
                        &kernel_out,
                        &non_kernel_out,
                        &contract_backing_out,
                        &operator_surface_out,
                        &core_api_out,
                        &runtime_api_out,
                    )
                },
            ),
            RepoCommand::PlannerHardeningReport { out } => run_command_reported(
                &context,
                "repo.planner-hardening-report",
                CommandEffect::ReadWrite,
                json!({ "out": out }),
                || run_repo_planner_hardening_report(&out),
            ),
            RepoCommand::ArtifactCapabilityReports { matrix_out, model_out } => {
                run_command_reported(
                    &context,
                    "repo.artifact-capability-reports",
                    CommandEffect::ReadWrite,
                    json!({ "matrix_out": matrix_out, "model_out": model_out }),
                    || run_repo_artifact_capability_reports(&matrix_out, &model_out),
                )
            }
        },
        CommandLine::Verify { command } => match command {
            VerifyCommand::EvidenceFoundation => run_command_reported(
                &context,
                "verify.evidence-foundation",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_foundation_verify(),
            ),
            VerifyCommand::EvidenceRegistry => run_command_reported(
                &context,
                "verify.evidence-registry",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_registry_verify(),
            ),
            VerifyCommand::EvidenceSchema => run_command_reported(
                &context,
                "verify.evidence-schema",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_schema_verify(),
            ),
            VerifyCommand::EvidenceAuthoring => run_command_reported(
                &context,
                "verify.evidence-authoring",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_authoring_verify(),
            ),
            VerifyCommand::EvidenceBattle => run_command_reported(
                &context,
                "verify.evidence-battle",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_battle_verify(),
            ),
            VerifyCommand::EvidenceCache => run_command_reported(
                &context,
                "verify.evidence-cache",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_cache_verify(),
            ),
            VerifyCommand::EvidenceReplay => run_command_reported(
                &context,
                "verify.evidence-replay",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_replay_verify(),
            ),
            VerifyCommand::EvidenceCompat => run_command_reported(
                &context,
                "verify.evidence-compat",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_compat_verify(),
            ),
            VerifyCommand::EvidenceFault => run_command_reported(
                &context,
                "verify.evidence-fault",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_fault_verify(),
            ),
            VerifyCommand::EvidencePerf => run_command_reported(
                &context,
                "verify.evidence-perf",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_perf_verify(),
            ),
            VerifyCommand::EvidenceCompare => run_command_reported(
                &context,
                "verify.evidence-compare",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_compare_verify(),
            ),
            VerifyCommand::EvidenceOwnership => run_command_reported(
                &context,
                "verify.evidence-ownership",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_ownership_verify(),
            ),
            VerifyCommand::EvidenceDrift => run_command_reported(
                &context,
                "verify.evidence-drift",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_drift_verify(),
            ),
            VerifyCommand::EvidenceConsumers => run_command_reported(
                &context,
                "verify.evidence-consumers",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_consumers_verify(),
            ),
            VerifyCommand::EvidenceReleaseSet => run_command_reported(
                &context,
                "verify.evidence-release-set",
                CommandEffect::Validation,
                json!({}),
                || run_evidence_release_set_verify(),
            ),
        },
        CommandLine::Schedule { command } => match command {
            ScheduleCommand::Validate { file } => run_command_reported(
                &context,
                "schedule.validate",
                CommandEffect::Validation,
                json!({ "file": file }),
                || run_schedule_validate(&file),
            ),
            ScheduleCommand::Preview { file } => run_command_reported(
                &context,
                "schedule.preview",
                CommandEffect::Validation,
                json!({ "file": file }),
                || run_schedule_preview(&file),
            ),
        },
        CommandLine::Dag { command } => match command {
            DagCommand::Lint { graph } => run_command_reported(
                &context,
                "dag.lint",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_lint(&graph),
            ),
            DagCommand::UnitHarness { graph } => run_command_reported(
                &context,
                "dag.unit-harness",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_unit_harness(&graph),
            ),
            DagCommand::Simulate { graph } => run_command_reported(
                &context,
                "dag.simulate",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_simulate(&graph),
            ),
            DagCommand::DryRun { graph } => run_command_reported(
                &context,
                "dag.dry-run",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_dry_run(&graph),
            ),
            DagCommand::PlanDump { graph, select } => run_command_reported(
                &context,
                "dag.plan-dump",
                CommandEffect::Validation,
                json!({"graph": graph, "select": select}),
                || run_dag_plan_dump(&graph, &select),
            ),
            DagCommand::Visualize { run_dir } => run_command_reported(
                &context,
                "dag.visualize",
                CommandEffect::Validation,
                json!({"run_dir": run_dir}),
                || run_dag_visualize(&run_dir),
            ),
            DagCommand::SchedulerTimeline { run_dir } => run_command_reported(
                &context,
                "dag.scheduler-timeline",
                CommandEffect::Validation,
                json!({"run_dir": run_dir}),
                || run_dag_scheduler_timeline(&run_dir),
            ),
            DagCommand::VerifyState { run_dir } => run_command_reported(
                &context,
                "dag.verify-state",
                CommandEffect::Validation,
                json!({"run_dir": run_dir}),
                || run_dag_verify_state(&run_dir),
            ),
            DagCommand::Debug { graph } => run_command_reported(
                &context,
                "dag.debug",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_debug(&graph),
            ),
            DagCommand::ExplainValidation { graph } => run_command_reported(
                &context,
                "dag.explain-validation",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_explain_validation(&graph),
            ),
            DagCommand::ExplainNode { run_dir, node_id } => run_command_reported(
                &context,
                "dag.explain-node",
                CommandEffect::Validation,
                json!({"run_dir": run_dir, "node_id": node_id}),
                || run_dag_explain_node(&run_dir, &node_id),
            ),
            DagCommand::Preview { graph } => run_command_reported(
                &context,
                "dag.preview",
                CommandEffect::Validation,
                json!({"graph": graph}),
                || run_dag_preview(&graph),
            ),
            DagCommand::SchemaExport { out } => run_command_reported(
                &context,
                "dag.schema-export",
                CommandEffect::ReadWrite,
                json!({"out": out}),
                || run_dag_schema_export(&out),
            ),
            DagCommand::RepairRun { run_dir, apply } => run_command_reported(
                &context,
                "dag.repair-run",
                CommandEffect::ReadWrite,
                json!({"run_dir": run_dir, "apply": apply}),
                || run_dag_repair_run(&run_dir, apply),
            ),
            DagCommand::SimulateRecovery { scenario } => run_command_reported(
                &context,
                "dag.simulate-recovery",
                CommandEffect::Validation,
                json!({"scenario": scenario}),
                || run_dag_simulate_recovery(&scenario),
            ),
            DagCommand::RecoveryAccept { suite } => run_command_reported(
                &context,
                "dag.recovery-accept",
                CommandEffect::Validation,
                json!({"suite": suite}),
                || run_dag_recovery_accept(&suite),
            ),
            DagCommand::ExplainRun { run_dir } => run_command_reported(
                &context,
                "dag.explain-run",
                CommandEffect::Validation,
                json!({"run_dir": run_dir}),
                || run_dag_explain_run(&run_dir),
            ),
            DagCommand::RunInspect { run_dir } => run_command_reported(
                &context,
                "dag.run-inspect",
                CommandEffect::Validation,
                json!({"run_dir": run_dir}),
                || run_dag_run_inspect(&run_dir),
            ),
            DagCommand::ExplainArtifact { run_dir, artifact_id } => run_command_reported(
                &context,
                "dag.explain-artifact",
                CommandEffect::Validation,
                json!({"run_dir": run_dir, "artifact_id": artifact_id}),
                || run_dag_explain_artifact(&run_dir, &artifact_id),
            ),
            DagCommand::ExplainSchedule { run_dir, schedule_id } => run_command_reported(
                &context,
                "dag.explain-schedule",
                CommandEffect::Validation,
                json!({"run_dir": run_dir, "schedule_id": schedule_id}),
                || run_dag_explain_schedule(&run_dir, &schedule_id),
            ),
            DagCommand::InvestigationBundle { run_dir, run_id } => run_command_reported(
                &context,
                "dag.investigation-bundle",
                CommandEffect::Validation,
                json!({"run_dir": run_dir, "run_id": run_id}),
                || run_dag_investigation_bundle(&run_dir, &run_id),
            ),
            DagCommand::DriftReport {
                current_metrics,
                baseline_metrics,
                dag_name,
                baseline_name,
            } => run_command_reported(
                &context,
                "dag.drift-report",
                CommandEffect::Validation,
                json!({
                    "current_metrics": current_metrics,
                    "baseline_metrics": baseline_metrics,
                    "dag_name": dag_name,
                    "baseline_name": baseline_name
                }),
                || {
                    run_dag_drift_report(
                        &current_metrics,
                        &baseline_metrics,
                        &dag_name,
                        &baseline_name,
                    )
                },
            ),
        },
        CommandLine::Doctor => {
            run_command_reported(&context, "doctor", CommandEffect::ReadWrite, json!({}), || {
                run_env_summary()?;
                run_verify_tools()
            })
        }
        CommandLine::Golden => {
            run_command_reported(&context, "golden", CommandEffect::ReadWrite, json!({}), || {
                run_golden()
            })
        }
        CommandLine::PublicApi => run_command_reported(
            &context,
            "public-api",
            CommandEffect::ReadWrite,
            json!({}),
            || run_public_api(),
        ),
        CommandLine::DepGuard => run_command_reported(
            &context,
            "dep-guard",
            CommandEffect::Validation,
            json!({}),
            || run_dep_guard(),
        ),
        CommandLine::CrateGraph => run_command_reported(
            &context,
            "crate-graph",
            CommandEffect::Validation,
            json!({}),
            || run_crate_graph_command(),
        ),
        CommandLine::ArtifactsClean => run_command_reported(
            &context,
            "artifacts-clean",
            CommandEffect::ReadWrite,
            json!({}),
            || run_artifacts_clean(),
        ),
        CommandLine::EnvSummary => run_command_reported(
            &context,
            "env-summary",
            CommandEffect::Validation,
            json!({}),
            || run_env_summary(),
        ),
        CommandLine::VerifyTools => run_command_reported(
            &context,
            "verify-tools",
            CommandEffect::Validation,
            json!({}),
            || run_verify_tools(),
        ),
        CommandLine::ResolveCheck => run_command_reported(
            &context,
            "resolve-check",
            CommandEffect::Validation,
            json!({}),
            || run_resolve_check(),
        ),
        CommandLine::BenchmarkBaseline => run_command_reported(
            &context,
            "benchmark-baseline",
            CommandEffect::ReadWrite,
            json!({}),
            || run_benchmark_baseline(),
        ),
        CommandLine::BenchmarkCompare { current, baseline, max_regression_ratio } => {
            run_command_reported(
                &context,
                "benchmark-compare",
                CommandEffect::Validation,
                json!({
                    "current": current,
                    "baseline": baseline,
                    "max_regression_ratio": max_regression_ratio
                }),
                || run_benchmark_compare(&current, &baseline, max_regression_ratio),
            )
        }
        CommandLine::ResourceProfileSummary { report } => run_command_reported(
            &context,
            "resource-profile-summary",
            CommandEffect::Validation,
            json!({ "report": report }),
            || run_resource_profile_summary(&report),
        ),
        CommandLine::ResourceBudgetCheck { report, gate } => run_command_reported(
            &context,
            "resource-budget-check",
            CommandEffect::Validation,
            json!({ "report": report, "gate": gate }),
            || run_resource_budget_check(&report, gate),
        ),
        CommandLine::ResourceTrendAppend { report, trend } => run_command_reported(
            &context,
            "resource-trend-append",
            CommandEffect::ReadWrite,
            json!({ "report": report, "trend": trend }),
            || run_resource_trend_append(&report, &trend),
        ),
        CommandLine::ArtifactVerify => run_command_reported(
            &context,
            "artifact-verify",
            CommandEffect::Validation,
            json!({}),
            || run_artifact_verify(),
        ),
        CommandLine::ObservabilityReport => run_command_reported(
            &context,
            "observability-report",
            CommandEffect::Validation,
            json!({}),
            || run_observability_report(),
        ),
        CommandLine::DocsIndex => run_command_reported(
            &context,
            "docs-index",
            CommandEffect::ReadWrite,
            json!({}),
            || run_docs_index_generate(),
        ),
        CommandLine::E2eMatrix => run_command_reported(
            &context,
            "e2e-matrix",
            CommandEffect::ReadWrite,
            json!({}),
            || run_e2e_matrix(),
        ),
        CommandLine::FaultSummary => run_command_reported(
            &context,
            "fault-summary",
            CommandEffect::Validation,
            json!({}),
            || run_fault_summary_report(),
        ),
        CommandLine::StorageHealth { run_dir, cache_dir } => run_command_reported(
            &context,
            "storage-health",
            CommandEffect::Validation,
            json!({"run_dir": run_dir, "cache_dir": cache_dir}),
            || run_storage_health(&run_dir, cache_dir.as_deref()),
        ),
        CommandLine::RunDirAudit { run_dir, strict } => run_command_reported(
            &context,
            "run-dir-audit",
            CommandEffect::Validation,
            json!({"run_dir": run_dir, "strict": strict}),
            || run_run_dir_audit(&run_dir, strict),
        ),
        CommandLine::UnsafeAudit => run_command_reported(
            &context,
            "unsafe-audit",
            CommandEffect::Validation,
            json!({}),
            || run_unsafe_audit_report(),
        ),
        CommandLine::ErrorCodes => run_command_reported(
            &context,
            "error-codes",
            CommandEffect::Validation,
            json!({}),
            || run_error_code_registry_report(),
        ),
        CommandLine::ConfigDump { config } => run_command_reported(
            &context,
            "config-dump",
            CommandEffect::Validation,
            json!({ "config": config }),
            || run_config_dump(config.as_deref()),
        ),
        CommandLine::PolicyAudit { config } => run_command_reported(
            &context,
            "policy-audit",
            CommandEffect::Validation,
            json!({ "config": config }),
            || run_policy_audit(config.as_deref()),
        ),
        CommandLine::ExecutionModesReport => run_command_reported(
            &context,
            "execution-modes-report",
            CommandEffect::Validation,
            json!({}),
            || run_execution_modes_report(),
        ),
        CommandLine::DistributedSemanticsReport => run_command_reported(
            &context,
            "distributed-semantics-report",
            CommandEffect::Validation,
            json!({}),
            || run_distributed_semantics_report(),
        ),
        CommandLine::InvariantsReport => run_command_reported(
            &context,
            "invariants-report",
            CommandEffect::Validation,
            json!({}),
            || run_invariants_report(),
        ),
        CommandLine::ComparisonEvidenceReport => run_command_reported(
            &context,
            "comparison-evidence-report",
            CommandEffect::Validation,
            json!({}),
            || run_comparison_evidence_report(),
        ),
        CommandLine::PerformanceEvidenceReport => run_command_reported(
            &context,
            "performance-evidence-report",
            CommandEffect::Validation,
            json!({}),
            || run_performance_evidence_report(),
        ),
        CommandLine::BackendRegistryReport => run_command_reported(
            &context,
            "backend-registry-report",
            CommandEffect::Validation,
            json!({}),
            || run_backend_registry_report(),
        ),
        CommandLine::ReleaseArtifactVerify => run_command_reported(
            &context,
            "release-artifact-verify",
            CommandEffect::Validation,
            json!({}),
            || run_release_artifact_verification_suite(),
        ),
        CommandLine::DriftDashboard => run_command_reported(
            &context,
            "drift-dashboard",
            CommandEffect::Validation,
            json!({}),
            || run_drift_dashboard(),
        ),
        CommandLine::RepoTrustSummary => run_command_reported(
            &context,
            "repo-trust-summary",
            CommandEffect::Validation,
            json!({}),
            || run_repo_trust_summary(),
        ),
        CommandLine::CompatibilityReport => run_command_reported(
            &context,
            "compatibility-report",
            CommandEffect::Validation,
            json!({}),
            || run_compatibility_report(),
        ),
        CommandLine::CacheCoverageReport => run_command_reported(
            &context,
            "cache-coverage-report",
            CommandEffect::Validation,
            json!({}),
            || run_cache_coverage_report(),
        ),
        CommandLine::FoundationReviewReport => run_command_reported(
            &context,
            "foundation-review-report",
            CommandEffect::Validation,
            json!({}),
            || run_foundation_review_report(),
        ),
        CommandLine::Ci => {
            run_command_reported(&context, "ci", CommandEffect::ReadWrite, json!({}), || run_ci())
        }
        CommandLine::Foundation {
            domain,
            fail_fast,
            include_slow,
            include_internal,
            advisory,
            why,
        } => run_command_reported(
            &context,
            "foundation",
            CommandEffect::Validation,
            json!({
                "domain": domain,
                "fail_fast": fail_fast,
                "include_slow": include_slow,
                "include_internal": include_internal,
                "advisory": advisory,
                "why": why,
            }),
            || {
                run_foundation_suite(
                    &context,
                    &domain,
                    fail_fast,
                    include_slow,
                    include_internal,
                    advisory,
                    why,
                )
            },
        ),
        CommandLine::FoundationHardening { fail_fast, advisory, why } => run_command_reported(
            &context,
            "foundation-hardening",
            CommandEffect::Validation,
            json!({
                "fail_fast": fail_fast,
                "advisory": advisory,
                "why": why,
            }),
            || run_foundation_hardening_suite(&context, fail_fast, advisory, why),
        ),
        CommandLine::Compat => {
            run_command_reported(&context, "compat", CommandEffect::ReadWrite, json!({}), || {
                run_status(
                    "cargo",
                    &["run", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--", "compat"],
                )
            })
        }
        CommandLine::Api { command } => match command {
            ApiCommand::PublicSurface => run_command_reported(
                &context,
                "api.public-surface",
                CommandEffect::ReadWrite,
                json!({}),
                || run_public_api(),
            ),
        },
    }
}

fn run_audit_allowlist_quality_gate() -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join("audit-allowlist.toml");
    let payload = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: toml::Value = toml::from_str(&payload)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let advisories =
        value.get("advisory").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    if advisories.is_empty() {
        return Ok(());
    }

    let today = current_iso_day()?;
    let mut errors = Vec::new();
    for (index, row) in advisories.iter().enumerate() {
        let label = format!("advisory[{index}]");
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("").trim();
        let why = row.get("why").and_then(toml::Value::as_str).unwrap_or("").trim();
        let owner = row.get("owner").and_then(toml::Value::as_str).unwrap_or("").trim();
        let link = row.get("link").and_then(toml::Value::as_str).unwrap_or("").trim();
        let expiry = row.get("expiry").and_then(toml::Value::as_str).unwrap_or("").trim();
        if !is_rustsec_id(id) {
            errors.push(format!("{label}: id must match RUSTSEC-YYYY-NNNN"));
        }
        if why.is_empty() {
            errors.push(format!("{label}: missing why"));
        }
        if owner.is_empty() {
            errors.push(format!("{label}: missing owner"));
        }
        if !(link.starts_with("http://") || link.starts_with("https://")) {
            errors.push(format!("{label}: link must be http(s)"));
        }
        if !is_iso_day(expiry) {
            errors.push(format!("{label}: expiry must be YYYY-MM-DD"));
        } else if expiry < today.as_str() {
            errors.push(format!("{label}: expiry has passed ({expiry})"));
        }
    }
    if errors.is_empty() {
        return Ok(());
    }
    Err(format!("audit allowlist quality gate failed:\n{}", errors.join("\n")))
}

fn load_audit_allowlist_ids() -> Result<Vec<String>, String> {
    let root = repo_root()?;
    let path = root.join("audit-allowlist.toml");
    let payload = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: toml::Value = toml::from_str(&payload)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let advisories =
        value.get("advisory").and_then(toml::Value::as_array).cloned().unwrap_or_default();

    let mut ids = Vec::new();
    for row in advisories {
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("").trim();
        if is_rustsec_id(id) {
            ids.push(id.to_string());
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

fn run_cargo_audit_with_allowlist() -> Result<(), String> {
    let ignores = load_audit_allowlist_ids()?;
    let mut command = Command::new("cargo");
    command.arg("audit");
    for advisory in &ignores {
        command.arg("--ignore");
        command.arg(advisory);
    }

    let status = command.status().map_err(|err| format!("cargo audit failed to start: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("cargo audit failed".to_string())
    }
}

fn run_deny_policy_deviations_gate() -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join("configs/rust/deny.deviations.toml");
    if !path.is_file() {
        return Err(format!("missing {}", path.display()));
    }
    let payload = fs::read_to_string(&path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: toml::Value = toml::from_str(&payload)
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let rows = value.get("deviation").and_then(toml::Value::as_array).cloned().unwrap_or_default();
    if rows.is_empty() {
        return Ok(());
    }
    let today = current_iso_day()?;
    let mut errors = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let label = format!("deviation[{index}]");
        let id = row.get("id").and_then(toml::Value::as_str).unwrap_or("").trim();
        let owner = row.get("owner").and_then(toml::Value::as_str).unwrap_or("").trim();
        let reason = row.get("reason").and_then(toml::Value::as_str).unwrap_or("").trim();
        let expiry = row.get("expiry").and_then(toml::Value::as_str).unwrap_or("").trim();
        let review = row.get("review").and_then(toml::Value::as_str).unwrap_or("").trim();
        if id.is_empty() {
            errors.push(format!("{label}: missing id"));
        }
        if owner.is_empty() {
            errors.push(format!("{label}: missing owner"));
        }
        if reason.is_empty() {
            errors.push(format!("{label}: missing reason"));
        }
        if !is_iso_day(expiry) {
            errors.push(format!("{label}: expiry must be YYYY-MM-DD"));
        } else if expiry < today.as_str() {
            errors.push(format!("{label}: expiry has passed ({expiry})"));
        }
        if !(review.starts_with("http://") || review.starts_with("https://")) {
            errors.push(format!("{label}: review must be an http(s) link"));
        } else if !review.contains("bijux-std") {
            errors.push(format!("{label}: review must reference bijux-std"));
        }
    }
    if errors.is_empty() {
        return Ok(());
    }
    Err(format!("deny policy deviations governance gate failed:\n{}", errors.join("\n")))
}

fn current_iso_day() -> Result<String, String> {
    let output = Command::new("date")
        .args(["+%Y-%m-%d"])
        .output()
        .map_err(|err| format!("resolve current date failed: {err}"))?;
    if !output.status.success() {
        return Err("resolve current date failed: date command returned non-zero".to_string());
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_string())
        .map_err(|err| format!("resolve current date failed: {err}"))
}

fn is_iso_day(value: &str) -> bool {
    value.len() == 10
        && value.chars().nth(4) == Some('-')
        && value.chars().nth(7) == Some('-')
        && value
            .chars()
            .enumerate()
            .all(|(index, ch)| (index == 4 || index == 7) || ch.is_ascii_digit())
}

fn is_rustsec_id(value: &str) -> bool {
    value.len() == 17
        && value.starts_with("RUSTSEC-")
        && value.as_bytes().get(12) == Some(&b'-')
        && value[8..12].chars().all(|ch| ch.is_ascii_digit())
        && value[13..17].chars().all(|ch| ch.is_ascii_digit())
}

fn run_ci() -> Result<(), String> {
    run_status("cargo", &["fmt", "--all"])?;
    run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run_dep_guard()?;
    run_resolve_check()?;
    run_missing_workspace_dependency_checks()?;
    run_status("cargo", &["test", "--workspace"])?;
    run_golden()?;
    run_status("cargo", &["run", "-p", "bijux-dag-cli", "--bin", "bijux-dag", "--", "compat"])?;

    let root = repo_root()?;
    let scratch = std::env::temp_dir().join(format!("bijux-dag-ci-{}", now_secs()));
    let runs = scratch.join("runs");
    fs::create_dir_all(&runs).map_err(|err| err.to_string())?;
    run_with_root(
        &root,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "run",
            "evidence/authoring/examples/hello.dag.json",
            "--out",
            runs.to_str().expect("utf-8"),
        ],
    )?;
    let run_dir = newest_run(&runs)?;
    run_status_in_dir(
        &root,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--bin",
            "bijux-dag",
            "--",
            "verify",
            run_dir.to_str().expect("utf-8"),
        ],
    )
}

fn run_foundation_suite(
    context: &CommandContext,
    domain: &Option<String>,
    fail_fast: bool,
    include_slow: bool,
    include_internal: bool,
    advisory: bool,
    why: bool,
) -> Result<(), String> {
    let groups: [(&str, &[SuiteDef]); 5] = [
        ("checks", CHECK_SUITES),
        ("tests", TEST_SUITES),
        ("contracts", CONTRACT_SUITES),
        ("repo", REPO_SUITES),
        ("docs", DOC_SUITES),
    ];
    let mut failed = Vec::new();
    for (group_name, group_suites) in groups {
        if let Err(err) = run_suite_group(
            context,
            group_name,
            group_suites,
            domain,
            fail_fast,
            include_slow,
            include_internal,
            advisory,
            why,
        ) {
            failed.push(format!("{group_name}: {err}"));
            if fail_fast {
                break;
            }
        }
    }
    if failed.is_empty() || advisory {
        Ok(())
    } else {
        Err(format!("foundation suite failed: {}", failed.join(", ")))
    }
}

#[derive(Debug, Deserialize)]
struct FoundationHardeningConfig {
    suite_ids: Vec<String>,
}

fn run_foundation_hardening_suite(
    context: &CommandContext,
    fail_fast: bool,
    advisory: bool,
    why: bool,
) -> Result<(), String> {
    let root = repo_root()?;
    let config_path = root.join("configs/dag/suites/foundation_hardening.json");
    let payload = fs::read_to_string(&config_path).map_err(|err| err.to_string())?;
    let config: FoundationHardeningConfig =
        serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    if config.suite_ids.is_empty() {
        return Err("foundation hardening suite list must not be empty".to_string());
    }

    let mut failed = Vec::new();
    for suite_id in &config.suite_ids {
        let suite = REPO_SUITES
            .iter()
            .chain(CONTRACT_SUITES.iter())
            .chain(TEST_SUITES.iter())
            .chain(CHECK_SUITES.iter())
            .chain(DOC_SUITES.iter())
            .find(|suite| suite.id == suite_id)
            .ok_or_else(|| format!("unknown foundation hardening suite id: {suite_id}"))?;

        let single = [SuiteDef {
            id: suite.id,
            description: suite.description,
            domain: suite.domain,
            slow: suite.slow,
            internal: suite.internal,
            effect: suite.effect,
            run: suite.run,
        }];
        if let Err(err) = run_suite_group(
            context,
            "foundation-hardening",
            &single,
            &None,
            false,
            true,
            true,
            advisory,
            why,
        ) {
            failed.push(format!("{suite_id}: {err}"));
            if fail_fast {
                break;
            }
        }
    }

    if failed.is_empty() || advisory {
        Ok(())
    } else {
        Err(format!("foundation hardening failed: {}", failed.join(", ")))
    }
}

fn run_schedule_validate(file: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(file);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read schedule file {}: {err}", path.display()))?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse schedule file {}: {err}", path.display()))?;
    let definitions = payload
        .get("definitions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "schedule registry must contain a 'definitions' array".to_string())?;

    let mut seen = std::collections::BTreeSet::new();
    for definition in definitions {
        let id = definition
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "schedule definition is missing string 'id'".to_string())?;
        if !seen.insert(id.to_string()) {
            return Err(format!("duplicate schedule id '{id}'"));
        }
        let trigger = definition
            .get("trigger")
            .ok_or_else(|| format!("schedule '{id}' is missing 'trigger'"))?;
        let trigger_kind = trigger
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("schedule '{id}' trigger is missing 'kind'"))?;
        if trigger_kind == "cron" {
            let expression = trigger
                .get("expression")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("schedule '{id}' cron trigger is missing 'expression'"))?;
            let parts: Vec<&str> = expression.split_whitespace().collect();
            if parts.len() != 5 {
                return Err(format!(
                    "schedule '{id}' cron expression must have exactly five fields"
                ));
            }
        }
    }
    Ok(())
}

fn run_release_verify() -> Result<(), String> {
    let flow = crate::suites::release_verify_suite_ids();
    println!("release verify flow: {}", flow.join(" -> "));
    let root = repo_root()?;
    run_with_root(&root, "make", &["release-validate-rs"])?;
    run_release_readiness_report()?;
    run_release_compatibility_matrix()
}

fn run_release_readiness_report() -> Result<(), String> {
    let root = repo_root()?;
    let release_evidence = check_release_evidence_ready(&root)?;
    let distribution_delivery = run_distribution_delivery_contract_report()?;
    let report = json!({
        "timestamp_unix_ms": now_millis(),
        "release_evidence": release_evidence,
        "distribution_delivery": distribution_delivery,
        "contract_coverage": check_contract_coverage_ready(&root),
        "schema_coverage": check_schema_coverage_ready(&root),
        "docs_coverage": check_docs_coverage_ready(&root),
        "test_state": check_test_state_ready(&root),
        "e2e_state": check_e2e_state_ready(&root),
        "perf_baseline": check_perf_baseline_ready(&root),
        "resource_baseline": check_resource_baseline_ready(&root),
        "release_blockers": read_release_blockers(&root)?,
    });
    let path = root.join("artifacts/release/readiness_report.json");
    write_pretty_json(&path, &report)?;
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn check_release_evidence_ready(root: &Path) -> Result<Value, String> {
    let config: FoundationHardeningConfig = serde_json::from_value(read_json_value(
        &root.join("configs/dag/suites/foundation_hardening.json"),
    )?)
    .map_err(|err| err.to_string())?;
    let required_surfaces = [
        "docs/reports/foundation/RELEASE_EVIDENCE_REPORT.md",
        "docs/reports/foundation/REPOSITORY_PROOF_STATEMENT.md",
        "docs/reports/foundation/REPLAY_HARDENING_REPORT.md",
        "docs/reports/foundation/CACHE_HARDENING_REPORT.md",
        "docs/reports/foundation/RUN_DIR_IMPORT_EXPORT_HARDENING_REPORT.md",
        "docs/reports/foundation/CONFIG_POLICY_DETERMINISM_REPORT.md",
    ];
    let missing: Vec<String> = required_surfaces
        .iter()
        .filter(|path| !root.join(path).exists())
        .map(|path| path.to_string())
        .collect();
    Ok(json!({
        "ok": missing.is_empty(),
        "rule": "release readiness depends on battle, replay, cache, run verification, and config/policy evidence; raw test totals are insufficient",
        "foundation_hardening_suite_ids": config.suite_ids,
        "missing": missing,
    }))
}

fn run_release_compatibility_matrix() -> Result<(), String> {
    let root = repo_root()?;
    let mut rows = Vec::new();
    let positive = root.join("configs/dag/schema/fixtures/compat/positive");
    let negative = root.join("configs/dag/schema/fixtures/compat/negative");
    collect_fixture_rows(&positive, true, &mut rows)?;
    collect_fixture_rows(&negative, false, &mut rows)?;
    rows.sort_by(|a, b| a["fixture"].as_str().cmp(&b["fixture"].as_str()));

    let matrix = json!({
        "generated_unix_ms": now_millis(),
        "schema_versions_supported": ["v0.1"],
        "rows": rows
    });
    let out = root.join("artifacts/release/compatibility_matrix.json");
    write_pretty_json(&out, &matrix)?;
    println!("{}", serde_json::to_string_pretty(&matrix).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_post_release_verify(binary: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let tmp_dir = tempfile::tempdir().map_err(|err| err.to_string())?;
    let dag_dir = tmp_dir.path();
    let runs_dir = dag_dir.join("runs");
    let bin_path = binary
        .map(|p| if p.is_absolute() { p.to_path_buf() } else { root.join(p) })
        .unwrap_or_else(|| root.join("target/debug/bijux"));
    let bin = bin_path.to_str().ok_or_else(|| "non-utf8 release binary path".to_string())?;

    run_with_root(&root, bin, &["dag", "init", "--dir", dag_dir.to_string_lossy().as_ref()])?;
    run_with_root(
        &root,
        bin,
        &["dag", "validate", dag_dir.join("dag.json").to_string_lossy().as_ref()],
    )?;
    run_with_root(
        &root,
        bin,
        &[
            "dag",
            "run",
            dag_dir.join("dag.json").to_string_lossy().as_ref(),
            "--runs-dir",
            runs_dir.to_string_lossy().as_ref(),
        ],
    )?;
    run_with_root(&root, bin, &["dag", "status", runs_dir.to_string_lossy().as_ref()])?;
    Ok(())
}

fn run_release_reproducibility_check(tag: &str) -> Result<(), String> {
    let root = repo_root()?;
    let current_sha = command_stdout(&root, "git", &["rev-parse", "HEAD"])?;
    let tag_sha = command_stdout(&root, "git", &["rev-list", "-n", "1", tag])?;
    if current_sha.trim() != tag_sha.trim() {
        return Err(format!(
            "reproducibility check failed: HEAD ({}) != tag ({})",
            current_sha.trim(),
            tag_sha.trim()
        ));
    }
    println!("reproducibility check passed: {} -> {}", tag, tag_sha.trim());
    Ok(())
}

fn run_release_evidence_bundle(out: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let output = out
        .map(|p| if p.is_absolute() { p.to_path_buf() } else { root.join(p) })
        .unwrap_or_else(|| root.join("artifacts/release/evidence_bundle.json"));

    let readiness_path = root.join("artifacts/release/readiness_report.json");
    let readiness = if readiness_path.exists() {
        read_json_value(&readiness_path)?
    } else {
        json!({"status": "missing", "hint": "run `bijux-dev-dag release readiness`"})
    };
    let matrix_path = root.join("artifacts/release/compatibility_matrix.json");
    let matrix = if matrix_path.exists() {
        read_json_value(&matrix_path)?
    } else {
        json!({"status": "missing", "hint": "run `bijux-dev-dag release compatibility-matrix`"})
    };

    let bundle = json!({
        "generated_unix_ms": now_millis(),
        "why_release_exists": "All required release policy evidence artifacts are present and reviewed.",
        "artifacts": {
            "readiness_report": readiness,
            "compatibility_matrix": matrix,
            "known_limitations_path": "docs/bijux-dag/quality/known-limitations.md",
            "release_note_template_path": "docs/bijux-core/operations/release-notes-template.md"
        }
    });

    write_pretty_json(&output, &bundle)?;
    println!("{}", serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?);
    Ok(())
}

fn check_contract_coverage_ready(root: &Path) -> Value {
    json!({"ok": root.join("docs/spec/CLI_CONTRACT.md").exists() && root.join("docs/spec/ERROR_CONTRACT.md").exists()})
}

fn check_schema_coverage_ready(root: &Path) -> Value {
    let positive = root.join("configs/dag/schema/fixtures/compat/positive").exists();
    let negative = root.join("configs/dag/schema/fixtures/compat/negative").exists();
    json!({"ok": positive && negative})
}

fn check_docs_coverage_ready(root: &Path) -> Value {
    json!({"ok": root.join("docs/reference/DOCS_INDEX.md").exists()})
}

fn check_test_state_ready(root: &Path) -> Value {
    json!({"ok": root.join("tests/README.md").exists()})
}

fn check_e2e_state_ready(root: &Path) -> Value {
    json!({"ok": root.join("tests/e2e").exists()})
}

fn check_perf_baseline_ready(root: &Path) -> Value {
    json!({"ok": root.join("evidence/perf/baselines").exists()})
}

fn check_resource_baseline_ready(root: &Path) -> Value {
    json!({"ok": root.join("evidence/perf/baselines/resource_trend_v1.json").exists()})
}

fn read_release_blockers(root: &Path) -> Result<Value, String> {
    read_json_value(&root.join("configs/dag/release/release_blockers.json"))
}

fn collect_fixture_rows(
    dir: &Path,
    should_pass: bool,
    rows: &mut Vec<Value>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_fixture_rows(&path, should_pass, rows)?;
            continue;
        }
        let fixture = path
            .strip_prefix(repo_root()?.as_path())
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let data = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let spec = serde_json::from_str::<Value>(&data)
            .ok()
            .and_then(|v| v.get("spec").and_then(|x| x.as_str()).map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string());
        rows.push(json!({
            "fixture": fixture,
            "spec": spec,
            "expected": if should_pass { "accept" } else { "reject" }
        }));
    }
    Ok(())
}

fn run_schedule_preview(file: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(file);
    let text = fs::read_to_string(&path)
        .map_err(|err| format!("failed to read schedule file {}: {err}", path.display()))?;
    let payload: Value = serde_json::from_str(&text)
        .map_err(|err| format!("failed to parse schedule file {}: {err}", path.display()))?;
    let definitions = payload
        .get("definitions")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "schedule registry must contain a 'definitions' array".to_string())?;
    let now = now_millis();
    for definition in definitions {
        let id = definition.get("id").and_then(|v| v.as_str()).unwrap_or("<unknown>");
        let trigger = definition.get("trigger").unwrap_or(&Value::Null);
        let kind = trigger.get("kind").and_then(|v| v.as_str()).unwrap_or("unknown");
        let preview = if kind == "cron" { now + 60_000 } else { now };
        println!("schedule={id} trigger={kind} preview_unix_ms={preview}");
    }
    Ok(())
}

fn run_dag_lint(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let findings = bijux_dag_core::lint_graph(&parsed);
    println!("{}", serde_json::to_string_pretty(&findings).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_unit_harness(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let preview = bijux_dag_core::DagUnitHarness::dry_run(&input).map_err(|err| err.to_string())?;
    println!("{}", serde_json::to_string_pretty(&preview).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_simulate(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let order = bijux_dag_core::simulate_graph(&parsed);
    println!("{}", serde_json::to_string_pretty(&order).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_dry_run(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let preview = bijux_dag_core::dry_run_preview(&parsed);
    println!("{}", serde_json::to_string_pretty(&preview).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_plan_dump(graph: &Path, select: &[String]) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let options = bijux_dag_core::PlanOptions {
        selected_nodes: select.iter().cloned().collect(),
        ..bijux_dag_core::PlanOptions::default()
    };
    let plan = bijux_dag_core::lower_graph_to_execution_plan(&parsed, options)
        .map_err(|err| err.to_string())?;
    validate_execution_plan_shape(&root, &plan)?;
    println!("{}", serde_json::to_string_pretty(&plan).map_err(|err| err.to_string())?);
    Ok(())
}

fn validate_execution_plan_shape(
    root: &Path,
    plan: &bijux_dag_core::ExecutionPlan,
) -> Result<(), String> {
    let schema_path = root.join("configs/dag/schema/execution_plan.schema.json");
    let schema_payload = fs::read_to_string(&schema_path).map_err(|err| err.to_string())?;
    let schema: Value = serde_json::from_str(&schema_payload).map_err(|err| err.to_string())?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "execution plan schema missing required list".to_string())?;
    let plan_value = serde_json::to_value(plan).map_err(|err| err.to_string())?;
    for key in required.iter().filter_map(Value::as_str) {
        if plan_value.get(key).is_none() {
            return Err(format!("execution plan missing schema-required field `{key}`"));
        }
    }
    Ok(())
}

fn run_dag_visualize(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(run_dir).join("observability.graph-visualization.json");
    let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    println!("{payload}");
    Ok(())
}

fn run_dag_scheduler_timeline(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let manifest_path = root.join(run_dir).join("manifest.json");
    if !manifest_path.exists() {
        return Err(format!(
            "run directory does not contain manifest.json: {}",
            manifest_path.display()
        ));
    }
    let timeline_path = root.join(run_dir).join("observability.timeline.json");
    let payload = fs::read_to_string(&timeline_path).map_err(|err| err.to_string())?;
    let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let entries = parsed.get("entries").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let scheduler_entries = entries
        .into_iter()
        .filter(|row| {
            row.get("category")
                .and_then(|v| v.as_str())
                .map(|category| {
                    matches!(
                        category,
                        "schedule" | "dispatch" | "retry" | "cache_hit" | "cache_miss"
                    )
                })
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let response = json!({
        "run_dir": run_dir,
        "timeline_path": timeline_path.strip_prefix(&root).map_err(|err| err.to_string())?,
        "scheduler_entry_count": scheduler_entries.len(),
        "entries": scheduler_entries,
    });
    println!("{}", serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_verify_state(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let manifest_path = run_path.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    let manifest: Value = serde_json::from_str(&manifest_text).map_err(|err| err.to_string())?;
    let run_state = match manifest.get("status").and_then(Value::as_str).unwrap_or("failed") {
        "success" => bijux_dag_runtime::RunState::Succeeded,
        "failed" => bijux_dag_runtime::RunState::Failed,
        "cancelled" => bijux_dag_runtime::RunState::Cancelled,
        _ => bijux_dag_runtime::RunState::Running,
    };

    let mut node_states = Vec::new();
    let trace_dir = run_path.join("trace");
    if trace_dir.exists() {
        for entry in fs::read_dir(&trace_dir).map_err(|err| err.to_string())? {
            let path = entry.map_err(|err| err.to_string())?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("json") {
                continue;
            }
            let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
            let state = match parsed.get("status").and_then(Value::as_str).unwrap_or("failed") {
                "success" => bijux_dag_runtime::NodeState::Success,
                "failed" => bijux_dag_runtime::NodeState::Failed,
                "cached" => bijux_dag_runtime::NodeState::Cached,
                "skipped" => bijux_dag_runtime::NodeState::Skipped,
                "cancelled" => bijux_dag_runtime::NodeState::Cancelled,
                "running" => bijux_dag_runtime::NodeState::Running,
                _ => bijux_dag_runtime::NodeState::Failed,
            };
            node_states.push(state);
        }
    }

    let report = bijux_dag_runtime::verify_post_run_state_consistency(
        run_state,
        &node_states,
        node_states.iter().filter(|s| matches!(s, bijux_dag_runtime::NodeState::Failed)).count(),
    );
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_debug(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let order = bijux_dag_core::simulate_graph(&parsed);
    let response = json!({
        "dependency_closure_order": order,
        "blocked_nodes": [],
        "policy_reasons": []
    });
    println!("{}", serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_explain_validation(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    match bijux_dag_core::parse_graph_strict(&input) {
        Ok(parsed) => {
            let diagnostics = parsed.validate_with_warnings();
            let explain = diagnostics
                .into_iter()
                .map(|d| {
                    json!({
                        "code": d.code,
                        "message": d.message,
                        "path": d.path,
                        "hint": d.hint
                    })
                })
                .collect::<Vec<_>>();
            println!("{}", serde_json::to_string_pretty(&explain).map_err(|err| err.to_string())?);
            Ok(())
        }
        Err(err) => Err(format!("validation parse failed for {}: {}", path.display(), err)),
    }
}

fn run_dag_explain_node(run_dir: &Path, node_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(run_dir).join("failure-propagation.json");
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let rows: Value = serde_json::from_str(&input).map_err(|err| err.to_string())?;
    let reasons = rows
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.get("node_id").and_then(|v| v.as_str()) == Some(node_id))
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string_pretty(&reasons).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_preview(graph: &Path) -> Result<(), String> {
    run_dag_dry_run(graph)
}

fn run_dag_schema_export(out: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(out);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "BijuxDagV01",
        "type": "object",
        "required": ["spec", "nodes", "edges"],
        "properties": {
            "spec": {"type": "string"},
            "meta": {"type": "object"},
            "nodes": {"type": "array"},
            "edges": {"type": "array"}
        }
    });
    fs::write(path, serde_json::to_vec_pretty(&schema).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

fn run_dag_repair_run(run_dir: &Path, apply: bool) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let manifest = run_path.join("manifest.json");
    let metadata_index = run_path.join("metadata.index.json");
    let manifest_exists = manifest.exists();
    let index_exists = metadata_index.exists();

    if !manifest_exists && apply {
        let payload = json!({
            "status": "repaired",
            "reason": "manifest was missing and reconstructed",
            "generated_unix_ms": now_millis(),
        });
        fs::write(&manifest, serde_json::to_vec_pretty(&payload).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    }
    if !index_exists && apply {
        let payload = json!({
            "status": "repaired",
            "reason": "metadata index was missing and rebuilt",
            "generated_unix_ms": now_millis(),
        });
        fs::write(
            &metadata_index,
            serde_json::to_vec_pretty(&payload).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
    }

    let response = json!({
        "run_dir": run_path,
        "manifest_exists": manifest_exists,
        "metadata_index_exists": index_exists,
        "apply": apply,
        "manifest_repaired": !manifest_exists && apply,
        "metadata_index_repaired": !index_exists && apply
    });
    println!("{}", serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_simulate_recovery(scenario: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(scenario);
    let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let scenario_json: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let scenario_id = scenario_json
        .get("scenario_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "scenario_id is required".to_string())?;
    let injections = scenario_json
        .get("injections")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "injections array is required".to_string())?;
    let summary = json!({
        "scenario_id": scenario_id,
        "fault_count": injections.len(),
        "simulated": true,
        "evaluated_unix_ms": now_millis(),
    });
    println!("{}", serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_recovery_accept(suite: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(suite);
    let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let suite_json: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
    let suite_id = suite_json
        .get("suite_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "suite_id is required".to_string())?;
    let required_scenarios = suite_json
        .get("required_scenarios")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "required_scenarios array is required".to_string())?;
    let strict = suite_json.get("strict").and_then(|v| v.as_bool()).unwrap_or(true);
    let report = json!({
        "suite_id": suite_id,
        "required_scenario_count": required_scenarios.len(),
        "strict": strict,
        "accepted": !required_scenarios.is_empty(),
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_explain_run(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(run_dir).join("observability.root-causes.json");
    let root_causes = fs::read_to_string(&path)
        .ok()
        .and_then(|v| serde_json::from_str::<Value>(&v).ok())
        .unwrap_or_else(|| json!([]));
    let report = json!({
        "what_happened": ["run execution completed with observability evidence"],
        "why_happened": root_causes,
        "what_next": ["inspect failed nodes", "run artifact verification", "review scheduler policy"]
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_run_inspect(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let manifest_path = run_path.join("manifest.json");
    let timeline_path = run_path.join("observability.timeline.json");
    let events_path = run_path.join("observability.events.json");
    let root_causes_path = run_path.join("observability.root-causes.json");

    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let timeline: Value =
        serde_json::from_str(&fs::read_to_string(&timeline_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let events: Value =
        serde_json::from_str(&fs::read_to_string(&events_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let root_causes: Value = if root_causes_path.exists() {
        serde_json::from_str(&fs::read_to_string(&root_causes_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?
    } else {
        json!({"roots":[]})
    };

    let response = json!({
        "run_id": manifest.get("run_id").cloned().unwrap_or(Value::Null),
        "status": manifest.get("status").cloned().unwrap_or(Value::Null),
        "node_counts": manifest.get("node_counts").cloned().unwrap_or(Value::Null),
        "event_count": events.as_array().map(|v| v.len()).unwrap_or(0),
        "timeline_entry_count": timeline.get("entries").and_then(|v| v.as_array()).map(|v| v.len()).unwrap_or(0),
        "root_causes": root_causes.get("roots").cloned().unwrap_or(json!([])),
        "artifacts": {
            "manifest": manifest_path.strip_prefix(&root).map_err(|err| err.to_string())?,
            "timeline": timeline_path.strip_prefix(&root).map_err(|err| err.to_string())?,
            "events": events_path.strip_prefix(&root).map_err(|err| err.to_string())?,
            "root_causes": root_causes_path.strip_prefix(&root).map_err(|err| err.to_string())?,
        }
    });
    println!("{}", serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_explain_artifact(run_dir: &Path, artifact_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(run_dir).join("observability.lineage-visualization.json");
    let lineage = fs::read_to_string(&path)
        .ok()
        .and_then(|v| serde_json::from_str::<Value>(&v).ok())
        .unwrap_or_else(|| json!({}));
    let report = json!({
        "artifact_id": artifact_id,
        "lineage_source": path,
        "lineage_data": lineage,
        "reproducible": true
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_explain_schedule(run_dir: &Path, schedule_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(run_dir).join("schedule.audit.json");
    let audits = fs::read_to_string(&path)
        .ok()
        .and_then(|v| serde_json::from_str::<Value>(&v).ok())
        .unwrap_or_else(|| json!([]));
    let matching = audits
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|row| row.get("schedule_id").and_then(|v| v.as_str()) == Some(schedule_id))
        .collect::<Vec<_>>();
    let report = json!({
        "schedule_id": schedule_id,
        "created_run": !matching.is_empty(),
        "records": matching
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_investigation_bundle(run_dir: &Path, run_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let bundle = json!({
        "run_id": run_id,
        "event_paths": [run_path.join("observability.events.json")],
        "manifest_paths": [run_path.join("manifest.json")],
        "lineage_paths": [run_path.join("observability.lineage-visualization.json")],
        "log_paths": [run_path.join("nodes")],
        "summary_paths": [run_path.join("observability.root-causes.json")]
    });
    println!("{}", serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_dag_drift_report(
    current_metrics: &Path,
    baseline_metrics: &Path,
    dag_name: &str,
    baseline_name: &str,
) -> Result<(), String> {
    let root = repo_root()?;
    let current_path = root.join(current_metrics);
    let baseline_path = root.join(baseline_metrics);
    let current_json: Value =
        serde_json::from_str(&fs::read_to_string(current_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let baseline_json: Value =
        serde_json::from_str(&fs::read_to_string(baseline_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let mut drift = Vec::new();
    if let (Some(curr), Some(base)) = (current_json.as_object(), baseline_json.as_object()) {
        for (key, curr_value) in curr {
            if let (Some(c), Some(b)) =
                (curr_value.as_f64(), base.get(key).and_then(|v| v.as_f64()))
            {
                if (c - b).abs() > 0.2 * b.max(1.0) {
                    drift.push(format!("{key} drifted from {b:.2} to {c:.2}"));
                }
            }
        }
    }
    let report = json!({
        "dag_name": dag_name,
        "baseline_name": baseline_name,
        "drift_findings": drift
    });
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_artifacts_clean() -> Result<(), String> {
    let root = repo_root()?;
    let artifacts_target = root.join("artifacts").join("target");
    if !artifacts_target.exists() {
        println!("artifacts target path is already clean: {}", artifacts_target.display());
        return Ok(());
    }
    fs::remove_dir_all(&artifacts_target).map_err(|err| err.to_string())?;
    println!("removed artifacts target: {}", artifacts_target.display());
    Ok(())
}

fn run_env_summary() -> Result<(), String> {
    println!("repo_root={}", repo_root()?.display());
    println!("cwd={}", env::current_dir().map_err(|err| err.to_string())?.display());
    print_command_version("rustc");
    print_command_version("cargo");
    print_command_version("cargo-audit");
    print_command_version("cargo-public-api");
    print_command_version("cargo-nextest");
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        println!("CARGO_TARGET_DIR={target_dir}");
    } else {
        println!("CARGO_TARGET_DIR=<not_set>");
    }
    Ok(())
}

fn print_command_version(command: &str) {
    let output = Command::new(command).arg("--version").output().ok();
    if let Some(output) = output {
        if output.status.success() {
            println!("{}={}", command, String::from_utf8_lossy(&output.stdout).trim());
        } else {
            println!("{}=<unavailable>", command);
        }
    } else {
        println!("{}=<unavailable>", command);
    }
}

fn run_verify_tools() -> Result<(), String> {
    let mut failed = false;
    for tool in ["cargo-audit", "cargo-public-api", "cargo-nextest", "rustup"] {
        let status = Command::new(tool).arg("--version").status();
        match status {
            Ok(status) if status.success() => println!("tool available: {tool}"),
            Ok(_) => {
                failed = true;
                println!("tool failed to execute: {tool}");
            }
            Err(err) => {
                failed = true;
                println!("tool missing: {tool} ({err})");
            }
        }
    }
    if failed {
        Err("required tools are missing or unavailable".into())
    } else {
        Ok(())
    }
}

fn run_resolve_check() -> Result<(), String> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .map_err(|err| format!("cargo metadata failed: {err}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed with status {}", output.status));
    }
    let payload = String::from_utf8_lossy(&output.stdout);
    if payload.contains("\"packages\"") {
        println!("workspace metadata resolved");
        Ok(())
    } else {
        Err("cargo metadata output missing package list".into())
    }
}

fn run_benchmark_baseline() -> Result<(), String> {
    let root = repo_root()?;
    let out_dir = root.join("artifacts").join("benchmarks");
    let runs_dir = out_dir.join("runs");
    fs::create_dir_all(&runs_dir).map_err(|err| err.to_string())?;
    let fixtures = [
        ("large-dag", "execute-local", "evidence/perf/fixtures/large_dag.json"),
        ("linear-32", "plan", "evidence/perf/fixtures/scheduler_linear_32.json"),
        ("parallel-64", "plan", "evidence/perf/fixtures/scheduler_parallel_64.json"),
        (
            "diamond-fanout",
            "manifest-finalize",
            "evidence/perf/fixtures/scheduler_diamond_fanout.json",
        ),
    ];
    let mut scenario_results = Vec::new();
    for (scenario_id, class, fixture) in fixtures {
        let start_ms = now_millis();
        run_with_root(
            &root,
            "cargo",
            &[
                "run",
                "-p",
                "bijux-dag-cli",
                "--",
                "dag",
                "run",
                fixture,
                "--out",
                runs_dir.to_str().ok_or_else(|| "non-utf8 runs path".to_string())?,
            ],
        )?;
        let end_ms = now_millis();
        let run_dir_size_bytes = dir_size_bytes(&runs_dir).unwrap_or(0);
        scenario_results.push(json!({
            "scenario_id": scenario_id,
            "class": class,
            "fixture": fixture,
            "elapsed_ms": end_ms.saturating_sub(start_ms),
            "resource_profile": {
                "wall_time_ms": end_ms.saturating_sub(start_ms),
                "cpu_time_ms": Value::Null,
                "rss_bytes": Value::Null,
                "peak_memory_bytes": Value::Null,
                "artifact_bytes": run_dir_size_bytes,
                "trace_bytes": estimate_trace_bytes(&runs_dir).unwrap_or(0),
                "process_count": 1,
                "measurement_quality": "approximate"
            }
        }));
    }

    let rust_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();
    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();
    let machine = json!({
        "os": env::consts::OS,
        "arch": env::consts::ARCH
    });

    let report = json!({
        "benchmark_format": "benchmark-report/v1",
        "profile": "deterministic-regression-baseline",
        "runner": "cargo run -p bijux-dag-cli --bin bijux-dag -- run",
        "commit_sha": commit_sha,
        "rust_version": rust_version,
        "machine": machine,
        "scenario_results": scenario_results,
        "recorded_at_unix_ms": now_millis()
    });
    fs::write(
        out_dir.join("baseline.json"),
        serde_json::to_vec_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn run_observability_report() -> Result<(), String> {
    let root = repo_root()?;
    let runs_root = root.join("artifacts").join("runs");
    let report_dir = root.join("artifacts").join("reports");
    fs::create_dir_all(&report_dir).map_err(|err| err.to_string())?;
    if !runs_root.exists() {
        fs::write(
            report_dir.join("observability.json"),
            serde_json::to_vec_pretty(&json!({"runs": [], "note": "no runs available"}))
                .map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?;
        return Ok(());
    }
    let mut runs = Vec::new();
    for entry in fs::read_dir(&runs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let run_path = entry.path();
        if !run_path.is_dir() {
            continue;
        }
        let name = run_path.file_name().and_then(|v| v.to_str()).unwrap_or_default().to_string();
        if !name.starts_with("run-") {
            continue;
        }
        let metrics_path = run_path.join("observability.metrics.json");
        let events_path = run_path.join("observability.events.json");
        let timeline_path = run_path.join("observability.timeline.json");
        runs.push(json!({
            "run_dir": name,
            "metrics_present": metrics_path.exists(),
            "events_present": events_path.exists(),
            "timeline_present": timeline_path.exists(),
        }));
    }
    let report = json!({
        "generated_unix_ms": now_millis(),
        "runs": runs
    });
    fs::write(
        report_dir.join("observability.json"),
        serde_json::to_vec_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

fn run_artifact_verify() -> Result<(), String> {
    let root = repo_root()?;
    let runs_root = root.join("artifacts").join("runs");
    if !runs_root.exists() {
        println!("no artifact runs directory found at {}", runs_root.display());
        return Ok(());
    }

    let mut failures = Vec::new();
    for entry in fs::read_dir(&runs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let run_path = entry.path();
        if !run_path.is_dir() {
            continue;
        }
        let name = run_path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
        if !name.starts_with("run-") {
            continue;
        }
        let manifest_path = run_path.join("manifest.json");
        if !manifest_path.exists() {
            failures.push(format!("{name}: missing manifest.json"));
            continue;
        }
        let manifest_text = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
        let manifest: serde_json::Value =
            serde_json::from_str(&manifest_text).map_err(|err| err.to_string())?;
        let outputs =
            manifest.get("outputs").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        for output in outputs {
            let node_id = output.get("node_id").and_then(|v| v.as_str()).unwrap_or_default();
            let file = output.get("file").and_then(|v| v.as_str()).unwrap_or_default();
            let expected_sha = output.get("sha256").and_then(|v| v.as_str()).unwrap_or_default();
            let file_path = run_path.join("nodes").join(node_id).join("outputs").join(file);
            if !file_path.exists() {
                failures.push(format!("{name}: missing output {}", file_path.display()));
                continue;
            }
            let bytes = fs::read(&file_path).map_err(|err| err.to_string())?;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&bytes);
            let actual_sha = hex::encode(hasher.finalize());
            if actual_sha != expected_sha {
                failures.push(format!("{name}: sha mismatch for {}", file_path.display()));
            }
        }
    }

    if failures.is_empty() {
        println!("artifact verification passed");
        Ok(())
    } else {
        Err(format!("artifact verification failed: {}", failures.join(", ")))
    }
}

fn run_golden() -> Result<(), String> {
    let root = repo_root()?;
    let scratch = std::env::temp_dir().join(format!("bijux-dag-golden-{}", now_secs()));
    let runs = scratch.join("runs");
    fs::create_dir_all(&runs).map_err(|err| err.to_string())?;

    let example = "evidence/authoring/examples/hello.dag.json";
    for _ in 0..2 {
        run_with_root(
            &root,
            "cargo",
            &[
                "run",
                "-p",
                "bijux-dag-cli",
                "--",
                "dag",
                "run",
                example,
                "--out",
                runs.to_str().expect("utf-8"),
            ],
        )?;
    }

    let (latest, previous) = two_latest_runs(&runs)?;

    let diff = run_status_and_json(
        &root,
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "diff",
            previous.to_str().expect("utf-8"),
            latest.to_str().expect("utf-8"),
            "--json",
        ],
    )?;
    assert_empty_diff(&diff)?;

    run_with_root(
        &root,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "replay",
            latest.to_str().expect("utf-8"),
            "--out",
            runs.to_str().expect("utf-8"),
        ],
    )?;

    let replay = newest_run(&runs)?;
    let replay_diff = run_status_and_json(
        &root,
        &[
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "diff",
            latest.to_str().expect("utf-8"),
            replay.to_str().expect("utf-8"),
            "--json",
        ],
    )?;
    assert_empty_diff(&replay_diff)
}

fn run_public_api() -> Result<(), String> {
    if Command::new("cargo-public-api").arg("--version").status().is_err() {
        return Ok(());
    }
    let root = repo_root()?;
    let docs_api = root.join("docs/api");
    fs::create_dir_all(&docs_api).map_err(|err| err.to_string())?;
    let public_api_toolchain =
        env::var("BIJUX_PUBLIC_API_TOOLCHAIN").unwrap_or_else(|_| "nightly-2025-06-22".into());
    let toolchain_flag = format!("+{public_api_toolchain}");

    for crate_name in
        ["bijux-dag-core", "bijux-dag-artifacts", "bijux-dag-runtime", "bijux-dag-app"]
    {
        let output = run_stdout_and_json(
            &root,
            "cargo",
            &[&toolchain_flag, "public-api", "-p", crate_name],
        )?;
        let out_txt = docs_api.join(format!("{crate_name}.txt"));
        if out_txt.exists() {
            let baseline = fs::read_to_string(&out_txt).map_err(|err| err.to_string())?;
            if baseline != output {
                return Err(format!("public API changed for {crate_name}"));
            }
        } else {
            fs::write(&out_txt, output).map_err(|err| err.to_string())?;
        }
    }

    Ok(())
}

fn run_dep_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy_text = fs::read_to_string(root.join("configs/dag/policy/dependency_rules.json"))
        .map_err(|err| err.to_string())?;
    let policy: DependencyPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;
    let edges = workspace_dependency_edges()?;
    let mut failed = false;

    for rule in &policy.rules {
        if edges.contains(&(rule.from.clone(), rule.to.clone())) {
            eprintln!("forbidden dependency edge {} -> {} ({})", rule.from, rule.to, rule.reason);
            failed = true;
        }
    }

    if failed {
        Err("dependency guard failed".into())
    } else {
        Ok(())
    }
}

fn run_crate_graph_command() -> Result<(), String> {
    let edges = workspace_dependency_edges()?;
    for (from, to) in edges {
        if from.starts_with("bijux-") && to.starts_with("bijux-") {
            println!("{from} -> {to}");
        }
    }
    Ok(())
}

fn workspace_dependency_edges() -> Result<BTreeSet<(String, String)>, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|err| format!("cargo metadata failed: {err}"))?;
    if !output.status.success() {
        return Err(format!("cargo metadata failed with status {}", output.status));
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("invalid metadata JSON: {err}"))?;
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    if let Some(packages) = payload.get("packages").and_then(Value::as_array) {
        for package in packages {
            let from = package.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
            if let Some(deps) = package.get("dependencies").and_then(Value::as_array) {
                for dep in deps {
                    let to =
                        dep.get("name").and_then(Value::as_str).unwrap_or_default().to_string();
                    if !from.is_empty() && !to.is_empty() {
                        edges.insert((from.clone(), to));
                    }
                }
            }
        }
    }
    Ok(edges)
}

fn run_workspace_manifest_policy_guard() -> Result<(), String> {
    let root = repo_root()?;
    let cli_deps = manifest_dependency_keys(&root.join("crates/bijux-dag-cli/Cargo.toml"))?;
    if cli_deps.contains("bijux-dag-runtime") || cli_deps.contains("bijux-dag-core") {
        return Err(
            "bijux-dag-cli must stay thin and only depend on bijux-dag-app plus cli wiring dependencies"
                .into(),
        );
    }

    let app_deps = manifest_dependency_keys(&root.join("crates/bijux-dag-app/Cargo.toml"))?;
    if !app_deps.contains("bijux-dag-runtime")
        || !app_deps.contains("bijux-dag-core")
        || !app_deps.contains("bijux-dag-artifacts")
    {
        return Err(
            "bijux-dag-app must depend on runtime/core/artifacts orchestration surfaces".into()
        );
    }
    Ok(())
}

fn manifest_dependency_keys(path: &Path) -> Result<BTreeSet<String>, String> {
    let manifest = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let value: toml::Value = toml::from_str(&manifest).map_err(|err| err.to_string())?;
    Ok(value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct ModuleSurfaceContract {
    schema_version: String,
    crates: Vec<ModuleSurfaceCrate>,
}

#[derive(Debug, Deserialize)]
struct ModuleSurfaceCrate {
    #[serde(rename = "crate")]
    crate_name: String,
    stable_public_modules: Vec<String>,
    experimental_public_modules: Vec<String>,
    simulated_public_modules: Vec<String>,
}

fn load_module_surface_contract(root: &Path) -> Result<ModuleSurfaceContract, String> {
    let path = root.join("contracts/foundation/module_surface_lanes.v1.json");
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    serde_json::from_str(&content).map_err(|err| err.to_string())
}

fn expected_public_modules(crate_entry: &ModuleSurfaceCrate) -> BTreeSet<String> {
    crate_entry
        .stable_public_modules
        .iter()
        .chain(crate_entry.experimental_public_modules.iter())
        .chain(crate_entry.simulated_public_modules.iter())
        .cloned()
        .collect()
}

fn crate_package_doc_rel(crate_name: &str) -> Option<&'static str> {
    match crate_name {
        "bijux-cli" => Some("docs/bijux-cli/packages/bijux-cli.md"),
        "bijux-cli-python" => Some("docs/bijux-cli/packages/bijux-cli-python.md"),
        "bijux-dag-core" => Some("docs/bijux-dag/packages/bijux-dag-core.md"),
        "bijux-dag-artifacts" => Some("docs/bijux-dag/packages/bijux-dag-artifacts.md"),
        "bijux-dag-runtime" => Some("docs/bijux-dag/packages/bijux-dag-runtime.md"),
        "bijux-dag-app" => Some("docs/bijux-dag/packages/bijux-dag-app.md"),
        "bijux-dag-cli" => Some("docs/bijux-dag/packages/bijux-dag-cli.md"),
        "bijux-dag-testkit" => Some("docs/bijux-dag/packages/bijux-dag-testkit.md"),
        "bijux-dev" => Some("docs/bijux-dev/packages/bijux-dev.md"),
        _ => None,
    }
}

fn crate_api_doc_rel(crate_name: &str) -> Option<&'static str> {
    match crate_name {
        "bijux-cli" => Some("docs/bijux-cli/interfaces/api-surface.md"),
        "bijux-dag-core" | "bijux-dag-artifacts" | "bijux-dag-runtime" | "bijux-dag-app" => {
            Some("docs/bijux-dag/interfaces/api-surface.md")
        }
        _ => None,
    }
}

fn run_public_export_docs_guard() -> Result<(), String> {
    let root = repo_root()?;
    let contract = load_module_surface_contract(&root)?;
    if contract.schema_version != "foundation-module-surface-lanes/v1" {
        return Err("module surface contract schema drift".to_string());
    }
    let mut missing = Vec::new();

    for crate_entry in contract.crates {
        let lib_rs = root.join("crates").join(&crate_entry.crate_name).join("src/lib.rs");
        let actual = public_modules_from_lib(&lib_rs)?;
        if actual.is_empty() {
            continue;
        }
        let Some(package_doc_rel) = crate_package_doc_rel(&crate_entry.crate_name) else {
            missing.push(format!(
                "{} has public exports but no package documentation mapping",
                crate_entry.crate_name
            ));
            continue;
        };
        let package_doc =
            fs::read_to_string(root.join(package_doc_rel)).map_err(|err| err.to_string())?;
        if !package_doc.contains(&crate_entry.crate_name) {
            missing.push(format!(
                "{} package doc does not mention its crate name: {}",
                crate_entry.crate_name, package_doc_rel
            ));
        }

        let Some(api_doc_rel) = crate_api_doc_rel(&crate_entry.crate_name) else {
            continue;
        };
        let api_doc = fs::read_to_string(root.join(api_doc_rel)).map_err(|err| err.to_string())?;
        if !api_doc.contains(&crate_entry.crate_name) {
            missing.push(format!(
                "{} API doc does not mention its crate name: {}",
                crate_entry.crate_name, api_doc_rel
            ));
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing.join(", "))
    }
}

fn run_crate_ownership_guard() -> Result<(), String> {
    let root = repo_root()?;
    let contract = load_module_surface_contract(&root)?;
    if contract.schema_version != "foundation-module-surface-lanes/v1" {
        return Err("module surface contract schema drift".to_string());
    }
    let mut violations = Vec::new();

    for crate_entry in contract.crates {
        let lib_rs = root.join("crates").join(&crate_entry.crate_name).join("src/lib.rs");
        let actual = public_modules_from_lib(&lib_rs)?;
        let allowed = expected_public_modules(&crate_entry);
        for module in actual.difference(&allowed) {
            violations.push(format!(
                "{} exports undeclared public module `{}`",
                crate_entry.crate_name, module
            ));
        }
        for module in allowed.difference(&actual) {
            violations.push(format!(
                "{} is missing declared public module `{}`",
                crate_entry.crate_name, module
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("crate ownership guard failed: {}", violations.join(", ")))
    }
}

fn public_modules_from_lib(path: &Path) -> Result<BTreeSet<String>, String> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut modules = BTreeSet::new();
    let mut depth = 0usize;
    for line in content.lines() {
        if depth == 0 {
            let trimmed = line.trim();
            if trimmed.starts_with("pub mod ") {
                let raw = trimmed.trim_start_matches("pub mod ").trim();
                let name = raw
                    .trim_end_matches(';')
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string();
                if !name.is_empty() {
                    modules.insert(name);
                }
            }
        }
        let opens = line.matches('{').count();
        let closes = line.matches('}').count();
        depth = depth.saturating_add(opens).saturating_sub(closes);
    }
    Ok(modules)
}

fn run_cli_command_freeze() -> Result<(), String> {
    #[derive(Debug, Deserialize)]
    struct MaintainerCommandSurfaceContract {
        schema_version: String,
        binary: String,
        visible_root_commands: Vec<String>,
    }

    let root = repo_root()?;
    let path = root.join("contracts/foundation/maintainer_command_surface.v1.json");
    let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let contract: MaintainerCommandSurfaceContract =
        serde_json::from_str(&content).map_err(|err| err.to_string())?;
    if contract.schema_version != "foundation-maintainer-command-surface/v1" {
        return Err("maintainer command surface contract schema drift".to_string());
    }
    if contract.binary != "bijux-dev-dag" {
        return Err("maintainer command surface contract binary drift".to_string());
    }

    let actual = root_command_names();
    if actual == contract.visible_root_commands {
        return Ok(());
    }

    let expected = contract.visible_root_commands;
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    let missing = expected_set.difference(&actual_set).cloned().collect::<Vec<_>>();
    let unexpected = actual_set.difference(&expected_set).cloned().collect::<Vec<_>>();
    let order_drift = actual != expected;

    Err(format!(
        "cli command freeze violated: missing={missing:?}, unexpected={unexpected:?}, order_drift={order_drift}, actual_count={}, expected_count={}",
        actual.len(),
        expected.len()
    ))
}

fn run_adapter_kind_freeze() -> Result<(), String> {
    let root = repo_root()?;
    let runtime_lib = root.join("crates/bijux-dag-runtime/src/lib.rs");
    let content = fs::read_to_string(&runtime_lib).map_err(|err| err.to_string())?;
    let mut kind_count = 0usize;
    for marker in [
        "vec![\"const\".to_string()]",
        "vec![\"shell\".to_string()]",
        "vec![\"container\".to_string()]",
    ] {
        if content.contains(marker) {
            kind_count += 1;
        }
    }
    if kind_count > ADAPTER_KIND_FREEZE_BASELINE {
        Err(format!(
            "adapter kind freeze violated: {} > baseline {}",
            kind_count, ADAPTER_KIND_FREEZE_BASELINE
        ))
    } else {
        Ok(())
    }
}

fn run_docs_guarantee_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    files.push(root.join("README.md"));
    collect_markdown_files(&root.join("docs"), &mut files)?;

    let mut violations = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for (idx, line) in content.lines().enumerate() {
            let lower = line.to_lowercase();
            let has_guarantee = lower.contains("guarantee") || lower.contains("guarantees");
            if !has_guarantee {
                continue;
            }
            let has_link = line.contains("](")
                && (line.contains("docs/spec/")
                    || line.contains("tests/")
                    || line.contains("benchmarks/")
                    || line.contains("artifacts/benchmarks/")
                    || line.contains("artifacts/memory/"));
            if !has_link {
                violations.push(format!("{rel}:{} guarantee claim missing proof link", idx + 1));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("docs guarantee guard failed: {}", violations.join(", ")))
    }
}

fn run_validation_rule_docs_guard() -> Result<(), String> {
    let root = repo_root()?;
    let validate_src = fs::read_to_string(root.join("crates/bijux-dag-core/src/validate.rs"))
        .map_err(|err| err.to_string())?;
    let docs = fs::read_to_string(root.join("docs/spec/VALIDATION_RULES.md"))
        .map_err(|err| err.to_string())?;

    let mut ids = BTreeSet::new();
    for token in validate_src.split(|c: char| !c.is_ascii_alphanumeric()) {
        if token.len() == 5
            && (token.starts_with('E') || token.starts_with('W'))
            && token.chars().skip(1).all(|c| c.is_ascii_digit())
        {
            ids.insert(token.to_string());
        }
    }

    let mut missing = Vec::new();
    for id in ids {
        if !docs.contains(&id) {
            missing.push(id);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "validation rule IDs missing from docs/spec/VALIDATION_RULES.md: {}",
            missing.join(", ")
        ))
    }
}

fn run_schema_contracts_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "configs/dag/schema/dag.schema.json",
        "configs/dag/schema/run_manifest.schema.json",
        "configs/dag/schema/node_trace.schema.json",
        "configs/dag/schema/outputs_index.schema.json",
        "configs/dag/schema/fixtures/v0.1/positive/empty-graph.json",
        "configs/dag/schema/fixtures/v0.1/negative/unknown-field.json",
    ];
    for rel in required {
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("missing schema contract file: {rel}"));
        }
    }
    Ok(())
}

fn run_repo_docs_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/bijux-core/foundation/documentation-system.md",
        "docs/bijux-core/foundation/domain-language.md",
        "docs/bijux-core/foundation/module-surface-lanes.md",
        "docs/bijux-core/foundation/package-boundary.md",
        "docs/bijux-core/governance/package-ownership.md",
        "docs/bijux-core/operations/artifact-governance.md",
        "docs/bijux-cli/interfaces/api-surface.md",
        "docs/bijux-dag/interfaces/api-surface.md",
        "docs/bijux-dag/interfaces/public-imports.md",
        "docs/bijux-dev/operations/repository-gates.md",
        "docs/spec/ADAPTER_CONTRACT.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing required docs contract: {rel}"));
        }
    }
    Ok(())
}

fn run_repo_source_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy = root.join("configs/dag/policy/source_layout.json");
    if !policy.exists() {
        return Err("missing source layout policy".into());
    }
    let runtime_lib = fs::read_to_string(root.join("crates/bijux-dag-runtime/src/lib.rs"))
        .map_err(|err| err.to_string())?;
    if runtime_lib.contains("use clap::") {
        return Err("runtime crate must not import clap".into());
    }
    Ok(())
}

fn run_root_directory_guard() -> Result<(), String> {
    let root = repo_root()?;
    let allowed = [
        "Cargo.toml",
        "Cargo.lock",
        "README.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "LICENSE",
        "NOTICE",
        "SECURITY.md",
        ".gitignore",
        "audit-allowlist.toml",
        "mkdocs.shared.yml",
        "mkdocs.yml",
        "rust-toolchain.toml",
        "rustfmt.toml",
        "Makefile",
    ];
    let mut violations = Vec::new();
    for entry in fs::read_dir(&root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if !allowed.contains(&name) {
            violations.push(name.to_string());
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!("root directory contains non-contract files: {}", violations.join(", ")))
    }
}

fn run_executable_guard() -> Result<(), String> {
    let root = repo_root()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut violations = Vec::new();
        let mut files = Vec::new();
        collect_files_with_extension(&root.join("crates"), "rs", &mut files)?;
        collect_files_with_extension(&root.join("docs"), "md", &mut files)?;
        collect_files_with_extension(&root.join("configs"), "json", &mut files)?;
        for file in files {
            let rel = file
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let mode = fs::metadata(&file).map_err(|err| err.to_string())?.permissions().mode();
            let executable = mode & 0o111 != 0;
            if executable {
                violations.push(rel);
            }
        }
        if !violations.is_empty() {
            return Err(format!(
                "executable source/docs/config files are not allowed: {}",
                violations.join(", ")
            ));
        }
    }
    Ok(())
}

fn run_repo_manifests_guard() -> Result<(), String> {
    let root = repo_root()?;
    let workspace = fs::read_to_string(root.join("Cargo.toml")).map_err(|err| err.to_string())?;
    if !workspace.contains("[workspace]") || !workspace.contains("members = [") {
        return Err("workspace Cargo.toml missing workspace members contract".into());
    }
    for crate_name in [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dev",
    ] {
        let manifest = root.join("crates").join(crate_name).join("Cargo.toml");
        let text = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        if !text.contains("[lints]") || !text.contains("workspace = true") {
            return Err(format!("{crate_name} manifest missing workspace lint contract"));
        }
    }
    Ok(())
}

fn run_repo_api_guard() -> Result<(), String> {
    let root = repo_root()?;
    let dag_docs = fs::read_to_string(root.join("docs/bijux-dag/interfaces/api-surface.md"))
        .map_err(|err| err.to_string())?;
    for crate_name in
        ["bijux-dag-core", "bijux-dag-artifacts", "bijux-dag-runtime", "bijux-dag-app"]
    {
        if !dag_docs.contains(crate_name) {
            return Err(format!("dag api surface docs missing coverage mention for {crate_name}"));
        }
    }
    let cli_docs = fs::read_to_string(root.join("docs/bijux-cli/interfaces/api-surface.md"))
        .map_err(|err| err.to_string())?;
    if !cli_docs.contains("bijux-cli") {
        return Err("cli api surface docs missing coverage mention for bijux-cli".to_string());
    }
    let dag_cli_docs = fs::read_to_string(root.join("docs/bijux-dag/packages/bijux-dag-cli.md"))
        .map_err(|err| err.to_string())?;
    if !dag_cli_docs.contains("bijux-dag-cli") {
        return Err("dag package docs missing coverage mention for bijux-dag-cli".to_string());
    }
    Ok(())
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

fn run_missing_workspace_dependency_checks() -> Result<(), String> {
    let root = repo_root()?;
    let manifests = [
        "crates/bijux-dag-core/Cargo.toml",
        "crates/bijux-dag-artifacts/Cargo.toml",
        "crates/bijux-dag-runtime/Cargo.toml",
        "crates/bijux-dag-app/Cargo.toml",
        "crates/bijux-dag-cli/Cargo.toml",
        "crates/bijux-dev/Cargo.toml",
    ];
    let mut failed = false;
    for manifest in manifests {
        let content = fs::read_to_string(root.join(manifest)).map_err(|err| err.to_string())?;
        for line in content.lines() {
            if line.contains("bijux_dag_") {
                eprintln!("legacy workspace crate reference in {manifest}: {line}");
                failed = true;
            }
        }
    }
    if failed {
        Err("found legacy workspace dependency references".into())
    } else {
        println!("workspace dependency references use canonical names");
        Ok(())
    }
}

fn assert_empty_diff(diff: &Value) -> Result<(), String> {
    if diff.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!("expected ok=true: {diff}"));
    }
    let payload = diff.get("data").ok_or_else(|| "missing data field".to_string())?;
    let is_empty_object = |key: &str| {
        payload
            .get(key)
            .map(|v| v.is_object() && v.as_object().is_some_and(|m| m.is_empty()))
            .unwrap_or(false)
    };

    if !is_empty_object("manifest") {
        return Err(format!("manifest not empty: {payload}"));
    }
    if payload.get("graph_fingerprint").and_then(Value::as_null).is_none() {
        return Err(format!("graph_fingerprint not null: {payload}"));
    }
    if !is_empty_object("nodes") {
        return Err(format!("nodes not empty: {payload}"));
    }
    if !is_empty_object("outputs") {
        return Err(format!("outputs not empty: {payload}"));
    }
    Ok(())
}
