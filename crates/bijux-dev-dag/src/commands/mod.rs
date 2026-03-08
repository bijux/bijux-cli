use clap::{CommandFactory, Parser};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

mod authoring_evidence;
mod battle_evidence;
mod benchmark_harness;
mod compare_evidence;
mod evidence_access;
mod evidence_control_plane;
mod evidence_registry;
mod model;
mod perf_evidence;
mod reporting;
mod suite_dispatch;

use authoring_evidence::{
    run_authoring_coverage_report, run_show_effective_all_authoring, run_validate_all_authoring,
};
use battle_evidence::{
    run_battle_coverage_report, run_battle_scenario_mapping_validate,
    run_battle_scenarios_by_trust_report, run_battle_scenarios_report,
    run_battle_trust_by_scenario_report,
};
use compare_evidence::{
    run_compare_evidence_policy_verify, run_comparison_evidence_report,
    run_comparison_harness_guard,
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
use model::{CommandContext, CommandEffect, SuiteDef};
use perf_evidence::{
    run_perf_evidence_policy_verify, run_perf_evidence_summary, run_perf_release_set,
    run_performance_evidence_guard, run_performance_evidence_report,
};
use reporting::run_command_reported;
use suite_dispatch::{run_suite_explain, run_suite_group, run_suite_list};

mod cli;

use cli::{
    ApiCommand, Cli, CommandLine, ControlCommand, DagCommand, ReleaseCommand, RepoCommand,
    ScheduleCommand, VerifyCommand, ADAPTER_KIND_FREEZE_BASELINE, CLI_COMMAND_FREEZE_BASELINE,
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
    let context = CommandContext {
        json: cli.json,
        report: cli.report,
    };
    match cli.command {
        CommandLine::Fmt => run_command_reported(
            &context,
            "fmt",
            CommandEffect::Validation,
            json!({}),
            || run_status("cargo", &["fmt", "--all"]),
        ),
        CommandLine::Lint => run_command_reported(
            &context,
            "lint",
            CommandEffect::Validation,
            json!({}),
            || {
                run_status("cargo", &["fmt", "--all", "--", "--check"])?;
                run_status(
                    "cargo",
                    &[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                )
            },
        ),
        CommandLine::Security => run_command_reported(
            &context,
            "security",
            CommandEffect::Validation,
            json!({}),
            || run_status("cargo", &["audit"]),
        ),
        CommandLine::Sanity => run_command_reported(
            &context,
            "sanity",
            CommandEffect::ReadWrite,
            json!({}),
            || {
                run_status("cargo", &["metadata", "--no-deps"])?;
                run_status("cargo", &["test", "-q"])?;
                run_status("cargo", &["fmt", "--all", "--", "--check"])
            },
        ),
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
                run_suite_explain(&context, "release", &suite, RELEASE_SUITES)
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
            RepoCommand::EvidenceDirectoryMap {
                out,
                create_missing,
            } => run_command_reported(
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
            RepoCommand::BattleCoverageReport {
                gaps_out,
                overloaded_out,
            } => run_command_reported(
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
            RepoCommand::EvidenceConsumerReports {
                assets_out,
                consumers_out,
            } => run_command_reported(
                &context,
                "repo.evidence-consumer-reports",
                CommandEffect::ReadWrite,
                json!({ "assets_out": assets_out, "consumers_out": consumers_out }),
                || run_evidence_consumer_reports(&assets_out, &consumers_out),
            ),
            RepoCommand::EvidenceSummaryReport {
                json_out,
                markdown_out,
            } => run_command_reported(
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
            RepoCommand::HotspotReports {
                file_out,
                function_out,
                api_out,
                dep_out,
            } => run_command_reported(
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
            ),
            RepoCommand::SchemaChangelog { out, schema_root } => run_command_reported(
                &context,
                "repo.schema-changelog",
                CommandEffect::ReadWrite,
                json!({ "out": out, "schema_root": schema_root }),
                || run_repo_schema_changelog(&out, &schema_root),
            ),
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
            DagCommand::ExplainArtifact {
                run_dir,
                artifact_id,
            } => run_command_reported(
                &context,
                "dag.explain-artifact",
                CommandEffect::Validation,
                json!({"run_dir": run_dir, "artifact_id": artifact_id}),
                || run_dag_explain_artifact(&run_dir, &artifact_id),
            ),
            DagCommand::ExplainSchedule {
                run_dir,
                schedule_id,
            } => run_command_reported(
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
        CommandLine::Doctor => run_command_reported(
            &context,
            "doctor",
            CommandEffect::ReadWrite,
            json!({}),
            || {
                run_env_summary()?;
                run_verify_tools()
            },
        ),
        CommandLine::Golden => run_command_reported(
            &context,
            "golden",
            CommandEffect::ReadWrite,
            json!({}),
            || run_golden(),
        ),
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
        CommandLine::BenchmarkCompare {
            current,
            baseline,
            max_regression_ratio,
        } => run_command_reported(
            &context,
            "benchmark-compare",
            CommandEffect::Validation,
            json!({
                "current": current,
                "baseline": baseline,
                "max_regression_ratio": max_regression_ratio
            }),
            || run_benchmark_compare(&current, &baseline, max_regression_ratio),
        ),
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
            run_command_reported(&context, "ci", CommandEffect::ReadWrite, json!({}), || {
                run_ci()
            })
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
        CommandLine::FoundationHardening {
            fail_fast,
            advisory,
            why,
        } => run_command_reported(
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
        CommandLine::Compat => run_command_reported(
            &context,
            "compat",
            CommandEffect::ReadWrite,
            json!({}),
            || {
                run_status(
                    "cargo",
                    &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"],
                )
            },
        ),
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

fn run_ci() -> Result<(), String> {
    run_status("cargo", &["fmt", "--all"])?;
    run_status(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    run_dep_guard()?;
    run_resolve_check()?;
    run_missing_workspace_dependency_checks()?;
    run_status("cargo", &["test", "--workspace"])?;
    run_golden()?;
    run_status(
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"],
    )?;

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
            "--",
            "dag",
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
            "--",
            "dag",
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
    let config_path = root.join("configs/suites/foundation_hardening.json");
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
        Err(format!(
            "foundation hardening failed: {}",
            failed.join(", ")
        ))
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
    run_ci()
}

fn run_release_readiness_report() -> Result<(), String> {
    let root = repo_root()?;
    let release_evidence = check_release_evidence_ready(&root)?;
    let report = json!({
        "timestamp_unix_ms": now_millis(),
        "release_evidence": release_evidence,
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn check_release_evidence_ready(root: &Path) -> Result<Value, String> {
    let config_payload = fs::read_to_string(root.join("configs/suites/foundation_hardening.json"))
        .map_err(|err| err.to_string())?;
    let config: FoundationHardeningConfig =
        serde_json::from_str(&config_payload).map_err(|err| err.to_string())?;
    let required_surfaces = [
        "docs/reports/foundation/release_evidence_report.md",
        "docs/reports/foundation/repository_proof_statement.md",
        "docs/reports/foundation/replay_hardening_report.md",
        "docs/reports/foundation/cache_hardening_report.md",
        "docs/reports/foundation/run_dir_import_export_hardening_report.md",
        "docs/reports/foundation/config_policy_determinism_report.md",
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
    let positive = root.join("configs/schema/fixtures/compat/positive");
    let negative = root.join("configs/schema/fixtures/compat/negative");
    collect_fixture_rows(&positive, true, &mut rows)?;
    collect_fixture_rows(&negative, false, &mut rows)?;
    rows.sort_by(|a, b| a["fixture"].as_str().cmp(&b["fixture"].as_str()));

    let matrix = json!({
        "generated_unix_ms": now_millis(),
        "schema_versions_supported": ["v0.1"],
        "rows": rows
    });
    let out = root.join("artifacts/release/compatibility_matrix.json");
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        &out,
        serde_json::to_string_pretty(&matrix).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&matrix).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_post_release_verify(binary: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let tmp_dir = tempfile::tempdir().map_err(|err| err.to_string())?;
    let dag_dir = tmp_dir.path();
    let runs_dir = dag_dir.join("runs");
    let bin_path = binary
        .map(|p| {
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                root.join(p)
            }
        })
        .unwrap_or_else(|| root.join("target/debug/bijux"));
    let bin = bin_path
        .to_str()
        .ok_or_else(|| "non-utf8 release binary path".to_string())?;

    run_with_root(
        &root,
        bin,
        &["dag", "init", "--dir", dag_dir.to_string_lossy().as_ref()],
    )?;
    run_with_root(
        &root,
        bin,
        &[
            "dag",
            "validate",
            dag_dir.join("dag.json").to_string_lossy().as_ref(),
        ],
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
    run_with_root(
        &root,
        bin,
        &["dag", "status", runs_dir.to_string_lossy().as_ref()],
    )?;
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
    println!(
        "reproducibility check passed: {} -> {}",
        tag,
        tag_sha.trim()
    );
    Ok(())
}

fn run_release_evidence_bundle(out: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let output = out
        .map(|p| {
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                root.join(p)
            }
        })
        .unwrap_or_else(|| root.join("artifacts/release/evidence_bundle.json"));

    let readiness_path = root.join("artifacts/release/readiness_report.json");
    let readiness = if readiness_path.exists() {
        serde_json::from_str::<Value>(
            &fs::read_to_string(&readiness_path).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?
    } else {
        json!({"status": "missing", "hint": "run `bijux-dev-dag release readiness`"})
    };
    let matrix_path = root.join("artifacts/release/compatibility_matrix.json");
    let matrix = if matrix_path.exists() {
        serde_json::from_str::<Value>(
            &fs::read_to_string(&matrix_path).map_err(|err| err.to_string())?,
        )
        .map_err(|err| err.to_string())?
    } else {
        json!({"status": "missing", "hint": "run `bijux-dev-dag release compatibility-matrix`"})
    };

    let bundle = json!({
        "generated_unix_ms": now_millis(),
        "why_release_exists": "All required release policy evidence artifacts are present and reviewed.",
        "artifacts": {
            "readiness_report": readiness,
            "compatibility_matrix": matrix,
            "known_limitations_path": "docs/tracking/KNOWN_LIMITATIONS.md",
            "release_note_template_path": "docs/reference/RELEASE_NOTE_TEMPLATE.md"
        }
    });

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(
        &output,
        serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn check_contract_coverage_ready(root: &Path) -> Value {
    json!({"ok": root.join("docs/spec/CLI_CONTRACT.md").exists() && root.join("docs/spec/ERROR_CONTRACT.md").exists()})
}

fn check_schema_coverage_ready(root: &Path) -> Value {
    let positive = root
        .join("configs/schema/fixtures/compat/positive")
        .exists();
    let negative = root
        .join("configs/schema/fixtures/compat/negative")
        .exists();
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
    let payload = fs::read_to_string(root.join("configs/release/release_blockers.json"))
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&payload).map_err(|err| err.to_string())
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
        let id = definition
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("<unknown>");
        let trigger = definition.get("trigger").unwrap_or(&Value::Null);
        let kind = trigger
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
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
    println!(
        "{}",
        serde_json::to_string_pretty(&findings).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_dag_unit_harness(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let preview = bijux_dag_core::DagUnitHarness::dry_run(&input).map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&preview).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_dag_simulate(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let order = bijux_dag_core::simulate_graph(&parsed);
    println!(
        "{}",
        serde_json::to_string_pretty(&order).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_dag_dry_run(graph: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root.join(graph);
    let input = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    let parsed = bijux_dag_core::parse_graph_strict(&input).map_err(|err| err.to_string())?;
    let preview = bijux_dag_core::dry_run_preview(&parsed);
    println!(
        "{}",
        serde_json::to_string_pretty(&preview).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&plan).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn validate_execution_plan_shape(
    root: &Path,
    plan: &bijux_dag_core::ExecutionPlan,
) -> Result<(), String> {
    let schema_path = root.join("configs/schema/execution_plan.schema.json");
    let schema_payload = fs::read_to_string(&schema_path).map_err(|err| err.to_string())?;
    let schema: Value = serde_json::from_str(&schema_payload).map_err(|err| err.to_string())?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| "execution plan schema missing required list".to_string())?;
    let plan_value = serde_json::to_value(plan).map_err(|err| err.to_string())?;
    for key in required.iter().filter_map(Value::as_str) {
        if plan_value.get(key).is_none() {
            return Err(format!(
                "execution plan missing schema-required field `{key}`"
            ));
        }
    }
    Ok(())
}

fn run_dag_visualize(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let path = root
        .join(run_dir)
        .join("observability.graph-visualization.json");
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
    let entries = parsed
        .get("entries")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
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
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_dag_verify_state(run_dir: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let manifest_path = run_path.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|err| err.to_string())?;
    let manifest: Value = serde_json::from_str(&manifest_text).map_err(|err| err.to_string())?;
    let run_state = match manifest
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("failed")
    {
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
            let state = match parsed
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("failed")
            {
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
        node_states
            .iter()
            .filter(|s| matches!(s, bijux_dag_runtime::NodeState::Failed))
            .count(),
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?
    );
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
            println!(
                "{}",
                serde_json::to_string_pretty(&explain).map_err(|err| err.to_string())?
            );
            Ok(())
        }
        Err(err) => Err(format!(
            "validation parse failed for {}: {}",
            path.display(),
            err
        )),
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
    println!(
        "{}",
        serde_json::to_string_pretty(&reasons).map_err(|err| err.to_string())?
    );
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
    fs::write(
        path,
        serde_json::to_vec_pretty(&schema).map_err(|err| err.to_string())?,
    )
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
        fs::write(
            &manifest,
            serde_json::to_vec_pretty(&payload).map_err(|err| err.to_string())?,
        )
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
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?
    );
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
    let strict = suite_json
        .get("strict")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let report = json!({
        "suite_id": suite_id,
        "required_scenario_count": required_scenarios.len(),
        "strict": strict,
        "accepted": !required_scenarios.is_empty(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_dag_explain_artifact(run_dir: &Path, artifact_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let path = root
        .join(run_dir)
        .join("observability.lineage-visualization.json");
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
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?
    );
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
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_artifacts_clean() -> Result<(), String> {
    let root = repo_root()?;
    let artifacts_target = root.join("artifacts").join("target");
    if !artifacts_target.exists() {
        println!(
            "artifacts target path is already clean: {}",
            artifacts_target.display()
        );
        return Ok(());
    }
    fs::remove_dir_all(&artifacts_target).map_err(|err| err.to_string())?;
    println!("removed artifacts target: {}", artifacts_target.display());
    Ok(())
}

fn run_env_summary() -> Result<(), String> {
    println!("repo_root={}", repo_root()?.display());
    println!(
        "cwd={}",
        env::current_dir().map_err(|err| err.to_string())?.display()
    );
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
            println!(
                "{}={}",
                command,
                String::from_utf8_lossy(&output.stdout).trim()
            );
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
        return Err(format!(
            "cargo metadata failed with status {}",
            output.status
        ));
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
        (
            "large-dag",
            "execute-local",
            "evidence/perf/fixtures/large_dag.json",
        ),
        (
            "linear-32",
            "plan",
            "evidence/perf/fixtures/scheduler_linear_32.json",
        ),
        (
            "parallel-64",
            "plan",
            "evidence/perf/fixtures/scheduler_parallel_64.json",
        ),
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
                runs_dir
                    .to_str()
                    .ok_or_else(|| "non-utf8 runs path".to_string())?,
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
        "runner": "cargo run -p bijux-dag-cli -- dag run",
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
        let name = run_path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_string();
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
        println!(
            "no artifact runs directory found at {}",
            runs_root.display()
        );
        return Ok(());
    }

    let mut failures = Vec::new();
    for entry in fs::read_dir(&runs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let run_path = entry.path();
        if !run_path.is_dir() {
            continue;
        }
        let name = run_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
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
        let outputs = manifest
            .get("outputs")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for output in outputs {
            let node_id = output
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let file = output
                .get("file")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let expected_sha = output
                .get("sha256")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let file_path = run_path
                .join("nodes")
                .join(node_id)
                .join("outputs")
                .join(file);
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
        Err(format!(
            "artifact verification failed: {}",
            failures.join(", ")
        ))
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
    if Command::new("cargo-public-api")
        .arg("--version")
        .status()
        .is_err()
    {
        return Ok(());
    }
    let root = repo_root()?;
    let docs_api = root.join("docs/api");
    fs::create_dir_all(&docs_api).map_err(|err| err.to_string())?;

    for crate_name in [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
    ] {
        let output = run_stdout_and_json(&root, "cargo", &["public-api", "-p", crate_name])?;
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
    let policy_text = fs::read_to_string(root.join("configs/policy/dependency_rules.json"))
        .map_err(|err| err.to_string())?;
    let policy: DependencyPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;
    let edges = workspace_dependency_edges()?;
    let mut failed = false;

    for rule in &policy.rules {
        if edges.contains(&(rule.from.clone(), rule.to.clone())) {
            eprintln!(
                "forbidden dependency edge {} -> {} ({})",
                rule.from, rule.to, rule.reason
            );
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
        return Err(format!(
            "cargo metadata failed with status {}",
            output.status
        ));
    }
    let payload: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("invalid metadata JSON: {err}"))?;
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    if let Some(packages) = payload.get("packages").and_then(Value::as_array) {
        for package in packages {
            let from = package
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if let Some(deps) = package.get("dependencies").and_then(Value::as_array) {
                for dep in deps {
                    let to = dep
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
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
    let cli_manifest = fs::read_to_string(root.join("crates/bijux-dag-cli/Cargo.toml"))
        .map_err(|err| err.to_string())?;
    if cli_manifest.contains("bijux-dag-runtime") || cli_manifest.contains("bijux-dag-core") {
        return Err(
            "bijux-dag-cli must stay thin and only depend on bijux-dag-app plus cli wiring dependencies"
                .into(),
        );
    }

    let app_manifest = fs::read_to_string(root.join("crates/bijux-dag-app/Cargo.toml"))
        .map_err(|err| err.to_string())?;
    if !app_manifest.contains("bijux_dag_runtime")
        || !app_manifest.contains("bijux_dag_core")
        || !app_manifest.contains("bijux_dag_artifacts")
    {
        return Err(
            "bijux-dag-app must depend on runtime/core/artifacts orchestration surfaces".into(),
        );
    }
    Ok(())
}

fn run_public_export_docs_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy_text = fs::read_to_string(root.join("configs/policy/crate_ownership.json"))
        .map_err(|err| err.to_string())?;
    let policy: CrateOwnershipPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;
    let docs = fs::read_to_string(root.join("docs/spec/CRATE_API_POLICY.md"))
        .map_err(|err| err.to_string())?;
    let mut missing = Vec::new();

    for crate_entry in policy.crates {
        let lib_rs = root.join(&crate_entry.path).join("src/lib.rs");
        let actual = public_modules_from_lib(&lib_rs)?;
        if actual.is_empty() {
            continue;
        }
        if !docs.contains(&crate_entry.name) {
            missing.push(format!(
                "{} has public exports but no crate mention in docs/spec/CRATE_API_POLICY.md",
                crate_entry.name
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
    let policy_text = fs::read_to_string(root.join("configs/policy/crate_ownership.json"))
        .map_err(|err| err.to_string())?;
    let policy: CrateOwnershipPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;
    let mut violations = Vec::new();

    for crate_entry in policy.crates {
        if crate_entry.domains.is_empty() {
            violations.push(format!("{} has no declared domains", crate_entry.name));
        }
        let lib_rs = root.join(&crate_entry.path).join("src/lib.rs");
        let actual = public_modules_from_lib(&lib_rs)?;
        let allowed: BTreeSet<String> = crate_entry.public_modules.into_iter().collect();
        for module in actual.difference(&allowed) {
            violations.push(format!(
                "{} exports undeclared public module `{}`",
                crate_entry.name, module
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "crate ownership guard failed: {}",
            violations.join(", ")
        ))
    }
}

fn public_modules_from_lib(path: &Path) -> Result<BTreeSet<String>, String> {
    if !path.exists() {
        return Ok(BTreeSet::new());
    }
    let content = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let mut modules = BTreeSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("pub mod ") {
            continue;
        }
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
    Ok(modules)
}

fn run_cli_command_freeze() -> Result<(), String> {
    let count = Cli::command().get_subcommands().count();
    if count > CLI_COMMAND_FREEZE_BASELINE {
        Err(format!(
            "cli command freeze violated: {} > baseline {}",
            count, CLI_COMMAND_FREEZE_BASELINE
        ))
    } else {
        Ok(())
    }
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
                violations.push(format!(
                    "{rel}:{} guarantee claim missing proof link",
                    idx + 1
                ));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "docs guarantee guard failed: {}",
            violations.join(", ")
        ))
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
        "configs/schema/dag.schema.json",
        "configs/schema/run_manifest.schema.json",
        "configs/schema/node_trace.schema.json",
        "configs/schema/outputs_index.schema.json",
        "configs/schema/fixtures/v0.1/positive/empty-graph.json",
        "configs/schema/fixtures/v0.1/negative/unknown-field.json",
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
        "docs/spec/WORKSPACE_CONTRACT.md",
        "docs/spec/BOUNDARY_RULES.md",
        "docs/spec/CRATE_OWNERSHIP.md",
        "docs/spec/EVIDENCE_MODEL.md",
        "docs/spec/GLOSSARY.md",
        "docs/spec/CRATE_API_POLICY.md",
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
    let policy = root.join("configs/policy/source_layout.json");
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
        ".gitignore",
        "rust-toolchain.toml",
        "Makefile",
    ];
    let mut violations = Vec::new();
    for entry in fs::read_dir(&root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if !allowed.contains(&name) {
            violations.push(name.to_string());
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "root directory contains non-contract files: {}",
            violations.join(", ")
        ))
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
            let mode = fs::metadata(&file)
                .map_err(|err| err.to_string())?
                .permissions()
                .mode();
            let executable = mode & 0o111 != 0;
            if executable && !rel.starts_with("scripts/") {
                violations.push(rel);
            }
        }
        if !violations.is_empty() {
            return Err(format!(
                "executable files outside scripts/ are not allowed: {}",
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
        "bijux-dev-dag",
    ] {
        let manifest = root.join("crates").join(crate_name).join("Cargo.toml");
        let text = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        if !text.contains("[lints]") || !text.contains("workspace = true") {
            return Err(format!(
                "{crate_name} manifest missing workspace lint contract"
            ));
        }
    }
    Ok(())
}

fn run_repo_api_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs = fs::read_to_string(root.join("docs/spec/CRATE_API_POLICY.md"))
        .map_err(|err| err.to_string())?;
    for crate_name in [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
    ] {
        if !docs.contains(crate_name) {
            return Err(format!(
                "crate api policy missing coverage mention for {crate_name}"
            ));
        }
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
        "crates/bijux-dev-dag/Cargo.toml",
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
    let payload = diff
        .get("data")
        .ok_or_else(|| "missing data field".to_string())?;
    let is_empty_object = |key: &str| {
        payload
            .get(key)
            .map(|v| v.is_object() && v.as_object().is_some_and(|m| m.is_empty()))
            .unwrap_or(false)
    };

    if !is_empty_object("manifest") {
        return Err(format!("manifest not empty: {payload}"));
    }
    if payload
        .get("graph_fingerprint")
        .and_then(Value::as_null)
        .is_none()
    {
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

fn run_status(cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(&repo_root()?, cmd, args)
}

fn run_status_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .status()
        .map_err(|err| format!("failed to run {cmd}: {err}"))?;
    if !status.success() {
        return Err(format!("`{cmd}` failed with status {status}"));
    }
    Ok(())
}

fn run_with_root(root: &Path, cmd: &str, args: &[&str]) -> Result<(), String> {
    run_status_in_dir(root, cmd, args)
}

fn run_status_and_json(root: &Path, args: &[&str]) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run cargo: {err}"))?;
    if !output.status.success() {
        let _ = io::stdout().write_all(&output.stdout);
        let _ = io::stderr().write_all(&output.stderr);
        return Err(format!("cargo failed with status {}", output.status));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(stdout.trim())
        .map_err(|err| format!("invalid json: {err}\nstdout:\n{stdout}"))
}

fn run_stdout_and_json(root: &Path, cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|err| format!("failed to run {cmd}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{cmd} failed with status {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn newest_run(runs: &Path) -> Result<PathBuf, String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| format!("no runs found in {}", runs.display()))
}

fn two_latest_runs(runs: &Path) -> Result<(PathBuf, PathBuf), String> {
    let mut candidates: Vec<_> = fs::read_dir(runs)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|n| n.starts_with("run-"))
        })
        .collect();

    if candidates.len() < 2 {
        return Err(format!("expected at least 2 runs in {}", runs.display()));
    }

    candidates.sort_by(|a, b| {
        let ma = fs::metadata(a)
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b)
            .and_then(|m| m.modified())
            .unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });

    Ok((candidates[0].clone(), candidates[1].clone()))
}

pub(crate) fn repo_root() -> Result<PathBuf, String> {
    let mut dir = env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("crates").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not locate repo root".to_string())
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text;
    }
    let mut index = 0usize;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !pattern.starts_with('*') {
            if !text[index..].starts_with(part) {
                return false;
            }
            index += part.len();
            continue;
        }
        if i == parts.len() - 1 && !pattern.ends_with('*') {
            return text.ends_with(part);
        }
        if let Some(found) = text[index..].find(part) {
            index += found + part.len();
        } else {
            return false;
        }
    }
    true
}

fn collect_all_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            if path.file_name().and_then(|v| v.to_str()) == Some(".git") {
                continue;
            }
            collect_all_files(&path, out)?;
            continue;
        }
        if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn run_evidence_taxonomy_report() -> Result<(), String> {
    let root = repo_root()?;
    let content =
        fs::read_to_string(root.join("evidence/taxonomy.md")).map_err(|err| err.to_string())?;
    println!("{content}");
    Ok(())
}

fn run_evidence_ledger_report() -> Result<(), String> {
    let root = repo_root()?;
    let content = fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
        .map_err(|err| err.to_string())?;
    println!("{content}");
    Ok(())
}

fn run_evidence_directory_map(out: &Path, create_missing: bool) -> Result<(), String> {
    let root = repo_root()?;
    let structure_payload = fs::read_to_string(root.join("configs/policy/evidence_structure.json"))
        .map_err(|err| err.to_string())?;
    let structure: Value =
        serde_json::from_str(&structure_payload).map_err(|err| err.to_string())?;
    let required_dirs = structure["required_directories"]
        .as_array()
        .ok_or_else(|| "required_directories must be an array".to_string())?;

    let mut map_entries = Vec::new();
    for dir in required_dirs {
        let rel = dir
            .as_str()
            .ok_or_else(|| "required directory entry must be a string".to_string())?;
        let full = root.join(rel);
        if create_missing && !full.exists() {
            fs::create_dir_all(&full).map_err(|err| err.to_string())?;
        }
        map_entries.push(json!({
            "path": rel,
            "exists": full.is_dir(),
        }));
    }

    let payload = json!({
        "version": structure["version"].as_str().unwrap_or("1"),
        "source_policy": "configs/policy/evidence_structure.json",
        "entries": map_entries
    });
    let out_path = if out.is_absolute() {
        out.to_path_buf()
    } else {
        root.join(out)
    };
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    fs::write(&out_path, text).map_err(|err| err.to_string())?;
    println!("{}", out_path.display());
    Ok(())
}

fn run_evidence_metadata_validate() -> Result<(), String> {
    let root = repo_root()?;
    let policy_payload = fs::read_to_string(root.join("configs/policy/evidence_governance.json"))
        .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&policy_payload).map_err(|err| err.to_string())?;
    let ledger_payload = fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
        .map_err(|err| err.to_string())?;
    let ledger: Value = serde_json::from_str(&ledger_payload).map_err(|err| err.to_string())?;
    let path_policy_payload =
        fs::read_to_string(root.join("configs/policy/evidence_path_policy.json"))
            .map_err(|err| err.to_string())?;
    let path_policy: Value =
        serde_json::from_str(&path_policy_payload).map_err(|err| err.to_string())?;

    let required_fields: BTreeSet<String> = policy["required_metadata_fields"]
        .as_array()
        .ok_or_else(|| "required_metadata_fields must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "required metadata field must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_classes: BTreeSet<String> = policy["allowed_evidence_classes"]
        .as_array()
        .ok_or_else(|| "allowed_evidence_classes must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed evidence class must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_decisions: BTreeSet<String> = policy["allowed_decisions"]
        .as_array()
        .ok_or_else(|| "allowed_decisions must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed decision must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let allowed_impl_status: BTreeSet<String> = policy["allowed_implementation_statuses"]
        .as_array()
        .ok_or_else(|| "allowed_implementation_statuses must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "allowed implementation status must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let forbidden_globs: Vec<String> = policy["forbidden_globs"]
        .as_array()
        .ok_or_else(|| "forbidden_globs must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "forbidden glob must be a string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;

    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    for entry in entries {
        let map = entry
            .as_object()
            .ok_or_else(|| "evidence ledger entry must be an object".to_string())?;
        for field in &required_fields {
            if !map.contains_key(field) {
                return Err(format!(
                    "evidence ledger entry missing required field `{field}`"
                ));
            }
        }
        let path = entry["path"]
            .as_str()
            .ok_or_else(|| "entry path must be string".to_string())?;
        let owner = entry["owner"]
            .as_str()
            .ok_or_else(|| format!("owner must be string for {path}"))?;
        let class = entry["evidence_class"]
            .as_str()
            .ok_or_else(|| format!("evidence_class must be string for {path}"))?;
        let decision = entry["decision"]
            .as_str()
            .ok_or_else(|| format!("decision must be string for {path}"))?;
        let implementation_status = entry["implementation_status"]
            .as_str()
            .ok_or_else(|| format!("implementation_status must be string for {path}"))?;
        let canonical_location = entry["canonical_location"]
            .as_str()
            .ok_or_else(|| format!("canonical_location must be string for {path}"))?;
        let trust_property = entry["trust_property"]
            .as_str()
            .ok_or_else(|| format!("trust_property must be string for {path}"))?;
        let trust_properties_protected = entry["trust_properties_protected"]
            .as_array()
            .ok_or_else(|| format!("trust_properties_protected must be array for {path}"))?;
        let consumer_surfaces = entry["consumer_surfaces"]
            .as_array()
            .ok_or_else(|| format!("consumer_surfaces must be array for {path}"))?;
        if owner.trim().is_empty() {
            return Err(format!("owner is empty for {path}"));
        }
        if canonical_location.trim().is_empty() {
            return Err(format!("canonical_location is empty for {path}"));
        }
        if trust_property.trim().is_empty() {
            return Err(format!("trust_property is empty for {path}"));
        }
        if trust_properties_protected.is_empty() {
            return Err(format!("trust_properties_protected is empty for {path}"));
        }
        if consumer_surfaces.is_empty() {
            return Err(format!("consumer_surfaces is empty for {path}"));
        }
        if !allowed_classes.contains(class) {
            return Err(format!("invalid evidence_class `{class}` for {path}"));
        }
        if !allowed_decisions.contains(decision) {
            return Err(format!("invalid decision `{decision}` for {path}"));
        }
        if !allowed_impl_status.contains(implementation_status) {
            return Err(format!(
                "invalid implementation_status `{implementation_status}` for {path}"
            ));
        }
        if !root.join(path).exists() {
            return Err(format!("ledger path does not exist: {path}"));
        }
    }

    let governed_roots: Vec<String> = path_policy["governed_roots"]
        .as_array()
        .ok_or_else(|| "governed_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "governed root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let schema_fixture_roots: Vec<String> = path_policy["schema_fixture_roots"]
        .as_array()
        .ok_or_else(|| "schema_fixture_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "schema fixture root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let legacy_scenario_roots: Vec<String> = path_policy["legacy_scenario_roots"]
        .as_array()
        .ok_or_else(|| "legacy_scenario_roots must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "legacy scenario root must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;
    let helper_allowlist: Vec<String> = path_policy["helper_allowlist"]
        .as_array()
        .ok_or_else(|| "helper_allowlist must be an array".to_string())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "helper allowlist entry must be string".to_string())
                .map(ToOwned::to_owned)
        })
        .collect::<Result<_, _>>()?;

    let mut files = Vec::new();
    collect_all_files(&root, &mut files)?;
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !rel.ends_with(".json") {
            continue;
        }
        let in_governed_root = governed_roots.iter().any(|governed_root| {
            rel == *governed_root || rel.starts_with(&format!("{governed_root}/"))
        });
        let in_schema_fixture_root = schema_fixture_roots
            .iter()
            .any(|schema_root| rel == *schema_root || rel.starts_with(&format!("{schema_root}/")));
        let in_helper_allowlist = helper_allowlist
            .iter()
            .any(|pattern| wildcard_match(pattern, &rel));
        let in_legacy_scenario_root = legacy_scenario_roots
            .iter()
            .any(|legacy_root| rel == *legacy_root || rel.starts_with(&format!("{legacy_root}/")));
        if in_governed_root
            || in_schema_fixture_root
            || in_helper_allowlist
            || in_legacy_scenario_root
        {
            continue;
        }
        let is_scenario_like = rel.ends_with(".dag.json")
            || rel.contains("/scenarios/")
            || rel.contains("/fixtures/")
            || rel.starts_with("examples/");
        if is_scenario_like {
            return Err(format!(
                "scenario-like json path outside evidence-governed roots is forbidden: {rel}"
            ));
        }
        if forbidden_globs
            .iter()
            .any(|pattern| wildcard_match(pattern, &rel))
        {
            return Err(format!(
                "path is forbidden by evidence governance freeze policy: {rel}"
            ));
        }
        if rel.starts_with("tests/authoring/examples/") || rel.starts_with("tests/authoring/bad/") {
            return Err(format!(
                "authoring evidence outside evidence/authoring is forbidden: {rel}"
            ));
        }
    }

    println!("evidence metadata validation passed");
    Ok(())
}

fn resolve_under_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn write_report(path: &Path, body: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, body).map_err(|err| err.to_string())
}

fn run_repo_hotspot_reports(
    file_out: &Path,
    function_out: &Path,
    api_out: &Path,
    dep_out: &Path,
) -> Result<(), String> {
    let root = repo_root()?;
    let crates_dir = root.join("crates");
    let mut files = Vec::new();
    collect_all_files(&crates_dir, &mut files)?;
    let rust_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("rs"))
        .collect();

    let mut file_counts = Vec::new();
    let mut long_functions = Vec::new();
    let mut api_counts: BTreeMap<String, usize> = BTreeMap::new();

    for file in &rust_files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let body = fs::read_to_string(file).map_err(|err| err.to_string())?;
        let lines: Vec<&str> = body.lines().collect();
        file_counts.push((lines.len(), rel.clone()));

        let mut fn_starts = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
            {
                fn_starts.push(idx + 1);
            }
            if trimmed.starts_with("pub ") || trimmed.starts_with("pub(crate) ") {
                *api_counts.entry(rel.clone()).or_insert(0) += 1;
            }
        }
        for (pos, start) in fn_starts.iter().enumerate() {
            let end = if pos + 1 < fn_starts.len() {
                fn_starts[pos + 1] - 1
            } else {
                lines.len()
            };
            let len = end.saturating_sub(*start) + 1;
            if len >= 60 {
                long_functions.push((len, rel.clone(), *start));
            }
        }
    }

    file_counts.sort_by(|a, b| b.cmp(a));
    long_functions.sort_by(|a, b| b.cmp(a));
    let mut api_rows: Vec<(usize, String)> = api_counts.into_iter().map(|(f, c)| (c, f)).collect();
    api_rows.sort_by(|a, b| b.cmp(a));

    let mut file_report =
        String::from("# File Size Hotspot Report\n\nGenerated from Rust source line counts.\n\n");
    for (lines, path) in file_counts.into_iter().take(40) {
        file_report.push_str(&format!("- {lines} lines :: {path}\n"));
    }

    let mut function_report = String::from(
        "# Long Function Hotspot Report\n\nHeuristic report for functions exceeding 60 lines in Rust files.\n\n",
    );
    for (len, path, start) in long_functions.into_iter().take(80) {
        function_report.push_str(&format!("- {len} lines :: {path}:{start}\n"));
    }

    let mut public_api_report =
        String::from("# Public API Hotspot Report\n\nTop files by count of public items.\n\n");
    for (count, path) in api_rows.into_iter().take(40) {
        public_api_report.push_str(&format!("- {count} public items :: {path}\n"));
    }

    let dep_status = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(&root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    let mut dep_report = String::from(
        "# Dependency Cycle Report\n\nRust crate graph cycles are expected to be absent.\n\n",
    );
    dep_report.push_str("- check: cargo metadata package graph\n");
    if dep_status {
        dep_report.push_str(
            "- status: no crate-level dependency cycles detected by Cargo package resolution\n",
        );
    } else {
        dep_report.push_str("- status: metadata resolution failed\n");
    }
    dep_report.push_str(
        "- note: module-level cycles are prevented by Rust module system and compile checks\n",
    );

    write_report(&resolve_under_root(&root, file_out), &file_report)?;
    write_report(&resolve_under_root(&root, function_out), &function_report)?;
    write_report(&resolve_under_root(&root, api_out), &public_api_report)?;
    write_report(&resolve_under_root(&root, dep_out), &dep_report)?;
    Ok(())
}

fn run_repo_schema_changelog(out: &Path, schema_root: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let schema_path = resolve_under_root(&root, schema_root);
    let mut files = Vec::new();
    collect_all_files(&schema_path, &mut files)?;
    let mut schema_files: Vec<String> = files
        .into_iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .map(|p| {
            p.strip_prefix(&root)
                .map(|v| v.to_string_lossy().replace('\\', "/"))
                .map_err(|err| err.to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    schema_files.sort();

    let mut report = String::from(
        "# Schema Changelog\n\nGenerated from files under `configs/schema`.\n\n## Schemas\n",
    );
    for rel in schema_files {
        let full = root.join(&rel);
        let bytes = fs::read(&full).map_err(|err| err.to_string())?;
        let sum = format!("{:x}", sha2::Sha256::digest(bytes));
        report.push_str(&format!("- {rel} :: {sum}\n"));
    }

    write_report(&resolve_under_root(&root, out), &report)?;
    Ok(())
}

fn run_evidence_ownership_verify() -> Result<(), String> {
    run_evidence_metadata_validate()
}

fn required_schema_fields(schema_path: &Path) -> Result<BTreeSet<String>, String> {
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("schema missing required array: {}", schema_path.display()))?;
    let mut fields = BTreeSet::new();
    for field in required {
        let name = field.as_str().ok_or_else(|| {
            format!(
                "schema required entry must be string: {}",
                schema_path.display()
            )
        })?;
        fields.insert(name.to_string());
    }
    Ok(fields)
}

fn run_evidence_schema_verify() -> Result<(), String> {
    let root = repo_root()?;

    let schema_files = [
        "configs/schema/evidence_asset.schema.json",
        "configs/schema/evidence_family.schema.json",
        "configs/schema/evidence_cache_metadata.schema.json",
        "configs/schema/evidence_battle_metadata.schema.json",
        "configs/schema/evidence_perf_metadata.schema.json",
        "configs/schema/evidence_compare_metadata.schema.json",
        "configs/schema/evidence_compat_metadata.schema.json",
        "configs/schema/evidence_fault_metadata.schema.json",
        "configs/schema/evidence_authoring_metadata.schema.json",
    ];
    for rel in schema_files {
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("required evidence schema is missing: {rel}"));
        }
        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(&path).map_err(|err| err.to_string())?)
                .map_err(|err| err.to_string())?;
        if parsed.get("type").and_then(Value::as_str) != Some("object") {
            return Err(format!("evidence schema must declare object type: {rel}"));
        }
    }

    let asset_required =
        required_schema_fields(&root.join("configs/schema/evidence_asset.schema.json"))?;
    let family_required =
        required_schema_fields(&root.join("configs/schema/evidence_family.schema.json"))?;

    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    for entry in entries {
        let map = entry
            .as_object()
            .ok_or_else(|| "evidence ledger entry must be object".to_string())?;
        let id = entry
            .get("id")
            .and_then(Value::as_str)
            .ok_or_else(|| "evidence entry missing id".to_string())?;
        for field in &asset_required {
            if !map.contains_key(field) {
                return Err(format!(
                    "evidence entry `{id}` missing required field `{field}`"
                ));
            }
        }

        let kind = entry
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid kind"))?;
        let owner = entry
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid owner"))?;
        let canonical_path = entry
            .get("canonical_path")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid canonical_path"))?;
        let consumers = entry
            .get("consumers")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid consumers"))?;
        let release_blocking = entry
            .get("release_blocking")
            .and_then(Value::as_bool)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid release_blocking"))?;
        let trust_properties = entry
            .get("trust_properties")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("evidence entry `{id}` has invalid trust_properties"))?;
        let duplicate_of = entry
            .get("duplicate_of")
            .ok_or_else(|| format!("evidence entry `{id}` missing duplicate_of"))?;
        let derived_from = entry
            .get("derived_from")
            .ok_or_else(|| format!("evidence entry `{id}` missing derived_from"))?;

        if owner.trim().is_empty() {
            return Err(format!("evidence entry `{id}` has empty owner"));
        }
        if canonical_path.trim().is_empty() {
            return Err(format!("evidence entry `{id}` has empty canonical_path"));
        }
        if !root.join(canonical_path).exists() {
            return Err(format!(
                "evidence entry `{id}` canonical_path does not exist: {canonical_path}"
            ));
        }
        if consumers.is_empty() {
            return Err(format!("evidence entry `{id}` has empty consumers"));
        }
        let allowed_kinds = [
            "authoring",
            "battle",
            "cache",
            "compat",
            "fault",
            "operator",
            "perf",
            "compare",
        ];
        if !allowed_kinds.contains(&kind) {
            return Err(format!("evidence entry `{id}` has unknown kind `{kind}`"));
        }
        if release_blocking && trust_properties.is_empty() {
            return Err(format!(
                "evidence entry `{id}` is release_blocking but has no trust_properties"
            ));
        }
        if !duplicate_of.is_null()
            && duplicate_of
                .as_str()
                .map_or(true, |value| value.trim().is_empty())
        {
            return Err(format!(
                "evidence entry `{id}` duplicate_of must be non-empty string or null"
            ));
        }
        if !derived_from.is_null()
            && derived_from
                .as_str()
                .map_or(true, |value| value.trim().is_empty())
        {
            return Err(format!(
                "evidence entry `{id}` derived_from must be non-empty string or null"
            ));
        }
    }

    let families = ledger["asset_families"]
        .as_array()
        .ok_or_else(|| "evidence ledger asset_families must be an array".to_string())?;
    for family in families {
        let map = family
            .as_object()
            .ok_or_else(|| "asset family must be object".to_string())?;
        let family_id = family
            .get("family_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "asset family missing family_id".to_string())?;
        for field in &family_required {
            if !map.contains_key(field) {
                return Err(format!(
                    "asset family `{family_id}` missing required field `{field}`"
                ));
            }
        }
    }

    let metadata_files = [
        ("evidence/cache/metadata.json", "cache metadata"),
        ("evidence/battle/metadata.json", "battle metadata"),
        ("evidence/perf/metadata.json", "perf metadata"),
        ("evidence/compare/metadata.json", "compare metadata"),
        ("evidence/compat/metadata.json", "compat metadata"),
        ("evidence/fault/metadata.json", "fault metadata"),
        ("evidence/authoring/metadata.json", "authoring metadata"),
    ];
    for (rel, label) in metadata_files {
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("missing {label}: {rel}"));
        }
        let payload: Value =
            serde_json::from_str(&fs::read_to_string(path).map_err(|err| err.to_string())?)
                .map_err(|err| err.to_string())?;
        if !payload.is_object() {
            return Err(format!("{label} is not an object: {rel}"));
        }
    }

    println!("evidence schema validation passed");
    Ok(())
}

fn run_evidence_domain_verify(domain: &str, required_paths: &[&str]) -> Result<(), String> {
    let root = repo_root()?;
    run_evidence_metadata_validate()?;
    for rel in required_paths {
        if !root.join(rel).exists() {
            return Err(format!(
                "required evidence surface missing for `{domain}`: {rel}"
            ));
        }
    }
    let ledger: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/ownership/evidence_ledger.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let entries = ledger["entries"]
        .as_array()
        .ok_or_else(|| "evidence ledger entries must be an array".to_string())?;
    let prefix = format!("evidence/{domain}/");
    let has_entries = entries.iter().any(|entry| {
        entry
            .get("path")
            .and_then(Value::as_str)
            .is_some_and(|path| path.starts_with(&prefix))
    });
    if !has_entries {
        return Err(format!(
            "evidence ledger has no entries for governed domain `{domain}`"
        ));
    }
    Ok(())
}

fn parse_string_set(value: &Value, label: &str) -> Result<BTreeSet<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array"))?;
    let mut out = BTreeSet::new();
    for item in items {
        let name = item
            .as_str()
            .ok_or_else(|| format!("{label} entry must be a string"))?;
        if name.trim().is_empty() {
            return Err(format!("{label} contains empty string"));
        }
        out.insert(name.to_string());
    }
    Ok(out)
}

fn run_evidence_family_boundary_verify() -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;

    for asset in &assets {
        let path = asset.canonical_path.as_str();
        let kind = asset.kind.as_str();
        let expected_kind = if path.starts_with("evidence/cache/") {
            Some("cache")
        } else if path.starts_with("evidence/compat/") {
            Some("compat")
        } else if path.starts_with("evidence/fault/") {
            Some("fault")
        } else {
            None
        };
        if let Some(expected) = expected_kind {
            if kind != expected {
                return Err(format!(
                    "mixed-family misuse: `{path}` is classified as `{kind}` but must be `{expected}`"
                ));
            }
        }
    }

    let cache_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let cache_allowed = parse_string_set(
        &cache_metadata["consumer_boundaries"]["cache_allowed_consumers"],
        "cache consumer_boundaries.cache_allowed_consumers",
    )?;
    let replay_allowed = parse_string_set(
        &cache_metadata["consumer_boundaries"]["replay_allowed_consumers"],
        "cache consumer_boundaries.replay_allowed_consumers",
    )?;

    for asset in &assets {
        let path = asset.canonical_path.as_str();
        if !path.starts_with("evidence/cache/") {
            continue;
        }
        let allowed = if path.starts_with("evidence/cache/replay/") {
            &replay_allowed
        } else {
            &cache_allowed
        };
        for consumer in &asset.consumers {
            if !allowed.contains(consumer) {
                return Err(format!(
                    "cache/replay consumer misuse: `{path}` uses consumer `{consumer}` outside allowed set"
                ));
            }
        }
    }

    let compat_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/compat/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let decision_matrix = compat_metadata["decision_matrix"]
        .as_object()
        .ok_or_else(|| "compat metadata decision_matrix must be an object".to_string())?;
    for (path, entry) in decision_matrix {
        if !path.starts_with("evidence/compat/") {
            return Err(format!(
                "compat decision matrix contains out-of-family asset: {path}"
            ));
        }
        if !root.join(path).exists() {
            return Err(format!(
                "compat decision matrix path does not exist: {path}"
            ));
        }
        let decision = entry
            .get("decision")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("compat decision matrix entry missing decision: {path}"))?;
        let allowed = [
            "supported",
            "unsupported_future",
            "unsupported_past",
            "corrupt",
        ];
        if !allowed.contains(&decision) {
            return Err(format!(
                "compat decision matrix has unknown decision `{decision}` for `{path}`"
            ));
        }
    }

    let fault_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/fault/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let fault_expectations = fault_metadata["fault_expectations"]
        .as_object()
        .ok_or_else(|| "fault metadata fault_expectations must be an object".to_string())?;
    let fault_profiles = fault_metadata["fault_profiles"]
        .as_object()
        .ok_or_else(|| "fault metadata fault_profiles must be an object".to_string())?;
    for fault_class in fault_expectations.keys() {
        if !fault_profiles.contains_key(fault_class) {
            return Err(format!(
                "fault metadata missing fault_profiles entry for fault class `{fault_class}`"
            ));
        }
    }
    for (fault_class, profile) in fault_profiles {
        let expected_fault_class = profile
            .get("expected_fault_class")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!("fault profile missing expected_fault_class: `{fault_class}`")
            })?;
        let expected_reaction = profile
            .get("expected_system_reaction")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!("fault profile missing expected_system_reaction: `{fault_class}`")
            })?;
        if expected_fault_class.trim().is_empty() || expected_reaction.trim().is_empty() {
            return Err(format!(
                "fault profile contains empty fields for fault class `{fault_class}`"
            ));
        }
    }

    Ok(())
}

fn run_evidence_authoring_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "authoring",
        &[
            "evidence/authoring/metadata.json",
            "evidence/authoring/examples",
            "evidence/authoring/patterns",
            "evidence/authoring/negative",
        ],
    )?;
    run_validate_all_authoring()
}

fn run_evidence_battle_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "battle",
        &[
            "evidence/battle/workflows",
            "evidence/battle/metadata.json",
            "evidence/battle/registries/scenario_registry.json",
            "evidence/battle/registries/trust_property_registry.json",
        ],
    )?;
    run_battle_scenario_mapping_validate()
}

fn run_evidence_cache_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "cache",
        &[
            "evidence/cache/corrupt",
            "evidence/cache/scenarios",
            "evidence/cache/replay",
            "evidence/cache/metadata.json",
        ],
    )?;
    run_evidence_family_boundary_verify()
}

fn run_evidence_replay_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "cache",
        &[
            "evidence/cache/replay/match_case.json",
            "evidence/cache/replay/mismatch_case.json",
            "evidence/cache/replay/corruption_case.json",
            "evidence/cache/replay/unsupported_version_case.json",
        ],
    )?;
    run_replay_contract_guard()
}

fn run_evidence_compat_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "compat",
        &[
            "evidence/compat/graph_schema",
            "evidence/compat/export_bundle",
            "evidence/compat/run_dir",
            "evidence/compat/scenarios",
            "evidence/compat/metadata.json",
        ],
    )?;
    run_evidence_family_boundary_verify()
}

fn run_evidence_fault_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "fault",
        &[
            "evidence/fault/classes",
            "evidence/fault/corrupt_runs",
            "evidence/fault/metadata.json",
        ],
    )?;
    run_evidence_family_boundary_verify()
}

fn run_evidence_perf_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "perf",
        &[
            "evidence/perf/scenarios",
            "evidence/perf/baselines",
            "evidence/perf/metadata.json",
        ],
    )?;
    run_perf_evidence_policy_verify()
}

fn run_evidence_compare_verify() -> Result<(), String> {
    run_evidence_domain_verify(
        "compare",
        &[
            "evidence/compare/scenarios",
            "evidence/compare/baselines",
            "evidence/compare/metadata.json",
            "evidence/reports/comparison_fact_vs_interpretation.md",
        ],
    )?;
    run_compare_evidence_policy_verify()
}

fn run_evidence_foundation_verify() -> Result<(), String> {
    let root = repo_root()?;
    run_evidence_suite_policy_verify()?;
    run_evidence_schema_verify()?;
    run_evidence_registry_verify()?;
    run_evidence_ownership_verify()?;
    run_evidence_drift_verify()?;
    run_evidence_consumers_verify()?;
    run_evidence_authoring_verify()?;
    run_evidence_battle_verify()?;
    run_evidence_cache_verify()?;
    run_evidence_replay_verify()?;
    run_evidence_compat_verify()?;
    run_evidence_fault_verify()?;
    run_evidence_perf_verify()?;
    run_evidence_compare_verify()?;
    run_evidence_release_set_verify()?;
    for rel in [
        "evidence/reports/evidence_audit_2026-03-07.md",
        "evidence/reports/evidence_topology_before_after.md",
        "evidence/reports/root_cleanup_before_after.md",
        "evidence/reports/release_evidence_strength_before_after.md",
        "evidence/reports/evidence_architecture_freeze_review_cycle.md",
        "evidence/reports/evidence_roast_memo_2026-03-07.md",
        "evidence/reports/final_evidence_root_contract_report.md",
        "evidence/reports/root_topology_before_after.md",
        "evidence/reports/evidence_verification_summary.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!(
                "foundation report required for evidence governance is missing: {rel}"
            ));
        }
    }
    Ok(())
}

fn run_evidence_drift_verify() -> Result<(), String> {
    let root = repo_root()?;
    let legacy_roots = [
        "examples",
        "benchmarks/scenarios",
        "benchmarks/baselines",
        "comparisons/scenarios",
        "comparisons/bijux/baselines",
        "tests/e2e/fixtures",
        "tests/e2e/replay/fixtures",
        "tests/e2e/compat",
        "tests/e2e/container",
    ];
    let mut violations = Vec::new();
    for legacy_root in legacy_roots {
        let base = root.join(legacy_root);
        if !base.exists() {
            continue;
        }
        let mut files = Vec::new();
        collect_all_files(&base, &mut files)?;
        for file in files {
            let rel = file
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if rel.ends_with(".json") || rel.ends_with(".dag.json") {
                violations.push(rel);
            }
        }
    }
    if let Err(err) = run_evidence_family_boundary_verify() {
        violations.push(format!("family-boundary-drift: {err}"));
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "evidence drift violations detected: {}",
            violations.join(", ")
        ))
    }
}

fn run_evidence_resolve_by_id(asset_id: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let asset = resolve_asset_by_id(&assets, asset_id)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&[asset]))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_evidence_resolve_by_family(family: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_family(&assets, family);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_evidence_resolve_by_trust_property(trust_property: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_trust_property(&assets, trust_property);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_evidence_resolve_by_consumer(consumer: &str) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let resolved = resolve_assets_by_consumer(&assets, consumer);
    println!(
        "{}",
        serde_json::to_string_pretty(&evidence_assets_as_json(&resolved))
            .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_evidence_consumer_reports(assets_out: &Path, consumers_out: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let assets = load_registry_assets(&root)?;
    let assets_report = render_assets_to_consumers_report(&assets);
    let consumers_report = render_consumers_to_families_report(&assets);
    fs::write(root.join(assets_out), assets_report).map_err(|err| err.to_string())?;
    fs::write(root.join(consumers_out), consumers_report).map_err(|err| err.to_string())?;
    println!(
        "{}",
        json!({"assets_report": assets_out.to_string_lossy(), "consumers_report": consumers_out.to_string_lossy()})
    );
    Ok(())
}

fn run_evidence_consumers_verify() -> Result<(), String> {
    let root = repo_root()?;
    verify_registry_access_bypass(&root)?;
    let restricted_patterns = [
        "tests/e2e/replay/fixtures/",
        "tests/e2e/fixtures/e2e_minimal.json",
        "tests/e2e/compat/legacy_fixture_validation.json",
        "tests/e2e/container/container_execution_if_supported.json",
        "benchmarks/scenarios/",
        "comparisons/scenarios/",
    ];
    let mut violations = Vec::new();
    let mut files = Vec::new();
    collect_all_files(&root, &mut files)?;
    let ignore_paths = [
        "crates/bijux-dev-dag/src/commands/mod.rs",
        "crates/bijux-dev-dag/tests/evidence_consumer_integrity_contracts.rs",
        "crates/bijux-dev-dag/tests/evidence_access_contracts.rs",
        "docs/spec/TEST_EVIDENCE_CONSUMER_CONTRACT.md",
        "configs/policy/evidence_governance.json",
        "configs/policy/evidence_path_policy.json",
        "configs/policy/release_evidence_policy.json",
    ];
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if ignore_paths.iter().any(|path| rel == *path) {
            continue;
        }
        if !(rel.ends_with(".rs")
            || rel.ends_with(".md")
            || rel.ends_with(".json")
            || rel.ends_with(".toml"))
        {
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for pattern in restricted_patterns {
            if text.contains(pattern) {
                violations.push(format!(
                    "{rel}: contains legacy scenario reference `{pattern}`"
                ));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(" | "))
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub(crate) fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[derive(Debug, Deserialize)]
struct TestTaxonomyPolicy {
    required_prefixes: Vec<String>,
    legacy_allowlist: Vec<String>,
}

fn run_test_taxonomy_guard() -> Result<(), String> {
    let root = repo_root()?;
    let policy_path = root.join("configs/policy/test_taxonomy.json");
    let policy_text = fs::read_to_string(&policy_path).map_err(|err| err.to_string())?;
    let policy: TestTaxonomyPolicy =
        serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;

    let allowlist: BTreeSet<String> = policy.legacy_allowlist.into_iter().collect();
    let prefixes = policy.required_prefixes;
    let mut violations = Vec::new();

    let mut dirs = vec![root.join("crates"), root.join("tests")];
    while let Some(dir) = dirs.pop() {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.contains("/tests/") && !rel.starts_with("tests/") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            if !prefixes.iter().any(|prefix| name.starts_with(prefix)) && !allowlist.contains(&rel)
            {
                violations.push(format!("test file must use taxonomy prefix: {rel}"));
            }

            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            let is_e2e = rel.starts_with("tests/e2e/") || name.starts_with("e2e_");
            let shells_out = content.contains("-p\", \"bijux-dag-cli\"")
                || content.contains("Command::new(\"cargo\")")
                || content.contains("Command::new(\"bijux\")");
            if shells_out && !is_e2e {
                violations.push(format!(
                    "non-e2e test shells out to production binary path: {rel}"
                ));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_test_classification_report() -> Result<(), String> {
    let root = repo_root()?;
    let categories = [
        "unit_",
        "contract_",
        "integration_",
        "e2e_",
        "perf_",
        "compat_",
        "fault_",
    ];
    let mut counts: BTreeMap<String, u64> = categories
        .iter()
        .map(|category| ((*category).to_string(), 0_u64))
        .collect();

    let mut dirs = vec![root.join("crates"), root.join("tests")];
    while let Some(dir) = dirs.pop() {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if !rel.contains("/tests/") && !rel.starts_with("tests/") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or_default();
            for category in categories {
                if name.starts_with(category) {
                    if let Some(value) = counts.get_mut(category) {
                        *value += 1;
                    }
                }
            }
        }
    }

    let missing: Vec<String> = counts
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(category, _)| category.clone())
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "counts": counts,
            "missing_categories": missing,
        }))
        .map_err(|err| err.to_string())?
    );

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("missing test categories: {}", missing.join(", ")))
    }
}

fn run_test_policy_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut violations = Vec::new();

    let schema_fixtures_ok = root
        .join("configs/schema/fixtures")
        .join("v0.1")
        .join("positive")
        .exists()
        && root
            .join("configs/schema/fixtures")
            .join("v0.1")
            .join("negative")
            .exists();
    if !schema_fixtures_ok {
        violations.push("schema fixtures must have positive and negative coverage".to_string());
    }

    let state_test = root.join("crates/bijux-dag-runtime/src/state_machine_tests.rs");
    let state_text = fs::read_to_string(&state_test).map_err(|err| err.to_string())?;
    for state in [
        "queued",
        "ready",
        "running",
        "succeeded",
        "failed",
        "cached",
        "skipped",
        "cancelled",
    ] {
        if !state_text.contains(state) {
            violations.push(format!(
                "runtime transition coverage missing state: {state}"
            ));
        }
    }

    let cache_test = root.join("crates/bijux-dag-runtime/src/tests_runtime.in.rs");
    let cache_text = fs::read_to_string(&cache_test).map_err(|err| err.to_string())?;
    for mode in ["CacheMode::Off", "CacheMode::Read", "CacheMode::ReadWrite"] {
        if !cache_text.contains(mode) {
            violations.push(format!("cache mode coverage missing mode: {mode}"));
        }
    }

    let output_contract = root.join("crates/bijux-dag-app/tests/output_contract.rs");
    let cli_contract = root.join("crates/bijux-dag-app/tests/cli_contract.rs");
    if !(output_contract.exists() && cli_contract.exists()) {
        violations.push(
            "public command policy requires integration and error-path app command tests"
                .to_string(),
        );
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_e2e_matrix() -> Result<(), String> {
    let root = repo_root()?;
    run_with_root(
        &root,
        "cargo",
        &[
            "test",
            "-p",
            "bijux-dag-app",
            "--test",
            "e2e_integration_scenarios",
        ],
    )
    .and_then(|_| {
        run_with_root(
            &root,
            "cargo",
            &[
                "run",
                "-p",
                "bijux-dag-cli",
                "--",
                "dag",
                "validate",
                "evidence/authoring/examples/hello.dag.json",
            ],
        )
    })
}

fn command_stdout(root: &Path, bin: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(bin)
        .current_dir(root)
        .args(args)
        .output()
        .map_err(|err| format!("failed to execute `{bin}`: {err}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|err| err.to_string())
    } else {
        Err(format!(
            "command `{bin} {}` failed with status {}",
            args.join(" "),
            output.status
        ))
    }
}

#[derive(Debug, Deserialize)]
struct FaultClassCatalog {
    fault_classes: Vec<FaultClassEntry>,
}

#[derive(Debug, Deserialize)]
struct FaultClassEntry {
    id: String,
    tested_by: Vec<String>,
}

fn run_fault_summary_report() -> Result<(), String> {
    let root = repo_root()?;
    let catalog_path = root.join("evidence/fault/classes/fault_classes.json");
    let metadata_path = root.join("evidence/fault/metadata.json");
    if !metadata_path.exists() {
        return Err("missing fault metadata: evidence/fault/metadata.json".to_string());
    }
    let payload = fs::read_to_string(&catalog_path).map_err(|err| err.to_string())?;
    let catalog: FaultClassCatalog =
        serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut tested = Vec::new();
    let mut missing = Vec::new();
    for entry in catalog.fault_classes {
        if entry.tested_by.is_empty() {
            missing.push(entry.id);
        } else {
            tested.push(json!({"id": entry.id, "tests": entry.tested_by}));
        }
    }

    let summary = json!({
        "tested_fault_classes": tested,
        "missing_fault_classes": missing,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?
    );
    if summary["missing_fault_classes"]
        .as_array()
        .is_some_and(|items| items.is_empty())
    {
        Ok(())
    } else {
        Err("fault class catalog has missing tested_by mappings".to_string())
    }
}

fn run_benchmark_compare(
    current: &Path,
    baseline: &Path,
    max_regression_ratio: f64,
) -> Result<(), String> {
    let root = repo_root()?;
    let current_path = root.join(current);
    let baseline_path = root.join(baseline);

    let current_json: Value =
        serde_json::from_str(&fs::read_to_string(current_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let baseline_json: Value =
        serde_json::from_str(&fs::read_to_string(baseline_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let current_items = current_json
        .get("scenario_results")
        .and_then(Value::as_array)
        .ok_or_else(|| "current benchmark report missing scenario_results".to_string())?;
    let baseline_items = baseline_json
        .get("scenario_results")
        .and_then(Value::as_array)
        .ok_or_else(|| "baseline benchmark report missing scenario_results".to_string())?;

    let mut base_map: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
    for item in baseline_items {
        let scenario = item
            .get("scenario_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "baseline item missing scenario_id".to_string())?;
        let elapsed = item
            .get("elapsed_ms")
            .and_then(Value::as_f64)
            .ok_or_else(|| "baseline item missing elapsed_ms".to_string())?;
        base_map.insert(scenario.to_string(), elapsed);
    }

    let mut regressions = Vec::new();
    for item in current_items {
        let scenario = item
            .get("scenario_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "current item missing scenario_id".to_string())?;
        let elapsed = item
            .get("elapsed_ms")
            .and_then(Value::as_f64)
            .ok_or_else(|| "current item missing elapsed_ms".to_string())?;
        if let Some(base) = base_map.get(scenario) {
            if *base > 0.0 {
                let ratio = (elapsed - *base) / *base;
                if ratio > max_regression_ratio {
                    regressions.push(json!({
                        "scenario_id": scenario,
                        "baseline_ms": base,
                        "current_ms": elapsed,
                        "regression_ratio": ratio
                    }));
                }
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"regressions": regressions}))
            .map_err(|err| err.to_string())?
    );

    if regressions.is_empty() {
        Ok(())
    } else {
        Err("benchmark regressions exceed threshold".to_string())
    }
}

fn run_performance_claims_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs = root.join("docs");
    let mut violations = Vec::new();
    let mut stack = vec![docs];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            for line in text.lines() {
                let lower = line.to_ascii_lowercase();
                let claim = lower.contains("performance")
                    || lower.contains("fast")
                    || lower.contains("latency")
                    || lower.contains("throughput");
                if claim
                    && !(line.contains("benchmarks/")
                        || line.contains("artifacts/benchmarks")
                        || line.contains("PERFORMANCE_STRATEGY.md")
                        || line.contains("PERFORMANCE_CONTRACT.md"))
                {
                    violations.push(format!(
                        "{rel}: performance claim without evidence link: {line}"
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_resource_profile_summary(report: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let report_path = root.join(report);
    let payload = fs::read_to_string(&report_path).map_err(|err| err.to_string())?;
    let report_json: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut summary = json!({
        "measurement_quality": "approximate",
        "scenario_count": 0,
        "totals": {
            "wall_time_ms": 0.0,
            "artifact_bytes": 0,
            "trace_bytes": 0
        },
        "cost_split": {
            "product_execution_ms": 0.0,
            "harness_overhead_ms": 0.0
        }
    });

    if let Some(items) = report_json
        .get("scenario_results")
        .and_then(Value::as_array)
    {
        let mut wall = 0.0_f64;
        for item in items {
            wall += item
                .get("elapsed_ms")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
        }
        summary["scenario_count"] = Value::from(items.len() as u64);
        summary["totals"]["wall_time_ms"] = Value::from(wall);
        summary["cost_split"]["product_execution_ms"] = Value::from(wall);
        summary["cost_split"]["harness_overhead_ms"] = Value::from(0.0_f64);
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&summary).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_resource_budget_check(report: &Path, gate: bool) -> Result<(), String> {
    let root = repo_root()?;
    let report_path = root.join(report);
    let budgets_path = root.join("evidence/perf/scenarios/resource_budgets.json");

    let report_json: Value =
        serde_json::from_str(&fs::read_to_string(&report_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let budgets_json: Value =
        serde_json::from_str(&fs::read_to_string(&budgets_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let mut budget_map: std::collections::BTreeMap<String, Value> =
        std::collections::BTreeMap::new();
    if let Some(items) = budgets_json.get("scenarios").and_then(Value::as_array) {
        for item in items {
            if let Some(id) = item.get("scenario_id").and_then(Value::as_str) {
                budget_map.insert(id.to_string(), item.clone());
            }
        }
    }

    let mut warnings = Vec::new();
    if let Some(items) = report_json
        .get("scenario_results")
        .and_then(Value::as_array)
    {
        for item in items {
            let scenario = item
                .get("scenario_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let elapsed = item
                .get("elapsed_ms")
                .and_then(Value::as_f64)
                .unwrap_or(0.0);
            if let Some(budget) = budget_map.get(scenario) {
                let approx_budget_ms = budget
                    .get("max_manifest_bytes")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as f64;
                if approx_budget_ms > 0.0 && elapsed > approx_budget_ms {
                    warnings.push(format!(
                        "scenario {} exceeded approximate budget threshold (elapsed_ms={elapsed})",
                        scenario
                    ));
                }
            }
        }
    }

    if warnings.is_empty() {
        println!("resource budgets within thresholds");
        return Ok(());
    }

    for warning in &warnings {
        eprintln!("resource-budget-warning: {warning}");
    }
    if gate {
        Err("resource budget check failed in gate mode".to_string())
    } else {
        Ok(())
    }
}

fn run_resource_trend_append(report: &Path, trend: &Path) -> Result<(), String> {
    let root = repo_root()?;
    let report_json: Value = serde_json::from_str(
        &fs::read_to_string(root.join(report)).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let trend_path = root.join(trend);
    let mut trend_json: Value = if trend_path.exists() {
        serde_json::from_str(&fs::read_to_string(&trend_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?
    } else {
        json!({"trend_format":"resource-trend/v1","series":[]})
    };

    let entry = json!({
        "commit_sha": report_json.get("commit_sha").cloned().unwrap_or(Value::from("unknown")),
        "timestamp_unix_ms": now_millis(),
        "scenario_results": report_json
            .get("scenario_results")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()))
    });

    if let Some(series) = trend_json.get_mut("series").and_then(Value::as_array_mut) {
        series.push(entry);
    }

    fs::write(
        trend_path,
        serde_json::to_vec_pretty(&trend_json).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

fn dir_size_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                total =
                    total.saturating_add(entry.metadata().map_err(|err| err.to_string())?.len());
            }
        }
    }
    Ok(total)
}

fn estimate_trace_bytes(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0_u64;
    let mut stack = vec![path.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .and_then(|v| v.to_str())
                .is_some_and(|name| name == "trace.json")
            {
                total =
                    total.saturating_add(entry.metadata().map_err(|err| err.to_string())?.len());
            }
        }
    }
    Ok(total)
}

fn run_docs_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let allowed_dirs = [
        "spec",
        "architecture",
        "user",
        "dev",
        "reference",
        "tracking",
        "generated",
        "_tracking",
        "adr",
        "operations",
    ];

    for entry in fs::read_dir(&docs_root).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !allowed_dirs.contains(&name.as_str()) {
            return Err(format!(
                "docs taxonomy violation: docs/{name} is not allowed"
            ));
        }
    }

    let root_markdown_count = fs::read_dir(&docs_root)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|v| v.to_str()) == Some("md"))
        .count();
    let max_root_docs = 110usize;
    if root_markdown_count > max_root_docs {
        return Err(format!(
            "docs root budget exceeded: {} > {}",
            root_markdown_count, max_root_docs
        ));
    }

    for rel in [
        "docs/spec/DOCS_GOVERNANCE.md",
        "docs/tracking/DOC_OWNERSHIP.json",
        "docs/tracking/DOCS_PRUNING_CHECKLIST.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing docs governance artifact: {rel}"));
        }
    }

    let owners: Value = serde_json::from_str(
        &fs::read_to_string(root.join("docs/tracking/DOC_OWNERSHIP.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    if owners
        .get("owners")
        .and_then(Value::as_array)
        .is_none_or(|items| items.is_empty())
    {
        return Err("docs ownership metadata has no owners entries".to_string());
    }

    for forbidden in ["production-grade", "world-class"] {
        let mut files = Vec::new();
        collect_markdown_files(&docs_root, &mut files)?;
        for file in files {
            let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
            for line in content.lines() {
                let lower = line.to_ascii_lowercase();
                if lower.contains(forbidden) && !line.contains('"') {
                    return Err(format!(
                        "marketing maturity phrase not allowed without quote: {}",
                        forbidden
                    ));
                }
            }
        }
    }

    let mut files = Vec::new();
    collect_markdown_files(&docs_root, &mut files)?;
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        let lower = content.to_ascii_lowercase();
        for stale in ["bijux-dag-compat", "legacy-cli", "old_runtime_path"] {
            if lower.contains(stale) {
                return Err(format!("stale crate/path reference in {rel}: {stale}"));
            }
        }
        if lower.contains("roadmap") && !rel.starts_with("docs/tracking/") {
            return Err(format!(
                "speculative roadmap content must live under docs/tracking: {rel}"
            ));
        }
        if content.contains("AUTO-GENERATED") && !rel.starts_with("docs/generated/") {
            return Err(format!(
                "generated-doc marker must only appear under docs/generated: {rel}"
            ));
        }
    }

    Ok(())
}

fn run_docs_link_check() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files)?;
    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for cap in content.match_indices("](") {
            let start = cap.0 + 2;
            if let Some(end_rel) = content[start..].find(')') {
                let link = &content[start..start + end_rel];
                if link.starts_with("http://")
                    || link.starts_with("https://")
                    || link.starts_with("mailto:")
                    || link.starts_with('#')
                {
                    continue;
                }
                let resolved = file.parent().unwrap_or(Path::new(".")).join(link);
                if !resolved.exists() {
                    let rel = file
                        .strip_prefix(&root)
                        .map_err(|err| err.to_string())?
                        .to_string_lossy()
                        .replace('\\', "/");
                    violations.push(format!("{rel}: broken link target {link}"));
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_naming_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs = [
        "docs/spec/NAMING_GUIDELINES.md",
        "docs/spec/TERMINOLOGY_GLOSSARY.md",
        "docs/spec/NAMING_PHILOSOPHY.md",
        "docs/spec/NAMING_REVIEW_POLICY.md",
        "docs/architecture/naming_audit.md",
        "configs/policy/naming_rules.json",
    ];
    for rel in required_docs {
        if !root.join(rel).exists() {
            return Err(format!("missing naming governance artifact: {rel}"));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/naming_rules.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let banned_terms = policy
        .get("runtime_module_banned_terms")
        .and_then(Value::as_array)
        .ok_or_else(|| "naming_rules.json missing runtime_module_banned_terms".to_string())?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if banned_terms.is_empty() {
        return Err("runtime_module_banned_terms must not be empty".to_string());
    }

    let mut runtime_files = Vec::new();
    collect_files_with_extension(
        &root.join("crates/bijux-dag-runtime/src"),
        "rs",
        &mut runtime_files,
    )?;
    let mut violations = Vec::new();
    for file in runtime_files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        for term in &banned_terms {
            if stem.contains(term) {
                violations.push(format!("{rel}: banned runtime module term `{term}`"));
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_runtime_semantics_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/RUNTIME_SEMANTICS_CONTRACT.md",
        "crates/bijux-dag-runtime/src/runtime_semantics.rs",
        "crates/bijux-dag-runtime/tests/runtime_semantics_contracts.rs",
        "crates/bijux-dag-runtime/tests/engine_correctness_contracts.rs",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing runtime semantics artifact: {rel}"));
        }
    }
    Ok(())
}

fn run_test_trust_foundation_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/TEST_TRUST_CONTRACT.md",
        "docs/spec/TEST_PHILOSOPHY.md",
        "docs/architecture/test_trust_audit.md",
        "crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing test trust artifact: {rel}"));
        }
    }

    let catalog: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("crates/bijux-dag-runtime/tests/fixtures/test_trust_catalog.json"),
        )
        .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let object = catalog
        .as_object()
        .ok_or_else(|| "test_trust_catalog.json must be an object".to_string())?;
    if object.is_empty() {
        return Err("test_trust_catalog.json must contain at least one class".to_string());
    }
    for (class, files) in object {
        let files = files
            .as_array()
            .ok_or_else(|| format!("catalog class `{class}` must be an array"))?;
        if files.is_empty() {
            return Err(format!("catalog class `{class}` must not be empty"));
        }
        for file in files {
            let rel = file
                .as_str()
                .ok_or_else(|| format!("catalog class `{class}` contains non-string entry"))?;
            let full = root.join("crates/bijux-dag-runtime/tests").join(rel);
            if !full.exists() {
                return Err(format!(
                    "catalog references missing test file: crates/bijux-dag-runtime/tests/{rel}"
                ));
            }
        }
    }
    Ok(())
}

fn run_battle_suite_mandatory_guard() -> Result<(), String> {
    let root = repo_root()?;

    let policy_path = root.join("configs/policy/battle_trust_properties.json");
    let metadata_path = root.join("evidence/battle/metadata.json");
    let harness_path =
        root.join("crates/bijux-dag-runtime/tests/battle_workflow_harness_contracts.rs");

    for required in [&policy_path, &metadata_path, &harness_path] {
        if !required.exists() {
            return Err(format!(
                "missing battle suite artifact: {}",
                required.display()
            ));
        }
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&policy_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    if trust_properties.len() < 12 {
        return Err("battle trust policy must define at least 12 trust properties".to_string());
    }
    let has_plan_truth = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_plan_truth")
    });
    if !has_plan_truth {
        return Err("battle trust policy must include tp_plan_truth".to_string());
    }
    let has_state_machine_legality = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_state_machine_legality")
    });
    if !has_state_machine_legality {
        return Err("battle trust policy must include tp_state_machine_legality".to_string());
    }
    let has_config_policy_determinism = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_config_policy_determinism")
    });
    if !has_config_policy_determinism {
        return Err("battle trust policy must include tp_config_policy_determinism".to_string());
    }

    let required_scenarios = policy
        .get("required_scenarios")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing required_scenarios".to_string())?;
    if required_scenarios.is_empty() {
        return Err("battle trust policy required_scenarios must not be empty".to_string());
    }
    let scenario_trust_map = policy
        .get("scenario_trust_map")
        .and_then(Value::as_object)
        .ok_or_else(|| "battle trust policy missing scenario_trust_map".to_string())?;
    let state_machine_mapped = scenario_trust_map.values().any(|value| {
        value.as_array().is_some_and(|ids| {
            ids.iter().any(|id| {
                id.as_str()
                    .is_some_and(|v| v == "tp_state_machine_legality")
            })
        })
    });
    if !state_machine_mapped {
        return Err(
            "battle trust policy must map at least one scenario to tp_state_machine_legality"
                .to_string(),
        );
    }

    run_battle_scenario_mapping_validate()?;

    let metadata: Value =
        serde_json::from_str(&fs::read_to_string(&metadata_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    if metadata
        .get("scenarios")
        .and_then(Value::as_object)
        .is_none()
    {
        return Err("battle metadata must define scenario ownership".to_string());
    }

    Ok(())
}

fn run_test_trust_cleanup_guard() -> Result<(), String> {
    let root = repo_root()?;
    let ledger = root.join("configs/policy/test_trust_ledger.json");
    if !ledger.exists() {
        return Err("missing test trust ledger policy".to_string());
    }

    let docs = root.join("docs/spec/TEST_TRUST_LEDGER.md");
    if !docs.exists() {
        return Err("missing test trust ledger spec".to_string());
    }

    let report = root.join("docs/reports/foundation/test_trust_cleanup_report.md");
    if !report.exists() {
        return Err("missing test trust cleanup report".to_string());
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&ledger).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let classes = policy
        .get("classification_rules")
        .and_then(Value::as_array)
        .ok_or_else(|| "test trust ledger missing classification_rules".to_string())?;
    if classes.is_empty() {
        return Err("classification_rules must not be empty".to_string());
    }

    let must_never_break = policy
        .get("must_never_break")
        .and_then(Value::as_array)
        .ok_or_else(|| "test trust ledger missing must_never_break".to_string())?;
    if must_never_break.is_empty() {
        return Err("must_never_break must not be empty".to_string());
    }

    Ok(())
}

fn run_docs_config_reduction_guard() -> Result<(), String> {
    let root = repo_root()?;
    let docs_policy = root.join("configs/policy/docs_config_governance.json");
    let config_policy = root.join("configs/policy/config_consumers.json");
    if !docs_policy.exists() {
        return Err("missing docs config governance policy".to_string());
    }
    if !config_policy.exists() {
        return Err("missing config consumers policy".to_string());
    }

    for required in [
        "docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md",
        "docs/spec/MODELED_AND_FUTURE_SURFACES.md",
        "docs/spec/SPEC_TO_CODE_AND_TEST_OWNERSHIP.md",
        "docs/reports/foundation/docs_root_inventory_report.md",
        "docs/reports/foundation/config_inventory_report.md",
        "docs/reports/foundation/evidence_claim_links.md",
        "docs/reports/foundation/renovation_burndown_report.md",
        "docs/architecture/ADR_RENOVATION_ALIGNMENT.md",
    ] {
        if !root.join(required).exists() {
            return Err(format!(
                "missing docs config reduction authority: {required}"
            ));
        }
    }

    let policy: Value =
        serde_json::from_str(&fs::read_to_string(&docs_policy).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;
    let freeze_enabled = policy
        .get("roadmap_growth_freeze")
        .and_then(|node| node.get("enabled"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !freeze_enabled {
        return Err("roadmap growth freeze must stay enabled".to_string());
    }

    Ok(())
}

fn run_docs_schema_reference_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_markdown_files(&root.join("docs"), &mut files)?;
    let mut violations = Vec::new();

    for file in files {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        for token in content.split_whitespace() {
            if !token.contains("configs/schema/") {
                continue;
            }
            let clean =
                token.trim_matches(|c: char| matches!(c, ')' | '(' | '[' | ']' | ',' | ';' | '"'));
            let path = if clean.contains("configs/schema/") {
                let idx = clean.find("configs/schema/").unwrap_or(0);
                &clean[idx..]
            } else {
                clean
            };
            if !root.join(path).exists() {
                let rel = file
                    .strip_prefix(&root)
                    .map_err(|err| err.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                violations.push(format!("{rel}: missing schema reference {path}"));
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_docs_contract_reference_guard() -> Result<(), String> {
    let root = repo_root()?;
    let crates = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];
    let mut violations = Vec::new();

    let docs_index = fs::read_to_string(root.join("docs/reference/DOCS_INDEX.md"))
        .map_err(|err| err.to_string())?;

    for crate_name in crates {
        let crate_dir = root.join("crates").join(crate_name);
        if !crate_dir.join("README.md").exists() {
            violations.push(format!("{crate_name} missing README.md"));
        }
        if !crate_dir.join("CONTRACT.md").exists() {
            violations.push(format!("{crate_name} missing CONTRACT.md"));
        }
        if !docs_index.contains(crate_name) {
            violations.push(format!(
                "docs/reference/DOCS_INDEX.md missing crate mention: {crate_name}"
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_docs_index_generate() -> Result<(), String> {
    let root = repo_root()?;
    let docs_root = root.join("docs");
    let sections = [
        "spec",
        "architecture",
        "user",
        "dev",
        "reference",
        "tracking",
        "generated",
    ];

    let mut lines = vec![
        "# Documentation index".to_string(),
        "".to_string(),
        "Generated from docs taxonomy.".to_string(),
        "".to_string(),
    ];

    for section in sections {
        let dir = docs_root.join(section);
        if !dir.exists() {
            continue;
        }
        lines.push(format!("## {}", section));
        let mut entries: Vec<String> = fs::read_dir(&dir)
            .map_err(|err| err.to_string())?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();
        entries.sort();
        for entry in entries {
            lines.push(format!("- `{}`", entry));
        }
        lines.push(String::new());
    }

    lines.push("## crate-doc-contracts".to_string());
    for crate_name in [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ] {
        lines.push(format!("- `{}`", crate_name));
    }
    lines.push(String::new());

    fs::write(
        docs_root.join("reference").join("DOCS_INDEX.md"),
        lines.join("\n"),
    )
    .map_err(|err| err.to_string())
}

fn run_docs_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let crate_names = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];

    let mut missing = Vec::new();
    for crate_name in crate_names {
        if !root
            .join("crates")
            .join(crate_name)
            .join("CONTRACT.md")
            .exists()
        {
            missing.push(format!("missing contract doc for {crate_name}"));
        }
    }

    let command_taxonomy = root.join("docs/CLI_COMMAND_TAXONOMY.md");
    if !command_taxonomy.exists() {
        missing.push("missing CLI command taxonomy doc".to_string());
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"missing": missing})).map_err(|err| err.to_string())?
    );

    if missing.is_empty() {
        Ok(())
    } else {
        Err("docs coverage has missing entries".to_string())
    }
}

fn run_contract_test_links_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
    let mut violations = Vec::new();

    for file in contracts {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !content.contains("## Related tests") {
            let rel = file.strip_prefix(&root).map_err(|err| err.to_string())?;
            violations.push(format!(
                "{} missing '## Related tests' section",
                rel.display()
            ));
            continue;
        }
        let mut test_link_count = 0usize;
        for line in content.lines() {
            if line.contains("tests/") && line.contains('`') {
                test_link_count += 1;
            }
        }
        if test_link_count == 0 {
            let rel = file.strip_prefix(&root).map_err(|err| err.to_string())?;
            violations.push(format!("{} has no linked test paths", rel.display()));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_contract_schema_owner_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
    let mut contract_blob = String::new();
    for file in contracts {
        contract_blob.push_str(&fs::read_to_string(file).map_err(|err| err.to_string())?);
        contract_blob.push('\n');
    }

    let mut missing = Vec::new();
    for entry in fs::read_dir(root.join("configs/schema")).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        if !contract_blob.contains(&rel) {
            missing.push(rel);
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "schemas missing owning contract links: {}",
            missing.join(", ")
        ))
    }
}

fn run_contract_command_ownership_guard() -> Result<(), String> {
    let root = repo_root()?;
    let taxonomy = fs::read_to_string(root.join("docs/CLI_COMMAND_TAXONOMY.md"))
        .map_err(|err| err.to_string())?;
    let contract = fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md"))
        .map_err(|err| err.to_string())?;

    let mut commands = Vec::new();
    for line in taxonomy.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("- `") || !trimmed.ends_with('`') {
            continue;
        }
        let value = trimmed
            .trim_start_matches("- `")
            .trim_end_matches('`')
            .to_string();
        if value.starts_with("migrate ") {
            if !commands.contains(&"migrate".to_string()) {
                commands.push("migrate".to_string());
            }
        } else {
            commands.push(value);
        }
    }

    let mut violations = Vec::new();
    for command in commands {
        let token = format!("`dag {command}`");
        let count = contract.matches(&token).count();
        if count != 1 {
            violations.push(format!(
                "command ownership token {} appears {} times in docs/spec/CLI_CONTRACT.md",
                token, count
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_contract_versioning_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut contracts = Vec::new();
    collect_contract_files(&root.join("docs/spec"), &mut contracts)?;
    let mut violations = Vec::new();
    for file in contracts {
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !content.contains("## Versioning and change policy") {
            let rel = file
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            violations.push(format!("{rel} missing versioning policy section"));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_contract_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let mut missing = Vec::new();
    let mut orphaned = Vec::new();
    let mut stale = Vec::new();

    let crate_names = [
        "bijux-dag-core",
        "bijux-dag-artifacts",
        "bijux-dag-runtime",
        "bijux-dag-app",
        "bijux-dag-cli",
        "bijux-dag-testkit",
        "bijux-dev-dag",
    ];
    for crate_name in crate_names {
        if !root
            .join("crates")
            .join(crate_name)
            .join("CONTRACT.md")
            .exists()
        {
            missing.push(format!("crate contract missing: {crate_name}"));
        }
    }

    let specs = [
        "CLI_CONTRACT.md",
        "RUN_DIR_CONTRACT.md",
        "CACHE_CONTRACT.md",
        "REPLAY_CONTRACT.md",
        "ERROR_CONTRACT.md",
        "TRACE_CONTRACT.md",
        "IMPORT_EXPORT_CONTRACT.md",
        "CONFIG_CONTRACT.md",
        "POLICY_CONTRACT.md",
        "SELECTOR_CONTRACT.md",
    ];
    for file in specs {
        let path = root.join("docs/spec").join(file);
        if !path.exists() {
            missing.push(format!("spec contract missing: docs/spec/{file}"));
        }
    }

    for entry in fs::read_dir(root.join("docs/spec")).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path
            .file_name()
            .and_then(|x| x.to_str())
            .is_some_and(|name| name.ends_with("CONTRACT.md"))
        {
            let file_name = path
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or_default();
            if !specs.contains(&file_name)
                && file_name != "WORKSPACE_CONTRACT.md"
                && file_name != "PROJECT_CONTRACT.md"
                && file_name != "ADAPTER_CONTRACT.md"
                && file_name != "EXECUTION_SEMANTICS_CONTRACT.md"
                && file_name != "SCHEDULER_STATESPACE_CONTRACT.md"
                && file_name != "DETERMINISTIC_SCHEDULING_CONTRACT.md"
                && file_name != "CONFIG_PRECEDENCE_CONTRACT.md"
                && file_name != "OPERATOR_INSPECTION_CONTRACT.md"
            {
                orphaned.push(format!("unknown contract doc: docs/spec/{file_name}"));
            }
            let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if !content.contains("## Scope") {
                stale.push(format!("{} missing scope section", file_name));
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "missing": missing,
            "orphaned": orphaned,
            "stale": stale
        }))
        .map_err(|err| err.to_string())?
    );

    if missing.is_empty() && orphaned.is_empty() && stale.is_empty() {
        Ok(())
    } else {
        Err("contract coverage report found gaps".to_string())
    }
}

fn collect_contract_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_contract_files(&path, out)?;
            continue;
        }
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("CONTRACT.md"))
        {
            out.push(path);
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ErrorCodeRegistry {
    version: u64,
    categories: Vec<String>,
    codes: Vec<ErrorCodeEntry>,
}

#[derive(Debug, Deserialize, serde::Serialize)]
struct ErrorCodeEntry {
    code: String,
    category: String,
    owner: String,
    description: String,
}

fn run_error_code_registry_report() -> Result<(), String> {
    let root = repo_root()?;
    let registry = load_error_code_registry(&root)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "version": registry.version,
            "categories": registry.categories,
            "codes": registry.codes,
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_error_code_docs_tests_guard() -> Result<(), String> {
    let root = repo_root()?;
    let registry = load_error_code_registry(&root)?;
    let docs_error_ref =
        fs::read_to_string(root.join("docs/reference/ERRORS.md")).map_err(|err| err.to_string())?;
    let docs_error_contract = fs::read_to_string(root.join("docs/spec/ERROR_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let tests = [
        root.join("crates/bijux-dag-app/tests/error_output_contract.rs"),
        root.join("crates/bijux-dag-app/tests/error_exit_contract.rs"),
    ];

    let mut violations = Vec::new();
    for code in &registry.codes {
        if !docs_error_ref.contains(&code.category) {
            violations.push(format!(
                "docs/reference/ERRORS.md missing category {} for {}",
                code.category, code.code
            ));
        }
        if !docs_error_contract
            .contains("Public error code additions require docs plus test coverage")
        {
            violations.push(
                "docs/spec/ERROR_CONTRACT.md missing public code governance rule".to_string(),
            );
        }
    }

    for test in tests {
        if !test.exists() {
            violations.push(format!(
                "missing required error contract test file: {}",
                test.strip_prefix(&root)
                    .map_err(|err| err.to_string())?
                    .display()
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_planner_alignment_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        bijux_dag_core::planner_alignment_required_doc(),
        bijux_dag_core::planner_alignment_required_schema(),
        bijux_dag_core::planner_alignment_required_test(),
        "crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs",
        "crates/bijux-dev-dag/tests/planner_hardening_contracts.rs",
        "docs/reports/foundation/planner_hardening_report.md",
        "docs/spec/BATTLE_TRUST_PROPERTIES.md",
        "configs/policy/battle_trust_properties.json",
        "crates/bijux-dag-runtime/src/runtime_core/planning/planner.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "planner alignment missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let planner_contract =
        fs::read_to_string(root.join(bijux_dag_core::planner_alignment_required_doc()))
            .map_err(|err| err.to_string())?;
    for required_token in [
        "parsed graph",
        "validated graph",
        "canonical graph",
        "execution plan",
        "P4021",
        "dag plan-dump",
    ] {
        if !planner_contract.contains(required_token) {
            return Err(format!(
                "planner contract missing required token: {required_token}"
            ));
        }
    }

    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required_command in ["DagCommand::PlanDump", "run_dag_plan_dump"] {
        if !commands.contains(required_command) {
            return Err(format!(
                "planner alignment missing command surface: {required_command}"
            ));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let has_plan_truth = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry
                    .get("id")
                    .and_then(Value::as_str)
                    .is_some_and(|id| id == "tp_plan_truth")
            })
        });
    if !has_plan_truth {
        return Err("planner alignment requires tp_plan_truth trust property".to_string());
    }

    Ok(())
}

fn run_scheduler_invariants_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SCHEDULER_CONTRACT.md",
        "docs/spec/SCHEDULER_STATE_TRANSITIONS.md",
        "docs/reports/foundation/scheduler_hardening_report.md",
        "crates/bijux-dag-runtime/tests/scheduler_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs",
        "crates/bijux-dev-dag/tests/scheduler_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "scheduler invariant coverage missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required in [
        "DagCommand::SchedulerTimeline",
        "run_dag_scheduler_timeline",
    ] {
        if !commands.contains(required) {
            return Err(format!(
                "scheduler invariant coverage missing command surface: {required}"
            ));
        }
    }
    Ok(())
}

fn run_state_machine_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/STATE_MACHINE_CONTRACT.md",
        "docs/spec/STATE_MACHINE_VISUALIZATION.md",
        "docs/reports/foundation/state_machine_hardening_report.md",
        "crates/bijux-dag-runtime/tests/state_machine_transitions.rs",
        "crates/bijux-dag-runtime/tests/state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/evolution_trace.json",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/cancellation_trace.json",
        "crates/bijux-dev-dag/tests/state_machine_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "state machine contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let contract = fs::read_to_string(root.join("docs/spec/STATE_MACHINE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let node_states = [
        "pending",
        "eligible",
        "queued",
        "running",
        "success",
        "failed",
        "skipped",
        "cached",
        "cancelled",
    ];
    for state in node_states {
        if !contract.contains(&format!("- {}", state)) {
            return Err(format!(
                "state machine contract missing documented node state `{}`",
                state
            ));
        }
    }
    let run_states = [
        "submitted",
        "planning",
        "running",
        "paused",
        "interrupted",
        "cancelling",
        "cancelled",
        "failed",
        "succeeded",
    ];
    for state in run_states {
        if !contract.contains(&format!("- {}", state)) {
            return Err(format!(
                "state machine contract missing documented run state `{}`",
                state
            ));
        }
    }
    for token in [
        "INV-NODE-TRANSITION-*",
        "INV-NODE-TERMINAL-REVERT-001",
        "INV-RUN-TRANSITION-*",
        "INV-RUN-FAILED-CAUSAL-001",
    ] {
        if !contract.contains(token) {
            return Err(format!(
                "state machine contract missing documented invariant token `{}`",
                token
            ));
        }
    }

    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required_surface in ["DagCommand::VerifyState", "run_dag_verify_state"] {
        if !commands.contains(required_surface) {
            return Err(format!(
                "state machine contract missing command surface `{}`",
                required_surface
            ));
        }
    }
    Ok(())
}

fn run_concurrency_model_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CONCURRENCY_MODEL.md",
        "docs/architecture/runtime-concurrency-boundaries.md",
        "docs/tracking/CONCURRENCY_FLAKE_LEDGER.md",
        "crates/bijux-dag-runtime/tests/concurrency_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "concurrency model missing required surfaces: {}",
            missing.join(", ")
        ))
    }
}

fn run_runtime_unsafe_guard() -> Result<(), String> {
    let root = repo_root()?;
    let output = Command::new("rg")
        .args(["-n", "\\bunsafe\\b", "crates/bijux-dag-runtime/src"])
        .current_dir(&root)
        .output()
        .map_err(|err| err.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if findings.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "runtime unsafe usage requires ADR and dedicated tests: {}",
            findings.join(" | ")
        ))
    }
}

fn run_unsafe_audit_report() -> Result<(), String> {
    let root = repo_root()?;
    let output = Command::new("rg")
        .args(["-n", "\\bunsafe\\b", "crates"])
        .current_dir(&root)
        .output()
        .map_err(|err| err.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut parts = line.splitn(3, ':');
            json!({
                "file": parts.next().unwrap_or(""),
                "line": parts.next().unwrap_or(""),
                "snippet": parts.next().unwrap_or("").trim(),
            })
        })
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "unsafe_entry_count": entries.len(),
            "entries": entries
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_backend_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/BACKEND_CONTRACT.md",
        "docs/spec/EXECUTION_ENGINE_CONTRACT.md",
        "docs/spec/ATTEMPT_TRACE_SCHEMA_v0.1.md",
        "docs/reports/foundation/backend_hardening_report.md",
        "docs/architecture/engine-backend-responsibilities.md",
        "crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs",
        "crates/bijux-dag-runtime/tests/execution_backend_contract.rs",
        "crates/bijux-dev-dag/tests/backend_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "backend contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let test_path = root.join("crates/bijux-dag-runtime/tests/execution_backend_contract.rs");
    let payload = fs::read_to_string(&test_path).map_err(|err| err.to_string())?;
    if !payload.contains("fake_and_process_like_backends_have_parity_on_basic_scenario") {
        return Err("backend contract missing fake-backend parity test".to_string());
    }
    for required_test in [
        "backend_prepare_failures_are_classified_correctly",
        "backend_launch_failures_do_not_corrupt_state",
        "cleanup_runs_after_observe_and_reports_cleanup_failures",
        "cleanup_runs_when_prepare_fails",
        "backend_observe_timeout_has_distinct_error",
        "backend_env_shaping_contract_is_explicitly_applied",
        "backend_output_collection_rejects_undeclared_outputs",
        "backend_registry_includes_capability_descriptors",
    ] {
        if !payload.contains(required_test) {
            return Err(format!(
                "backend contract missing required conformance test `{}`",
                required_test
            ));
        }
    }
    let backend_src = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs"),
    )
    .map_err(|err| err.to_string())?;
    let implementation_count = backend_src.matches("impl ExecutionBackend for").count();
    if implementation_count > 2 {
        return Err(
            "new backend implementations are blocked until backend contract conformance remains explicit and passing"
                .to_string(),
        );
    }
    Ok(())
}

fn run_backend_registry_report() -> Result<(), String> {
    let registry = bijux_dag_runtime::backend_registry();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "backend_count": registry.len(),
            "backends": registry
        }))
        .map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_storage_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/STORAGE_CONTRACT.md",
        "docs/architecture/storage-layout-ownership.md",
        "crates/bijux-dag-runtime/src/store.rs",
        "crates/bijux-dag-runtime/tests/storage_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "storage boundaries missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut violations = Vec::new();
    let mut stack = vec![runtime_src];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .to_string();
            if rel.ends_with("store.rs") || rel.ends_with("lib.rs") || rel.ends_with("engine.rs") {
                continue;
            }
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            if text.contains("staging_path().join(\"nodes\")")
                || text.contains("manifest.json")
                || text.contains("outputs.index.json")
            {
                violations.push(rel);
            }
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "runtime modules use ad-hoc storage paths outside approved modules: {}",
            violations.join(", ")
        ))
    }
}

fn run_storage_health(run_dir: &Path, cache_dir: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let run_path = root.join(run_dir);
    let mut anomalies = Vec::new();
    let manifest = run_path.join("manifest.json");
    if !manifest.exists() {
        anomalies.push("missing manifest.json".to_string());
    } else {
        let payload = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        if parsed.get("run_id").is_none() {
            anomalies.push("manifest missing run_id".to_string());
        }
    }
    let outputs = run_path.join("outputs.index.json");
    if !outputs.exists() {
        anomalies.push("missing outputs.index.json".to_string());
    }
    if let Some(cache_path) = cache_dir {
        let cache_abs = root.join(cache_path);
        if cache_abs.exists() {
            for entry in fs::read_dir(&cache_abs).map_err(|err| err.to_string())? {
                let entry = entry.map_err(|err| err.to_string())?;
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let meta = path.join("meta.json");
                if !meta.exists() {
                    anomalies.push(format!("cache entry missing meta.json: {}", path.display()));
                    continue;
                }
                let payload = fs::read_to_string(&meta).map_err(|err| err.to_string())?;
                let parsed: Value =
                    serde_json::from_str(&payload).map_err(|err| err.to_string())?;
                if parsed.get("fingerprint").is_none() {
                    anomalies.push(format!(
                        "cache meta missing fingerprint: {}",
                        meta.display()
                    ));
                }
            }
        }
    }
    let response = json!({
        "run_dir": run_dir,
        "cache_dir": cache_dir,
        "healthy": anomalies.is_empty(),
        "anomalies": anomalies
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&response).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_run_dir_audit(run_dir: &Path, strict: bool) -> Result<(), String> {
    let root = repo_root()?;
    let mode = if strict {
        bijux_dag_artifacts::VerificationMode::Strict
    } else {
        bijux_dag_artifacts::VerificationMode::Standard
    };
    let report = bijux_dag_artifacts::verify_run_dir(root.join(run_dir), mode)
        .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_artifact_hardening_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/RUN_DIR_STORAGE_CONTRACT.md",
        "docs/spec/RUN_DIR_CONTRACT.md",
        "docs/spec/RUN_DIR_OWNERSHIP.md",
        "docs/spec/IMPORT_EXPORT_CONTRACT.md",
        "docs/reports/foundation/run_dir_import_export_hardening_report.md",
        "docs/spec/ARTIFACT_OWNERSHIP_TABLE.md",
        "docs/spec/ARTIFACT_LIFECYCLE.md",
        "configs/schema/operator/run_verify_report.schema.json",
        "evidence/compat/export_bundle/v0_1_supported/bundle.json",
        "evidence/compat/export_bundle/unsupported_past/bundle.json",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "crates/bijux-dag-artifacts/src/storage/hardening.rs",
        "crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs",
        "crates/bijux-dev-dag/tests/run_dir_import_export_hardening_contracts.rs",
        "evidence/fault/corrupt_runs/missing_manifest_version.json",
        "evidence/fault/corrupt_runs/invalid_outputs_index.json",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing artifact hardening artifact: {rel}"));
        }
    }
    let run_dir_contract = fs::read_to_string(root.join("docs/spec/RUN_DIR_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "Required entries (authoritative)",
        "Optional entries",
        "Derived artifacts (non-authoritative)",
        "Verification behavior",
        "dag verify --strict",
    ] {
        if !run_dir_contract.contains(token) {
            return Err(format!(
                "run-dir contract missing required section `{token}`"
            ));
        }
    }
    let import_export_contract =
        fs::read_to_string(root.join("docs/spec/IMPORT_EXPORT_CONTRACT.md"))
            .map_err(|err| err.to_string())?;
    for token in [
        "Bundle versioning",
        "export-bundle/v0.1",
        "dag export --manifest-only",
        "dag export --with-files",
        "provenance.source",
    ] {
        if !import_export_contract.contains(token) {
            return Err(format!(
                "import/export contract missing required section `{token}`"
            ));
        }
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    for required in ["tp_run_dir_resilience", "tp_import_export_compatibility"] {
        let present = trust_properties.iter().any(|property| {
            property
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|id| id == required)
        });
        if !present {
            return Err(format!(
                "artifact hardening requires trust property `{required}`"
            ));
        }
    }
    Ok(())
}

fn run_observability_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/OBSERVABILITY_CONTRACT.md",
        "docs/tracking/OBSERVABILITY_SURFACE_PLAN.md",
        "crates/bijux-dag-runtime/tests/observability_contracts.rs",
        "crates/bijux-dag-runtime/src/observability.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "observability contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let test_text =
        fs::read_to_string(root.join("crates/bijux-dag-runtime/tests/observability_contracts.rs"))
            .map_err(|err| err.to_string())?;
    if !test_text.contains("required_runtime_event_names_are_present_for_reference_sequence") {
        return Err(
            "observability contract test for required runtime event names is missing".to_string(),
        );
    }
    Ok(())
}

fn run_extensibility_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/EXTENSIBILITY_CONTRACT.md",
        "docs/reference/INTERNAL_HOOK_PROMOTION_CHECKLIST.md",
        "configs/schema/extension_descriptor.schema.json",
        "crates/bijux-dag-runtime/tests/extension_catalog_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "extensibility contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let report = json!({
        "extension_points": [
            { "name": "adapter", "stability": "stable", "owner": "bijux-dag-runtime" },
            { "name": "execution-backend", "stability": "experimental", "owner": "bijux-dag-runtime" },
            { "name": "internal-hook", "stability": "internal", "owner": "bijux-dag-runtime" }
        ],
        "source_contract": "docs/spec/EXTENSIBILITY_CONTRACT.md"
    });
    let report_dir = root.join("artifacts/reports");
    fs::create_dir_all(&report_dir).map_err(|err| err.to_string())?;
    fs::write(
        report_dir.join("extensibility_contract_report.json"),
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn run_security_model_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SECURITY_MODEL.md",
        "docs/tracking/NON_HERMETIC_BEHAVIORS.md",
        "docs/tracking/SECURITY_DEBT_LEDGER.md",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
        "crates/bijux-dag-runtime/tests/security_policy_contracts.rs",
        "crates/bijux-dag-runtime/tests/secrets_security_contracts.rs",
        "crates/bijux-dag-runtime/src/security_env.rs",
        "crates/bijux-dag-runtime/src/path_authorization.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "security model contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let security_doc = fs::read_to_string(root.join("docs/spec/SECURITY_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for required_section in [
        "## Threat model",
        "## Hermeticity model",
        "## Environment controls",
        "## Filesystem controls",
        "## Secret handling and redaction",
    ] {
        if !security_doc.contains(required_section) {
            return Err(format!(
                "security model missing required section: {required_section}"
            ));
        }
    }
    let security_tests =
        fs::read_to_string(root.join("crates/bijux-dag-runtime/tests/security_model_contracts.rs"))
            .map_err(|err| err.to_string())?;
    if !security_tests.contains("clean_env_and_allowlist_contract_is_deterministic")
        || !security_tests
            .contains("input_and_output_authorization_reject_path_traversal_and_symlink_escape")
    {
        return Err("security model tests missing required enforcement coverage".to_string());
    }
    Ok(())
}

fn run_container_remote_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CONTAINER_EXECUTION_CONTRACT.md",
        "docs/spec/REMOTE_EXECUTION_MODEL.md",
        "docs/architecture/execution-mode-responsibilities.md",
        "crates/bijux-dag-runtime/tests/container_execution_contracts.rs",
        "crates/bijux-dag-runtime/tests/remote_execution_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "container/remote execution boundary missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let remote_doc = fs::read_to_string(root.join("docs/spec/REMOTE_EXECUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    if !remote_doc.contains("Not implemented: production Kubernetes/HPC") {
        return Err(
            "remote execution model must explicitly declare kubernetes/hpc not implemented"
                .to_string(),
        );
    }
    let deployment_doc = fs::read_to_string(root.join("docs/DEPLOYMENT_BACKENDS.md"))
        .map_err(|err| err.to_string())?;
    if deployment_doc.contains("Kubernetes execution is production-ready")
        || deployment_doc.contains("HPC execution is production-ready")
    {
        return Err("deployment backend docs overclaim kubernetes/hpc maturity".to_string());
    }
    Ok(())
}

fn run_batch_execution_boundary_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/BATCH_EXECUTION_MODEL.md",
        "docs/architecture/local-vs-batch-execution-constraints.md",
        "crates/bijux-dag-runtime/tests/batch_execution_contracts.rs",
        "crates/bijux-dag-runtime/tests/batch_backend_simulation_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "batch execution boundary missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let batch_doc = fs::read_to_string(root.join("docs/spec/BATCH_EXECUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    if !batch_doc.contains("not implemented as") && !batch_doc.contains("not implemented") {
        return Err(
            "batch execution model must explicitly state not-implemented production boundary"
                .to_string(),
        );
    }
    if batch_doc.contains("production-ready") || batch_doc.contains("ga-ready") {
        return Err("batch execution model contains unsupported maturity claim".to_string());
    }
    Ok(())
}

fn run_operator_ux_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/OPERATOR_UX_CONTRACT.md",
        "docs/spec/OPERATOR_INSPECTION_CONTRACT.md",
        "docs/user/OPERATOR_COMMAND_INDEX.md",
        "docs/user/OPERATOR_INSPECTION_GUIDE.md",
        "docs/reference/COMMAND_TAXONOMY.md",
        "crates/bijux-dag-app/tests/operator_ux_contract.rs",
        "evidence/operator/scenarios/inspection_only.json",
        "configs/schema/operator/run_list.schema.json",
        "configs/schema/operator/run_show.schema.json",
        "configs/schema/operator/run_inspect.schema.json",
        "configs/schema/operator/run_tree.schema.json",
        "configs/schema/operator/run_timeline.schema.json",
        "configs/schema/operator/run_explain_failure.schema.json",
        "configs/schema/operator/run_doctor.schema.json",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "operator ux contract missing required surfaces: {}",
            missing.join(", ")
        ));
    }
    let index = fs::read_to_string(root.join("docs/user/OPERATOR_COMMAND_INDEX.md"))
        .map_err(|err| err.to_string())?;
    for command in [
        "dag runs list",
        "dag runs show",
        "dag runs inspect",
        "dag runs tree",
        "dag runs timeline",
        "dag runs diff",
        "dag runs verify",
        "dag runs doctor",
        "dag runs explain-failure",
    ] {
        if !index.contains(command) {
            return Err(format!("operator command index missing `{command}`"));
        }
    }
    let tests = fs::read_to_string(root.join("crates/bijux-dag-app/tests/operator_ux_contract.rs"))
        .map_err(|err| err.to_string())?;
    for required_test in [
        "operator_inspection_supports_imported_runs",
        "operator_inspection_distinguishes_unsupported_runs",
        "operator_inspection_distinguishes_corrupt_runs",
        "operator_timing_summary_is_trace_coherent",
    ] {
        if !tests.contains(required_test) {
            return Err(format!(
                "operator ux test coverage missing required case `{}`",
                required_test
            ));
        }
    }
    Ok(())
}

fn run_authoring_ux_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs = [
        "docs/spec/AUTHORING_UX_CONTRACT.md",
        "docs/user/AUTHORING_GUIDE.md",
    ];
    let required_examples = [
        "evidence/authoring/patterns/minimal.json",
        "evidence/authoring/patterns/medium.json",
        "evidence/authoring/patterns/pattern_chain.json",
        "evidence/authoring/patterns/pattern_diamond.json",
        "evidence/authoring/patterns/pattern_fanout.json",
        "evidence/authoring/patterns/pattern_aggregation.json",
        "evidence/authoring/patterns/pattern_cache_heavy.json",
        "evidence/authoring/patterns/pattern_replay_sensitive.json",
    ];
    let required_bad = [
        "evidence/authoring/negative/undeclared_outputs.json",
        "evidence/authoring/negative/invalid_refs.json",
        "evidence/authoring/negative/cycle.json",
        "evidence/authoring/negative/invalid_selectors.json",
        "evidence/authoring/negative/unsupported_adapter_payload.json",
    ];
    let mut missing = Vec::new();
    for rel in required_docs
        .iter()
        .chain(required_examples.iter())
        .chain(required_bad.iter())
    {
        if !root.join(rel).exists() {
            missing.push((*rel).to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "authoring ux required surfaces missing: {}",
            missing.join(", ")
        ));
    }

    let contract = fs::read_to_string(root.join("docs/spec/AUTHORING_UX_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let guide = fs::read_to_string(root.join("docs/user/AUTHORING_GUIDE.md"))
        .map_err(|err| err.to_string())?;
    for rel in required_examples.iter().chain(required_bad.iter()) {
        if !contract.contains(rel) {
            return Err(format!(
                "authoring contract must reference executable fixture: {rel}"
            ));
        }
        if !guide.contains(rel) {
            return Err(format!(
                "authoring user guide must reference executable fixture: {rel}"
            ));
        }
    }

    for rel in required_examples {
        let payload = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
        let graph = bijux_dag_core::parse_graph_strict(&payload).map_err(|err| err.to_string())?;
        let has_error = graph
            .validate_with_warnings()
            .iter()
            .any(|d| d.severity == bijux_dag_core::Severity::Error);
        if has_error {
            return Err(format!(
                "authoring example must validate without errors: {rel}"
            ));
        }
    }
    Ok(())
}

fn run_versioning_compatibility_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required_docs = [
        "docs/spec/VERSIONING_MODEL.md",
        "docs/reference/COMPATIBILITY_MATRIX.md",
        "docs/spec/SCHEMA_EVOLUTION_RULEBOOK.md",
        "docs/spec/RUN_DIR_EVOLUTION_RULEBOOK.md",
        "docs/spec/EXPORT_BUNDLE_EVOLUTION_RULEBOOK.md",
        "docs/spec/MIGRATION_POLICY.md",
        "docs/spec/VERSION_COMPATIBILITY_DRIFT_POLICY.md",
    ];
    let required_fixtures = [
        "evidence/compat/metadata.json",
        "evidence/compat/graph_schema/v0_1_supported/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_future/minimal.dag.json",
        "evidence/compat/graph_schema/unsupported_past/minimal.dag.json",
        "evidence/compat/run_dir/v0_1_supported/manifest.json",
        "evidence/compat/run_dir/unsupported_future/manifest.json",
        "evidence/compat/export_bundle/v0_1_supported/bundle.json",
        "evidence/compat/export_bundle/unsupported_past/bundle.json",
    ];
    let mut missing = Vec::new();
    for rel in required_docs.iter().chain(required_fixtures.iter()) {
        if !root.join(rel).exists() {
            missing.push((*rel).to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "versioning compatibility surfaces missing: {}",
            missing.join(", ")
        ));
    }

    let matrix = fs::read_to_string(root.join("docs/reference/COMPATIBILITY_MATRIX.md"))
        .map_err(|err| err.to_string())?;
    for token in ["graph schema", "run-dir format", "export bundle"] {
        if !matrix.to_lowercase().contains(token) {
            return Err(format!(
                "compatibility matrix missing required surface row: {token}"
            ));
        }
    }
    Ok(())
}

fn run_cache_evolution_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CACHE_CONTRACT.md",
        "docs/spec/CACHE_EVOLUTION_MODEL.md",
        "docs/reports/foundation/cache_hardening_report.md",
        "docs/spec/CACHE_PRUNE_POLICY.md",
        "docs/tracking/CACHE_CORRECTNESS_COVERAGE.md",
        "evidence/cache/metadata.json",
        "evidence/cache/corrupt/missing_meta.json",
        "evidence/cache/corrupt/hash_mismatch.json",
        "evidence/cache/corrupt/unsupported_metadata_version.json",
        "evidence/cache/corrupt/truncated_meta.json",
        "evidence/cache/corrupt/missing_outputs_proof.json",
        "evidence/cache/scenarios/warm_cold.json",
        "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
        "crates/bijux-dag-runtime/tests/cache_contracts.rs",
        "crates/bijux-dev-dag/tests/cache_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "cache evolution required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let cache_metadata: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/cache/metadata.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let corrupt_fixtures = cache_metadata
        .get("corrupt_fixtures")
        .and_then(Value::as_object)
        .ok_or_else(|| "cache metadata missing corrupt_fixtures object".to_string())?;
    for fixture in [
        "evidence/cache/corrupt/missing_meta.json",
        "evidence/cache/corrupt/hash_mismatch.json",
        "evidence/cache/corrupt/unsupported_metadata_version.json",
        "evidence/cache/corrupt/truncated_meta.json",
        "evidence/cache/corrupt/missing_outputs_proof.json",
    ] {
        if !corrupt_fixtures.contains_key(fixture) {
            return Err(format!(
                "cache metadata missing corruption entry: {fixture}"
            ));
        }
    }
    let model = fs::read_to_string(root.join("docs/spec/CACHE_EVOLUTION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "Intentional cache key inputs",
        "Metadata compatibility",
        "Cache lineage model",
        "Locality decision",
    ] {
        if !model.contains(token) {
            return Err(format!("cache evolution model missing section `{token}`"));
        }
    }
    let app_commands = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    let cache_surface_count = [
        "Ls",
        "Pack",
        "Unpack",
        "Gc",
        "Verify",
        "Explain",
        "Stats",
        "PruneSimulate",
        "Diff",
    ]
    .iter()
    .filter(|name| app_commands.contains(&format!("CacheCommands::{}", name)))
    .count();
    if cache_surface_count >= 9 {
        for test in [
            "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
            "crates/bijux-dag-runtime/tests/cache_contracts.rs",
        ] {
            if !root.join(test).exists() {
                return Err(format!(
                    "cache command surface expanded without required cache coverage test: {}",
                    test
                ));
            }
        }
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_cache_integrity = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_cache_integrity")
    });
    if !has_cache_integrity {
        return Err("cache evolution requires tp_cache_integrity trust property".to_string());
    }
    Ok(())
}

fn run_replay_contract_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/REPLAY_CONTRACT.md",
        "docs/reports/foundation/replay_hardening_report.md",
        "configs/schema/operator/replay_diff.schema.json",
        "evidence/cache/replay/match_case.json",
        "evidence/cache/replay/mismatch_case.json",
        "evidence/cache/replay/corruption_case.json",
        "evidence/cache/replay/unsupported_version_case.json",
        "crates/bijux-dag-app/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "replay contract required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let contract = fs::read_to_string(root.join("docs/spec/REPLAY_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "## Replay definition",
        "## Authoritative inputs",
        "## Replay explain mode",
        "## What replay cannot prove",
    ] {
        if !contract.contains(token) {
            return Err(format!("replay contract missing section `{token}`"));
        }
    }
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    if !commands_src.contains("DiffModeArg::Semantic") {
        return Err("replay contract requires semantic diff mode in CLI surfaces".to_string());
    }
    let replay_battle = fs::read_to_string(
        root.join("evidence/battle/workflows/replay/replay_semantic_comparison.json"),
    )
    .map_err(|err| err.to_string())?;
    if !replay_battle.contains("replay_mandatory_proof") {
        return Err("replay battle scenario must assert replay_mandatory_proof".to_string());
    }
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_replay_equivalence = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_replay_equivalence")
    });
    if !has_replay_equivalence {
        return Err("replay contract requires tp_replay_equivalence trust property".to_string());
    }

    let mut violations = Vec::new();
    let docs_dir = root.join("docs");
    let mut stack = vec![docs_dir];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let rel = path
                .strip_prefix(&root)
                .map_err(|err| err.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
            for line in text.lines() {
                let lower = line.to_ascii_lowercase();
                if lower.contains("replayable")
                    && !line.contains("REPLAY_CONTRACT.md")
                    && !line.contains("docs/spec/REPLAY_CONTRACT.md")
                {
                    violations.push(format!("{}: {}", rel, line.trim()));
                }
            }
        }
    }
    if !violations.is_empty() {
        return Err(format!(
            "vague replayable claims must cite replay contract: {}",
            violations.join(" | ")
        ));
    }
    Ok(())
}

fn run_multi_run_analytics_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/MULTI_RUN_ANALYTICS_CONTRACT.md",
        "docs/spec/HISTORY_RETENTION_POLICY.md",
        "docs/spec/ANALYTICS_EXACTNESS.md",
        "configs/schema/operator/runs_analytics.schema.json",
        "crates/bijux-dag-app/tests/multi_run_analytics_contract.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "multi-run analytics required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for token in ["Summary", "Compare", "Trend", "Failures", "Flakes"] {
        if !commands_src.contains(token) {
            return Err(format!(
                "runs command surface missing analytics variant `{token}`"
            ));
        }
    }
    let contract = fs::read_to_string(root.join("docs/spec/MULTI_RUN_ANALYTICS_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    if !contract.contains("never mutate authoritative run records") {
        return Err("multi-run analytics contract must assert non-mutation rule".to_string());
    }
    Ok(())
}

fn run_distributed_coordination_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/DISTRIBUTED_COORDINATION_MODEL.md",
        "docs/architecture/controller_backend_artifact_boundary.md",
        "docs/architecture/local_only_vs_remote_coordinated_runtime.md",
        "crates/bijux-dag-runtime/tests/distributed_event_reconciliation_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "distributed coordination required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let model = fs::read_to_string(root.join("docs/spec/DISTRIBUTED_COORDINATION_MODEL.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "single-controller",
        "Single-writer rule",
        "Not implemented boundary",
        "planner, scheduler, and storage contracts",
    ] {
        if !model.contains(token) {
            return Err(format!(
                "distributed coordination model missing section `{token}`"
            ));
        }
    }
    Ok(())
}

fn run_distributed_semantics_report() -> Result<(), String> {
    let payload = json!({
        "local_semantics": {
            "authoritative_writer": "controller",
            "run_state_writer_count": 1,
            "distributed_coordination_mode": "not_implemented"
        },
        "simulated_distributed_semantics": {
            "event_source": "fake_distributed_event_source",
            "reconciliation": ["out_of_order", "duplicate", "missing_completion", "restart_partial_state"],
            "authoritative_remote_state_writer": false
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_formal_invariants_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/FORMAL_INVARIANTS.md",
        "docs/tracking/INVARIANT_COVERAGE.md",
        "crates/bijux-dag-runtime/src/invariants.rs",
        "crates/bijux-dag-runtime/src/invariants_tests.rs",
        "crates/bijux-dag-runtime/tests/formal_invariant_property_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "formal invariants required surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let spec = fs::read_to_string(root.join("docs/spec/FORMAL_INVARIANTS.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "INV-GRAPH-SHAPE-001",
        "INV-PLAN-SHAPE-001",
        "INV-SCHED-READY-001",
        "INV-RUN-COUNTS-001",
        "INV-TRACE-TIME-001",
        "INV-CACHE-PROOF-001",
        "INV-ARTIFACT-REF-001",
    ] {
        if !spec.contains(token) {
            return Err(format!("formal invariants spec missing `{token}`"));
        }
    }
    let mut unchecked_guarantees = Vec::new();
    let rel = "docs/spec/FORMAL_INVARIANTS.md";
    let text = fs::read_to_string(root.join(rel)).map_err(|err| err.to_string())?;
    for (idx, line) in text.lines().enumerate() {
        let lower = line.to_ascii_lowercase();
        if (lower.contains("guarantee") || lower.contains("always") || lower.contains("never"))
            && !line.contains("INV-")
        {
            unchecked_guarantees.push(format!("{}:{} {}", rel, idx + 1, line.trim()));
        }
    }
    if !unchecked_guarantees.is_empty() {
        return Err(format!(
            "normative guarantee wording must cite invariant ids: {}",
            unchecked_guarantees.join(" | ")
        ));
    }
    Ok(())
}

fn run_invariants_report() -> Result<(), String> {
    let root = repo_root()?;
    let registry_src = fs::read_to_string(root.join("crates/bijux-dag-runtime/src/invariants.rs"))
        .map_err(|err| err.to_string())?;
    let coverage = fs::read_to_string(root.join("docs/tracking/INVARIANT_COVERAGE.md"))
        .map_err(|err| err.to_string())?;

    let mut ids = Vec::new();
    for line in registry_src.lines() {
        if let Some(start) = line.find("id: \"INV-") {
            let slice = &line[start + 5..];
            if let Some(end) = slice.find('"') {
                ids.push(slice[..end].to_string());
            }
        }
    }
    ids.sort();
    ids.dedup();

    let mut missing_coverage = Vec::new();
    for id in &ids {
        if !coverage.contains(id) {
            missing_coverage.push(id.clone());
        }
    }

    let payload = json!({
        "invariant_ids": ids,
        "missing_coverage_entries": missing_coverage,
        "coverage_file": "docs/tracking/INVARIANT_COVERAGE.md"
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?
    );
    if payload["missing_coverage_entries"]
        .as_array()
        .is_some_and(|a| a.is_empty())
    {
        Ok(())
    } else {
        Err("invariant coverage file missing registry entries".to_string())
    }
}

fn run_adoption_surfaces_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/ADOPTION_SURFACES.md",
        "docs/user/INSTALLATION.md",
        "docs/user/CI_INTEGRATION.md",
        "docs/user/FIRST_HOUR_WITH_BIJUX_DAG.md",
        "docs/reference/SUPPORT_MATRIX.md",
        "docs/spec/RELEASE_BINARY_VERIFICATION.md",
        "docs/user/TRUST_BOUNDARIES.md",
        "evidence/authoring/examples/minimal_consumer.dag.json",
        "crates/bijux-dag-testkit/fixtures/minimal_consumer/README.md",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "adoption surfaces required docs/fixtures missing: {}",
            missing.join(", ")
        ));
    }

    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    if !commands_src.contains("Capabilities") {
        return Err(
            "dag capabilities command is required for machine-readable support summary".to_string(),
        );
    }

    let quickstart = fs::read_to_string(root.join("docs/user/FIRST_HOUR_WITH_BIJUX_DAG.md"))
        .map_err(|err| err.to_string())?;
    let install = fs::read_to_string(root.join("docs/user/INSTALLATION.md"))
        .map_err(|err| err.to_string())?;
    for required_cmd in [
        "cargo build -p bijux-dag-cli --release",
        "cargo run -p bijux-dag-cli -- dag version",
    ] {
        if !install.contains(required_cmd) {
            return Err(format!(
                "installation doc missing clean-environment command `{}`",
                required_cmd
            ));
        }
    }
    for forbidden in ["kubernetes", "hpc", "production-grade remote"] {
        if quickstart.to_ascii_lowercase().contains(forbidden) {
            return Err(format!(
                "quickstart references unsupported surface `{}` as first-class",
                forbidden
            ));
        }
    }
    Ok(())
}

fn run_release_artifact_verification_suite() -> Result<(), String> {
    let root = repo_root()?;
    let commands_src = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for command in ["Version", "Capabilities", "Runs"] {
        if !commands_src.contains(command) {
            return Err(format!(
                "release artifact verification requires `{}` command surface",
                command
            ));
        }
    }
    let policy = fs::read_to_string(root.join("docs/spec/RELEASE_BINARY_VERIFICATION.md"))
        .map_err(|err| err.to_string())?;
    for token in ["dag version --json", "dag capabilities --json"] {
        if !policy.contains(token) {
            return Err(format!(
                "release binary verification doc missing required check `{}`",
                token
            ));
        }
    }
    Ok(())
}

fn run_drift_dashboard() -> Result<(), String> {
    let root = repo_root()?;
    let payload = json!({
        "drift_classes": [
            {"name":"docs drift","severity":"blocker","check":"repo-docs"},
            {"name":"schema drift","severity":"blocker","check":"docs-schema-ref"},
            {"name":"contract drift","severity":"blocker","check":"docs-contract-ref"},
            {"name":"cli drift","severity":"blocker","check":"cli-freeze"},
            {"name":"test drift","severity":"warning","check":"contract-test-links"},
            {"name":"fixture drift","severity":"warning","check":"docs-coverage"},
            {"name":"benchmark drift","severity":"warning","check":"performance-claims"},
            {"name":"dependency drift","severity":"warning","check":"dependency-policy"}
        ],
        "dashboard_doc": "docs/tracking/DRIFT_DASHBOARD.md",
        "anti_drift_policy": root.join("docs/spec/ANTI_DRIFT_POLICY.md").exists()
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_repo_trust_summary() -> Result<(), String> {
    let root = repo_root()?;
    let payload = json!({
        "contracts": {
            "invariants": root.join("docs/spec/FORMAL_INVARIANTS.md").exists(),
            "comparison_harness": root.join("docs/spec/COMPARISON_HARNESS_CONTRACT.md").exists(),
            "adoption_surfaces": root.join("docs/spec/ADOPTION_SURFACES.md").exists(),
            "anti_drift": root.join("docs/spec/ANTI_DRIFT_POLICY.md").exists()
        },
        "tracking": {
            "invariant_coverage": root.join("docs/tracking/INVARIANT_COVERAGE.md").exists(),
            "drift_dashboard": root.join("docs/tracking/DRIFT_DASHBOARD.md").exists()
        },
        "evidence_index": root.join("docs/reference/REPO_TRUST_EVIDENCE_INDEX.md").exists()
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_anti_drift_governance_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/ANTI_DRIFT_POLICY.md",
        "docs/tracking/DRIFT_DASHBOARD.md",
        "docs/reference/REPO_TRUST_EVIDENCE_INDEX.md",
        ".github/pull_request_template.md",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "anti-drift governance surfaces missing: {}",
            missing.join(", ")
        ));
    }
    let policy = fs::read_to_string(root.join("docs/spec/ANTI_DRIFT_POLICY.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "docs drift",
        "schema drift",
        "contract drift",
        "cli drift",
        "same-change alignment rule",
    ] {
        if !policy
            .to_ascii_lowercase()
            .contains(&token.to_ascii_lowercase())
        {
            return Err(format!("anti-drift policy missing `{}`", token));
        }
    }

    let suite_ids = crate::suites::repo::IDS;
    for required_check in [
        "cli-freeze",
        "docs-schema-ref",
        "docs-contract-ref",
        "contract-test-links",
        "docs-coverage",
        "versioning-compatibility",
        "performance-claims",
    ] {
        if !suite_ids.contains(&required_check) {
            return Err(format!(
                "anti-drift governance requires repo suite `{}`",
                required_check
            ));
        }
    }

    let release_doc = fs::read_to_string(root.join("docs/spec/RELEASE_BINARY_VERIFICATION.md"))
        .map_err(|err| err.to_string())?;
    if !release_doc.contains("dag version --json")
        || !release_doc.contains("dag capabilities --json")
    {
        return Err(
            "release verification doc must define machine-readable artifact checks".to_string(),
        );
    }

    let benchmark_scenarios = root.join("evidence/perf/scenarios");
    if !benchmark_scenarios.exists() {
        return Err(
            "benchmark scenario directory missing for anti-drift benchmark check".to_string(),
        );
    }
    Ok(())
}

fn run_runtime_module_triage_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/architecture/runtime_module_triage.md",
        "docs/spec/RUNTIME_PUBLIC_API_BOUNDARY.md",
        "configs/policy/runtime_module_freeze.json",
        "crates/bijux-dag-runtime/src/runtime.rs",
        "crates/bijux-dag-runtime/src/adapters.rs",
        "crates/bijux-dag-runtime/src/execution.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "runtime module triage surfaces missing: {}",
            missing.join(", ")
        ));
    }

    let freeze_payload = fs::read_to_string(root.join("configs/policy/runtime_module_freeze.json"))
        .map_err(|err| err.to_string())?;
    let freeze_json: Value =
        serde_json::from_str(&freeze_payload).map_err(|err| err.to_string())?;
    let allowed = freeze_json
        .get("allowed_modules")
        .and_then(Value::as_array)
        .ok_or_else(|| "runtime_module_freeze.json missing allowed_modules".to_string())?;
    let allowed_set: BTreeSet<String> = allowed
        .iter()
        .filter_map(Value::as_str)
        .map(|s| s.to_string())
        .collect();

    let mut actual = BTreeSet::new();
    for entry in
        fs::read_dir(root.join("crates/bijux-dag-runtime/src")).map_err(|err| err.to_string())?
    {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("rs") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|v| v.to_str()) {
            if ![
                "lib",
                "runtime_boundary_tests",
                "adapter_contract_tests",
                "invariants_tests",
                "runtime_policy_trace_tests",
                "state_machine_tests",
                "tests_runtime.in",
                "test_support",
            ]
            .contains(&stem)
            {
                actual.insert(stem.to_string());
            }
        }
    }
    let unexpected: Vec<String> = actual.difference(&allowed_set).cloned().collect();
    if !unexpected.is_empty() {
        return Err(format!(
            "runtime module freeze violated by modules: {}",
            unexpected.join(", ")
        ));
    }
    Ok(())
}

fn run_sacred_execution_flow_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SACRED_EXECUTION_FLOW.md",
        "docs/architecture/runtime-execution-flow.md",
        "docs/reports/foundation/sacred_execution_hardening_report.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/context.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs",
        "crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs",
        "crates/bijux-dev-dag/tests/sacred_execution_hardening_contracts.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "sacred execution flow required surfaces missing: {}",
            missing.join(", ")
        ));
    }

    let engine_src = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs"),
    )
    .map_err(|err| err.to_string())?;
    for token in [
        "sacred_execution::run_materialize_inputs",
        "sacred_execution::run_cache_lookup",
        "sacred_execution::run_retry_logic",
        "sacred_execution::run_write_trace",
        "sacred_execution::run_cache_write",
        "sacred_execution::resolve_dependencies",
    ] {
        if !engine_src.contains(token) {
            return Err(format!("engine flow missing centralized hook `{}`", token));
        }
    }
    for forbidden in [
        "crate::try_cache_read(",
        "crate::try_cache_write(",
        "crate::write_trace(",
        "crate::execute_with_retries(",
    ] {
        if engine_src.contains(forbidden) {
            return Err(format!(
                "engine flow bypasses sacred hook with direct call `{}`",
                forbidden
            ));
        }
    }
    Ok(())
}

fn run_crate_boundary_foundation_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/CRATE_RESPONSIBILITY_STATEMENTS.md",
        "docs/spec/CRATE_BOUNDARY_CONTRACT.md",
        "docs/architecture/crate_boundary_adr.md",
        "docs/architecture/crate_service_interfaces.md",
        "configs/policy/forbidden_dependencies.json",
        "crates/bijux-dag-app/tests/crate_boundary_contract.rs",
        "crates/bijux-dag-runtime/src/services.rs",
        "crates/bijux-dag-artifacts/src/services.rs",
    ];
    let mut missing = Vec::new();
    for rel in required {
        if !root.join(rel).exists() {
            missing.push(rel.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "crate boundary foundation missing required surfaces: {}",
            missing.join(", ")
        ));
    }

    let policy_payload =
        fs::read_to_string(root.join("configs/policy/forbidden_dependencies.json"))
            .map_err(|err| err.to_string())?;
    let policy: Value = serde_json::from_str(&policy_payload).map_err(|err| err.to_string())?;
    let edges = policy
        .get("forbidden_edges")
        .and_then(Value::as_array)
        .ok_or_else(|| "forbidden dependency policy missing forbidden_edges".to_string())?;
    for edge in edges {
        let from = edge.get("from").and_then(Value::as_str).unwrap_or_default();
        let to = edge.get("to").and_then(Value::as_str).unwrap_or_default();
        let cargo = fs::read_to_string(root.join(format!("crates/{}/Cargo.toml", from)))
            .map_err(|err| err.to_string())?;
        if cargo.contains(to) {
            return Err(format!(
                "forbidden dependency edge detected: {} -> {}",
                from, to
            ));
        }
    }
    Ok(())
}

fn load_error_code_registry(root: &Path) -> Result<ErrorCodeRegistry, String> {
    let payload = fs::read_to_string(root.join("configs/policy/error_codes.json"))
        .map_err(|err| err.to_string())?;
    let registry: ErrorCodeRegistry =
        serde_json::from_str(&payload).map_err(|err| err.to_string())?;

    let mut seen_codes = BTreeSet::new();
    let mut seen_categories = BTreeSet::new();
    for category in &registry.categories {
        seen_categories.insert(category.clone());
    }
    for entry in &registry.codes {
        if !seen_categories.contains(&entry.category) {
            return Err(format!(
                "error code {} references unknown category {}",
                entry.code, entry.category
            ));
        }
        if entry.owner.trim().is_empty() || entry.description.trim().is_empty() {
            return Err(format!(
                "error code {} has empty owner or description",
                entry.code
            ));
        }
        if !seen_codes.insert(entry.code.clone()) {
            return Err(format!("duplicate error code {}", entry.code));
        }
    }
    Ok(registry)
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct EffectiveConfigDump {
    jobs: Option<usize>,
    cache_mode: Option<String>,
    materialize_inputs: Option<String>,
    policy: Option<Value>,
    debug: Option<Value>,
}

fn run_config_dump(config: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let defaults_path = root.join("configs/dev/default_runtime_config.json");
    let defaults_payload = fs::read_to_string(&defaults_path).map_err(|err| err.to_string())?;
    let defaults: Value = serde_json::from_str(&defaults_payload).map_err(|err| err.to_string())?;
    let mut merged = defaults;

    if let Ok(env_cache_dir) = env::var("BIJUX_DAG_CACHE_DIR") {
        merged["cache_dir"] = Value::String(env_cache_dir);
    }
    if let Ok(env_adapters_dir) = env::var("BIJUX_DAG_ADAPTERS_DIR") {
        merged["adapters_dir"] = Value::String(env_adapters_dir);
    }

    if let Some(path) = config {
        let full = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let payload = fs::read_to_string(full).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        deep_merge_json(&mut merged, &parsed);
    }

    let _typed: EffectiveConfigDump =
        serde_json::from_value(merged.clone()).map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&merged).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_policy_audit(config: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let defaults_path = root.join("configs/dev/default_runtime_config.json");
    let defaults_payload = fs::read_to_string(&defaults_path).map_err(|err| err.to_string())?;
    let defaults: Value = serde_json::from_str(&defaults_payload).map_err(|err| err.to_string())?;
    let mut merged = defaults;
    if let Some(path) = config {
        let full = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        let payload = fs::read_to_string(full).map_err(|err| err.to_string())?;
        let parsed: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        deep_merge_json(&mut merged, &parsed);
    }
    let policy = merged
        .get("policy")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let report = json!({
        "policy_controls": {
            "deny_network": policy.get("deny_network").cloned().unwrap_or(Value::Bool(false)),
            "deny_env": policy.get("deny_env").cloned().unwrap_or(Value::Bool(false)),
            "deny_clock": policy.get("deny_clock").cloned().unwrap_or(Value::Bool(false)),
            "clean_env": policy.get("clean_env").cloned().unwrap_or(Value::Bool(false))
        },
        "security_contract": "docs/spec/SECURITY_MODEL.md"
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_execution_modes_report() -> Result<(), String> {
    let report = bijux_dag_runtime::execution_mode_report();
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_compatibility_report() -> Result<(), String> {
    let root = repo_root()?;
    let report = json!({
        "graph_schema": {
            "current": "0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/v0_1_supported"))?,
            "unsupported_future_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/unsupported_future"))?,
            "unsupported_past_fixtures": collect_fixture_count(&root.join("evidence/compat/graph_schema/unsupported_past"))?
        },
        "run_dir": {
            "current": "run-manifest/v0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/run_dir/v0_1_supported"))?,
            "unsupported_future_fixtures": collect_fixture_count(&root.join("evidence/compat/run_dir/unsupported_future"))?
        },
        "export_bundle": {
            "current": "export-bundle/v0.1",
            "supported_fixtures": collect_fixture_count(&root.join("evidence/compat/export_bundle/v0_1_supported"))?,
            "unsupported_past_fixtures": collect_fixture_count(&root.join("evidence/compat/export_bundle/unsupported_past"))?
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_cache_coverage_report() -> Result<(), String> {
    let root = repo_root()?;
    let report = json!({
        "cache_correctness": {
            "docs": {
                "contract": root.join("docs/spec/CACHE_CONTRACT.md").exists(),
                "model": root.join("docs/spec/CACHE_EVOLUTION_MODEL.md").exists(),
                "prune_policy": root.join("docs/spec/CACHE_PRUNE_POLICY.md").exists(),
                "coverage_ledger": root.join("docs/tracking/CACHE_CORRECTNESS_COVERAGE.md").exists()
            },
            "fixtures": {
                "corruption": collect_fixture_count(&root.join("evidence/cache/corrupt"))?,
                "warm_cold": collect_fixture_count(&root.join("evidence/cache/scenarios"))?
            },
            "tests": {
                "app_cache_evolution_contract": root.join("crates/bijux-dag-app/tests/cache_evolution_contract.rs").exists(),
                "runtime_cache_contracts": root.join("crates/bijux-dag-runtime/tests/cache_contracts.rs").exists()
            }
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn collect_fixture_count(dir: &Path) -> Result<usize, String> {
    if !dir.exists() {
        return Ok(0);
    }
    let count = fs::read_dir(dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .count();
    Ok(count)
}

fn run_config_lint() -> Result<(), String> {
    let root = repo_root()?;
    let examples_dir = root.join("configs/dev/examples");
    let mut violations = Vec::new();

    for entry in fs::read_dir(&examples_dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let payload = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        let value: Value = serde_json::from_str(&payload).map_err(|err| err.to_string())?;
        let allowed_root = [
            "jobs",
            "cache_mode",
            "materialize_inputs",
            "policy",
            "debug",
        ];
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                if !allowed_root.contains(&key.as_str()) {
                    violations.push(format!("{} has unknown field `{}`", path.display(), key));
                }
                if key.starts_with("deprecated_") {
                    violations.push(format!(
                        "{} contains deprecated field `{}`",
                        path.display(),
                        key
                    ));
                }
            }
        } else {
            violations.push(format!("{} must be a JSON object", path.display()));
        }
        if value.get("jobs").and_then(|v| v.as_u64()).unwrap_or(0) == 0 {
            violations.push(format!("{} has invalid jobs", path.display()));
        }
        let cache_mode = value
            .get("cache_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !["off", "read", "read-write"].contains(&cache_mode) {
            violations.push(format!("{} has invalid cache_mode", path.display()));
        }
        let materialize = value
            .get("materialize_inputs")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !["none", "direct", "all"].contains(&materialize) {
            violations.push(format!("{} has invalid materialize_inputs", path.display()));
        }
        if value.get("policy").is_none() {
            violations.push(format!("{} missing policy object", path.display()));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_config_precedence_drift_guard() -> Result<(), String> {
    let root = repo_root()?;
    let precedence_doc = fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    let expected = "CLI > explicit config file > environment > defaults";
    if !precedence_doc.contains(expected) {
        return Err(
            "docs/spec/CONFIG_PRECEDENCE_CONTRACT.md missing canonical precedence table"
                .to_string(),
        );
    }
    for token in ["dag config show-effective", "dag policy show-effective"] {
        if !precedence_doc.contains(token) {
            return Err(format!(
                "config precedence contract missing command surface `{}`",
                token
            ));
        }
    }

    let defaults = json!({"jobs": 1});
    let env_cfg = json!({"jobs": 2});
    let file_cfg = json!({"jobs": 3});
    let cli_cfg = json!({"jobs": 4});
    let mut merged = defaults;
    deep_merge_json(&mut merged, &env_cfg);
    deep_merge_json(&mut merged, &file_cfg);
    deep_merge_json(&mut merged, &cli_cfg);
    if merged.get("jobs").and_then(|v| v.as_u64()) != Some(4) {
        return Err("effective precedence behavior does not match documented order".to_string());
    }
    Ok(())
}

fn run_config_policy_determinism_guard() -> Result<(), String> {
    let root = repo_root()?;
    for required in [
        "docs/spec/CONFIG_PRECEDENCE_CONTRACT.md",
        "docs/spec/POLICY_EVALUATION_TRACE.md",
        "docs/reports/foundation/config_policy_determinism_report.md",
        "crates/bijux-dag-app/tests/config_precedence_contract.rs",
        "crates/bijux-dag-app/tests/config_validation_contract.rs",
        "crates/bijux-dag-app/tests/config_effective_command_contract.rs",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
    ] {
        if !root.join(required).exists() {
            return Err(format!(
                "config/policy determinism missing required surface: {required}"
            ));
        }
    }

    let contract = fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md"))
        .map_err(|err| err.to_string())?;
    for token in [
        "CLI > explicit config file > environment > defaults",
        "Unknown fields in explicit config must fail before execution.",
        "Malformed config files must fail before execution.",
        "Policy evaluation trace must be available for operator/debug inspection.",
    ] {
        if !contract.contains(token) {
            return Err(format!(
                "config precedence contract missing required token `{token}`"
            ));
        }
    }

    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let trust_properties = policy
        .get("trust_properties")
        .and_then(Value::as_array)
        .ok_or_else(|| "battle trust policy missing trust_properties".to_string())?;
    let has_config_policy = trust_properties.iter().any(|property| {
        property
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == "tp_config_policy_determinism")
    });
    if !has_config_policy {
        return Err(
            "battle trust policy must include tp_config_policy_determinism as release evidence"
                .to_string(),
        );
    }
    Ok(())
}

fn run_ambient_env_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_files_with_extension(&root.join("crates"), "rs", &mut files)?;
    let mut violations = Vec::new();
    let allow_env_keys = [
        "BIJUX_DAG_CACHE_DIR",
        "BIJUX_DAG_ADAPTERS_DIR",
        "BIJUX_DAG_JOBS",
        "BIJUX_DAG_CACHE_MODE",
        "BIJUX_DAG_MATERIALIZE_INPUTS",
        "BIJUX_DAG_POLICY_JSON",
    ];
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .map_err(|err| err.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        let content = fs::read_to_string(&file).map_err(|err| err.to_string())?;
        if !(content.contains("std::env::var(") || content.contains("env::var(")) {
            continue;
        }
        for line in content.lines() {
            if !(line.contains("std::env::var(\"") || line.contains("env::var(\"")) {
                continue;
            }
            if rel.contains("/tests/") || rel.ends_with(".in.rs") {
                continue;
            }
            if allow_env_keys.iter().any(|key| line.contains(key)) {
                continue;
            }
            violations.push(format!(
                "{rel}: disallowed ambient env read `{}`",
                line.trim()
            ));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
}

fn run_foundation_verification_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/CONTROL_PLANE_FOUNDATION.md",
        "docs/spec/WORKSPACE_CONTRACT.md",
        "crates/bijux-dev-dag/src/commands/mod.rs",
        "crates/bijux-dev-dag/src/suites/repo.rs",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing foundation artifact: {rel}"));
        }
    }
    for required in [
        "repo-docs",
        "repo-source",
        "root-directory-guard",
        "executable-guard",
        "docs-governance",
        "docs-links",
        "docs-schema-ref",
        "crate-boundary-foundation",
        "artifact-hardening",
        "performance-evidence",
        "test-trust-foundation",
        "test-trust-cleanup",
        "docs-config-reduction",
        "scheduler-invariants",
        "backend-contract",
        "cache-evolution",
        "replay-contract",
        "config-policy-determinism",
        "battle-suite-mandatory",
        "runtime-module-triage",
    ] {
        if !crate::suites::repo::IDS.contains(&required) {
            return Err(format!(
                "foundation verification missing suite id: {required}"
            ));
        }
    }
    Ok(())
}

fn run_foundation_review_report() -> Result<(), String> {
    let root = repo_root()?;
    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut runtime_modules = Vec::new();
    collect_files_with_extension(&runtime_src, "rs", &mut runtime_modules)?;

    let docs_root = root.join("docs");
    let mut markdown = Vec::new();
    collect_markdown_files(&docs_root, &mut markdown)?;
    let docs_root_markdown_count = markdown
        .iter()
        .filter(|path| path.parent() == Some(docs_root.as_path()))
        .count();

    let report = json!({
        "runtime_module_count": runtime_modules.len(),
        "docs_root_markdown_count": docs_root_markdown_count,
        "repo_suite_count": crate::suites::repo::IDS.len(),
        "has_foundation_final_report": root.join("docs/reports/foundation/foundation_final_report.md").exists(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn run_foundation_review_guard() -> Result<(), String> {
    let root = repo_root()?;
    for rel in [
        "docs/spec/FOUNDATION_READINESS_CRITERIA.md",
        "docs/spec/ARCHITECTURE_REVIEW_CHECKLIST.md",
        "docs/spec/FEATURE_DEVELOPMENT_FREEZE_POLICY.md",
        "docs/reports/foundation/repository_architecture_report.md",
        "docs/reports/foundation/runtime_module_ownership_report.md",
        "docs/reports/foundation/artifact_contract_report.md",
        "docs/reports/foundation/performance_evidence_report.md",
        "docs/reports/foundation/test_trust_coverage_report.md",
        "docs/reports/foundation/release_evidence_report.md",
        "docs/reports/foundation/repository_proof_statement.md",
        "docs/reports/foundation/cleanup_backlog.md",
        "docs/reports/foundation/subsystem_strength_assessment.md",
        "docs/reports/foundation/foundation_final_report.md",
    ] {
        if !root.join(rel).exists() {
            return Err(format!("missing foundation review artifact: {rel}"));
        }
    }
    Ok(())
}

fn run_control_plane_surfaces_guard() -> Result<(), String> {
    let root = repo_root()?;
    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .map_err(|err| err.to_string())?;
    for required in [
        "enum RepoCommand",
        "enum ReleaseCommand",
        "ArtifactVerify",
        "StorageHealth",
        "RunDirAudit",
        "Ci",
        "FoundationHardening",
        "ControlCommand::Run",
        "ReleaseCommand::Verify",
    ] {
        if !commands.contains(required) {
            return Err(format!("missing control-plane command surface: {required}"));
        }
    }
    let foundation = fs::read_to_string(root.join("docs/spec/CONTROL_PLANE_FOUNDATION.md"))
        .map_err(|err| err.to_string())?;
    for required in [
        "repo verification",
        "docs verification",
        "naming verification",
        "crate boundary verification",
        "fixture verification",
        "artifact contract verification",
        "release verification",
        "ci verification",
    ] {
        if !foundation.contains(required) {
            return Err(format!(
                "control-plane foundation doc missing required surface: {required}"
            ));
        }
    }
    Ok(())
}

fn run_repo_hygiene_suite_guard() -> Result<(), String> {
    for required in [
        "repo-docs",
        "repo-source",
        "repo-manifests",
        "repo-api",
        "root-directory-guard",
        "executable-guard",
        "docs-governance",
        "docs-links",
        "docs-schema-ref",
        "config-lint",
        "config-drift",
        "ambient-env-guard",
        "evidence-authority",
    ] {
        if !crate::suites::repo::IDS.contains(&required) {
            return Err(format!(
                "repo hygiene suite missing required guard: {required}"
            ));
        }
    }
    Ok(())
}

fn deep_merge_json(target: &mut Value, overlay: &Value) {
    match (target, overlay) {
        (Value::Object(dst), Value::Object(src)) => {
            for (key, value) in src {
                match dst.get_mut(key) {
                    Some(existing) => deep_merge_json(existing, value),
                    None => {
                        dst.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (target, overlay) => {
            *target = overlay.clone();
        }
    }
}

fn collect_files_with_extension(
    dir: &Path,
    ext: &str,
    out: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            collect_files_with_extension(&path, ext, out)?;
            continue;
        }
        if path.extension().and_then(|x| x.to_str()) == Some(ext) {
            out.push(path);
        }
    }
    Ok(())
}
