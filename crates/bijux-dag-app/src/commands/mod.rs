use clap::{Args, Command, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

mod surface_policy;

pub(crate) use surface_policy::{
    command_access_denial, command_access_for_path, lane_label, CommandAccessDenial,
    CommandAvailability, CommandLane,
};

const PUBLIC_ROOT_COMMANDS: &[&str] = &[
    "artifact",
    "artifact-inspect",
    "cache",
    "commands",
    "diff",
    "doctor",
    "explain",
    "plan",
    "replay",
    "run",
    "runs",
    "validate",
    "verify",
    "version",
];

const HIDDEN_DEFAULT_COMMAND_PATHS: &[&str] = &["artifact fetch", "explain-plan", "run-bundle"];

const DENY_NETWORK_HELP: &str =
    "deny declared network effects; shell execution does not firewall sockets, and container execution only enforces this when the runtime can honor a no-network mode";
const DENY_ENV_HELP: &str =
    "deny declared environment effects; this is a policy gate over declared DAG effects, not a syscall sandbox over arbitrary process reads";
const DENY_CLOCK_HELP: &str =
    "deny declared clock effects; this does not virtualize wall clock access inside spawned processes";
const CLEAN_ENV_HELP: &str =
    "run with the curated bijux environment instead of inheriting the full parent environment; this shapes environment variables only";
const HERMETIC_HELP: &str =
    "enable the best-effort local policy profile by forcing --deny-network, --deny-clock, and --clean-env; this does not claim syscall sandboxing or host filesystem isolation";
const RESUME_RUN_HELP: &str =
    "resume an existing run directory by run id, reusing only nodes whose persisted outputs still match their recorded evidence";
const RESUME_FAILURE_MODE_HELP: &str =
    "choose whether nodes that cannot be safely reused are rerun or rejected during resume";
const RESOURCE_CAPACITY_HELP: &str =
    "declare a named runtime capacity as <name=count>; repeat for resources such as license tokens or database slots";
const REPLAY_SANDBOX_HELP: &str =
    "forbid replay outputs from being written inside the source run directory; this is a write-boundary check, not a process sandbox";
const REPLAY_SOURCE_RUN_ID_HELP: &str =
    "resolve the replay source run by run id instead of passing a source run directory path";
const REPLAY_SOURCE_RUN_ROOT_HELP: &str =
    "root directory used when resolving --source-run-id; defaults to the replay output root when omitted";
const RUN_PROGRESS_HELP: &str =
    "show live progress for `bijux-dag run`; `compact` renders operator-readable updates on stderr in human mode and streams `dag.run.progress` JSON lines on stdout when `--json` is active";
const EXECUTION_BACKEND_HELP: &str =
    "choose the node execution backend; `kubernetes` runs container nodes as Kubernetes Jobs through kubectl plus a shared persistent volume claim, and `slurm` submits nodes through sbatch and polls sacct until each job reaches a terminal state";
const DAG_PRODUCT_SENTENCE: &str =
    "bijux-dag v0.4.1 is a local-first DAG runtime for reproducible workflows with explicit graph contracts, deterministic execution records, verified artifacts, cache explanation, and replayable run bundles.";
const ROOT_HELP_BOUNDARY_HELP: &str =
    "v0.4.0 surface truth table:\n  stable: validate, plan, run, replay, runs ..., artifact, artifact-inspect, diff, explain, verify, doctor, cache, version, commands\n  experimental: hidden explicit-path routes require deliberate inventory with `bijux-dag commands --lane experimental`\n  simulated: modeled platform namespaces require `bijux-dag commands --lane simulated` to inventory and BIJUX_DAG_ENABLE_SIMULATED=1 to execute\n  internal: maintainer namespaces require `bijux-dag commands --lane internal` to inventory and BIJUX_DAG_ENABLE_INTERNAL=1 to execute\n  future: generic hpc beyond the shared-filesystem slurm lane, public remote workers, and public scheduler services are not part of v0.4.0\n\nUse `bijux-dag commands` for the stable operator surface and add `--lane` only when you intentionally need repository-owned non-stable routes.";

pub(crate) fn root_command_hidden_from_public_help(name: &str) -> bool {
    !PUBLIC_ROOT_COMMANDS.contains(&name)
}

pub(crate) fn command_path_hidden_from_public_help(path: &str) -> bool {
    let head = path.split(' ').next().unwrap_or(path);
    if root_command_hidden_from_public_help(head) {
        return true;
    }
    HIDDEN_DEFAULT_COMMAND_PATHS.contains(&path)
}

pub(crate) fn hide_non_public_help(mut command: Command, prefix: &str) -> Command {
    let subcommand_names = command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_string())
        .collect::<Vec<_>>();
    for name in subcommand_names {
        let path = if prefix.is_empty() { name.clone() } else { format!("{prefix} {name}") };
        command = command.mut_subcommand(&name, |subcommand| {
            let subcommand = hide_non_public_help(subcommand, &path);
            if command_path_hidden_from_public_help(&path) {
                subcommand.hide(true)
            } else {
                subcommand
            }
        });
    }
    command
}

#[derive(Parser)]
#[command(
    about = DAG_PRODUCT_SENTENCE,
    long_about = None,
    after_help = ROOT_HELP_BOUNDARY_HELP
)]
pub(crate) struct DagCli {
    #[arg(long, global = true)]
    pub(crate) json: bool,
    #[arg(long, global = true)]
    pub(crate) quiet: bool,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum CommandCatalogLaneArg {
    Stable,
    Experimental,
    Simulated,
    Internal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ExecutionBackendArg {
    Local,
    Kubernetes,
    Slurm,
}

#[derive(Args)]
pub(crate) struct RunCommandArgs {
    #[arg(required = true)]
    pub(crate) dags: Vec<PathBuf>,
    #[arg(long)]
    pub(crate) out: PathBuf,
    #[arg(long = "input", action = clap::ArgAction::Append)]
    pub(crate) input: Vec<String>,
    #[arg(long)]
    pub(crate) inputs_file: Option<PathBuf>,
    #[arg(long)]
    pub(crate) run_id: Option<String>,
    #[arg(long, help = RESUME_RUN_HELP)]
    pub(crate) resume_run: Option<String>,
    #[arg(long, value_enum, default_value_t = ResumeFailureModeArg::RerunIncomplete, help = RESUME_FAILURE_MODE_HELP)]
    pub(crate) resume_failure_mode: ResumeFailureModeArg,
    #[arg(long)]
    pub(crate) latest: Option<PathBuf>,
    #[arg(long, default_value_t = 1)]
    pub(crate) jobs: usize,
    #[arg(long)]
    pub(crate) cpu_budget: Option<u32>,
    #[arg(long)]
    pub(crate) memory_budget_mb: Option<u32>,
    #[arg(long)]
    pub(crate) gpu_device_budget: Option<u32>,
    #[arg(
        long = "resource-capacity",
        action = clap::ArgAction::Append,
        value_name = "name=count",
        help = RESOURCE_CAPACITY_HELP
    )]
    pub(crate) resource_capacity: Vec<String>,
    #[arg(long)]
    pub(crate) node_timeout_ms: Option<u64>,
    #[arg(long)]
    pub(crate) run_timeout_ms: Option<u64>,
    #[arg(long, value_enum, default_value_t = RunTimeoutBehaviorArg::FinishRunning)]
    pub(crate) run_timeout_behavior: RunTimeoutBehaviorArg,
    #[arg(long, help = DENY_NETWORK_HELP)]
    pub(crate) deny_network: bool,
    #[arg(long, help = DENY_ENV_HELP)]
    pub(crate) deny_env: bool,
    #[arg(long, help = DENY_CLOCK_HELP)]
    pub(crate) deny_clock: bool,
    #[arg(long, help = CLEAN_ENV_HELP)]
    pub(crate) clean_env: bool,
    #[arg(long, help = HERMETIC_HELP)]
    pub(crate) hermetic: bool,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) select: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) exclude: Vec<String>,
    #[arg(long = "to-node", action = clap::ArgAction::Append)]
    pub(crate) to_node: Vec<String>,
    #[arg(long)]
    pub(crate) dependency_closure: bool,
    #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
    pub(crate) materialize_inputs: MaterializeModeArg,
    #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
    pub(crate) cache: CacheModeArg,
    #[arg(long)]
    pub(crate) cache_dir: Option<PathBuf>,
    #[arg(long)]
    pub(crate) remote_cache_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = AbsolutePathPolicyArg::AllowLiteral)]
    pub(crate) absolute_path_policy: AbsolutePathPolicyArg,
    #[arg(long)]
    pub(crate) preflight_only: bool,
    #[arg(long)]
    pub(crate) explain_scheduling: bool,
    #[arg(long, value_enum, default_value_t = RunProgressArg::Off, help = RUN_PROGRESS_HELP)]
    pub(crate) progress: RunProgressArg,
    #[arg(long, value_enum, default_value_t = ExecutionBackendArg::Local, help = EXECUTION_BACKEND_HELP)]
    pub(crate) backend: ExecutionBackendArg,
    #[arg(long, default_value = "bijux")]
    pub(crate) kubernetes_namespace: String,
    #[arg(long)]
    pub(crate) kubernetes_volume_claim: Option<String>,
    #[arg(long)]
    pub(crate) kubernetes_shared_root: Option<PathBuf>,
    #[arg(long, default_value = "general")]
    pub(crate) slurm_queue: String,
    #[arg(long, default_value = "cpu")]
    pub(crate) slurm_partition: String,
}

#[derive(Args)]
pub(crate) struct ReplayCommandArgs {
    #[arg(required_unless_present = "source_run_id", conflicts_with = "source_run_id")]
    pub(crate) run_dir: Option<PathBuf>,
    #[arg(long, help = REPLAY_SOURCE_RUN_ID_HELP)]
    pub(crate) source_run_id: Option<String>,
    #[arg(long, help = REPLAY_SOURCE_RUN_ROOT_HELP)]
    pub(crate) source_run_root: Option<PathBuf>,
    #[arg(long)]
    pub(crate) out: PathBuf,
    #[arg(long)]
    pub(crate) dry_run: bool,
    #[arg(long, help = REPLAY_SANDBOX_HELP)]
    pub(crate) sandbox: bool,
    #[arg(long)]
    pub(crate) prove: bool,
    #[arg(long)]
    pub(crate) reuse_cache: bool,
    #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
    pub(crate) cache: CacheModeArg,
    #[arg(long, default_value_t = 1)]
    pub(crate) jobs: usize,
    #[arg(long)]
    pub(crate) run_id: Option<String>,
    #[arg(long)]
    pub(crate) cpu_budget: Option<u32>,
    #[arg(long)]
    pub(crate) memory_budget_mb: Option<u32>,
    #[arg(long)]
    pub(crate) gpu_device_budget: Option<u32>,
    #[arg(
        long = "resource-capacity",
        action = clap::ArgAction::Append,
        value_name = "name=count",
        help = RESOURCE_CAPACITY_HELP
    )]
    pub(crate) resource_capacity: Vec<String>,
    #[arg(long, help = DENY_NETWORK_HELP)]
    pub(crate) deny_network: bool,
    #[arg(long, help = DENY_ENV_HELP)]
    pub(crate) deny_env: bool,
    #[arg(long, help = DENY_CLOCK_HELP)]
    pub(crate) deny_clock: bool,
    #[arg(long, help = CLEAN_ENV_HELP)]
    pub(crate) clean_env: bool,
    #[arg(long, help = HERMETIC_HELP)]
    pub(crate) hermetic: bool,
    #[arg(long = "from-node", action = clap::ArgAction::Append)]
    pub(crate) from_node: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) select: Vec<String>,
    #[arg(long, action = clap::ArgAction::Append)]
    pub(crate) exclude: Vec<String>,
    #[arg(long)]
    pub(crate) dependency_closure: bool,
    #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
    pub(crate) materialize_inputs: MaterializeModeArg,
    #[arg(long)]
    pub(crate) remote_cache_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Init {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Validate {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        print_fingerprints: bool,
        #[arg(long)]
        explain: bool,
    },
    Canonicalize {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
    },
    Lint {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        strict: bool,
    },
    #[command(name = "graph-lint")]
    GraphLint {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        strict: bool,
    },
    Fingerprint {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        explain: bool,
    },
    Hash {
        #[command(subcommand)]
        command: HashCommands,
    },
    #[command(name = "artifact-inspect")]
    ArtifactInspect {
        run_dir: PathBuf,
        artifact_id: String,
    },
    Artifact {
        #[command(subcommand)]
        command: ArtifactCommands,
    },
    #[command(name = "commands")]
    CommandCatalog {
        #[arg(long)]
        groups: bool,
        #[arg(long = "lane", value_enum, action = clap::ArgAction::Append)]
        lanes: Vec<CommandCatalogLaneArg>,
    },
    #[command(name = "control-plane")]
    ControlPlane {
        #[command(subcommand)]
        command: ControlPlaneCommands,
    },
    #[command(name = "state-store")]
    StateStore {
        #[command(subcommand)]
        command: StateStoreCommands,
    },
    Dataset {
        #[command(subcommand)]
        command: DatasetCommands,
    },
    Enterprise {
        #[command(subcommand)]
        command: EnterpriseCommands,
    },
    Fleet {
        #[command(subcommand)]
        command: FleetCommands,
    },
    Governance {
        #[command(subcommand)]
        command: GovernanceCommands,
    },
    Incident {
        #[command(subcommand)]
        command: IncidentCommands,
    },
    Lab {
        #[command(subcommand)]
        command: LabCommands,
    },
    Federation {
        #[command(subcommand)]
        command: FederationCommands,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    Durability {
        #[command(subcommand)]
        command: DurabilityCommands,
    },
    Performance {
        #[command(subcommand)]
        command: PerformanceCommands,
    },
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    CanonicalBytes {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
    },
    CanonicalDiff {
        dag: PathBuf,
    },
    ShowEffectiveGraph {
        #[arg(required_unless_present = "run_dir", conflicts_with = "run_dir")]
        dags: Vec<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long = "select", action = clap::ArgAction::Append, conflicts_with = "run_dir")]
        select: Vec<String>,
        #[arg(long = "exclude", action = clap::ArgAction::Append, conflicts_with = "run_dir")]
        exclude: Vec<String>,
        #[arg(long = "from-node", action = clap::ArgAction::Append, conflicts_with = "run_dir")]
        from_node: Vec<String>,
        #[arg(long = "to-node", action = clap::ArgAction::Append, conflicts_with = "run_dir")]
        to_node: Vec<String>,
        #[arg(long, conflicts_with = "run_dir")]
        dependency_closure: bool,
    },
    #[command(name = "explain-plan", alias = "show-effective-plan")]
    ExplainPlan {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = AbsolutePathPolicyArg::AllowLiteral)]
        absolute_path_policy: AbsolutePathPolicyArg,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        memory_budget_mb: Option<u32>,
        #[arg(long)]
        gpu_device_budget: Option<u32>,
        #[arg(
            long = "resource-capacity",
            action = clap::ArgAction::Append,
            value_name = "name=count",
            help = RESOURCE_CAPACITY_HELP
        )]
        resource_capacity: Vec<String>,
        #[arg(long = "from-node", action = clap::ArgAction::Append)]
        from_node: Vec<String>,
        #[arg(long = "to-node", action = clap::ArgAction::Append)]
        to_node: Vec<String>,
    },
    Plan {
        #[command(subcommand)]
        command: PlanCommands,
    },
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommands,
    },
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommands,
    },
    Run {
        #[command(flatten)]
        command: Box<RunCommandArgs>,
    },
    #[command(name = "run-bundle", alias = "bundle")]
    RunBundle {
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        redact: bool,
    },
    Replay {
        #[command(flatten)]
        command: Box<ReplayCommandArgs>,
    },
    Prove {
        run_dir: PathBuf,
    },
    #[command(name = "proof-summary")]
    ProofSummary {
        run_dir: PathBuf,
    },
    Graph {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long, value_enum, default_value_t = GraphFormatArg::Dot)]
        format: GraphFormatArg,
    },
    Runs {
        #[command(subcommand)]
        command: RunsCommands,
    },
    Diff {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long, value_enum, default_value_t = DiffModeArg::Semantic)]
        mode: DiffModeArg,
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        explain: bool,
    },
    #[command(name = "why-rerun")]
    WhyRerun {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long)]
        node: Option<String>,
    },
    #[command(name = "why-cache-missed")]
    WhyCacheMissed {
        #[arg(required_unless_present_all = ["run_dir", "node"])]
        key: Option<String>,
        #[arg(long, required_unless_present_all = ["run_dir", "node"])]
        expected_adapter_id: Option<String>,
        #[arg(long, required_unless_present_all = ["run_dir", "node"])]
        expected_adapter_version: Option<String>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long, requires = "run_dir")]
        node: Option<String>,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    #[command(name = "trace-artifact")]
    TraceArtifact {
        run_dir: PathBuf,
        artifact_id: String,
    },
    #[command(name = "trace-node")]
    TraceNode {
        run_dir: PathBuf,
        #[arg(long)]
        id: String,
    },
    Explain {
        run_dir: PathBuf,
        #[arg(long)]
        node: Option<String>,
    },
    #[command(name = "node")]
    Node {
        run_dir: PathBuf,
        #[arg(long)]
        id: String,
    },
    Status {
        run_dir: PathBuf,
    },
    #[command(name = "verify")]
    Verify {
        run_dir: PathBuf,
        #[arg(long)]
        deep: bool,
        #[arg(long)]
        strict: bool,
    },
    Fsck {
        run_dir: PathBuf,
        #[arg(long)]
        strict: bool,
    },
    Doctor,
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
    Adapters {
        #[command(subcommand)]
        command: AdaptersCommands,
    },
    Export {
        run_dir: Option<PathBuf>,
        #[arg(long)]
        from_run: Option<PathBuf>,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        manifest_only: bool,
        #[arg(long)]
        without_artifacts: bool,
        #[arg(long)]
        provenance_only: bool,
        #[arg(long)]
        redact: bool,
        #[arg(long)]
        with_files: bool,
        #[arg(long)]
        #[arg(hide = true)]
        include_files: bool,
    },
    Import {
        file: PathBuf,
        #[arg(long)]
        verify_only: bool,
    },
    VersionInspect {
        #[arg(long)]
        dag: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        export_bundle: Option<PathBuf>,
    },
    Capabilities {
        #[arg(long)]
        backend: Option<String>,
    },
    #[command(name = "semantic-portability")]
    SemanticPortability {
        #[arg(long)]
        backend: String,
    },
    #[command(name = "equivalence-proof")]
    EquivalenceProof {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long)]
        backend_a: String,
        #[arg(long)]
        backend_b: String,
    },
    Version,
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },
}

#[derive(Subcommand)]
pub(crate) enum HashCommands {
    Graph {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        explain: bool,
    },
    Run {
        run_dir: PathBuf,
    },
    Artifact {
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum PlanCommands {
    Explain {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = AbsolutePathPolicyArg::AllowLiteral)]
        absolute_path_policy: AbsolutePathPolicyArg,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        memory_budget_mb: Option<u32>,
        #[arg(long)]
        gpu_device_budget: Option<u32>,
        #[arg(
            long = "resource-capacity",
            action = clap::ArgAction::Append,
            value_name = "name=count",
            help = RESOURCE_CAPACITY_HELP
        )]
        resource_capacity: Vec<String>,
        #[arg(long = "from-node", action = clap::ArgAction::Append)]
        from_node: Vec<String>,
        #[arg(long = "to-node", action = clap::ArgAction::Append)]
        to_node: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        exclude: Vec<String>,
        #[arg(long)]
        dependency_closure: bool,
    },
    Diagnostics {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
    },
    Diff {
        before: PathBuf,
        after: PathBuf,
    },
    Equivalence {
        before: PathBuf,
        after: PathBuf,
    },
    Closure {
        #[arg(required = true)]
        dags: Vec<PathBuf>,
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
    },
    Backfill {
        #[arg(long)]
        window_start_unix_ms: u128,
        #[arg(long)]
        window_end_unix_ms: u128,
        #[arg(long, action = clap::ArgAction::Append)]
        partition_key: Vec<String>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ScheduleBackfillCommands {
    Plan {
        registry: PathBuf,
        #[arg(long)]
        schedule_id: String,
        #[arg(long)]
        planned_unix_ms: u128,
        #[arg(long)]
        backfill_id: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Status {
        state: PathBuf,
    },
    Summary {
        state: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Advance {
        state: PathBuf,
        request: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Pause {
        state: PathBuf,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Resume {
        state: PathBuf,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    #[command(name = "retry-failed")]
    RetryFailed {
        state: PathBuf,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Cancel {
        state: PathBuf,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ScheduleQueueCommands {
    Status {
        registry: PathBuf,
        #[arg(long, help = "existing submission ledger json used to reconstruct queue state")]
        ledger: Option<PathBuf>,
        #[arg(long, help = "write the queue state json to this path")]
        out: Option<PathBuf>,
    },
    Dispatch {
        ledger: PathBuf,
        #[arg(long, default_value_t = 1)]
        max_dispatches: usize,
        #[arg(long, help = "json file containing queue priority dispatch policy")]
        policy: Option<PathBuf>,
        #[arg(long, help = "write the updated submission ledger json to this path")]
        out: Option<PathBuf>,
    },
    Update {
        ledger: PathBuf,
        #[arg(help = "json file containing submission status updates")]
        updates: PathBuf,
        #[arg(long, help = "write the updated submission ledger json to this path")]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ScheduleControlCommands {
    Status {
        registry: PathBuf,
        #[arg(long, help = "existing schedule control json used to compute pause status")]
        overrides: Option<PathBuf>,
        #[arg(long, help = "write the schedule control status json to this path")]
        out: Option<PathBuf>,
    },
    Pause {
        overrides: PathBuf,
        #[arg(long)]
        schedule_id: String,
        #[arg(long)]
        operator: String,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long, help = "write the updated schedule control json to this path")]
        out: Option<PathBuf>,
    },
    Resume {
        overrides: PathBuf,
        #[arg(long)]
        schedule_id: String,
        #[arg(long)]
        operator: String,
        #[arg(long)]
        at_unix_ms: u128,
        #[arg(long)]
        reason: Option<String>,
        #[arg(long, help = "write the updated schedule control json to this path")]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub(crate) enum ScheduleCommands {
    Validate {
        registry: PathBuf,
    },
    #[command(
        about = "evaluate internal schedule trigger inputs into deterministic submission records"
    )]
    Submit {
        registry: PathBuf,
        #[arg(
            help = "json file containing now_unix_ms plus manual_requests[].arguments, events[].payload, dependency completions, and signals[].payload"
        )]
        inputs: PathBuf,
        #[arg(
            long,
            help = "existing submission ledger json used to suppress duplicate submissions"
        )]
        ledger: Option<PathBuf>,
        #[arg(long, help = "existing schedule control json used to pause schedules")]
        overrides: Option<PathBuf>,
        #[arg(long, help = "write the updated submission ledger json to this path")]
        out: Option<PathBuf>,
    },
    Preview {
        registry: PathBuf,
        #[arg(long)]
        now_unix_ms: u128,
        #[arg(long, default_value_t = 3)]
        next_runs: usize,
    },
    Compile {
        registry: PathBuf,
        #[arg(long)]
        schedule_id: String,
        #[arg(long, help = "requested timestamp used for schedule-derived graph input bindings")]
        requested_unix_ms: u128,
    },
    Audit {
        registry: PathBuf,
        #[arg(long)]
        now_unix_ms: u128,
        #[arg(long, default_value_t = 3)]
        next_runs: usize,
    },
    Dedup {
        events: PathBuf,
    },
    Sla {
        simulation: PathBuf,
    },
    Order {
        simulation: PathBuf,
    },
    Throttle {
        simulation: PathBuf,
    },
    Queue {
        #[command(subcommand)]
        command: ScheduleQueueCommands,
    },
    Control {
        #[command(subcommand)]
        command: ScheduleControlCommands,
    },
    Backfill {
        #[command(subcommand)]
        command: ScheduleBackfillCommands,
    },
}

#[derive(Subcommand)]
pub(crate) enum RuntimeCommands {
    #[command(name = "execute-payload")]
    ExecutePayload {
        payload: PathBuf,
        #[arg(long)]
        result: PathBuf,
        #[arg(long)]
        in_place: bool,
    },
    Isolation {
        dag: PathBuf,
    },
    Dispatch {
        simulation: PathBuf,
    },
    State {
        run_dir: PathBuf,
    },
    #[command(name = "write-discipline")]
    WriteDiscipline {
        run_dir: PathBuf,
    },
    #[command(name = "worker-recovery")]
    WorkerRecovery {
        simulation: PathBuf,
    },
    #[command(name = "control-recovery")]
    ControlRecovery {
        simulation: PathBuf,
    },
    Repair {
        run_dir: PathBuf,
        #[arg(long)]
        apply: bool,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
        materialize_inputs: MaterializeModeArg,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        remote_cache_dir: Option<PathBuf>,
    },
    Retry {
        dag: PathBuf,
        #[arg(long)]
        node_id: String,
        #[arg(long)]
        attempt: u32,
        #[arg(long)]
        failure_class: String,
        #[arg(long)]
        exit_code: Option<i32>,
    },
    Timeout {
        dag: PathBuf,
        #[arg(long)]
        node_id: String,
        #[arg(long)]
        queue_wait_ms: Option<u64>,
        #[arg(long)]
        execution_ms: Option<u64>,
        #[arg(long)]
        total_elapsed_ms: Option<u64>,
        #[arg(long)]
        heartbeat_gap_ms: Option<u64>,
        #[arg(long)]
        heartbeat_timeout_ms: Option<u64>,
        #[arg(long)]
        sla_timeout_ms: Option<u64>,
    },
    Heartbeat {
        simulation: PathBuf,
    },
    Cancel {
        simulation: PathBuf,
    },
    Pause {
        simulation: PathBuf,
    },
    Intervention {
        simulation: PathBuf,
    },
    Transition {
        simulation: PathBuf,
    },
    Events {
        run_dir: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum ArtifactCommands {
    Fetch {
        run_dir: PathBuf,
        artifact_id: String,
        #[arg(long)]
        out: PathBuf,
    },
    Registry {
        run_dir: PathBuf,
    },
    Lineage {
        run_dir: PathBuf,
        #[arg(long)]
        artifact_id: Option<String>,
    },
    Promote {
        run_dir: PathBuf,
        artifact_id: String,
        #[arg(long)]
        deliverables_root: PathBuf,
        #[arg(long, default_value = "release")]
        to: String,
    },
    Retention {
        root: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum ControlPlaneCommands {
    Api {
        simulation: PathBuf,
    },
    Leadership {
        simulation: PathBuf,
    },
    Planning {
        simulation: PathBuf,
    },
    Sharding {
        simulation: PathBuf,
    },
    Leases {
        simulation: PathBuf,
    },
    Idempotency {
        simulation: PathBuf,
    },
    Backpressure {
        simulation: PathBuf,
    },
    Cache {
        simulation: PathBuf,
    },
    Migration {
        simulation: PathBuf,
    },
    #[command(name = "fan-in")]
    FanIn {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum StateStoreCommands {
    Transaction { simulation: PathBuf },
    Journal { simulation: PathBuf },
    Snapshot { simulation: PathBuf },
    Index { simulation: PathBuf },
    Archive { simulation: PathBuf },
    Checksum { run_dir: PathBuf },
    Amplification { simulation: PathBuf },
    Retention { simulation: PathBuf },
    Consistency { simulation: PathBuf },
    Clock { simulation: PathBuf },
}

#[derive(Subcommand)]
pub(crate) enum DatasetCommands {
    Mapping { simulation: PathBuf },
    Staleness { simulation: PathBuf },
}

#[derive(Subcommand)]
pub(crate) enum EnterpriseCommands {
    Webhook {
        simulation: PathBuf,
    },
    Queue {
        simulation: PathBuf,
    },
    #[command(name = "service-contract")]
    ServiceContract {
        simulation: PathBuf,
    },
    #[command(name = "incident-hook")]
    IncidentHook {
        simulation: PathBuf,
    },
    #[command(name = "asset-link")]
    AssetLink {
        simulation: PathBuf,
    },
    Calendar {
        simulation: PathBuf,
    },
    Approval {
        simulation: PathBuf,
    },
    #[command(name = "dependency-catalog")]
    DependencyCatalog {
        simulation: PathBuf,
    },
    Credentials {
        simulation: PathBuf,
    },
    Export {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum FleetCommands {
    Register {
        simulation: PathBuf,
    },
    Capabilities {
        simulation: PathBuf,
    },
    Drain {
        simulation: PathBuf,
    },
    Autoscale {
        simulation: PathBuf,
    },
    #[command(name = "warm-pool")]
    WarmPool {
        simulation: PathBuf,
    },
    Isolation {
        simulation: PathBuf,
    },
    Preemption {
        simulation: PathBuf,
    },
    Trust {
        simulation: PathBuf,
    },
    Gossip {
        simulation: PathBuf,
    },
    Fragmentation {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum GovernanceCommands {
    Contracts {
        dag: PathBuf,
    },
    Ownership {
        dag: PathBuf,
    },
    Tags {
        dag: PathBuf,
    },
    Cost {
        dag: PathBuf,
        #[arg(long, default_value_t = 0.04)]
        cpu_core_hour_rate: f64,
        #[arg(long, default_value_t = 0.005)]
        memory_gb_hour_rate: f64,
    },
    Alerts {
        dag: PathBuf,
        #[arg(long, default_value = "run_failed")]
        event: String,
    },
    #[command(name = "policy-check")]
    PolicyCheck {
        dag: PathBuf,
        policy: PathBuf,
    },
    #[command(name = "catalog-export")]
    CatalogExport {
        dag: PathBuf,
        #[arg(long)]
        run_dir: Option<PathBuf>,
    },
    #[command(name = "audit-event")]
    AuditEvent {
        simulation: PathBuf,
    },
    Promotion {
        simulation: PathBuf,
    },
    Compliance {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum IncidentCommands {
    Mode {
        simulation: PathBuf,
    },
    #[command(name = "blast-radius")]
    BlastRadius {
        simulation: PathBuf,
    },
    #[command(name = "safe-stop")]
    SafeStop {
        simulation: PathBuf,
    },
    #[command(name = "degraded-mode")]
    DegradedMode {
        simulation: PathBuf,
    },
    Annotation {
        simulation: PathBuf,
    },
    #[command(name = "repair-window")]
    RepairWindow {
        simulation: PathBuf,
    },
    Timeline {
        simulation: PathBuf,
    },
    #[command(name = "replay-validation")]
    ReplayValidation {
        simulation: PathBuf,
    },
    #[command(name = "readiness-review")]
    ReadinessReview {
        simulation: PathBuf,
    },
    Scorecard {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum FederationCommands {
    Schedule {
        simulation: PathBuf,
    },
    Failover {
        simulation: PathBuf,
    },
    Lineage {
        simulation: PathBuf,
    },
    Sovereignty {
        simulation: PathBuf,
    },
    Replay {
        simulation: PathBuf,
    },
    #[command(name = "policy-distribution")]
    PolicyDistribution {
        simulation: PathBuf,
    },
    #[command(name = "audit-integrity")]
    AuditIntegrity {
        simulation: PathBuf,
    },
    #[command(name = "trust-tier")]
    TrustTier {
        simulation: PathBuf,
    },
    Delegation {
        simulation: PathBuf,
    },
    #[command(name = "config-inheritance")]
    ConfigInheritance {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum LabCommands {
    Federation {
        #[command(subcommand)]
        command: FederationCommands,
    },
    Incident {
        #[command(subcommand)]
        command: IncidentCommands,
    },
    Enterprise {
        #[command(subcommand)]
        command: EnterpriseCommands,
    },
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },
    Durability {
        #[command(subcommand)]
        command: DurabilityCommands,
    },
    Performance {
        #[command(subcommand)]
        command: PerformanceCommands,
    },
}

#[derive(Subcommand)]
pub(crate) enum SecurityCommands {
    #[command(name = "filesystem-allowlist")]
    FilesystemAllowlist {
        simulation: PathBuf,
    },
    #[command(name = "env-allowlist")]
    EnvAllowlist {
        simulation: PathBuf,
    },
    #[command(name = "network-policy")]
    NetworkPolicy {
        dag: PathBuf,
    },
    #[command(name = "command-injection")]
    CommandInjection {
        simulation: PathBuf,
    },
    #[command(name = "artifact-secrets")]
    ArtifactSecrets {
        simulation: PathBuf,
    },
    Auth {
        simulation: PathBuf,
    },
    Authz {
        simulation: PathBuf,
    },
    Tenant {
        simulation: PathBuf,
    },
    Secrets {
        simulation: PathBuf,
    },
    #[command(name = "supply-chain")]
    SupplyChain {
        simulation: PathBuf,
    },
    #[command(name = "supply-inventory")]
    SupplyInventory {
        simulation: PathBuf,
    },
    #[command(name = "trust-classes")]
    TrustClasses {
        simulation: PathBuf,
    },
    #[command(name = "malformed-input-fuzz")]
    MalformedInputFuzz {
        simulation: PathBuf,
    },
    #[command(name = "dependency-risk")]
    DependencyRisk {
        simulation: PathBuf,
    },
    #[command(name = "data-access")]
    DataAccess {
        simulation: PathBuf,
    },
    Override {
        simulation: PathBuf,
    },
    #[command(name = "override-audit")]
    OverrideAudit {
        simulation: PathBuf,
    },
    #[command(name = "safe-defaults")]
    SafeDefaults {
        dag: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum PerformanceCommands {
    #[command(name = "latency-budgets")]
    LatencyBudgets { simulation: PathBuf },
    #[command(name = "large-graph-corpus")]
    LargeGraphCorpus { simulation: PathBuf },
    #[command(name = "canonicalization-profile")]
    CanonicalizationProfile { simulation: PathBuf },
    #[command(name = "scheduler-churn")]
    SchedulerChurn { simulation: PathBuf },
    #[command(name = "artifact-write-profile")]
    ArtifactWriteProfile { simulation: PathBuf },
    #[command(name = "memory-ceilings")]
    MemoryCeilings { simulation: PathBuf },
    #[command(name = "streaming-output")]
    StreamingOutput { simulation: PathBuf },
    #[command(name = "run-history-compaction")]
    RunHistoryCompaction { simulation: PathBuf },
    #[command(name = "benchmark-report-governance")]
    BenchmarkReportGovernance { simulation: PathBuf },
    #[command(name = "performance-regression-gates")]
    PerformanceRegressionGates { simulation: PathBuf },
}

#[derive(Subcommand)]
pub(crate) enum DurabilityCommands {
    #[command(name = "module-surface-budgets")]
    ModuleSurfaceBudgets { simulation: PathBuf },
    #[command(name = "typed-contracts")]
    TypedContracts { simulation: PathBuf },
    #[command(name = "public-api-review")]
    PublicApiReview { simulation: PathBuf },
    #[command(name = "contract-alignment")]
    ContractAlignment { simulation: PathBuf },
    #[command(name = "compatibility-fixtures")]
    CompatibilityFixtures { simulation: PathBuf },
    #[command(name = "change-impact-labels")]
    ChangeImpactLabels { simulation: PathBuf },
    #[command(name = "release-notes-evidence")]
    ReleaseNotesEvidence { simulation: PathBuf },
    #[command(name = "medium-acceptance-gate")]
    MediumAcceptanceGate { simulation: PathBuf },
    #[command(name = "production-candidate")]
    ProductionCandidate { simulation: PathBuf },
}

#[derive(Subcommand)]
pub(crate) enum ReleaseCommands {
    Version { dag: PathBuf },
    Promotion { simulation: PathBuf },
    Deprecation { simulation: PathBuf },
    Checkpoint { simulation: PathBuf },
    Shadow { simulation: PathBuf },
    Canary { simulation: PathBuf },
    Rollback { simulation: PathBuf },
    Classify { before: PathBuf, after: PathBuf },
    Evidence { simulation: PathBuf },
    Health { simulation: PathBuf },
}

#[derive(Subcommand)]
pub(crate) enum RunsCommands {
    List {
        #[arg(long)]
        root: PathBuf,
    },
    Show {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Inspect {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    History {
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        graph: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        offset: Option<usize>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
    },
    IdExplain {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Tree {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Timeline {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        event: Option<String>,
        #[arg(long)]
        since_unix_ms: Option<u128>,
        #[arg(long)]
        until_unix_ms: Option<u128>,
    },
    #[command(name = "scheduler-checkpoint")]
    SchedulerCheckpoint {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Stop {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Diff {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long, value_enum, default_value_t = DiffModeArg::Semantic)]
        mode: DiffModeArg,
        #[arg(long)]
        node: Option<String>,
        #[arg(long)]
        explain: bool,
    },
    Verify {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        deep: bool,
        #[arg(long)]
        strict: bool,
    },
    Doctor {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    ExplainFailure {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
    },
    Summary {
        #[arg(long)]
        root: PathBuf,
    },
    Compare {
        run_a: String,
        run_b: String,
        #[arg(long)]
        root: PathBuf,
    },
    Trend {
        #[arg(long)]
        root: PathBuf,
    },
    Failures {
        #[arg(long)]
        root: PathBuf,
    },
    Flakes {
        #[arg(long)]
        root: PathBuf,
    },
    #[command(name = "diagnostics-bundle")]
    DiagnosticsBundle {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        redact: bool,
    },
    Index {
        #[arg(long)]
        root: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum CacheCommands {
    Ls {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Pack {
        node_fp: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Unpack {
        pack: PathBuf,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Gc {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Verify {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        remote: Option<PathBuf>,
    },
    Explain {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        key: String,
        #[arg(long)]
        expected_adapter_id: Option<String>,
        #[arg(long)]
        expected_adapter_version: Option<String>,
    },
    Stats {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    PruneSimulate {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
    },
    Diff {
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        key_a: String,
        #[arg(long)]
        key_b: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum AdaptersCommands {
    Ls,
    Dump,
    Describe,
    Admit {
        dag: PathBuf,
    },
    Conformance,
    #[command(name = "cache-compat")]
    CacheCompat {
        meta: PathBuf,
        #[arg(long)]
        expected_schema: String,
    },
    Reference,
    Doctor,
}

#[derive(Subcommand)]
pub(crate) enum MigrateCommands {
    Dag {
        file: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    Run {
        run_dir: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },
    Inspect {
        #[arg(long)]
        dag: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum ConfigCommands {
    ShowEffective {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        jobs: Option<usize>,
        #[arg(long, value_enum)]
        cache_mode: Option<CacheModeArg>,
        #[arg(long, value_enum)]
        materialize_inputs: Option<MaterializeModeArg>,
    },
}

#[derive(Subcommand)]
pub(crate) enum PolicyCommands {
    ShowEffective {
        #[arg(long)]
        config: Option<PathBuf>,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
        #[arg(long)]
        clean_env: bool,
        #[arg(long, action = clap::ArgAction::Append)]
        allow_env: Vec<String>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum CacheModeArg {
    Off,
    Read,
    Readwrite,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum MaterializeModeArg {
    Copy,
    Hardlink,
    Symlink,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum DiffModeArg {
    Summary,
    Semantic,
    Artifact,
    Provenance,
    Timing,
    Policy,
    Cache,
    Raw,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum GraphFormatArg {
    Dot,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum AbsolutePathPolicyArg {
    AllowLiteral,
    DenyLiteral,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum RunTimeoutBehaviorArg {
    FinishRunning,
    CancelRunning,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum ResumeFailureModeArg {
    RerunIncomplete,
    RejectIncomplete,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub(crate) enum RunProgressArg {
    Off,
    Compact,
}
