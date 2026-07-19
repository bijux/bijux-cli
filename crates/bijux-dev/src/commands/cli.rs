use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

pub(super) const ADAPTER_KIND_FREEZE_BASELINE: usize = 3;

#[derive(Parser)]
#[command(name = "bijux-dev-dag")]
#[command(about = "Developer workflow helpers for bijux-dag")]
pub(super) struct Cli {
    #[arg(long)]
    pub(super) json: bool,
    #[arg(long)]
    pub(super) report: Option<PathBuf>,
    #[command(subcommand)]
    pub(super) command: CommandLine,
}

#[derive(Subcommand)]
pub(super) enum CommandLine {
    /// Run cargo fmt on workspace
    Fmt,
    /// Run workspace format check + clippy
    Lint,
    /// Run cargo audit
    Security,
    /// Run metadata + tests + format check
    Sanity,
    /// Run legacy style checks
    Checks {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run legacy style tests
    Tests {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run compatibility and runtime contracts
    Contracts {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Produce documentation health or report views
    Docs {
        #[command(subcommand)]
        command: ControlCommand,
    },
    /// Run release preparation workflows
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    /// Run repo and governance policies
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    /// Run focused repository verification checks
    Verify {
        #[command(subcommand)]
        command: VerifyCommand,
    },
    /// Validate and preview scheduling definitions
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommand,
    },
    /// DAG developer-experience helpers
    Dag {
        #[command(subcommand)]
        command: DagCommand,
    },
    /// Print environment diagnostics and report status
    Doctor,
    /// Generate and verify golden run/replay contract
    Golden,
    /// Compare cargo-public-api output with docs/api baseline
    PublicApi,
    /// API surface commands
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },
    /// Check forbidden dependency usage in workspace Cargo manifests
    DepGuard,
    /// Print workspace crate dependency graph from cargo metadata
    CrateGraph,
    /// Remove workspace target artifacts
    ArtifactsClean,
    /// Print build environment summary
    EnvSummary,
    /// Validate required cargo tools are installed
    VerifyTools,
    /// Verify workspace dependencies resolve
    ResolveCheck,
    /// Record baseline benchmark artifact
    BenchmarkBaseline,
    /// Compare benchmark result with a baseline by threshold
    BenchmarkCompare {
        #[arg(long)]
        current: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long, default_value_t = 0.15)]
        max_regression_ratio: f64,
    },
    /// Print resource profile summary from benchmark report
    ResourceProfileSummary {
        #[arg(long)]
        report: PathBuf,
    },
    /// Validate resource budgets in warning or gate mode
    ResourceBudgetCheck {
        #[arg(long)]
        report: PathBuf,
        #[arg(long, default_value_t = false)]
        gate: bool,
    },
    /// Append benchmark report to resource trend series
    ResourceTrendAppend {
        #[arg(long)]
        report: PathBuf,
        #[arg(long)]
        trend: PathBuf,
    },
    /// Verify artifact reproducibility and integrity for local runs
    ArtifactVerify,
    /// Generate observability evidence report from run artifacts
    ObservabilityReport,
    /// Generate documentation inventory and consolidation evidence
    DocsInventory,
    /// Execute end-to-end matrix across binary and crate integration entrypoints
    E2eMatrix,
    /// Report tested and missing fault classes from fault suite catalog
    FaultSummary,
    /// Validate run and cache storage artifacts and report anomalies
    StorageHealth {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    /// Audit run-directory integrity and required storage surfaces
    RunDirAudit {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long, default_value_t = false)]
        strict: bool,
    },
    /// Enumerate unsafe blocks and owner files
    UnsafeAudit,
    /// Enumerate known public error codes and owners
    ErrorCodes,
    /// Print effective config resolution as machine-readable JSON
    ConfigDump {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Print effective security controls for a run policy configuration
    PolicyAudit {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Report implemented, simulated, and aspirational execution modes
    ExecutionModesReport,
    /// Report local semantics and simulated distributed semantics boundaries
    DistributedSemanticsReport,
    /// Enumerate invariant registry and coverage status
    InvariantsReport,
    /// Summarize committed comparison evidence surfaces
    ComparisonEvidenceReport,
    /// Summarize benchmark and performance evidence surfaces
    PerformanceEvidenceReport,
    /// Print execution backend registry and capability descriptors
    BackendRegistryReport,
    /// Verify release artifact command surfaces and machine-readable outputs
    ReleaseArtifactVerify,
    /// Summarize drift classes and checker coverage
    DriftDashboard,
    /// Print repository trust status by domain
    RepoTrustSummary,
    /// Summarize version support by versioned surface
    CompatibilityReport,
    /// Report cache correctness coverage surfaces
    CacheCoverageReport,
    /// Generate foundation review evidence summary
    FoundationReviewReport,
    /// Run full CI-like sequence
    Ci,
    /// Run control-plane foundation suites across checks tests contracts repo and docs
    Foundation {
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        include_slow: bool,
        #[arg(long)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        advisory: bool,
        #[arg(long, default_value_t = false)]
        why: bool,
    },
    /// Run curated high-trust foundation hardening suites only
    FoundationHardening {
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long, default_value_t = false)]
        advisory: bool,
        #[arg(long, default_value_t = false)]
        why: bool,
    },
    /// Run CLI compatibility command
    Compat,
}

include!("cli_control_command.rs");

#[derive(Subcommand)]
pub(super) enum RepoCommand {
    /// Run dependency policy checks
    Deps,
    /// Execute governance suites
    Run {
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        include_slow: bool,
        #[arg(long)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        advisory: bool,
        #[arg(long, default_value_t = false)]
        why: bool,
    },
    /// Show known repo suites
    List,
    /// Explain a repo suite
    Explain {
        #[arg(long)]
        suite: String,
    },
    /// Print evidence taxonomy contract
    EvidenceTaxonomy,
    /// Print evidence ownership ledger
    EvidenceLedger,
    /// Validate evidence metadata completeness and path governance
    EvidenceValidate,
    /// Validate all authoring evidence assets and policy semantics
    ValidateAllAuthoring,
    /// Print effective authoring metadata payload for all assets
    ShowEffectiveAllAuthoring,
    /// Generate authoring coverage and unused-asset reports
    AuthoringCoverageReport {
        #[arg(long, default_value = "evidence/reports/authoring_coverage_by_docs_and_commands.md")]
        out: PathBuf,
        #[arg(long, default_value = "evidence/reports/authoring_unused_assets.md")]
        unused_out: PathBuf,
    },
    /// Normalize evidence ledger ordering and representation
    EvidenceLedgerNormalize {
        #[arg(long, default_value_t = false)]
        check: bool,
    },
    /// Generate evidence directory map from policy
    EvidenceDirectoryMap {
        #[arg(long, default_value = "evidence/_meta/maps/directory_map.json")]
        out: PathBuf,
        #[arg(long, default_value_t = false)]
        create_missing: bool,
    },
    /// Rebuild canonical evidence registry
    EvidenceRegistryRebuild {
        #[arg(long, default_value = "evidence/_meta/registries/evidence_registry.json")]
        out: PathBuf,
        #[arg(long, default_value_t = false)]
        check: bool,
    },
    /// Diff generated evidence registry against current committed registry
    EvidenceRegistryDiff,
    /// Detect evidence files not represented by registry entries
    EvidenceRegistryOrphans,
    /// Detect registry entries whose canonical files do not exist
    EvidenceRegistryMissing,
    /// List battle scenarios and mapped trust properties
    BattleScenarios,
    /// List battle scenarios grouped by trust property
    BattleScenariosByTrust,
    /// List trust properties mapped by scenario
    BattleTrustByScenario,
    /// Generate battle trust coverage and overloaded-scenario reports
    BattleCoverageReport {
        #[arg(long, default_value = "evidence/reports/battle_coverage_gaps.md")]
        gaps_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/battle_overloaded_scenarios.md")]
        overloaded_out: PathBuf,
    },
    /// Validate battle scenario trust-property mappings
    BattleValidate,
    /// Print performance evidence summary from governed metadata
    PerfEvidenceSummary,
    /// Print release-relevant performance evidence set only
    PerfReleaseSet,
    /// Resolve one evidence asset by stable asset id
    EvidenceResolveById {
        #[arg(long)]
        id: String,
    },
    /// Resolve evidence assets by governed family kind
    EvidenceResolveByFamily {
        #[arg(long)]
        family: String,
    },
    /// Resolve evidence assets by trust property id
    EvidenceResolveByTrustProperty {
        #[arg(long)]
        trust_property: String,
    },
    /// Resolve evidence assets by consumer surface id
    EvidenceResolveByConsumer {
        #[arg(long)]
        consumer: String,
    },
    /// Generate reports that map assets to consumers and consumers to families
    EvidenceConsumerReports {
        #[arg(long, default_value = "evidence/reports/evidence_assets_to_consumers.md")]
        assets_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/evidence_consumers_to_families.md")]
        consumers_out: PathBuf,
    },
    /// Emit machine-readable and human-readable evidence suite summary reports
    EvidenceSummaryReport {
        #[arg(long, default_value = "artifacts/reports/evidence_suite_summary.json")]
        json_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/evidence_verification_summary.md")]
        markdown_out: PathBuf,
    },
    /// Generate release evidence summary and release proof scope reports
    ReleaseEvidenceReport {
        #[arg(long, default_value = "evidence/release/release_evidence.json")]
        json_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/what_this_release_proves.md")]
        proves_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/what_this_release_does_not_prove.md")]
        limits_out: PathBuf,
        #[arg(long, default_value = "evidence/reports/unsupported_or_simulated_areas.md")]
        unsupported_out: PathBuf,
    },
    /// Generate foundation hotspot reports for files, functions, API surface, and dependencies
    HotspotReports {
        #[arg(long, default_value = "docs/reports/foundation/FILE_SIZE_HOTSPOT_REPORT.md")]
        file_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/LONG_FUNCTION_HOTSPOT_REPORT.md")]
        function_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/PUBLIC_API_HOTSPOT_REPORT.md")]
        api_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/dependency_cycle_report.md")]
        dep_out: PathBuf,
    },
    /// Generate schema changelog from files under configs/dag/schema
    SchemaChangelog {
        #[arg(long, default_value = "docs/reports/foundation/SCHEMA_CHANGELOG.md")]
        out: PathBuf,
        #[arg(long, default_value = "configs/dag/schema")]
        schema_root: PathBuf,
    },
    /// Generate runtime kernel-boundary and API-scope reports from policy and source
    RuntimeScopeReports {
        #[arg(long, default_value = "docs/reports/foundation/KERNEL_OWNED_MODULES_REPORT.md")]
        kernel_out: PathBuf,
        #[arg(
            long,
            default_value = "docs/reports/foundation/RUNTIME_NON_KERNEL_MODULES_REPORT.md"
        )]
        non_kernel_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/RUNTIME_CONTRACT_BACKING_REPORT.md")]
        contract_backing_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/RUNTIME_OPERATOR_SURFACE_REPORT.md")]
        operator_surface_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/CORE_PUBLIC_API_SURFACE.md")]
        core_api_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/RUNTIME_PUBLIC_API_SURFACE.md")]
        runtime_api_out: PathBuf,
    },
    /// Generate planner hardening report from canonical graph fixtures
    PlannerHardeningReport {
        #[arg(long, default_value = "docs/reports/foundation/PLANNER_HARDENING_REPORT.md")]
        out: PathBuf,
    },
    /// Generate artifact store capability and content-addressed model reports from implementation
    ArtifactCapabilityReports {
        #[arg(long, default_value = "docs/reports/foundation/artifact_store_capability_matrix.md")]
        matrix_out: PathBuf,
        #[arg(long, default_value = "docs/reports/foundation/CONTENT_ADDRESSED_STORAGE_MODEL.md")]
        model_out: PathBuf,
    },
}

include!("cli_verify_command.rs");

#[derive(Subcommand)]
pub(super) enum ScheduleCommand {
    /// Validate schedule registry semantics
    Validate {
        #[arg(long, default_value = "configs/dag/schedules/registry.json")]
        file: PathBuf,
    },
    /// Preview next-fire behavior
    Preview {
        #[arg(long, default_value = "configs/dag/schedules/registry.json")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub(super) enum DagCommand {
    /// Lint DAG style and maintainability rules
    Lint {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Run compact unit harness checks for a DAG
    UnitHarness {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Simulate execution ordering without running adapters
    Simulate {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Dry-run compile and planning preview
    DryRun {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Dump lowered execution plan as structured JSON
    PlanDump {
        #[arg(long)]
        graph: PathBuf,
        #[arg(long, value_delimiter = ',')]
        select: Vec<String>,
    },
    /// Render graph visualization payload from run artifacts
    Visualize {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Emit scheduler timeline view from completed run artifacts
    SchedulerTimeline {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Verify node and run state-machine consistency from run artifacts
    VerifyState {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Debug dependency closure and blocked nodes from a graph
    Debug {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Explain validation diagnostics for a DAG file
    ExplainValidation {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Explain why a node did not run from run artifacts
    ExplainNode {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        node_id: String,
    },
    /// Preview what a run would do under deterministic planning
    Preview {
        #[arg(long)]
        graph: PathBuf,
    },
    /// Export a JSON schema contract skeleton for DAG documents
    SchemaExport {
        #[arg(long, default_value = "docs/spec/dag-schema-v0.1.json")]
        out: PathBuf,
    },
    /// Validate run metadata consistency and optionally repair missing metadata files
    RepairRun {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long, default_value_t = false)]
        apply: bool,
    },
    /// Run recovery fault simulation from a scenario fixture file
    SimulateRecovery {
        #[arg(long)]
        scenario: PathBuf,
    },
    /// Validate a recovery acceptance suite definition
    RecoveryAccept {
        #[arg(long)]
        suite: PathBuf,
    },
    /// Explain run-level behavior from observability artifacts
    ExplainRun {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Summarize run observability artifacts for operators
    RunInspect {
        #[arg(long)]
        run_dir: PathBuf,
    },
    /// Explain artifact lineage and reproducibility from artifacts
    ExplainArtifact {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        artifact_id: String,
    },
    /// Explain schedule creation decision from schedule audit records
    ExplainSchedule {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        schedule_id: String,
    },
    /// Build an investigation bundle report for operator debugging
    InvestigationBundle {
        #[arg(long)]
        run_dir: PathBuf,
        #[arg(long)]
        run_id: String,
    },
    /// Compare current metrics with a baseline metrics report
    DriftReport {
        #[arg(long)]
        current_metrics: PathBuf,
        #[arg(long)]
        baseline_metrics: PathBuf,
        #[arg(long)]
        dag_name: String,
        #[arg(long)]
        baseline_name: String,
    },
}

#[derive(Subcommand)]
pub(super) enum ApiCommand {
    /// Verify public API surface contracts
    PublicSurface,
}

include!("cli_release_command.rs");

pub(super) fn root_command_names() -> Vec<String> {
    let mut commands = Cli::command()
        .get_subcommands()
        .map(|command| command.get_name().to_string())
        .collect::<Vec<_>>();
    commands.push("help".to_string());
    commands
}
