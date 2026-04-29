use crate::commands::DagCli;
use crate::{dag_command, emit_json, ExitCode};
use clap::Command;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
enum CommandGroup {
    Core,
    Runtime,
    Evidence,
    Cache,
    Diagnostics,
    Lab,
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
        "cache" => CommandGroup::Cache,
        "artifact" | "artifact-inspect" | "diff" | "explain" | "explain-plan" | "export"
        | "fsck" | "import" | "node" | "proof-summary" | "prove" | "run-bundle" | "runs"
        | "status" | "trace-artifact" | "trace-node" | "verify" | "why-cache-missed"
        | "why-rerun" => CommandGroup::Evidence,
        "adapters"
        | "capabilities"
        | "commands"
        | "doctor"
        | "semantic-portability"
        | "version"
        | "version-inspect" => CommandGroup::Diagnostics,
        "control-plane" | "dataset" | "enterprise" | "federation" | "fleet" | "governance"
        | "incident" | "lab" | "release" | "runtime" | "schedule" | "security" | "state-store" => {
            CommandGroup::Lab
        }
        "replay" | "run" => CommandGroup::Runtime,
        _ => CommandGroup::Core,
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

fn command_catalog() -> Vec<CommandCatalogEntry> {
    let mut entries =
        dag_command().get_subcommands().map(|sub| build_entry("", sub)).collect::<Vec<_>>();
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
) -> Result<ExitCode, ExitCode> {
    let entries = command_catalog();
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
    fn command_catalog_contains_new_operator_surfaces() {
        let entries = command_catalog();
        let mut flattened = Vec::new();
        for entry in &entries {
            super::flatten(entry, &mut flattened);
        }
        assert!(flattened.iter().any(|entry| entry.path == "commands"));
        assert!(flattened.iter().any(|entry| entry.path == "lab federation schedule"));
        assert!(flattened.iter().any(|entry| entry.path == "artifact fetch"));
        assert!(flattened.iter().any(|entry| entry.path == "trace-node"));
        assert!(
            flattened
                .iter()
                .any(|entry| entry.path == "doctor"
                    && entry.maturity == CommandMaturity::Experimental)
        );
    }

    #[test]
    fn command_groups_cover_public_taxonomy() {
        let groups = command_groups(&command_catalog());
        assert!(groups.contains(&"core".to_string()));
        assert!(groups.contains(&"runtime".to_string()));
        assert!(groups.contains(&"evidence".to_string()));
        assert!(groups.contains(&"cache".to_string()));
        assert!(groups.contains(&"diagnostics".to_string()));
        assert!(groups.contains(&"lab".to_string()));
    }
}
