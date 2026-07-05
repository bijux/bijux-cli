use super::{command_path_hidden_from_public_help, Commands};
use serde::Serialize;
use std::env;

pub(crate) const ENABLE_SIMULATED_ENV: &str = "BIJUX_DAG_ENABLE_SIMULATED";
pub(crate) const ENABLE_INTERNAL_ENV: &str = "BIJUX_DAG_ENABLE_INTERNAL";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CommandLane {
    Stable,
    Experimental,
    Simulation,
    Internal,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum CommandAvailability {
    Default,
    ExplicitPath,
    OptIn,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub(crate) struct CommandAccess {
    pub(crate) lane: CommandLane,
    pub(crate) availability: CommandAvailability,
    pub(crate) opt_in_env: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommandAccessDenial {
    pub(crate) root_command: &'static str,
    pub(crate) lane: CommandLane,
    pub(crate) opt_in_env: &'static str,
}

impl CommandAccessDenial {
    pub(crate) fn message(self) -> String {
        format!(
            "`{}` belongs to the {} command lane outside the stable v0.4.0 operator surface; set {}=1 to run it intentionally",
            self.root_command,
            lane_label(self.lane),
            self.opt_in_env,
        )
    }
}

pub(crate) fn command_access_for_command(command: &Commands) -> CommandAccess {
    command_access_for_head(root_command(command))
}

pub(crate) fn command_access_for_path(path: &str) -> CommandAccess {
    if is_path_scoped_experimental_route(path) {
        return CommandAccess {
            lane: CommandLane::Experimental,
            availability: CommandAvailability::ExplicitPath,
            opt_in_env: None,
        };
    }
    let head = path.split(' ').next().unwrap_or(path);
    let mut access = command_access_for_head(head);
    if access.availability == CommandAvailability::Default
        && command_path_hidden_from_public_help(path)
    {
        access.availability = CommandAvailability::ExplicitPath;
    }
    access
}

pub(crate) fn command_access_denial(command: &Commands) -> Option<CommandAccessDenial> {
    let root_command = root_command(command);
    let access = command_access_for_head(root_command);
    let opt_in_env = access.opt_in_env?;
    if opt_in_enabled(opt_in_env) {
        return None;
    }
    Some(CommandAccessDenial { root_command, lane: access.lane, opt_in_env })
}

pub(crate) fn lane_label(lane: CommandLane) -> &'static str {
    match lane {
        CommandLane::Stable => "stable",
        CommandLane::Experimental => "experimental",
        CommandLane::Simulation => "simulated",
        CommandLane::Internal => "internal",
    }
}

fn command_access_for_head(head: &str) -> CommandAccess {
    if matches!(
        head,
        "control-plane"
            | "dataset"
            | "enterprise"
            | "federation"
            | "fleet"
            | "governance"
            | "incident"
            | "lab"
            | "state-store"
    ) {
        return CommandAccess {
            lane: CommandLane::Simulation,
            availability: CommandAvailability::OptIn,
            opt_in_env: Some(ENABLE_SIMULATED_ENV),
        };
    }
    if matches!(
        head,
        "capabilities"
            | "durability"
            | "equivalence-proof"
            | "performance"
            | "release"
            | "runtime"
            | "schedule"
            | "security"
            | "semantic-portability"
            | "version-inspect"
    ) {
        return CommandAccess {
            lane: CommandLane::Internal,
            availability: CommandAvailability::OptIn,
            opt_in_env: Some(ENABLE_INTERNAL_ENV),
        };
    }
    if matches!(
        head,
        "adapters"
            | "canonical-bytes"
            | "canonical-diff"
            | "canonicalize"
            | "config"
            | "export"
            | "fingerprint"
            | "fsck"
            | "graph"
            | "graph-lint"
            | "hash"
            | "import"
            | "init"
            | "lint"
            | "migrate"
            | "node"
            | "policy"
            | "proof-summary"
            | "prove"
            | "show-effective-graph"
            | "status"
            | "trace-artifact"
            | "why-cache-missed"
            | "why-rerun"
    ) {
        return CommandAccess {
            lane: CommandLane::Experimental,
            availability: CommandAvailability::ExplicitPath,
            opt_in_env: None,
        };
    }
    CommandAccess {
        lane: CommandLane::Stable,
        availability: CommandAvailability::Default,
        opt_in_env: None,
    }
}

fn is_path_scoped_experimental_route(path: &str) -> bool {
    matches!(path, "explain-plan" | "run-bundle" | "trace-node")
        || path.starts_with("artifact fetch")
}

fn root_command(command: &Commands) -> &'static str {
    match command {
        Commands::Init { .. } => "init",
        Commands::Validate { .. } => "validate",
        Commands::Canonicalize { .. } => "canonicalize",
        Commands::Lint { .. } => "lint",
        Commands::GraphLint { .. } => "graph-lint",
        Commands::Fingerprint { .. } => "fingerprint",
        Commands::Hash { .. } => "hash",
        Commands::ArtifactInspect { .. } => "artifact-inspect",
        Commands::Artifact { .. } => "artifact",
        Commands::CommandCatalog { .. } => "commands",
        Commands::ControlPlane { .. } => "control-plane",
        Commands::StateStore { .. } => "state-store",
        Commands::Dataset { .. } => "dataset",
        Commands::Enterprise { .. } => "enterprise",
        Commands::Fleet { .. } => "fleet",
        Commands::Governance { .. } => "governance",
        Commands::Incident { .. } => "incident",
        Commands::Lab { .. } => "lab",
        Commands::Federation { .. } => "federation",
        Commands::Security { .. } => "security",
        Commands::Durability { .. } => "durability",
        Commands::Performance { .. } => "performance",
        Commands::Release { .. } => "release",
        Commands::CanonicalBytes { .. } => "canonical-bytes",
        Commands::CanonicalDiff { .. } => "canonical-diff",
        Commands::ShowEffectiveGraph { .. } => "show-effective-graph",
        Commands::ExplainPlan { .. } => "explain-plan",
        Commands::Plan { .. } => "plan",
        Commands::Schedule { .. } => "schedule",
        Commands::Runtime { .. } => "runtime",
        Commands::Run { .. } => "run",
        Commands::RunBundle { .. } => "run-bundle",
        Commands::Replay { .. } => "replay",
        Commands::Prove { .. } => "prove",
        Commands::ProofSummary { .. } => "proof-summary",
        Commands::Graph { .. } => "graph",
        Commands::Runs { .. } => "runs",
        Commands::Diff { .. } => "diff",
        Commands::WhyRerun { .. } => "why-rerun",
        Commands::WhyCacheMissed { .. } => "why-cache-missed",
        Commands::TraceArtifact { .. } => "trace-artifact",
        Commands::TraceNode { .. } => "trace-node",
        Commands::Explain { .. } => "explain",
        Commands::Node { .. } => "node",
        Commands::Status { .. } => "status",
        Commands::Verify { .. } => "verify",
        Commands::Fsck { .. } => "fsck",
        Commands::Doctor => "doctor",
        Commands::Migrate { .. } => "migrate",
        Commands::Cache { .. } => "cache",
        Commands::Adapters { .. } => "adapters",
        Commands::Export { .. } => "export",
        Commands::Import { .. } => "import",
        Commands::VersionInspect { .. } => "version-inspect",
        Commands::Capabilities { .. } => "capabilities",
        Commands::SemanticPortability { .. } => "semantic-portability",
        Commands::EquivalenceProof { .. } => "equivalence-proof",
        Commands::Version => "version",
        Commands::Config { .. } => "config",
        Commands::Policy { .. } => "policy",
    }
}

fn opt_in_enabled(env_name: &str) -> bool {
    let Some(value) = env::var_os(env_name) else {
        return false;
    };
    let normalized = value.to_string_lossy().trim().to_ascii_lowercase();
    !normalized.is_empty() && !matches!(normalized.as_str(), "0" | "false" | "no" | "off")
}

#[cfg(test)]
mod tests {
    use super::{
        command_access_for_path, CommandAvailability, CommandLane, ENABLE_INTERNAL_ENV,
        ENABLE_SIMULATED_ENV,
    };

    #[test]
    fn stable_commands_stay_on_default_surface() {
        let access = command_access_for_path("run");
        assert_eq!(access.lane, CommandLane::Stable);
        assert_eq!(access.availability, CommandAvailability::Default);
        assert_eq!(access.opt_in_env, None);
    }

    #[test]
    fn experimental_commands_stay_explicit_path_only() {
        let access = command_access_for_path("trace-artifact");
        assert_eq!(access.lane, CommandLane::Experimental);
        assert_eq!(access.availability, CommandAvailability::ExplicitPath);
        assert_eq!(access.opt_in_env, None);
    }

    #[test]
    fn hidden_helper_paths_stay_explicit_path_only() {
        let access = command_access_for_path("artifact fetch");
        assert_eq!(access.lane, CommandLane::Experimental);
        assert_eq!(access.availability, CommandAvailability::ExplicitPath);
        assert_eq!(access.opt_in_env, None);
    }

    #[test]
    fn hidden_config_root_stays_experimental() {
        let access = command_access_for_path("config");
        assert_eq!(access.lane, CommandLane::Experimental);
        assert_eq!(access.availability, CommandAvailability::ExplicitPath);
        assert_eq!(access.opt_in_env, None);
    }

    #[test]
    fn simulated_commands_require_opt_in() {
        let access = command_access_for_path("enterprise webhook");
        assert_eq!(access.lane, CommandLane::Simulation);
        assert_eq!(access.availability, CommandAvailability::OptIn);
        assert_eq!(access.opt_in_env, Some(ENABLE_SIMULATED_ENV));
    }

    #[test]
    fn internal_commands_require_opt_in() {
        let access = command_access_for_path("capabilities");
        assert_eq!(access.lane, CommandLane::Internal);
        assert_eq!(access.availability, CommandAvailability::OptIn);
        assert_eq!(access.opt_in_env, Some(ENABLE_INTERNAL_ENV));
    }
}
