#![forbid(unsafe_code)]
//! Canonical command catalog for normalization and route recognition.

use crate::contracts::canonical_bijux_tool_namespace;

fn contains(values: &[&str], value: &str) -> bool {
    values.contains(&value)
}

/// Canonical CLI root aliases supported by route normalization.
#[must_use]
pub fn cli_root_aliases() -> &'static [&'static str] {
    super::model::CLI_ROOT_ALIASES
}

/// Canonical `cli config` subcommands supported by route normalization.
#[must_use]
pub fn cli_config_subcommands() -> &'static [&'static str] {
    super::model::CLI_CONFIG_SUBCOMMANDS
}

/// Canonical `cli plugins` subcommands supported by route normalization.
#[must_use]
pub fn cli_plugins_subcommands() -> &'static [&'static str] {
    super::model::CLI_PLUGINS_SUBCOMMANDS
}

/// Normalize path aliases into canonical command paths.
#[must_use]
pub fn normalize_command_path(path: &[String]) -> Vec<String> {
    match path {
        [a, rest @ ..] if canonical_bijux_tool_namespace(a).is_some() => {
            let mut normalized = vec![canonical_bijux_tool_namespace(a)
                .expect("checked canonical namespace")
                .to_string()];
            normalized.extend(rest.iter().cloned());
            normalized
        }
        [a] if contains(super::model::CLI_ROOT_ALIASES, a) => vec!["cli".to_string(), a.clone()],
        [a, b] if a == "config" && contains(super::model::CLI_CONFIG_SUBCOMMANDS, b) => {
            vec!["cli".to_string(), "config".to_string(), b.clone()]
        }
        [a, b] if a == "plugins" && contains(super::model::CLI_PLUGINS_SUBCOMMANDS, b) => {
            vec!["cli".to_string(), "plugins".to_string(), b.clone()]
        }
        _ => path.to_vec(),
    }
}

/// Return true when normalized path is a known built-in command route.
#[must_use]
pub fn is_known_route(path: &[String]) -> bool {
    super::model::is_known_route(path)
}

/// REPL reference commands rendered in command help.
#[must_use]
pub fn repl_reference_commands() -> &'static [&'static str] {
    super::model::REPL_REFERENCE_COMMANDS
}
