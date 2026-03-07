#[derive(Subcommand)]
pub(super) enum ReleaseCommand {
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
