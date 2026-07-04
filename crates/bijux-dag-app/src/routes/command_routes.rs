use crate::commands::{root_command_hidden_from_public_help, DagCli};
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

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum CommandMaturity {
    Stable,
    Experimental,
    Simulation,
    Internal,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct CommandCatalogEntry {
    path: String,
    group: CommandGroup,
    maturity: CommandMaturity,
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

fn command_maturity(path: &str) -> CommandMaturity {
    let head = path.split(' ').next().unwrap_or(path);
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
            | "release"
            | "runtime"
            | "schedule"
            | "security"
            | "state-store"
    ) {
        return CommandMaturity::Simulation;
    }
    if matches!(path, "commands" | "doctor" | "explain-plan" | "run-bundle" | "trace-node")
        || path.starts_with("artifact fetch")
    {
        return CommandMaturity::Experimental;
    }
    if matches!(
        head,
        "capabilities" | "equivalence-proof" | "semantic-portability" | "version-inspect"
    ) {
        return CommandMaturity::Internal;
    }
    CommandMaturity::Stable
}

fn build_entry(prefix: &str, command: &Command) -> CommandCatalogEntry {
    let path = if prefix.is_empty() {
        command.get_name().to_string()
    } else {
        format!("{prefix} {}", command.get_name())
    };
    let mut subcommands =
        command.get_subcommands().map(|sub| build_entry(&path, sub)).collect::<Vec<_>>();
    subcommands.sort_by(|left, right| left.path.cmp(&right.path));
    let mut aliases =
        command.get_all_aliases().map(std::string::ToString::to_string).collect::<Vec<_>>();
    aliases.sort();
    CommandCatalogEntry {
        path: path.clone(),
        group: command_group(&path),
        maturity: command_maturity(&path),
        about: command.get_about().map(|value| value.to_string()),
        aliases,
        subcommands,
    }
}

fn flatten(entry: &CommandCatalogEntry, out: &mut Vec<CommandCatalogEntry>) {
    out.push(entry.clone());
    for child in &entry.subcommands {
        flatten(child, out);
    }
}

fn command_catalog(include_hidden: bool) -> Vec<CommandCatalogEntry> {
    let mut entries =
        dag_command()
            .get_subcommands()
            .filter(|sub| include_hidden || !root_command_hidden_from_public_help(sub.get_name()))
            .map(|sub| build_entry("", sub))
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

fn maturity_label(maturity: CommandMaturity) -> &'static str {
    match maturity {
        CommandMaturity::Stable => "stable",
        CommandMaturity::Experimental => "experimental",
        CommandMaturity::Simulation => "simulation",
        CommandMaturity::Internal => "internal",
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
        println!("{} [{} | {}]", entry.path, group, maturity_label(entry.maturity));
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::{command_catalog, command_groups, CommandMaturity};

    #[test]
    fn command_catalog_exposes_public_surface_by_default() {
        let entries = command_catalog(false);
        let mut flattened = Vec::new();
        for entry in &entries {
            super::flatten(entry, &mut flattened);
        }
        assert!(flattened.iter().any(|entry| entry.path == "commands"));
        assert!(flattened.iter().any(|entry| entry.path == "artifact fetch"));
        assert!(
            flattened
                .iter()
                .any(|entry| entry.path == "doctor"
                    && entry.maturity == CommandMaturity::Experimental)
        );
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
    }

    #[test]
    fn command_groups_cover_public_taxonomy() {
        let groups = command_groups(&command_catalog(false));
        assert!(groups.contains(&"graph".to_string()));
        assert!(groups.contains(&"plan".to_string()));
        assert!(groups.contains(&"run".to_string()));
        assert!(groups.contains(&"inspect".to_string()));
        assert!(groups.contains(&"replay".to_string()));
        assert!(groups.contains(&"cache".to_string()));
        assert!(groups.contains(&"artifact".to_string()));
        assert!(groups.contains(&"config".to_string()));
        assert!(groups.contains(&"migrate".to_string()));
        assert!(groups.contains(&"doctor".to_string()));
        assert!(groups.contains(&"prove".to_string()));
        assert!(groups.contains(&"export-import".to_string()));
    }
}
