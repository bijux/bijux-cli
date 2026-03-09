#![forbid(unsafe_code)]

pub(crate) mod error;
pub(crate) mod serialization;
pub(crate) mod service;
pub(crate) mod storage;
pub(crate) mod validation;

use anyhow::{anyhow, Result};
use bijux_cli_install::CompatibilityPaths;
use serde_json::Value;
use std::path::PathBuf;
use std::io::{self, IsTerminal, Read};

use service::{ConfigService, DefaultConfigService, StaticConfigPathProvider};
use storage::FileConfigRepository;

fn command_positionals(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
    let mut extra_start = 1 + command_tokens.len();
    if argv.len() < extra_start {
        return Vec::new();
    }
    for (idx, token) in command_tokens.iter().enumerate() {
        if argv.get(idx + 1).map(String::as_str) != Some(*token) {
            extra_start = idx + 1;
            break;
        }
    }
    let extras = &argv[extra_start..];
    let mut positional = Vec::new();
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == "--quiet" || token == "-q" || token == "--pretty" || token == "--no-pretty" {
            i += 1;
            continue;
        }
        if token == "--format"
            || token == "-f"
            || token == "--log-level"
            || token == "--color"
            || token == "--config-path"
        {
            i += 2;
            continue;
        }
        if token.starts_with("--format=")
            || token.starts_with("--log-level=")
            || token.starts_with("--color=")
            || token.starts_with("--config-path=")
        {
            i += 1;
            continue;
        }
        if token.starts_with('-') {
            i += 1;
            continue;
        }
        positional.push(token.clone());
        i += 1;
    }
    positional
}

fn command_option_value(argv: &[String], command_tokens: &[&str], option: &str) -> Option<String> {
    let mut extra_start = 1 + command_tokens.len();
    if argv.len() < extra_start {
        return None;
    }
    for (idx, token) in command_tokens.iter().enumerate() {
        if argv.get(idx + 1).map(String::as_str) != Some(*token) {
            extra_start = idx + 1;
            break;
        }
    }

    let extras = &argv[extra_start..];
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == option {
            return extras.get(i + 1).cloned();
        }
        if token.starts_with(&(option.to_string() + "=")) {
            return token.split_once('=').map(|(_, value)| value.to_string());
        }
        i += 1;
    }

    None
}

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
        [a] if a == "config" => Some(service.list_entries().map_err(|err| anyhow!(err.to_string()))?),
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            let positional = command_positionals(argv, &["cli", "config", "get"]);
            let raw_key = positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
            Some(service.get_value(raw_key).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            let positional = command_positionals(argv, &["cli", "config", "set"]);
            let raw_pair = positional.first().cloned().or_else(read_pair_from_stdin_fallback);
            let raw_pair = raw_pair.ok_or_else(|| anyhow!("Missing argument: KEY=VALUE required"))?;
            Some(service.set_pair(&raw_pair).map_err(|err| anyhow!(err.to_string()))?)
        }
        [a, b, c] if a == "cli" && b == "config" && c == "unset" => {
            let positional = command_positionals(argv, &["cli", "config", "unset"]);
            let raw_key = positional.first().ok_or_else(|| anyhow!("Missing argument: KEY required"))?;
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
            let raw_path = positional
                .first()
                .ok_or_else(|| anyhow!("Missing parameter: path"))?;
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
            let raw_path = positional
                .first()
                .ok_or_else(|| anyhow!("Missing parameter: path"))?;
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
