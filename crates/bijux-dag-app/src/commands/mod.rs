use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Git for computation graphs", long_about = None)]
pub(crate) struct DagCli {
    #[arg(long, global = true)]
    pub(crate) json: bool,
    #[arg(long, global = true)]
    pub(crate) quiet: bool,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Init {
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    Validate {
        dag: PathBuf,
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        print_fingerprints: bool,
        #[arg(long)]
        explain: bool,
    },
    Canonicalize {
        dag: PathBuf,
    },
    Lint {
        dag: PathBuf,
        #[arg(long)]
        strict: bool,
    },
    #[command(name = "graph-lint")]
    GraphLint {
        dag: PathBuf,
        #[arg(long)]
        strict: bool,
    },
    Fingerprint {
        dag: PathBuf,
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
        dag: PathBuf,
    },
    CanonicalDiff {
        dag: PathBuf,
    },
    ShowEffectiveGraph {
        dag: PathBuf,
    },
    #[command(name = "explain-plan", alias = "show-effective-plan")]
    ExplainPlan {
        dag: PathBuf,
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
        dag: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        latest: Option<PathBuf>,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        node_timeout_ms: Option<u64>,
        #[arg(long)]
        run_timeout_ms: Option<u64>,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
        #[arg(long)]
        clean_env: bool,
        #[arg(long)]
        hermetic: bool,
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        exclude: Vec<String>,
        #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
        materialize_inputs: MaterializeModeArg,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long)]
        cache_dir: Option<PathBuf>,
        #[arg(long)]
        remote_cache_dir: Option<PathBuf>,
        #[arg(long)]
        preflight_only: bool,
        #[arg(long)]
        explain_scheduling: bool,
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
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        sandbox: bool,
        #[arg(long)]
        prove: bool,
        #[arg(long)]
        reuse_cache: bool,
        #[arg(long, value_enum, default_value_t = CacheModeArg::Off)]
        cache: CacheModeArg,
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long)]
        cpu_budget: Option<u32>,
        #[arg(long)]
        deny_network: bool,
        #[arg(long)]
        deny_env: bool,
        #[arg(long)]
        deny_clock: bool,
        #[arg(long)]
        clean_env: bool,
        #[arg(long)]
        hermetic: bool,
        #[arg(long, action = clap::ArgAction::Append)]
        select: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        exclude: Vec<String>,
        #[arg(long, value_enum, default_value_t = MaterializeModeArg::Copy)]
        materialize_inputs: MaterializeModeArg,
        #[arg(long)]
        remote_cache_dir: Option<PathBuf>,
    },
    Prove {
        run_dir: PathBuf,
    },
    #[command(name = "proof-summary")]
    ProofSummary {
        run_dir: PathBuf,
    },
    Graph {
        dag: PathBuf,
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
        key: String,
        #[arg(long)]
        expected_adapter_id: String,
        #[arg(long)]
        expected_adapter_version: String,
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
        dag: PathBuf,
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
        dag: PathBuf,
    },
    Diagnostics {
        dag: PathBuf,
    },
    Diff {
        before: PathBuf,
        after: PathBuf,
    },
    Closure {
        dag: PathBuf,
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
pub(crate) enum ScheduleCommands {
    Validate {
        registry: PathBuf,
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
        #[arg(long)]
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
}

#[derive(Subcommand)]
pub(crate) enum RuntimeCommands {
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
    },
    Retry {
        dag: PathBuf,
        #[arg(long)]
        node_id: String,
        #[arg(long)]
        attempt: u32,
        #[arg(long)]
        failure_class: String,
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
    LatencyBudgets {
        simulation: PathBuf,
    },
    #[command(name = "large-graph-corpus")]
    LargeGraphCorpus {
        simulation: PathBuf,
    },
    #[command(name = "canonicalization-profile")]
    CanonicalizationProfile {
        simulation: PathBuf,
    },
    #[command(name = "scheduler-churn")]
    SchedulerChurn {
        simulation: PathBuf,
    },
    #[command(name = "artifact-write-profile")]
    ArtifactWriteProfile {
        simulation: PathBuf,
    },
    #[command(name = "memory-ceilings")]
    MemoryCeilings {
        simulation: PathBuf,
    },
    #[command(name = "streaming-output")]
    StreamingOutput {
        simulation: PathBuf,
    },
    #[command(name = "run-history-compaction")]
    RunHistoryCompaction {
        simulation: PathBuf,
    },
    #[command(name = "benchmark-report-governance")]
    BenchmarkReportGovernance {
        simulation: PathBuf,
    },
    #[command(name = "performance-regression-gates")]
    PerformanceRegressionGates {
        simulation: PathBuf,
    },
}

#[derive(Subcommand)]
pub(crate) enum DurabilityCommands {
    #[command(name = "module-surface-budgets")]
    ModuleSurfaceBudgets {
        simulation: PathBuf,
    },
    #[command(name = "typed-contracts")]
    TypedContracts {
        simulation: PathBuf,
    },
    #[command(name = "public-api-review")]
    PublicApiReview {
        simulation: PathBuf,
    },
    #[command(name = "contract-alignment")]
    ContractAlignment {
        simulation: PathBuf,
    },
    #[command(name = "compatibility-fixtures")]
    CompatibilityFixtures {
        simulation: PathBuf,
    },
    #[command(name = "change-impact-labels")]
    ChangeImpactLabels {
        simulation: PathBuf,
    },
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
