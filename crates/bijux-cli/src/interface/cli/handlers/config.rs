#![forbid(unsafe_code)]
//! `config` command handlers.

use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::features::config::layered::LayeredConfigOptions;
use crate::features::config::operations as config_operations;
use crate::shared::argv::{command_has_flag, command_option_value, command_positionals};

pub(crate) fn execute_config_command(
    normalized_path: &[String],
    argv: &[String],
    config_file: &Path,
) -> Result<Option<Value>> {
    let get_tokens = config_command_tokens(argv, &["get"]);
    let set_tokens = config_command_tokens(argv, &["set"]);
    let unset_tokens = config_command_tokens(argv, &["unset"]);
    let clear_tokens = config_command_tokens(argv, &["clear"]);
    let reload_tokens = config_command_tokens(argv, &["reload"]);
    let validate_tokens = config_command_tokens(argv, &["validate"]);
    let schema_tokens = config_command_tokens(argv, &["schema"]);
    let explain_tokens = config_command_tokens(argv, &["explain"]);
    let repair_tokens = config_command_tokens(argv, &["repair"]);
    let export_tokens = config_command_tokens(argv, &["export"]);
    let load_tokens = config_command_tokens(argv, &["load"]);
    let current_dir = std::env::current_dir()
        .map_err(|err| anyhow!("Failed to resolve current directory: {err}"))?;

    let result = match normalized_path {
        [a] if a == "config" => Some(config_operations::list_entries(config_file)?),
        [a, b] if a == "config" && b == "list" => {
            Some(config_operations::list_entries(config_file)?)
        }
        [a, b] if a == "cli" && b == "config" => {
            Some(config_operations::list_entries(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "list" => {
            Some(config_operations::list_entries(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            let positional = command_positionals(argv, get_tokens);
            let raw_key =
                positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(config_operations::get_value(config_file, raw_key)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            let positional = command_positionals(argv, set_tokens);
            let raw_pair = positional.first().cloned().or_else(read_pair_from_stdin_fallback);
            let raw_pair =
                raw_pair.ok_or_else(|| anyhow!("Missing argument: KEY=VALUE required"))?;
            Some(config_operations::set_pair(config_file, &raw_pair)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "unset" => {
            let positional = command_positionals(argv, unset_tokens);
            let raw_key =
                positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(config_operations::unset_key(config_file, raw_key)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "clear" => {
            let _ = command_positionals(argv, clear_tokens);
            Some(config_operations::clear_all(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "reload" => {
            let _ = command_positionals(argv, reload_tokens);
            Some(config_operations::reload(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "validate" => {
            let _ = command_positionals(argv, validate_tokens);
            let profile = command_option_value(argv, validate_tokens, "--profile");
            Some(config_operations::validate(config_file, &current_dir, profile.as_deref())?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "schema" => {
            let positional = command_positionals(argv, schema_tokens);
            Some(config_operations::schema(positional.first().map(String::as_str))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "explain" => {
            let positional = command_positionals(argv, explain_tokens);
            let raw_key =
                positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            let profile = command_option_value(argv, explain_tokens, "--profile");
            let include_secrets = command_has_flag(argv, "--include-secrets");
            Some(config_operations::explain(
                config_file,
                &current_dir,
                raw_key,
                profile.as_deref(),
                include_secrets,
            )?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "repair" => {
            let _ = command_positionals(argv, repair_tokens);
            Some(config_operations::repair(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "export" => {
            let positional = command_positionals(argv, export_tokens);
            let raw_path = positional.first().ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let target_path = PathBuf::from(raw_path);
            let profile = command_option_value(argv, export_tokens, "--profile");
            let include_secrets = command_has_flag(argv, "--include-secrets");
            let portable = command_has_flag(argv, "--portable");
            if portable || profile.is_some() || include_secrets {
                Some(config_operations::export_with_options(
                    config_file,
                    &current_dir,
                    &target_path,
                    &LayeredConfigOptions { profile, include_secrets, portable },
                )?)
            } else {
                Some(config_operations::export_to(config_file, &target_path)?)
            }
        }
        [a, b, c] if a == "cli" && b == "config" && c == "load" => {
            let positional = command_positionals(argv, load_tokens);
            let raw_path = positional.first().ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let source_path = PathBuf::from(raw_path);
            let profile = command_option_value(argv, load_tokens, "--profile");
            let portable = command_has_flag(argv, "--portable");
            if portable || profile.is_some() {
                Some(config_operations::load_with_options(
                    config_file,
                    &source_path,
                    &LayeredConfigOptions { profile, include_secrets: false, portable },
                )?)
            } else {
                Some(config_operations::load_from(config_file, &source_path)?)
            }
        }
        _ => None,
    };

    Ok(result)
}

fn config_command_tokens<'a>(argv: &[String], suffix: &'a [&'a str]) -> &'a [&'a str] {
    if command_starts_with_root_config(argv) {
        match suffix {
            ["list"] => &["config", "list"],
            ["get"] => &["config", "get"],
            ["set"] => &["config", "set"],
            ["unset"] => &["config", "unset"],
            ["clear"] => &["config", "clear"],
            ["reload"] => &["config", "reload"],
            ["validate"] => &["config", "validate"],
            ["schema"] => &["config", "schema"],
            ["explain"] => &["config", "explain"],
            ["repair"] => &["config", "repair"],
            ["export"] => &["config", "export"],
            ["load"] => &["config", "load"],
            _ => &["config"],
        }
    } else {
        match suffix {
            ["list"] => &["cli", "config", "list"],
            ["get"] => &["cli", "config", "get"],
            ["set"] => &["cli", "config", "set"],
            ["unset"] => &["cli", "config", "unset"],
            ["clear"] => &["cli", "config", "clear"],
            ["reload"] => &["cli", "config", "reload"],
            ["validate"] => &["cli", "config", "validate"],
            ["schema"] => &["cli", "config", "schema"],
            ["explain"] => &["cli", "config", "explain"],
            ["repair"] => &["cli", "config", "repair"],
            ["export"] => &["cli", "config", "export"],
            ["load"] => &["cli", "config", "load"],
            _ => &["cli", "config"],
        }
    }
}

fn command_starts_with_root_config(argv: &[String]) -> bool {
    let mut command_start = 1;
    while command_start < argv.len() {
        let token = argv[command_start].as_str();
        if token == "--quiet" || token == "-q" || token == "--pretty" || token == "--no-pretty" {
            command_start += 1;
            continue;
        }
        if token == "--format"
            || token == "-f"
            || token == "--log-level"
            || token == "--color"
            || token == "--config-path"
        {
            command_start += 2;
            continue;
        }
        if token.starts_with("--format=")
            || token.starts_with("--log-level=")
            || token.starts_with("--color=")
            || token.starts_with("--config-path=")
            || token == "--json"
            || token == "--text"
        {
            command_start += 1;
            continue;
        }
        break;
    }
    argv.get(command_start).is_some_and(|segment| segment == "config")
}

fn read_pair_from_stdin_fallback() -> Option<String> {
    let mut stdin = io::stdin();
    if stdin.is_terminal() {
        return None;
    }

    let mut buf = String::new();
    if stdin.read_to_string(&mut buf).is_err() {
        return None;
    }

    let trimmed = buf.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
