#[derive(Subcommand)]
pub(super) enum ControlCommand {
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
        #[arg(long, default_value_t = false)]
        advisory: bool,
        #[arg(long, default_value_t = false)]
        why: bool,
    },
    /// Show known suites
    List,
    /// Explain a suite
    Explain {
        #[arg(long)]
        suite: String,
    },
}
