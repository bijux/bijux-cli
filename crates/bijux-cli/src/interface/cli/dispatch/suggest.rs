//! Unknown-route correction hints and next-command guidance.

use std::cmp::max;

use crate::contracts::KNOWN_BIJUX_TOOL_NAMESPACES;
use crate::routing::model::{CLI_CONFIG_SUBCOMMANDS, CLI_PLUGINS_SUBCOMMANDS, CLI_ROOT_ALIASES};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RouteCorrection {
    pub(super) nearest_command: String,
    pub(super) next_command: String,
    pub(super) next_help: String,
}

const ROOT_COMMANDS: &[&str] = &[
    "cli",
    "dev",
    "status",
    "audit",
    "docs",
    "sleep",
    "doctor",
    "version",
    "config",
    "plugins",
    "repl",
    "completion",
    "history",
    "memory",
    "help",
];

const CLI_COMMANDS: &[&str] = &["status", "paths", "config", "self-test", "plugins"];

pub(super) fn correction_for_unknown_route(path: &[String]) -> Option<RouteCorrection> {
    let suggested = suggest_path(path)?;
    let joined = suggested.join(" ");
    Some(RouteCorrection {
        nearest_command: joined.clone(),
        next_command: format!("bijux {joined}"),
        next_help: format!("bijux help {joined}"),
    })
}

fn suggest_path(path: &[String]) -> Option<Vec<String>> {
    let [first, rest @ ..] = path else {
        return None;
    };

    match first.as_str() {
        "config" => suggest_tail("config", rest, CLI_CONFIG_SUBCOMMANDS, "config"),
        "plugins" => suggest_tail("plugins", rest, CLI_PLUGINS_SUBCOMMANDS, "plugins"),
        "cli" => suggest_cli(rest),
        "dev" => suggest_dev(rest),
        known if ROOT_COMMANDS.contains(&known) => Some(path.to_vec()),
        _ => {
            let best_root = nearest(first, ROOT_COMMANDS)?;
            let mut out = vec![best_root.to_string()];
            out.extend(rest.iter().cloned());
            Some(out)
        }
    }
}

fn suggest_cli(rest: &[String]) -> Option<Vec<String>> {
    let [second, tail @ ..] = rest else {
        return Some(vec!["cli".to_string()]);
    };

    match second.as_str() {
        "config" => suggest_tail("cli", tail, CLI_CONFIG_SUBCOMMANDS, "config"),
        "plugins" => suggest_tail("cli", tail, CLI_PLUGINS_SUBCOMMANDS, "plugins"),
        known if CLI_COMMANDS.contains(&known) || CLI_ROOT_ALIASES.contains(&known) => {
            Some(vec!["cli".to_string(), known.to_string()])
        }
        _ => {
            let mut candidates = CLI_COMMANDS.to_vec();
            candidates.extend_from_slice(CLI_ROOT_ALIASES);
            let best = nearest(second, &candidates)?;
            Some(vec!["cli".to_string(), best.to_string()])
        }
    }
}

fn suggest_dev(rest: &[String]) -> Option<Vec<String>> {
    let [second, tail @ ..] = rest else {
        return Some(vec!["dev".to_string(), "cli".to_string()]);
    };

    if second == "cli" {
        let [third, _] = tail else {
            return Some(vec!["dev".to_string(), "cli".to_string()]);
        };

        return Some(vec!["dev".to_string(), "cli".to_string(), third.clone()]);
    }

    if KNOWN_BIJUX_TOOL_NAMESPACES.contains(&second.as_str()) {
        let mut out = vec!["dev".to_string(), second.clone()];
        out.extend(tail.iter().cloned());
        return Some(out);
    }

    let mut candidates = vec!["cli"];
    candidates.extend_from_slice(KNOWN_BIJUX_TOOL_NAMESPACES);
    let best = nearest(second, &candidates)?;
    if best == "cli" {
        Some(vec!["dev".to_string(), "cli".to_string()])
    } else {
        Some(vec!["dev".to_string(), best.to_string()])
    }
}

fn suggest_tail(
    root: &str,
    rest: &[String],
    candidates: &[&str],
    namespace: &str,
) -> Option<Vec<String>> {
    let [second, _] = rest else {
        return match root {
            "cli" => Some(vec!["cli".to_string(), namespace.to_string()]),
            _ => Some(vec![namespace.to_string()]),
        };
    };

    let best = nearest(second, candidates)?;
    if root == "cli" {
        Some(vec!["cli".to_string(), namespace.to_string(), best.to_string()])
    } else {
        Some(vec![namespace.to_string(), best.to_string()])
    }
}

fn nearest<'a>(query: &str, candidates: &'a [&'a str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .max_by_key(|candidate| similarity_score(&query.to_ascii_lowercase(), candidate))
}

fn similarity_score(left: &str, right: &str) -> usize {
    let right = right.to_ascii_lowercase();
    let prefix = common_prefix_len(left, &right);
    let distance = levenshtein_distance(left, &right);
    let normalized = max(left.chars().count(), right.chars().count());
    (prefix * 1000) + normalized.saturating_sub(distance)
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars().zip(right.chars()).take_while(|(a, b)| a == b).count()
}

fn levenshtein_distance(left: &str, right: &str) -> usize {
    let l: Vec<char> = left.chars().collect();
    let r: Vec<char> = right.chars().collect();
    if l.is_empty() {
        return r.len();
    }
    if r.is_empty() {
        return l.len();
    }

    let mut prev: Vec<usize> = (0..=r.len()).collect();
    let mut curr = vec![0; r.len() + 1];

    for (i, lc) in l.iter().enumerate() {
        curr[0] = i + 1;
        for (j, rc) in r.iter().enumerate() {
            let cost = usize::from(lc != rc);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        prev.clone_from(&curr);
    }

    prev[r.len()]
}
