use crate::commands::{
    command_access_for_path, command_path_hidden_from_public_help, lane_label, CommandAvailability,
    CommandLane, DagCli,
};
use crate::{dag_command, emit_json, ExitCode};
use clap::Command;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
enum CommandGroup {
    Graph,
    Plan,
    Run,
    Inspect,
    Replay,
    Cache,
    Artifact,
    Config,
    Migrate,
    Doctor,
    Prove,
    ExportImport,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CommandCatalogEntry {
    path: String,
    group: CommandGroup,
    lane: CommandLane,
    availability: CommandAvailability,
    opt_in_env: Option<&'static str>,
    about: Option<String>,
    aliases: Vec<String>,
    subcommands: Vec<CommandCatalogEntry>,
}

fn command_group(path: &str) -> CommandGroup {
    let head = path.split(' ').next().unwrap_or(path);
    match head {
        "init" | "validate" | "canonicalize" | "lint" | "graph-lint" | "canonical-bytes"
        | "canonical-diff" | "fingerprint" | "graph" => CommandGroup::Graph,
        "plan" | "explain-plan" | "show-effective-graph" => CommandGroup::Plan,
        "run" | "runtime" | "schedule" => CommandGroup::Run,
        "runs" | "status" | "node" | "trace-node" | "diff" | "why-rerun" => CommandGroup::Inspect,
        "replay" => CommandGroup::Replay,
        "cache" => CommandGroup::Cache,
        "artifact" | "artifact-inspect" | "trace-artifact" => CommandGroup::Artifact,
        "config"
        | "policy"
        | "version"
        | "version-inspect"
        | "capabilities"
        | "semantic-portability"
        | "equivalence-proof"
        | "commands"
        | "adapters"
        | "control-plane"
        | "state-store"
        | "dataset"
        | "enterprise"
        | "fleet"
        | "governance"
        | "incident"
        | "lab"
        | "federation"
        | "security"
        | "release" => CommandGroup::Config,
        "migrate" => CommandGroup::Migrate,
        "doctor" => CommandGroup::Doctor,
        "prove" | "proof-summary" | "verify" | "fsck" => CommandGroup::Prove,
        "export" | "import" | "run-bundle" => CommandGroup::ExportImport,
        _ => {
            if path.starts_with("plan ") {
                return CommandGroup::Plan;
            }
            if path.starts_with("runs ") {
                return CommandGroup::Inspect;
            }
            if path.starts_with("replay ") {
                return CommandGroup::Replay;
            }
            if path.starts_with("artifact ") {
                return CommandGroup::Artifact;
            }
            if path.starts_with("config ") || path.starts_with("policy ") {
                return CommandGroup::Config;
            }
            if path.starts_with("cache ") {
                return CommandGroup::Cache;
            }
            if path.starts_with("migrate ") {
                return CommandGroup::Migrate;
            }
            if path.starts_with("export ") || path.starts_with("import ") {
                return CommandGroup::ExportImport;
            }
            if path.starts_with("run ") {
                return CommandGroup::Run;
            }
            if path.starts_with("graph ") {
                return CommandGroup::Graph;
            }
            if path.starts_with("prove ") {
                return CommandGroup::Prove;
            }
            if path.starts_with("doctor ") {
                return CommandGroup::Doctor;
            }
            if path.starts_with("schedule ") || path.starts_with("runtime ") {
                return CommandGroup::Run;
            }
            if path.starts_with("status ") || path.starts_with("node ") || path.starts_with("diff ")
            {
                return CommandGroup::Inspect;
            }
            if path.starts_with("trace-artifact ") {
                return CommandGroup::Artifact;
            }
            if path.starts_with("version ")
                || path.starts_with("capabilities ")
                || path.starts_with("commands ")
            {
                return CommandGroup::Config;
            }
            if path.starts_with("canonicalize ")
                || path.starts_with("validate ")
                || path.starts_with("lint ")
            {
                return CommandGroup::Graph;
            }
            CommandGroup::Inspect
        }
    }
}

fn build_entry(
    prefix: &str,
    command: &Command,
    include_hidden: bool,
) -> Option<CommandCatalogEntry> {
    let path = if prefix.is_empty() {
        command.get_name().to_string()
    } else {
        format!("{prefix} {}", command.get_name())
    };
    if !include_hidden && command_path_hidden_from_public_help(&path) {
        return None;
    }
    let mut subcommands = command
        .get_subcommands()
        .filter_map(|sub| build_entry(&path, sub, include_hidden))
        .collect::<Vec<_>>();
    subcommands.sort_by(|left, right| left.path.cmp(&right.path));
    let mut aliases =
        command.get_all_aliases().map(std::string::ToString::to_string).collect::<Vec<_>>();
    aliases.sort();
    let access = command_access_for_path(&path);
    Some(CommandCatalogEntry {
        path: path.clone(),
        group: command_group(&path),
        lane: access.lane,
        availability: access.availability,
        opt_in_env: access.opt_in_env,
        about: command.get_about().map(|value| value.to_string()),
        aliases,
        subcommands,
    })
}

fn flatten(entry: &CommandCatalogEntry, out: &mut Vec<CommandCatalogEntry>) {
    out.push(entry.clone());
    for child in &entry.subcommands {
        flatten(child, out);
    }
}

fn command_catalog(include_hidden: bool) -> Vec<CommandCatalogEntry> {
    let mut entries = dag_command()
        .get_subcommands()
        .filter_map(|sub| build_entry("", sub, include_hidden))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    entries
}

fn command_groups(entries: &[CommandCatalogEntry]) -> Vec<String> {
    let mut groups = entries
        .iter()
        .map(|entry| entry.group)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|group| {
            serde_json::to_value(group)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
                .unwrap_or_else(|| "core".to_string())
        })
        .collect::<Vec<_>>();
    groups.sort();
    groups
}

fn availability_label(availability: CommandAvailability) -> &'static str {
    match availability {
        CommandAvailability::Default => "default",
        CommandAvailability::ExplicitPath => "explicit-path",
        CommandAvailability::OptIn => "opt-in",
    }
}

pub(crate) fn handle_command_catalog_command(
    cli: &DagCli,
    groups_only: bool,
    include_hidden: bool,
) -> Result<ExitCode, ExitCode> {
    let entries = command_catalog(include_hidden);
    let groups = command_groups(&entries);
    if cli.json {
        let mut flattened = Vec::new();
        for entry in &entries {
            flatten(entry, &mut flattened);
        }
        return emit_json(
            cli,
            "dag.commands",
            true,
            json!({
                "groups": groups,
                "commands": flattened,
            }),
            Vec::new(),
            ExitCode::SUCCESS,
        );
    }
    if groups_only {
        for group in groups {
            println!("{group}");
        }
        return Ok(ExitCode::SUCCESS);
    }
    let mut flattened = Vec::new();
    for entry in &entries {
        flatten(entry, &mut flattened);
    }
    for entry in flattened {
        let group = serde_json::to_value(entry.group)
            .ok()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_else(|| "core".to_string());
        if let Some(env_name) = entry.opt_in_env {
            println!(
                "{} [{} | {} | {} via {}]",
                entry.path,
                group,
                lane_label(entry.lane),
                availability_label(entry.availability),
                env_name
            );
        } else {
            println!(
                "{} [{} | {} | {}]",
                entry.path,
                group,
                lane_label(entry.lane),
                availability_label(entry.availability)
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{command_catalog, command_groups};
    use crate::commands::{CommandAvailability, CommandLane, ENABLE_SIMULATED_ENV};

    #[test]
    fn command_catalog_exposes_public_surface_by_default() {
        let entries = command_catalog(false);
        let mut flattened = Vec::new();
        for entry in &entries {
            super::flatten(entry, &mut flattened);
        }
        assert!(flattened.iter().any(|entry| entry.path == "commands"));
        assert!(flattened.iter().any(|entry| entry.path == "doctor"));
        assert!(flattened.iter().all(|entry| entry.lane == CommandLane::Stable));
        assert!(flattened.iter().all(|entry| entry.availability == CommandAvailability::Default));
        assert!(!flattened.iter().any(|entry| entry.path == "artifact fetch"));
        assert!(!flattened.iter().any(|entry| entry.path == "status"));
        assert!(!flattened.iter().any(|entry| entry.path == "init"));
        assert!(!flattened.iter().any(|entry| entry.path.starts_with("lab ")));
        assert!(!flattened.iter().any(|entry| entry.path == "trace-node"));
    }

    #[test]
    fn command_catalog_includes_hidden_namespaces_when_requested() {
        let entries = command_catalog(true);
        let mut flattened = Vec::new();
        for entry in &entries {
            super::flatten(entry, &mut flattened);
        }
        assert!(flattened.iter().any(|entry| entry.path == "lab federation schedule"));
        assert!(flattened.iter().any(|entry| entry.path == "trace-node"));
        assert!(flattened.iter().any(|entry| {
            entry.path == "artifact fetch"
                && entry.lane == CommandLane::Experimental
                && entry.availability == CommandAvailability::ExplicitPath
        }));
        assert!(flattened.iter().any(|entry| {
            entry.path == "governance ownership"
                && entry.lane == CommandLane::Simulation
                && entry.availability == CommandAvailability::OptIn
                && entry.opt_in_env == Some(ENABLE_SIMULATED_ENV)
        }));
    }

    #[test]
    fn command_groups_cover_public_taxonomy() {
        let groups = command_groups(&command_catalog(false));
        assert_eq!(
            groups,
            vec![
                "artifact".to_string(),
                "cache".to_string(),
                "config".to_string(),
                "doctor".to_string(),
                "graph".to_string(),
                "inspect".to_string(),
                "plan".to_string(),
                "prove".to_string(),
                "replay".to_string(),
                "run".to_string(),
            ]
        );
    }
}
