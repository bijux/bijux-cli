//! Top-level application entrypoint and route execution.

use std::env;

use crate::features::install::CompatibilityPaths;
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use crate::routing::registry::{RouteRegistry, RouteTarget};
use crate::routing::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use anyhow::Result;
use bijux_dev_cli::dispatch::owns_path as owns_dev_cli_path;
use serde_json::{json, Value};

use crate::features::config::execute_config_command;
use crate::features::diagnostics::{cli_command as cli_commands, root_command as root_commands};
use crate::features::{
    developer as developer_feature, history as history_feature, memory as memory_feature,
    plugins as plugins_feature,
};
use crate::infrastructure::state_paths::resolve_state_paths;
use crate::interface::cli::help::render_command_help;
use crate::shared::output::{render_value, EmitterConfig};

/// In-memory process output and exit result produced by the core app runner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunResult {
    /// Process exit code.
    pub exit_code: i32,
    /// Payload that should be written to stdout.
    pub stdout: String,
    /// Payload that should be written to stderr.
    pub stderr: String,
}

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: env::var("NO_COLOR").ok().as_deref() == Some("1"),
    }
}

fn route_response(
    normalized_path: &[String],
    argv: &[String],
    global_flags: &ParsedGlobalFlags,
) -> Result<Value> {
    let mut registry = RouteRegistry::default();
    let _ = registry.register_plugin_namespace("community");

    let target = match normalized_path {
        [a] if a == "config"
            || a == "history"
            || a == "memory"
            || a == "plugins"
            || a == "dev"
            || a == "atlas" =>
        {
            RouteTarget::BuiltIn
        }
        [a, b] if a == "history" && b == "clear" => RouteTarget::BuiltIn,
        [a, b]
            if a == "memory"
                && (b == "list" || b == "get" || b == "set" || b == "delete" || b == "clear") =>
        {
            RouteTarget::BuiltIn
        }
        [a, b]
            if a == "plugins"
                && (b == "list"
                    || b == "info"
                    || b == "inspect"
                    || b == "check"
                    || b == "reserved-names"
                    || b == "where"
                    || b == "explain"
                    || b == "schema") =>
        {
            RouteTarget::BuiltIn
        }
        _ if owns_dev_cli_path(normalized_path) => RouteTarget::BuiltIn,
        _ => registry.resolve(normalized_path)?,
    };
    if matches!(target, RouteTarget::Plugin(_)) {
        return Ok(json!({
            "status": "ok",
            "route": normalized_path.join(" "),
            "owner": "plugin"
        }));
    }

    let paths = resolve_state_paths(global_flags)?;
    let compatibility_paths = CompatibilityPaths {
        config_file: paths.config_file.clone(),
        history_file: paths.history_file.clone(),
        plugins_dir: paths.plugins_dir.clone(),
    };
    let plugin_registry_path = paths.plugin_registry_file.clone();
    if let Some(payload) = execute_config_command(normalized_path, argv, &compatibility_paths)? {
        return Ok(payload);
    }
    if let Some(payload) = history_feature::command::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) = memory_feature::command::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) =
        plugins_feature::command::try_handle(normalized_path, argv, &paths, &plugin_registry_path)?
    {
        return Ok(payload);
    }
    if let Some(payload) =
        developer_feature::command::try_handle(normalized_path, &plugin_registry_path)?
    {
        return Ok(payload);
    }
    if let Some(payload) = developer_feature::runtime_adapter::try_handle(
        normalized_path,
        argv,
        &registry,
        &paths,
        &plugin_registry_path,
    )? {
        return Ok(payload);
    }
    if let Some(payload) =
        cli_commands::try_handle(normalized_path, &paths, &registry, &plugin_registry_path)
    {
        return Ok(payload);
    }
    if let Some(payload) = root_commands::try_handle(normalized_path, argv) {
        return Ok(payload);
    }

    let payload = json!({"status": "error", "message": "unknown route"});

    Ok(payload)
}

fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(error.to_string())
        }
        Err(_) => None,
    }
}

/// Execute the CLI for provided argv and return output streams and exit code.
pub fn run_app(argv: &[String]) -> Result<AppRunResult> {
    if argv.len() == 1 {
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&[])?.trim_end()),
            stderr: String::new(),
        });
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path: Vec<&str> = argv[2..].iter().map(String::as_str).collect();
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: format!("{}\n", render_command_help(&path)?.trim_end()),
            stderr: String::new(),
        });
    }

    if let Some(help) = try_render_clap_help(argv) {
        return Ok(AppRunResult { exit_code: 0, stdout: help, stderr: String::new() });
    }

    let intent = parse_intent(argv)?;
    if intent.normalized_path.is_empty() {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: format!("{}\n", render_command_help(&[])?.trim_end()),
        });
    }

    let is_unknown = !is_known_catalog_route(&intent.normalized_path);

    let response = route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = if message.contains("Missing argument")
                || message.contains("Invalid argument")
                || message.contains("Key cannot be empty")
                || message.contains("Invalid key")
                || message.contains("Unknown config section")
                || message.contains("Config key not found")
                || message.contains("Missing parameter")
                || message.contains("Unsupported format")
                || message.contains("Failed to load config")
            {
                2
            } else if message.contains("Non-ASCII") || message.contains("Control characters") {
                3
            } else {
                1
            };
            let rendered_error = render_value(
                &json!({
                    "status": "error",
                    "code": code,
                    "message": message,
                    "command": intent.normalized_path.join(" "),
                }),
                emitter_config(&intent.global_flags),
            )?;
            let error_content = if rendered_error.ends_with('\n') {
                rendered_error
            } else {
                format!("{rendered_error}\n")
            };
            return Ok(AppRunResult {
                exit_code: code,
                stdout: String::new(),
                stderr: error_content,
            });
        }
    };

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') { rendered } else { format!("{rendered}\n") };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    if intent.global_flags.quiet {
        return Ok(AppRunResult { exit_code: 0, stdout: String::new(), stderr: String::new() });
    }

    Ok(AppRunResult { exit_code: 0, stdout: content, stderr: String::new() })
}
