#![forbid(unsafe_code)]

pub(crate) mod error;
pub(crate) mod serialization;
pub(crate) mod service;
pub(crate) mod storage;
pub(crate) mod validation;

use anyhow::{anyhow, Result};
use bijux_cli_install::CompatibilityPaths;
use serde_json::Value;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

use service::{ConfigService, DefaultConfigService, StaticConfigPathProvider};
use storage::FileConfigRepository;
use crate::argv::{command_option_value, command_positionals};

pub(crate) fn execute_config_command(
    normalized_path: &[String],
    argv: &[String],
    paths: &CompatibilityPaths,
) -> Result<Option<Value>> {
    let service = DefaultConfigService::new(
        StaticConfigPathProvider::new(paths.config_file.clone()),
        FileConfigRepository,
    );

    let result = match normalized_path {
        [a] if a == "config" => {
            Some(service.list_entries().map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            let positional = command_positionals(argv, &["cli", "config", "get"]);
            let raw_key =
                positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(service.get_value(raw_key).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            let positional = command_positionals(argv, &["cli", "config", "set"]);
            let raw_pair = positional.first().cloned().or_else(read_pair_from_stdin_fallback);
            let raw_pair =
                raw_pair.ok_or_else(|| anyhow!("Missing argument: KEY=VALUE required"))?;
            Some(service.set_pair(&raw_pair).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "unset" => {
            let positional = command_positionals(argv, &["cli", "config", "unset"]);
            let raw_key =
                positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(service.unset_key(raw_key).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "clear" => {
            let _ = command_positionals(argv, &["cli", "config", "clear"]);
            Some(service.clear_all().map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "reload" => {
            let _ = command_positionals(argv, &["cli", "config", "reload"]);
            Some(service.reload().map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "export" => {
            let positional = command_positionals(argv, &["cli", "config", "export"]);
            let raw_path = positional.first().ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let format = command_option_value(argv, &["cli", "config", "export"], "--format")
                .unwrap_or_else(|| "json".to_string());
            if format == "text" {
                return Err(anyhow!("Unsupported format: text"));
            }
            let target_path = PathBuf::from(raw_path);
            Some(service.export_to(&target_path).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "load" => {
            let positional = command_positionals(argv, &["cli", "config", "load"]);
            let raw_path = positional.first().ok_or_else(|| anyhow!("Missing parameter: path"))?;
            let source_path = PathBuf::from(raw_path);
            Some(
                service
                    .load_from(&source_path)
                    .map_err(|err| anyhow!("Failed to load config: {}", err))?,
            )
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
