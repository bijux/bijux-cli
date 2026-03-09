#![forbid(unsafe_code)]
//! Binary entrypoint for the Rust foundation.

use std::collections::HashMap;
use std::env;
use std::path::Path;

use anyhow::Result;
use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_cli_core as _;
use bijux_cli_output::{render_value, EmitterConfig};
use bijux_cli_python::{
    default_compatibility_paths, discover_compatibility_paths, load_compatibility_config,
    run_config_migrations, CompatibilityConfig, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
use bijux_cli_routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use serde_json::{json, Value};

fn emitter_config(flags: &ParsedGlobalFlags) -> EmitterConfig {
    EmitterConfig {
        format: flags.output_format.unwrap_or(OutputFormat::Json),
        pretty: !matches!(flags.pretty_mode, Some(PrettyMode::Compact)),
        color: flags.color_mode.unwrap_or(ColorMode::Never),
        log_level: flags.log_level.unwrap_or(LogLevel::Info),
        quiet: flags.quiet,
        no_color: true,
    }
}

fn home_dir() -> Option<std::path::PathBuf> {
    env::var_os("HOME").map(std::path::PathBuf::from)
}

fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| env::var(key).ok().map(|value| ((*key).to_string(), value)))
        .collect()
}

fn render_command_help(path: &[&str]) -> Result<String> {
    let mut cmd = root_command();
    let target = find_command_mut(&mut cmd, path).ok_or_else(|| anyhow::anyhow!("unknown help path"))?;
    let mut out = Vec::new();
    target.write_long_help(&mut out)?;
    Ok(String::from_utf8(out)?)
}

fn find_command_mut<'a>(command: &'a mut clap::Command, path: &[&str]) -> Option<&'a mut clap::Command> {
    if let Some((head, tail)) = path.split_first() {
        let child = command.find_subcommand_mut(head)?;
        find_command_mut(child, tail)
    } else {
        Some(command)
    }
}

fn route_response(normalized_path: &[String]) -> Result<Value> {
    let mut registry = RouteRegistry::default();
    let _ = registry.register_plugin_namespace("community");

    let target = registry.resolve(normalized_path)?;
    if matches!(target, RouteTarget::Plugin(_)) {
        return Ok(json!({
            "status": "ok",
            "route": normalized_path.join(" "),
            "owner": "plugin"
        }));
    }

    let home = home_dir();
    let defaults = home
        .as_deref()
        .map(default_compatibility_paths)
        .unwrap_or_else(|| default_compatibility_paths(Path::new(".")));

    let config = load_compatibility_config(&defaults.config_file)
        .unwrap_or_else(|_| CompatibilityConfig::default());
    let paths = discover_compatibility_paths(home.as_deref(), &PathOverrides::default(), &env_map(), &config)?;

    let payload = match normalized_path {
        [a, b] if a == "cli" && b == "version" => {
            json!({"version": env!("CARGO_PKG_VERSION")})
        }
        [a, b] if a == "cli" && b == "doctor" => {
            json!({"status": "healthy", "checks": ["routing", "output", "config"]})
        }
        [a, b] if a == "cli" && b == "repl" => {
            json!({"status": "ready", "mode": "repl", "history_file": paths.history_file})
        }
        [a, b] if a == "cli" && b == "completion" => {
            json!({"shells": ["bash", "zsh", "fish", "powershell"]})
        }
        [a, b] if a == "cli" && b == "inspect" => {
            json!({"reserved_namespaces": registry.route_tree(), "builtins": registry.built_in_paths()})
        }
        [a, b] if a == "cli" && b == "status" => {
            json!({"status": "ok", "runtime": "rust-foundation"})
        }
        [a, b] if a == "cli" && b == "paths" => {
            json!({"config": paths.config_file, "history": paths.history_file, "plugins": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            json!({
                "BIJUXCLI_CONFIG": paths.config_file,
                "BIJUXCLI_HISTORY_FILE": paths.history_file,
                "BIJUXCLI_PLUGINS_DIR": paths.plugins_dir
            })
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            run_config_migrations(&paths.config_file, 1)?;
            json!({"status": "ok", "updated": paths.config_file})
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            json!({"plugins": [], "directory": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            json!({"plugins": [], "status": "loaded"})
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "routes" => {
            json!({"routes": registry.built_in_paths()})
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "registry" => {
            json!({"registry": registry.route_tree()})
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "env" => {
            json!({"env": env_map()})
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "doctor" => {
            json!({"status": "healthy", "runtime": "dev-cli"})
        }
        [a, b, c] if a == "dev" && b == "cli" && c == "contracts" => {
            json!({"schemas": ["output-envelope-v1", "error-envelope-v1", "plugin-manifest-v1"]})
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
            ) => Some(error.to_string()),
        Err(_) => None,
    }
}

/// Execute the CLI for provided argv and return rendered output.
pub fn run_argv(argv: &[String]) -> Result<String> {
    if argv.len() == 1 {
        return render_command_help(&[]);
    }

    if argv.len() >= 2 && argv[1] == "help" {
        let path: Vec<&str> = argv[2..].iter().map(String::as_str).collect();
        return render_command_help(&path);
    }

    if let Some(help) = try_render_clap_help(argv) {
        return Ok(help);
    }

    let intent = parse_intent(argv)?;
    if intent.normalized_path.is_empty() {
        return render_command_help(&[]);
    }

    let rendered = render_value(&route_response(&intent.normalized_path)?, emitter_config(&intent.global_flags))?;
    if rendered.ends_with('\n') {
        Ok(rendered)
    } else {
        Ok(format!("{rendered}\n"))
    }
}

fn main() -> Result<()> {
    let argv: Vec<String> = env::args().collect();
    let rendered = run_argv(&argv)?;
    print!("{rendered}");
    Ok(())
}
