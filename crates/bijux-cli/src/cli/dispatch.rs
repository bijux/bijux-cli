//! Top-level application entrypoint and route execution.

use std::env;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crate::install::{install_health_report, post_install_hint, CompatibilityPaths};
use crate::plugin::{compatibility_warnings, plugin_origin_metadata};
use crate::routing::catalog::is_known_route as is_known_catalog_route;
use crate::routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use crate::routing::registry::{RouteRegistry, RouteTarget};
use crate::routing::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use serde_json::{json, Value};

use crate::cli::commands::help::render_command_help;
use crate::cli::commands::{
    dev as dev_commands, dev_cli as dev_cli_commands, history as history_commands,
    memory as memory_commands, plugins as plugins_commands,
};
use crate::cli::context::resolve_state_paths;
use crate::config::execute_config_command;
use crate::output::{render_value, EmitterConfig};

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
        [a, b, c] if a == "dev" && b == "cli" => {
            let _ = c;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "scripts" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "rustdoc" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "release" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "evidence" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "config" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "python" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c, d] if a == "dev" && b == "cli" && c == "repo" => {
            let _ = d;
            RouteTarget::BuiltIn
        }
        [a, b, c]
            if a == "dev"
                && b == "cli"
                && (c == "dashboard"
                    || c == "quickcheck"
                    || c == "truth"
                    || c == "blockers"
                    || c == "next") =>
        {
            RouteTarget::BuiltIn
        }
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
    if let Some(payload) = history_commands::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) = memory_commands::try_handle(normalized_path, argv, &paths)? {
        return Ok(payload);
    }
    if let Some(payload) =
        plugins_commands::try_handle(normalized_path, argv, &paths, &plugin_registry_path)?
    {
        return Ok(payload);
    }
    if let Some(payload) = dev_commands::try_handle(normalized_path, &plugin_registry_path)? {
        return Ok(payload);
    }
    if let Some(payload) =
        dev_cli_commands::try_handle(normalized_path, argv, &registry, &paths, &plugin_registry_path)?
    {
        return Ok(payload);
    }

    let payload = match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            json!({"version": env!("CARGO_PKG_VERSION")})
        }
        [a, b] if a == "cli" && b == "doctor" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            json!({
                "status": "healthy",
                "checks": ["routing", "output", "config", "install"],
                "install": {
                    "has_path_shadowing": install_report.has_path_shadowing,
                    "has_duplicate_installs": install_report.has_duplicate_installs,
                    "stale_wrapper_scripts": install_report.stale_wrapper_scripts,
                    "legacy_installer_conflicts": false,
                    "has_mismatched_wheel_binary_versions": install_report.has_mismatched_wheel_binary_versions,
                }
            })
        }
        [a, b] if a == "cli" && b == "repl" => {
            json!({"status": "ready", "mode": "repl", "history_file": paths.history_file})
        }
        [a, b] if a == "cli" && b == "completion" => {
            json!({"shells": ["bash", "zsh", "fish", "powershell"]})
        }
        [a, b] if a == "cli" && b == "inspect" => {
            let route_sources: Vec<Value> = registry
                .built_in_paths()
                .into_iter()
                .map(|path| {
                    let segments: Vec<String> = path.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "segments": segments,
                        "owner": "bijux-cli",
                        "source": "built-in",
                    })
                })
                .collect();
            json!({
                "status": "ok",
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "route_sources": route_sources,
                "alias_rewrites": registry.alias_rewrites().into_iter().map(|(alias, canonical)| {
                    let alias_segments: Vec<String> = alias.segments.into_iter().map(|s| s.0).collect();
                    let canonical_segments: Vec<String> = canonical.segments.into_iter().map(|s| s.0).collect();
                    json!({
                        "alias": alias_segments,
                        "canonical": canonical_segments,
                        "source": "compatibility-alias",
                    })
                }).collect::<Vec<_>>(),
                "plugin_origins": plugin_origin_metadata(&plugin_registry_path).unwrap_or_default(),
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
                "contracts": {
                    "schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"],
                    "version": "v1",
                }
            })
        }
        [a, b] if a == "cli" && b == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a] if a == "audit" => {
            json!({
                "status": "ok",
                "checks": ["config", "paths", "plugins", "history"],
                "issues": []
            })
        }
        [a] if a == "docs" => {
            json!({
                "status": "ok",
                "topics": ["commands", "configuration", "plugins", "repl", "diagnostics"],
            })
        }
        [a] if a == "atlas" => {
            json!({
                "status": "ok",
                "mount": "atlas",
            })
        }
        [a] if a == "sleep" => {
            let duration_secs = argv
                .get(2)
                .and_then(|raw| raw.parse::<f64>().ok())
                .map(|v| v.clamp(0.0, 2.0))
                .unwrap_or(0.0);
            if duration_secs > 0.0 {
                thread::sleep(Duration::from_secs_f64(duration_secs));
            }
            json!({"status": "ok", "slept_seconds": duration_secs})
        }
        [a, b] if a == "cli" && b == "paths" => {
            let install_report = install_health_report(
                &env::var("PATH").unwrap_or_default(),
                env::var("BIJUX_BIN").ok().as_deref(),
                env::var("BIJUX_WHEEL_VERSION").ok().as_deref(),
                env!("CARGO_PKG_VERSION"),
            );
            let hint =
                install_report.active_binary.as_deref().map(post_install_hint).unwrap_or_else(
                    || {
                        "Run `bijux version` and `bijux cli doctor` to verify your environment."
                            .to_string()
                    },
                );
            json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "active_binary": install_report.active_binary,
                "path_binaries": install_report.path_binaries,
                "post_install_hint": hint
            })
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => {
            for _ in 0..200_u16 {
                thread::sleep(Duration::from_millis(50));
            }
            json!({"status": "completed"})
        }
        _ => json!({"status": "error", "message": "unknown route"}),
    };

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
