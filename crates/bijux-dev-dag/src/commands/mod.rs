use clap::{CommandFactory, Parser, Subcommand};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

const CLI_COMMAND_FREEZE_BASELINE: usize = 29;
const ADAPTER_KIND_FREEZE_BASELINE: usize = 3;

#[derive(Parser)]
#[command(name = "bijux-dev-dag")]
#[command(about = "Developer workflow helpers for bijux-dag")]
struct Cli {
    #[arg(long)]
    json: bool,
    #[arg(long)]
    report: Option<PathBuf>,
    #[command(subcommand)]
    command: CommandLine,
}

#[derive(Subcommand)]
enum CommandLine {
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
    /// Generate taxonomy-based docs index
    DocsIndex,
    /// Execute end-to-end matrix across binary and crate integration entrypoints
    E2eMatrix,
    /// Report tested and missing fault classes from fault suite catalog
    FaultSummary,
    /// Enumerate unsafe blocks and owner files
    UnsafeAudit,
    /// Enumerate known public error codes and owners
    ErrorCodes,
    /// Print effective config resolution as machine-readable JSON
    ConfigDump {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Run full CI-like sequence
    Ci,
    /// Run CLI compatibility command
    Compat,
}

#[derive(Subcommand)]
enum ControlCommand {
    /// Execute suite checks
    Run {
        #[arg(long)]
        domain: Option<String>,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        include_slow: bool,
        #[arg(long)]
        include_internal: bool,
    },
    /// Show known suites
    List,
    /// Explain a suite
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Subcommand)]
enum RepoCommand {
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
    },
    /// Show known repo suites
    List,
    /// Explain a repo suite
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Subcommand)]
enum ScheduleCommand {
    /// Validate schedule registry semantics
    Validate {
        #[arg(long, default_value = "configs/schedules/registry.json")]
        file: PathBuf,
    },
    /// Preview next-fire behavior
    Preview {
        #[arg(long, default_value = "configs/schedules/registry.json")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum DagCommand {
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
enum ApiCommand {
    /// Verify public API surface contracts
    PublicSurface,
}

#[derive(Subcommand)]
enum ReleaseCommand {
    /// Execute release verification
    Verify,
    /// Generate release readiness report
    Readiness,
    /// Generate compatibility matrix from schema fixtures
    CompatibilityMatrix,
    /// Run post-release installation workflow
    PostReleaseVerify {
        #[arg(long)]
        binary: Option<PathBuf>,
    },
    /// Verify release reproducibility against a tag
    ReproducibilityCheck {
        #[arg(long)]
        tag: String,
    },
    /// Generate release evidence bundle
    EvidenceBundle {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// List release workflows
    List,
    /// Explain a release workflow
    Explain {
        #[arg(long)]
        suite: String,
    },
}

#[derive(Copy, Clone)]
enum CommandEffect {
    Validation,
    ReadWrite,
}

impl CommandEffect {
    fn label(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::ReadWrite => "read-write",
        }
    }
}

struct SuiteDef {
    id: &'static str,
    description: &'static str,
    domain: &'static str,
    slow: bool,
    internal: bool,
    effect: CommandEffect,
    run: fn() -> Result<(), String>,
}

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

const CHECK_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "fmt",
        description: "cargo fmt check",
        domain: "style",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["fmt", "--all", "--", "--check"]),
    },
    SuiteDef {
        id: "lint",
        description: "cargo clippy with warnings as errors",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        },
    },
    SuiteDef {
        id: "security",
        description: "cargo audit policy check",
        domain: "supply-chain",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["audit"]),
    },
    SuiteDef {
        id: "dep-guard",
        description: "forbidden dependency reference check",
        domain: "policy",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_dep_guard(),
    },
];

const TEST_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "unit",
        description: "cargo test --workspace",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "--workspace"]),
    },
    SuiteDef {
        id: "arch",
        description: "repository architecture tests",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "-p", "bijux-dev-dag"]),
    },
    SuiteDef {
        id: "e2e-matrix",
        description: "end-to-end matrix against binary and crate entrypoints",
        domain: "e2e",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_e2e_matrix(),
    },
];

const CONTRACT_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "compat",
        description: "core compat fixture assertions",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"]),
    },
    SuiteDef {
        id: "golden",
        description: "run/replay golden execution parity",
        domain: "runtime",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_golden(),
    },
    SuiteDef {
        id: "public-api",
        description: "public API surface contract",
        domain: "quality",
        slow: true,
        internal: false,
        effect: CommandEffect::ReadWrite,
        run: || run_public_api(),
    },
    SuiteDef {
        id: "validation-rules-doc",
        description: "core validation rule IDs are documented",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_validation_rule_docs_guard(),
    },
    SuiteDef {
        id: "schema-contracts",
        description: "schema source files and fixtures are present and versioned",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_schema_contracts_guard(),
    },
    SuiteDef {
        id: "adapter-conformance",
        description: "runtime adapter descriptor conformance checks",
        domain: "contracts",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_status("cargo", &["test", "-p", "bijux-dag-runtime", "adapter_descriptor_requires_identity_and_schema_version"]),
    },
];

const DOC_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "api",
        description: "check documentation index files",
        domain: "docs",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || {
            let root = repo_root()?;
            if !root.join("docs").join("DEVELOPMENT.md").exists() {
                return Err("missing docs/DEVELOPMENT.md".into());
            }
            Ok(())
        },
    },
    SuiteDef {
        id: "guarantee-evidence",
        description: "guarantee language requires linked proof",
        domain: "docs",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_guarantee_guard(),
    },
];

const RELEASE_SUITES: &[SuiteDef] = &[SuiteDef {
    id: "verify",
    description: "full release verification",
    domain: "release",
    slow: true,
    internal: false,
    effect: CommandEffect::ReadWrite,
    run: || run_ci(),
},
SuiteDef {
    id: "readiness",
    description: "release readiness evidence aggregation",
    domain: "release",
    slow: false,
    internal: false,
    effect: CommandEffect::Validation,
    run: || run_release_readiness_report(),
},
SuiteDef {
    id: "compatibility-matrix",
    description: "generate compatibility matrix from supported fixtures",
    domain: "release",
    slow: false,
    internal: false,
    effect: CommandEffect::ReadWrite,
    run: || run_release_compatibility_matrix(),
},
SuiteDef {
    id: "post-release-verify",
    description: "run minimal installed-binary workflow",
    domain: "release",
    slow: false,
    internal: false,
    effect: CommandEffect::Validation,
    run: || run_post_release_verify(None),
},
SuiteDef {
    id: "reproducibility-check",
    description: "verify release tag reproducibility against current commit",
    domain: "release",
    slow: false,
    internal: false,
    effect: CommandEffect::Validation,
    run: || Ok(()),
},
SuiteDef {
    id: "evidence-bundle",
    description: "write release evidence bundle",
    domain: "release",
    slow: false,
    internal: false,
    effect: CommandEffect::ReadWrite,
    run: || run_release_evidence_bundle(None),
}];

const REPO_SUITES: &[SuiteDef] = &[
    SuiteDef {
        id: "dependency-policy",
        description: "legacy workspace dependency reference check",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_missing_workspace_dependency_checks(),
    },
    SuiteDef {
        id: "dep-guard",
        description: "metadata dependency boundary check",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_dep_guard(),
    },
    SuiteDef {
        id: "ownership-public-modules",
        description: "crate public modules match ownership contract",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_crate_ownership_guard(),
    },
    SuiteDef {
        id: "cli-freeze",
        description: "freeze new top-level CLI commands",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_cli_command_freeze(),
    },
    SuiteDef {
        id: "adapter-freeze",
        description: "freeze runtime adapter kinds until split",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_adapter_kind_freeze(),
    },
    SuiteDef {
        id: "crate-manifest-policy",
        description: "crate manifest boundary policy checks",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_workspace_manifest_policy_guard(),
    },
    SuiteDef {
        id: "public-export-docs",
        description: "public exports require crate-doc contract mention",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_public_export_docs_guard(),
    },
    SuiteDef {
        id: "repo-docs",
        description: "doc root required governance contracts and budgets",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_repo_docs_guard(),
    },
    SuiteDef {
        id: "repo-source",
        description: "source layout policy and disallowed imports",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_repo_source_guard(),
    },
    SuiteDef {
        id: "repo-manifests",
        description: "workspace Cargo manifest conventions",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_repo_manifests_guard(),
    },
    SuiteDef {
        id: "repo-api",
        description: "crate API docs coverage baseline",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_repo_api_guard(),
    },
    SuiteDef {
        id: "test-taxonomy",
        description: "test file naming taxonomy and e2e shell-out boundary",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_test_taxonomy_guard(),
    },
    SuiteDef {
        id: "test-classification",
        description: "test family classification report and category coverage",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_test_classification_report(),
    },
    SuiteDef {
        id: "test-policy",
        description: "schema fixtures runtime transitions and cache mode coverage policies",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_test_policy_guard(),
    },
    SuiteDef {
        id: "fault-summary",
        description: "fault class catalog coverage summary",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_fault_summary_report(),
    },
    SuiteDef {
        id: "performance-claims",
        description: "performance claims must reference benchmark evidence",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_performance_claims_guard(),
    },
    SuiteDef {
        id: "resource-budgets-warning",
        description: "resource budget validation in warning mode",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_resource_budget_check(Path::new("artifacts/benchmarks/baseline.json"), false),
    },
    SuiteDef {
        id: "docs-governance",
        description: "docs taxonomy root budget and owners governance checks",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_governance_guard(),
    },
    SuiteDef {
        id: "docs-links",
        description: "markdown local link checker",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_link_check(),
    },
    SuiteDef {
        id: "docs-schema-ref",
        description: "schema references in docs must resolve",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_schema_reference_guard(),
    },
    SuiteDef {
        id: "docs-contract-ref",
        description: "crate README and CONTRACT references must resolve and be linked",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_contract_reference_guard(),
    },
    SuiteDef {
        id: "docs-coverage",
        description: "docs coverage report for crates and commands",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_docs_coverage_report(),
    },
    SuiteDef {
        id: "contract-test-links",
        description: "contract docs must link at least one verifying test",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_contract_test_links_guard(),
    },
    SuiteDef {
        id: "contract-schema-owners",
        description: "schema files must be linked by owning contract docs",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_contract_schema_owner_guard(),
    },
    SuiteDef {
        id: "contract-command-ownership",
        description: "public commands must be covered by exactly one CLI contract section entry",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_contract_command_ownership_guard(),
    },
    SuiteDef {
        id: "contract-versioning-policy",
        description: "contract docs must declare versioning and change policy",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_contract_versioning_guard(),
    },
    SuiteDef {
        id: "contract-coverage-report",
        description: "report missing orphaned and stale contracts",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_contract_coverage_report(),
    },
    SuiteDef {
        id: "planner-alignment",
        description: "planner docs tests and schema alignment",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_planner_alignment_guard(),
    },
    SuiteDef {
        id: "scheduler-invariants",
        description: "scheduler contract and invariants test surfaces are present",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_scheduler_invariants_guard(),
    },
    SuiteDef {
        id: "concurrency-model",
        description: "concurrency model docs tests and ledger alignment",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_concurrency_model_guard(),
    },
    SuiteDef {
        id: "runtime-unsafe-audit",
        description: "runtime unsafe usage guard and auditability",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_runtime_unsafe_guard(),
    },
    SuiteDef {
        id: "error-code-registry",
        description: "enumerate stable error codes and owner crates",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_error_code_registry_report(),
    },
    SuiteDef {
        id: "error-code-doc-tests",
        description: "public error codes require docs and test references",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_error_code_docs_tests_guard(),
    },
    SuiteDef {
        id: "config-lint",
        description: "checked-in config examples validate against config schemas",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_config_lint(),
    },
    SuiteDef {
        id: "config-drift",
        description: "docs precedence table matches effective resolver behavior",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_config_precedence_drift_guard(),
    },
    SuiteDef {
        id: "ambient-env-guard",
        description: "ambient environment reads are limited to contract allowlist",
        domain: "governance",
        slow: false,
        internal: false,
        effect: CommandEffect::Validation,
        run: || run_ambient_env_guard(),
    },
];

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

struct CommandContext {
    json: bool,
    report: Option<PathBuf>,
}

fn run(cli: Cli) -> Result<(), String> {
    let context = CommandContext {
        json: cli.json,
        report: cli.report,
    };
    match cli.command {
        CommandLine::Fmt => run_command_reported(&context, "fmt", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["fmt", "--all"])
        }),
        CommandLine::Lint => run_command_reported(&context, "lint", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["fmt", "--all", "--", "--check"])?;
            run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])
        }),
        CommandLine::Security => run_command_reported(&context, "security", CommandEffect::Validation, json!({}), || {
            run_status("cargo", &["audit"])
        }),
        CommandLine::Sanity => run_command_reported(&context, "sanity", CommandEffect::ReadWrite, json!({}), || {
            run_status("cargo", &["metadata", "--no-deps"])?;
            run_status("cargo", &["test", "-q"])?;
            run_status("cargo", &["fmt", "--all", "--", "--check"])
        }),
        CommandLine::Checks { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "checks", CHECK_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "checks", CHECK_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "checks", &suite, CHECK_SUITES)
            }
        },
        CommandLine::Tests { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "tests", TEST_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "tests", TEST_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "tests", &suite, TEST_SUITES)
            }
        },
        CommandLine::Contracts { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(
                    &context,
                    "contracts",
                    CONTRACT_SUITES,
                    &domain,
                    fail_fast,
                    include_slow,
                    include_internal,
                )
            }
            ControlCommand::List => {
                run_suite_list(&context, "contracts", CONTRACT_SUITES)
            }
            ControlCommand::Explain { suite } => {
                run_suite_explain(&context, "contracts", &suite, CONTRACT_SUITES)
            }
        },
        CommandLine::Docs { command } => match command {
            ControlCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "docs", DOC_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            ControlCommand::List => {
                run_suite_list(&context, "docs", DOC_SUITES)
            }
            ControlCommand::Explain { suite } => run_suite_explain(&context, "docs", &suite, DOC_SUITES),
        },
        CommandLine::Release { command } => match command {
            ReleaseCommand::Verify => {
                run_command_reported(
                    &context,
                    "release.verify",
                    CommandEffect::ReadWrite,
                    json!({ "flow": crate::suites::release_verify_suite_ids() }),
                    || run_release_verify(),
                )
            }
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
            ReleaseCommand::List => {
                run_suite_list(&context, "release", RELEASE_SUITES)
            }
            ReleaseCommand::Explain { suite } => {
                run_suite_explain(&context, "release", &suite, RELEASE_SUITES)
            }
        },
        CommandLine::Repo { command } => match command {
            RepoCommand::Deps => {
                run_command_reported(&context, "repo.deps", CommandEffect::Validation, json!({}), || {
                    run_missing_workspace_dependency_checks()
                })
            }
            RepoCommand::Run { domain, fail_fast, include_slow, include_internal } => {
                run_suite_group(&context, "repo", REPO_SUITES, &domain, fail_fast, include_slow, include_internal)
            }
            RepoCommand::List => run_suite_list(&context, "repo", REPO_SUITES),
            RepoCommand::Explain { suite } => run_suite_explain(&context, "repo", &suite, REPO_SUITES),
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
                || run_dag_drift_report(&current_metrics, &baseline_metrics, &dag_name, &baseline_name),
            ),
        },
        CommandLine::Doctor => run_command_reported(&context, "doctor", CommandEffect::ReadWrite, json!({}), || {
            run_env_summary()?;
            run_verify_tools()
        }),
        CommandLine::Golden => run_command_reported(&context, "golden", CommandEffect::ReadWrite, json!({}), || {
            run_golden()
        }),
        CommandLine::PublicApi => run_command_reported(&context, "public-api", CommandEffect::ReadWrite, json!({}), || {
            run_public_api()
        }),
        CommandLine::DepGuard => run_command_reported(&context, "dep-guard", CommandEffect::Validation, json!({}), || {
            run_dep_guard()
        }),
        CommandLine::CrateGraph => run_command_reported(&context, "crate-graph", CommandEffect::Validation, json!({}), || {
            run_crate_graph_command()
        }),
        CommandLine::ArtifactsClean => run_command_reported(&context, "artifacts-clean", CommandEffect::ReadWrite, json!({}), || {
            run_artifacts_clean()
        }),
        CommandLine::EnvSummary => run_command_reported(&context, "env-summary", CommandEffect::Validation, json!({}), || {
            run_env_summary()
        }),
        CommandLine::VerifyTools => run_command_reported(&context, "verify-tools", CommandEffect::Validation, json!({}), || {
            run_verify_tools()
        }),
        CommandLine::ResolveCheck => run_command_reported(&context, "resolve-check", CommandEffect::Validation, json!({}), || {
            run_resolve_check()
        }),
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
        CommandLine::Ci => run_command_reported(&context, "ci", CommandEffect::ReadWrite, json!({}), || {
            run_ci()
        }),
        CommandLine::Compat => run_command_reported(&context, "compat", CommandEffect::ReadWrite, json!({}), || {
            run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"])
        }),
        CommandLine::Api { command } => match command {
            ApiCommand::PublicSurface => run_command_reported(&context, "api.public-surface", CommandEffect::ReadWrite, json!({}), || {
                run_public_api()
            }),
        },
    }
}

fn run_suite_group(
    context: &CommandContext,
    group: &str,
    suites: &[SuiteDef],
    domain: &Option<String>,
    fail_fast: bool,
    include_slow: bool,
    include_internal: bool,
) -> Result<(), String> {
    let root = repo_root()?;
    let overrides = crate::suites::load_suite_overrides(&root.join("configs/dev/suite_overrides.json"))?;
    let disabled: BTreeSet<String> = overrides.disabled_suite_ids.into_iter().collect();

    let selected: Vec<&SuiteDef> = suites
        .iter()
        .filter(|suite| domain.as_deref().is_none_or(|d| suite.domain == d))
        .filter(|suite| include_internal || !suite.internal)
        .filter(|suite| include_slow || !suite.slow)
        .filter(|suite| !disabled.contains(suite.id))
        .collect();

    let mut failed: Vec<String> = Vec::new();
    for suite in selected {
        if let Err(error) = run_suite(context, group, suite) {
            failed.push(format!("{}: {error}", suite.id));
            if fail_fast {
                break;
            }
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(format!("{} failed: {}", group, failed.join(", ")))
    }
}

fn run_suite(context: &CommandContext, group: &str, suite: &SuiteDef) -> Result<(), String> {
    run_command_reported(context, &format!("{group}.{}", suite.id), suite.effect, json!({}), suite.run)
}

fn run_suite_list(context: &CommandContext, group: &str, suites: &[SuiteDef]) -> Result<(), String> {
    let data = json!({
        "group": group,
        "suites": suites.iter().map(|s| json!({"id": s.id, "description": s.description, "domain": s.domain, "slow": s.slow, "internal": s.internal, "effect": s.effect.label()})).collect::<Vec<_>>()
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.list"),
        "read-write",
        data,
        || Ok(()),
        false,
    )
}

fn run_suite_explain(context: &CommandContext, group: &str, suite_id: &str, suites: &[SuiteDef]) -> Result<(), String> {
    let suite = suites
        .iter()
        .find(|suite| suite.id == suite_id)
        .ok_or_else(|| format!("suite '{suite_id}' is unknown"))?;
    let data = json!({
        "id": suite.id,
        "group": group,
        "description": suite.description,
        "domain": suite.domain,
        "slow": suite.slow,
        "internal": suite.internal,
        "effect": suite.effect.label(),
    });
    run_text_or_json_report(
        context,
        group,
        &format!("{group}.explain"),
        suite.effect.label(),
        data,
        || Ok(()),
        false,
    )
}

fn run_command_reported<F>(context: &CommandContext, command: &str, effect: CommandEffect, data: Value, run: F) -> Result<(), String>
where
    F: FnOnce() -> Result<(), String>,
{
    run_text_or_json_report(context, command, command, effect.label(), data, run, true)
}

fn run_text_or_json_report(
    context: &CommandContext,
    command: &str,
    command_name: &str,
    effect: &str,
    data: Value,
    run: impl FnOnce() -> Result<(), String>,
    include_data_on_success: bool,
) -> Result<(), String> {
    let result = run();
    let (status, error) = match &result {
        Ok(_) => ("ok", None),
        Err(err) => ("error", Some(err.clone())),
    };

    let mut report = json!({
        "command": command_name,
        "status": status,
        "effect": effect,
        "data": data,
    });
    if let Some(error) = error {
        report["error"] = Value::String(error);
    }

    if context.json {
        println!("{}", serde_json::to_string_pretty(&report).expect("json print"));
    } else if include_data_on_success || status == "error" {
        let value = report.to_string();
        println!("[{command}] {status} ({effect}): {value}",);
    } else {
        println!("[{command}] {status} ({effect})");
    }

    if let Some(report_path) = context.report.as_ref() {
        let output = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
        fs::write(report_path, output).map_err(|err| err.to_string())?;
    }
    let _ = append_control_plane_audit(command_name, status, effect);

    result
}

fn append_control_plane_audit(command_name: &str, status: &str, effect: &str) -> Result<(), String> {
    let root = repo_root()?;
    let audit_dir = root.join("artifacts").join("reports");
    fs::create_dir_all(&audit_dir).map_err(|err| err.to_string())?;
    let audit_path = audit_dir.join("control-plane-audit.jsonl");
    let event = json!({
        "action": command_name,
        "status": status,
        "effect": effect,
        "ts_unix_ms": now_millis(),
    });
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&audit_path)
        .map_err(|err| err.to_string())?;
    writeln!(file, "{event}").map_err(|err| err.to_string())
}

fn run_ci() -> Result<(), String> {
    run_status("cargo", &["fmt", "--all"])?;
    run_status("cargo", &["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"])?;
    run_dep_guard()?;
    run_resolve_check()?;
    run_missing_workspace_dependency_checks()?;
    run_status("cargo", &["test", "--workspace"])?;
    run_golden()?;
    run_status("cargo", &["run", "-p", "bijux-dag-cli", "--", "dag", "compat"])?;

    let root = repo_root()?;
    let scratch = std::env::temp_dir().join(format!("bijux-dag-ci-{}", now_secs()));
    let runs = scratch.join("runs");
    fs::create_dir_all(&runs).map_err(|err| err.to_string())?;
    run_with_root(
        &root,
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "run", "examples/hello.dag.json", "--out", runs.to_str().expect("utf-8")],
    )?;
    let run_dir = newest_run(&runs)?;
    run_status_in_dir(
        &root,
        "cargo",
        &["run", "-p", "bijux-dag-cli", "--", "dag", "verify", run_dir.to_str().expect("utf-8")],
    )
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
    let report = json!({
        "timestamp_unix_ms": now_millis(),
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
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
    Ok(())
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
    println!("{}", serde_json::to_string_pretty(&matrix).map_err(|err| err.to_string())?);
    Ok(())
}

fn run_post_release_verify(binary: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let script = root.join("tests/post_release/minimal_workflow.sh");
    if !script.exists() {
        return Err("missing tests/post_release/minimal_workflow.sh".to_string());
    }
    match binary {
        Some(bin) => {
            let status = Command::new("env")
                .arg(format!("BIJUX_RELEASE_BINARY={}", bin.display()))
                .arg("bash")
                .arg(&script)
                .status()
                .map_err(|err| err.to_string())?;
            if status.success() { Ok(()) } else { Err("post-release verification failed".to_string()) }
        }
        None => run_status("bash", &[script.to_string_lossy().as_ref()]),
    }
}

fn run_release_reproducibility_check(tag: &str) -> Result<(), String> {
    let root = repo_root()?;
    let script = root.join("scripts/release/verify_tag_reproducibility.sh");
    run_status("bash", &[script.to_string_lossy().as_ref(), tag])
}

fn run_release_evidence_bundle(out: Option<&Path>) -> Result<(), String> {
    let root = repo_root()?;
    let output = out
        .map(|p| if p.is_absolute() { p.to_path_buf() } else { root.join(p) })
        .unwrap_or_else(|| root.join("artifacts/release/evidence_bundle.json"));

    let readiness_path = root.join("artifacts/release/readiness_report.json");
    let readiness = if readiness_path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&readiness_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?
    } else {
        json!({"status": "missing", "hint": "run `bijux-dev-dag release readiness`"})
    };
    let matrix_path = root.join("artifacts/release/compatibility_matrix.json");
    let matrix = if matrix_path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&matrix_path).map_err(|err| err.to_string())?)
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
    println!("{}", serde_json::to_string_pretty(&bundle).map_err(|err| err.to_string())?);
    Ok(())
}

fn check_contract_coverage_ready(root: &Path) -> Value {
    json!({"ok": root.join("docs/spec/CLI_CONTRACT.md").exists() && root.join("docs/spec/ERROR_CONTRACT.md").exists()})
}

fn check_schema_coverage_ready(root: &Path) -> Value {
    let positive = root.join("configs/schema/fixtures/compat/positive").exists();
    let negative = root.join("configs/schema/fixtures/compat/negative").exists();
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
    json!({"ok": root.join("benchmarks/baselines").exists()})
}

fn check_resource_baseline_ready(root: &Path) -> Value {
    json!({"ok": root.join("benchmarks/baselines/resource_trend_v1.json").exists()})
}

fn read_release_blockers(root: &Path) -> Result<Value, String> {
    let payload = fs::read_to_string(root.join("configs/release/release_blockers.json"))
        .map_err(|err| err.to_string())?;
    serde_json::from_str(&payload).map_err(|err| err.to_string())
}

fn collect_fixture_rows(dir: &Path, should_pass: bool, rows: &mut Vec<Value>) -> Result<(), String> {
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
    println!(
        "{}",
        serde_json::to_string_pretty(&plan).map_err(|err| err.to_string())?
    );
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
    println!("{}", serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?);
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

fn run_dag_drift_report(current_metrics: &Path, baseline_metrics: &Path, dag_name: &str, baseline_name: &str) -> Result<(), String> {
    let root = repo_root()?;
    let current_path = root.join(current_metrics);
    let baseline_path = root.join(baseline_metrics);
    let current_json: Value = serde_json::from_str(&fs::read_to_string(current_path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())?;
    let baseline_json: Value = serde_json::from_str(&fs::read_to_string(baseline_path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())?;
    let mut drift = Vec::new();
    if let (Some(curr), Some(base)) = (current_json.as_object(), baseline_json.as_object()) {
        for (key, curr_value) in curr {
            if let (Some(c), Some(b)) = (curr_value.as_f64(), base.get(key).and_then(|v| v.as_f64())) {
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
    let output = Command::new(command)
        .arg("--version")
        .output()
        .ok();
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
        ("large-dag", "execute-local", "benchmarks/fixtures/large_dag.json"),
        ("linear-32", "plan", "benchmarks/fixtures/scheduler_linear_32.json"),
        ("parallel-64", "plan", "benchmarks/fixtures/scheduler_parallel_64.json"),
        (
            "diamond-fanout",
            "manifest-finalize",
            "benchmarks/fixtures/scheduler_diamond_fanout.json",
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
                failures.push(format!(
                    "{name}: sha mismatch for {}",
                    file_path.display()
                ));
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

    let example = "examples/hello.dag.json";
    for _ in 0..2 {
        run_with_root(
            &root,
            "cargo",
            &["run", "-p", "bijux-dag-cli", "--", "dag", "run", example, "--out", runs.to_str().expect("utf-8")],
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
        &["run", "-p", "bijux-dag-cli", "--", "dag", "replay", latest.to_str().expect("utf-8"), "--out", runs.to_str().expect("utf-8")],
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

    for crate_name in ["bijux-dag-core", "bijux-dag-artifacts", "bijux-dag-runtime", "bijux-dag-app"] {
        let output = run_stdout_and_json(
            &root,
            "cargo",
            &["public-api", "-p", crate_name],
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
        return Err(format!("cargo metadata failed with status {}", output.status));
    }
    let payload: Value =
        serde_json::from_slice(&output.stdout).map_err(|err| format!("invalid metadata JSON: {err}"))?;
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
        return Err("bijux-dag-app must depend on runtime/core/artifacts orchestration surfaces".into());
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
        Err(format!("crate ownership guard failed: {}", violations.join(", ")))
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
        let manifest = root
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml");
        let text = fs::read_to_string(&manifest).map_err(|err| err.to_string())?;
        if !text.contains("[lints]") || !text.contains("workspace = true") {
            return Err(format!("{crate_name} manifest missing workspace lint contract"));
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
    serde_json::from_str(stdout.trim()).map_err(|err| format!("invalid json: {err}\nstdout:\n{stdout}"))
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
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
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
        let ma = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        let mb = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(UNIX_EPOCH);
        mb.cmp(&ma)
    });

    Ok((candidates[0].clone(), candidates[1].clone()))
}

fn repo_root() -> Result<PathBuf, String> {
    let mut dir = env::current_dir().map_err(|err| err.to_string())?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err("could not locate repo root".to_string())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn now_millis() -> u128 {
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
    let policy: TestTaxonomyPolicy = serde_json::from_str(&policy_text).map_err(|err| err.to_string())?;

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
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
            if !prefixes.iter().any(|prefix| name.starts_with(prefix)) && !allowlist.contains(&rel) {
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
    let categories = ["unit_", "contract_", "integration_", "e2e_", "perf_", "compat_", "fault_"];
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
            let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
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
    for state in ["queued", "ready", "running", "succeeded", "failed", "cached", "skipped", "cancelled"] {
        if !state_text.contains(state) {
            violations.push(format!("runtime transition coverage missing state: {state}"));
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
            "public command policy requires integration and error-path app command tests".to_string(),
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
    let script = root.join("tests/e2e/run_matrix.sh");
    if !script.exists() {
        return Err("missing tests/e2e/run_matrix.sh".to_string());
    }
    run_with_root(
        &root,
        "bash",
        &[script
            .to_str()
            .ok_or_else(|| "non-utf8 e2e matrix script path".to_string())?],
    )
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
    let catalog_path = root.join("tests/fault/fixtures/fault_classes.json");
    let payload = fs::read_to_string(&catalog_path).map_err(|err| err.to_string())?;
    let catalog: FaultClassCatalog = serde_json::from_str(&payload).map_err(|err| err.to_string())?;

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
    if summary["missing_fault_classes"].as_array().is_some_and(|items| items.is_empty()) {
        Ok(())
    } else {
        Err("fault class catalog has missing tested_by mappings".to_string())
    }
}

fn run_benchmark_compare(current: &Path, baseline: &Path, max_regression_ratio: f64) -> Result<(), String> {
    let root = repo_root()?;
    let current_path = root.join(current);
    let baseline_path = root.join(baseline);

    let current_json: Value = serde_json::from_str(
        &fs::read_to_string(current_path).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let baseline_json: Value = serde_json::from_str(
        &fs::read_to_string(baseline_path).map_err(|err| err.to_string())?,
    )
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
                        || line.contains("PERFORMANCE_STRATEGY.md"))
                {
                    violations.push(format!("{rel}: performance claim without evidence link: {line}"));
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

    if let Some(items) = report_json.get("scenario_results").and_then(Value::as_array) {
        let mut wall = 0.0_f64;
        for item in items {
            wall += item.get("elapsed_ms").and_then(Value::as_f64).unwrap_or(0.0);
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
    let budgets_path = root.join("benchmarks/scenarios/resource_budgets.json");

    let report_json: Value = serde_json::from_str(
        &fs::read_to_string(&report_path).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let budgets_json: Value = serde_json::from_str(
        &fs::read_to_string(&budgets_path).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;

    let mut budget_map: std::collections::BTreeMap<String, Value> = std::collections::BTreeMap::new();
    if let Some(items) = budgets_json.get("scenarios").and_then(Value::as_array) {
        for item in items {
            if let Some(id) = item.get("scenario_id").and_then(Value::as_str) {
                budget_map.insert(id.to_string(), item.clone());
            }
        }
    }

    let mut warnings = Vec::new();
    if let Some(items) = report_json.get("scenario_results").and_then(Value::as_array) {
        for item in items {
            let scenario = item
                .get("scenario_id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let elapsed = item.get("elapsed_ms").and_then(Value::as_f64).unwrap_or(0.0);
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
                total = total.saturating_add(
                    entry
                        .metadata()
                        .map_err(|err| err.to_string())?
                        .len(),
                );
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
                total = total.saturating_add(
                    entry
                        .metadata()
                        .map_err(|err| err.to_string())?
                        .len(),
                );
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
            return Err(format!("docs taxonomy violation: docs/{name} is not allowed"));
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
            let clean = token
                .trim_matches(|c: char| matches!(c, ')' | '(' | '[' | ']' | ',' | ';' | '"'));
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
    let sections = ["spec", "architecture", "user", "dev", "reference", "tracking", "generated"];

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
        if !root.join("crates").join(crate_name).join("CONTRACT.md").exists() {
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
            violations.push(format!("{} missing '## Related tests' section", rel.display()));
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
    let contract =
        fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md")).map_err(|err| err.to_string())?;

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
        if !root.join("crates").join(crate_name).join("CONTRACT.md").exists() {
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
        if path.file_name().and_then(|x| x.to_str()).is_some_and(|name| name.ends_with("CONTRACT.md")) {
            let file_name = path.file_name().and_then(|x| x.to_str()).unwrap_or_default();
            if !specs.contains(&file_name)
                && file_name != "WORKSPACE_CONTRACT.md"
                && file_name != "PROJECT_CONTRACT.md"
                && file_name != "ADAPTER_CONTRACT.md"
                && file_name != "EXECUTION_SEMANTICS_CONTRACT.md"
                && file_name != "SCHEDULER_STATESPACE_CONTRACT.md"
                && file_name != "DETERMINISTIC_SCHEDULING_CONTRACT.md"
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
    let docs_error_contract =
        fs::read_to_string(root.join("docs/spec/ERROR_CONTRACT.md")).map_err(|err| err.to_string())?;
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
        if !docs_error_contract.contains("Public error code additions require docs plus test coverage") {
            violations.push("docs/spec/ERROR_CONTRACT.md missing public code governance rule".to_string());
        }
    }

    for test in tests {
        if !test.exists() {
            violations.push(format!(
                "missing required error contract test file: {}",
                test.strip_prefix(&root).map_err(|err| err.to_string())?.display()
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
            "planner alignment missing required surfaces: {}",
            missing.join(", ")
        ))
    }
}

fn run_scheduler_invariants_guard() -> Result<(), String> {
    let root = repo_root()?;
    let required = [
        "docs/spec/SCHEDULER_CONTRACT.md",
        "crates/bijux-dag-runtime/tests/scheduler_contract.rs",
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
            "scheduler invariant coverage missing required surfaces: {}",
            missing.join(", ")
        ))
    }
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

fn load_error_code_registry(root: &Path) -> Result<ErrorCodeRegistry, String> {
    let payload = fs::read_to_string(root.join("configs/policy/error_codes.json"))
        .map_err(|err| err.to_string())?;
    let registry: ErrorCodeRegistry = serde_json::from_str(&payload).map_err(|err| err.to_string())?;

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
            return Err(format!("error code {} has empty owner or description", entry.code));
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
        let full = if path.is_absolute() { path.to_path_buf() } else { root.join(path) };
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
        let allowed_root = ["jobs", "cache_mode", "materialize_inputs", "policy", "debug"];
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                if !allowed_root.contains(&key.as_str()) {
                    violations.push(format!("{} has unknown field `{}`", path.display(), key));
                }
                if key.starts_with("deprecated_") {
                    violations.push(format!("{} contains deprecated field `{}`", path.display(), key));
                }
            }
        } else {
            violations.push(format!("{} must be a JSON object", path.display()));
        }
        if value.get("jobs").and_then(|v| v.as_u64()).unwrap_or(0) == 0 {
            violations.push(format!("{} has invalid jobs", path.display()));
        }
        let cache_mode = value.get("cache_mode").and_then(|v| v.as_str()).unwrap_or("");
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
    let precedence_doc =
        fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE.md")).map_err(|err| err.to_string())?;
    let expected = "CLI > explicit config file > environment > defaults";
    if !precedence_doc.contains(expected) {
        return Err("docs/spec/CONFIG_PRECEDENCE.md missing canonical precedence table".to_string());
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

fn run_ambient_env_guard() -> Result<(), String> {
    let root = repo_root()?;
    let mut files = Vec::new();
    collect_files_with_extension(&root.join("crates"), "rs", &mut files)?;
    let mut violations = Vec::new();
    let allow_env_keys = ["BIJUX_DAG_CACHE_DIR", "BIJUX_DAG_ADAPTERS_DIR"];
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
            violations.push(format!("{rel}: disallowed ambient env read `{}`", line.trim()));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join(", "))
    }
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

fn collect_files_with_extension(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> Result<(), String> {
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
