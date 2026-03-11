//! Top-level application entrypoint and route execution.

use std::env;
use std::process::Command;

use crate::features::developer::runtime_query_adapter;
use crate::interface::cli::handlers::{
    cli as cli_handlers, config as config_handlers, history as history_handlers,
    memory as memory_handlers, plugins as plugins_handlers, root as root_handlers,
};
use crate::interface::cli::parser::{parse_intent, root_command, ParsedGlobalFlags};
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::routing::known_bijux_tool;
use crate::routing::registry::{RouteRegistry, RouteTarget};
use crate::routing::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use anyhow::Result;
use bijux_dev_cli::dispatch::owns_path as owns_dev_cli_path;
use serde_json::{json, Value};

use crate::features::diagnostics::state_paths::resolve_state_paths;
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
    let plugin_registry_path = paths.plugin_registry_file.clone();
    if let Some(payload) =
        config_handlers::execute_config_command(normalized_path, argv, &paths.config_file)?
    {
        return Ok(payload);
    }
    if let Some(payload) = history_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) = memory_handlers::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) =
        plugins_handlers::try_handle(normalized_path, argv, &paths, &plugin_registry_path)?
    {
        return Ok(payload);
    }
    if let Some(payload) = runtime_query_adapter::try_handle(
        normalized_path,
        argv,
        &registry,
        &paths,
        &plugin_registry_path,
    )? {
        return Ok(payload);
    }
    if let Some(payload) =
        cli_handlers::try_handle(normalized_path, &paths, &registry, &plugin_registry_path)
    {
        return Ok(payload);
    }
    if let Some(payload) = root_handlers::try_handle(normalized_path, argv) {
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

fn delegate_to_external_binary(
    binary: &str,
    package_name: &str,
    command_surface: &str,
    forwarded_args: &[String],
) -> AppRunResult {
    match Command::new(binary).args(forwarded_args).output() {
        Ok(output) => AppRunResult {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
        Err(error) => {
            let message = format!(
                "failed to run `{command_surface}` via `{binary}`: {error}\ninstall with `cargo install {package_name}` or `pip install {package_name}`\n"
            );
            AppRunResult {
                exit_code: 1,
                stdout: String::new(),
                stderr: message,
            }
        }
    }
}

fn maintenance_route_exit_code(normalized_path: &[String], payload: &Value) -> Option<i32> {
    let is_maintenance_runner = matches!(
        normalized_path,
        [a, b, c, d]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && (d == "generate" || d == "generate-all")
    ) || matches!(
        normalized_path,
        [a, b, c, d, e]
            if a == "dev"
                && b == "cli"
                && c == "maintenance"
                && d == "status"
                && (e == "run" || e == "run-all")
    );

    if !is_maintenance_runner {
        return None;
    }

    if payload
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status == "failed" || status == "error")
    {
        let exit_code = payload
            .get("exit_code")
            .and_then(Value::as_i64)
            .filter(|code| *code > 0)
            .unwrap_or(1);
        return Some(exit_code as i32);
    }

    if payload
        .get("failed")
        .and_then(Value::as_u64)
        .is_some_and(|count| count > 0)
    {
        return Some(1);
    }

    if payload
        .get("results")
        .and_then(Value::as_array)
        .is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| status == "failed" || status == "error")
            })
        })
    {
        return Some(1);
    }

    Some(0)
}

fn try_delegate_known_bijux_tool(argv: &[String]) -> Option<AppRunResult> {
    let first = argv.get(1)?;

    if let Some(tool) = known_bijux_tool(first) {
        if tool.namespace == "atlas" && argv.len() == 2 {
            return None;
        }
        let command_surface = format!("bijux {}", tool.namespace);
        return Some(delegate_to_external_binary(
            tool.runtime_binary,
            tool.runtime_package,
            &command_surface,
            &argv[2..],
        ));
    }

    if first == "dev" {
        let tool_namespace = argv.get(2)?;
        if tool_namespace == "cli" {
            return None;
        }
        if let Some(tool) = known_bijux_tool(tool_namespace) {
            let command_surface = format!("bijux dev {}", tool.namespace);
            return Some(delegate_to_external_binary(
                tool.control_binary,
                tool.control_package,
                &command_surface,
                &argv[3..],
            ));
        }
    }

    None
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
        return Ok(AppRunResult {
            exit_code: 0,
            stdout: help,
            stderr: String::new(),
        });
    }

    if let Some(delegated) = try_delegate_known_bijux_tool(argv) {
        return Ok(delegated);
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
    let content = if rendered.ends_with('\n') {
        rendered
    } else {
        format!("{rendered}\n")
    };

    if is_unknown {
        return Ok(AppRunResult {
            exit_code: 2,
            stdout: String::new(),
            stderr: content,
        });
    }

    let route_exit_code =
        maintenance_route_exit_code(&intent.normalized_path, &payload).unwrap_or(0);

    if intent.global_flags.quiet {
        return Ok(AppRunResult {
            exit_code: route_exit_code,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    Ok(AppRunResult {
        exit_code: route_exit_code,
        stdout: content,
        stderr: String::new(),
    })
}
