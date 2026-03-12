#![forbid(unsafe_code)]
//! `config` command handlers.

use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::features::config::operations as config_operations;
use crate::shared::argv::command_positionals;

pub(crate) fn execute_config_command(
    normalized_path: &[String],
    argv: &[String],
    config_file: &Path,
) -> Result<Option<Value>> {
    let result = match normalized_path {
        [a] if a == "config" => Some(config_operations::list_entries(config_file)?),
        [a, b] if a == "config" && b == "list" => {
            Some(config_operations::list_entries(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "list" => {
            Some(config_operations::list_entries(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            let positional = command_positionals(argv, &["cli", "config", "get"]);
            let raw_key = positional
                .first()
                .ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(config_operations::get_value(config_file, raw_key)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            let positional = command_positionals(argv, &["cli", "config", "set"]);
            let raw_pair = positional
                .first()
                .cloned()
                .or_else(read_pair_from_stdin_fallback);
            let raw_pair =
                raw_pair.ok_or_else(|| anyhow!("Missing argument: KEY=VALUE required"))?;
            Some(config_operations::set_pair(config_file, &raw_pair)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "unset" => {
            let positional = command_positionals(argv, &["cli", "config", "unset"]);
            let raw_key = positional
                .first()
                .ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(config_operations::unset_key(config_file, raw_key)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "clear" => {
            let _ = command_positionals(argv, &["cli", "config", "clear"]);
            Some(config_operations::clear_all(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "reload" => {
            let _ = command_positionals(argv, &["cli", "config", "reload"]);
            Some(config_operations::reload(config_file)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "export" => {
            let positional = command_positionals(argv, &["cli", "config", "export"]);
            let raw_path = positional
                .first()
                .ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let target_path = PathBuf::from(raw_path);
            Some(config_operations::export_to(config_file, &target_path)?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "load" => {
            let positional = command_positionals(argv, &["cli", "config", "load"]);
            let raw_path = positional
                .first()
                .ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let source_path = PathBuf::from(raw_path);
            Some(config_operations::load_from(config_file, &source_path)?)
        }
        _ => None,
    };

    Ok(result)
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
