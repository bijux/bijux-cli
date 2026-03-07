use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Bijux DAG CLI", long_about = None)]
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
    },
    ShowEffectiveGraph {
        dag: PathBuf,
    },
    ShowEffectivePlan {
        dag: PathBuf,
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
    },
    Replay {
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
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
    Graph {
        dag: PathBuf,
        #[arg(long, value_enum, default_value_t = GraphFormatArg::Dot)]
        format: GraphFormatArg,
    },
    Runs {
        #[command(subcommand)]
        command: RunsCommands,
    },
    #[command(hide = true)]
    Diff {
        run_a: PathBuf,
        run_b: PathBuf,
        #[arg(long)]
        explain: bool,
    },
    #[command(hide = true)]
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
    #[command(hide = true)]
    Status {
        run_dir: PathBuf,
    },
    #[command(name = "verify")]
    #[command(hide = true)]
    Verify {
        run_dir: PathBuf,
        #[arg(long)]
        deep: bool,
    },
    #[command(hide = true)]
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
        run_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        include_files: bool,
    },
    Import {
        file: PathBuf,
    },
    VersionInspect {
        #[arg(long)]
        dag: Option<PathBuf>,
        #[arg(long)]
        run_dir: Option<PathBuf>,
        #[arg(long)]
        export_bundle: Option<PathBuf>,
    },
    Version,
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
        #[arg(long)]
        explain: bool,
    },
    Verify {
        run_id: String,
        #[arg(long)]
        root: PathBuf,
        #[arg(long)]
        deep: bool,
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
}

#[derive(Subcommand)]
pub(crate) enum AdaptersCommands {
    Ls,
    Dump,
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
    },
    Run {
        run_dir: PathBuf,
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
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
pub(crate) enum GraphFormatArg {
    Dot,
}
