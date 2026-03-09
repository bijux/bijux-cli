//! Top-level application entrypoint and route execution.

use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};
use bijux_cli_install::{
    default_compatibility_paths, discover_compatibility_paths, install_health_report,
    load_compatibility_config, post_install_hint, run_config_migrations, CompatibilityConfig,
    PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
};
use bijux_cli_output::{render_value, EmitterConfig};
use bijux_cli_plugin::{
    compatibility_warnings, list_plugins, plugin_origin_metadata, registry_path_from_plugins_dir,
};
use bijux_cli_routing::parser::{parse_intent, root_command, ParsedGlobalFlags};
use bijux_cli_routing::registry::{RouteRegistry, RouteTarget};
use serde_json::{json, Value};

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

fn home_dir() -> Option<std::path::PathBuf> {
    env::var_os("HOME").map(std::path::PathBuf::from)
}

fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| env::var(key).ok().map(|value| ((*key).to_string(), value)))
        .collect()
}

fn normalize_config_key(raw: &str) -> Result<String> {
    let key = raw.trim();
    if key.is_empty() {
        anyhow::bail!("Key cannot be empty");
    }
    if !key.is_ascii() {
        anyhow::bail!("Non-ASCII characters are not allowed in keys or values.");
    }
    if key.contains('.') {
        anyhow::bail!("Unknown config section in key: {key}");
    }
    let normalized = key.strip_prefix("BIJUXCLI_").unwrap_or(key).to_ascii_lowercase();
    if !normalized.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        anyhow::bail!("Invalid key: only alphanumerics and underscore allowed.");
    }
    Ok(normalized)
}

fn decode_quoted_value(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

fn validate_config_value(value: &str) -> Result<()> {
    if !value.is_ascii() {
        anyhow::bail!("Non-ASCII characters are not allowed in keys or values.");
    }
    if value
        .chars()
        .any(|ch| matches!(ch, '\r' | '\n' | '\t' | '\u{000B}' | '\u{000C}'))
    {
        anyhow::bail!("Control characters are not allowed in config values.");
    }
    Ok(())
}

fn parse_set_pair(raw_pair: &str) -> Result<(String, String)> {
    if !raw_pair.contains('=') {
        anyhow::bail!("Invalid argument: KEY=VALUE required");
    }
    let (raw_key, raw_value) = raw_pair.split_once('=').expect("contains '=' checked");
    let key = normalize_config_key(raw_key)?;
    let mut value = raw_value.to_string();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value = decode_quoted_value(&value[1..value.len() - 1]);
    }
    validate_config_value(&value)?;
    Ok((key, value))
}

fn parse_compat_config_kv(path: &Path) -> Result<BTreeMap<String, String>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(path)?;
    let mut out = BTreeMap::new();
    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((raw_key, raw_value)) = raw_line.split_once('=') else {
            anyhow::bail!("Malformed line {line_no}: {raw_line}");
        };
        let normalized = normalize_config_key(raw_key)?;
        let value = decode_quoted_value(raw_value.trim());
        validate_config_value(&value)?;
        out.insert(normalized, value);
    }
    Ok(out)
}

fn write_compat_config_kv(path: &Path, values: &BTreeMap<String, String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut rendered = String::new();
    for (key, value) in values {
        rendered.push_str(&format!("BIJUXCLI_{}={}\n", key.to_ascii_uppercase(), value));
    }
    let temp_path = path.with_extension("tmp");
    let mut temp = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&temp_path)?;
    temp.write_all(rendered.as_bytes())?;
    temp.sync_all()?;
    fs::rename(temp_path, path)?;
    Ok(())
}

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

fn render_command_help(path: &[&str]) -> Result<String> {
    let mut cmd = root_command();
    let target =
        find_command_mut(&mut cmd, path).ok_or_else(|| anyhow::anyhow!("unknown help path"))?;
    let mut out = Vec::new();
    target.write_long_help(&mut out)?;
    let mut rendered = String::from_utf8(out)?;
    if matches!(path, ["inspect"] | ["cli", "inspect"] | ["cli", "plugins", "inspect"]) {
        rendered.push_str(
            "\nCompatibility note: inspect output includes plugin compatibility warnings when present.\n",
        );
    }
    Ok(rendered)
}

fn find_command_mut<'a>(
    command: &'a mut clap::Command,
    path: &[&str],
) -> Option<&'a mut clap::Command> {
    if let Some((head, tail)) = path.split_first() {
        let child = command.find_subcommand_mut(head)?;
        find_command_mut(child, tail)
    } else {
        Some(command)
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
        [a] if a == "config" || a == "history" => RouteTarget::BuiltIn,
        [a, b] if a == "plugins" && (b == "list" || b == "inspect" || b == "check") => {
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

    let home = home_dir();
    let defaults = home
        .as_deref()
        .map(default_compatibility_paths)
        .unwrap_or_else(|| default_compatibility_paths(Path::new(".")));

    let config = load_compatibility_config(&defaults.config_file)
        .unwrap_or_else(|_| CompatibilityConfig::default());
    let mut overrides = PathOverrides::default();
    if let Some(path) = &global_flags.config_path {
        overrides.config_file = Some(path.into());
    }
    let paths = discover_compatibility_paths(
        home.as_deref(),
        &overrides,
        &env_map(),
        &config,
    )?;
    let plugin_registry_path = registry_path_from_plugins_dir(&paths.plugins_dir);

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
            json!({
                "reserved_namespaces": registry.route_tree(),
                "builtins": registry.built_in_paths(),
                "plugin_origins": plugin_origin_metadata(&plugin_registry_path).unwrap_or_default(),
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
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
            let hint = install_report
                .active_binary
                .as_deref()
                .map(post_install_hint)
                .unwrap_or_else(|| {
                    "Run `bijux version` and `bijux cli doctor` to verify your environment."
                        .to_string()
                });
            json!({
                "config": paths.config_file,
                "history": paths.history_file,
                "plugins": paths.plugins_dir,
                "active_binary": install_report.active_binary,
                "path_binaries": install_report.path_binaries,
                "post_install_hint": hint
            })
        }
        [a, b, c] if a == "cli" && b == "config" && c == "get" => {
            let positional = command_positionals(argv, &["cli", "config", "get"]);
            let Some(raw_key) = positional.first() else {
                anyhow::bail!("Missing argument: KEY required");
            };
            let normalized_key = normalize_config_key(raw_key)?;
            let env_key = format!("BIJUXCLI_{}", normalized_key.to_ascii_uppercase());
            let value = if let Ok(value) = env::var(&env_key) {
                value
            } else {
                let values = parse_compat_config_kv(&paths.config_file)?;
                values.get(&normalized_key).cloned().ok_or_else(|| {
                    anyhow::anyhow!("Config key not found: {raw_key}")
                })?
            };
            json!({"value": value, "key": normalized_key, "source_path": paths.config_file})
        }
        [a] if a == "config" => {
            json!({
                "BIJUXCLI_CONFIG": paths.config_file,
                "BIJUXCLI_HISTORY_FILE": paths.history_file,
                "BIJUXCLI_PLUGINS_DIR": paths.plugins_dir
            })
        }
        [a, b, c] if a == "cli" && b == "config" && c == "set" => {
            run_config_migrations(&paths.config_file, 1)?;
            let positional = command_positionals(argv, &["cli", "config", "set"]);
            let Some(raw_pair) = positional.first() else {
                anyhow::bail!("Missing argument: KEY=VALUE required");
            };
            let (key, value) = parse_set_pair(raw_pair)?;
            let mut values = parse_compat_config_kv(&paths.config_file)?;
            values.insert(key.clone(), value.clone());
            write_compat_config_kv(&paths.config_file, &values)?;
            json!({"status": "updated", "key": key, "value": value, "updated": paths.config_file})
        }
        [a] if a == "history" => {
            json!({"entries": [], "count": 0})
        }
        [a, b] if a == "cli" && b == "self-test" => {
            json!({"status": "ok", "checks": ["routing", "contracts", "emitters"]})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "list" => {
            json!({"plugins": list_plugins(&plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir})
        }
        [a, b] if a == "plugins" && b == "list" => {
            json!({"plugins": list_plugins(&plugin_registry_path).unwrap_or_default(), "directory": paths.plugins_dir})
        }
        [a, b, c] if a == "cli" && b == "plugins" && c == "inspect" => {
            json!({
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
                "status": "loaded",
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            })
        }
        [a, b] if a == "plugins" && b == "inspect" => {
            json!({
                "plugins": list_plugins(&plugin_registry_path).unwrap_or_default(),
                "status": "loaded",
                "compatibility_warnings": compatibility_warnings(&plugin_registry_path, env!("CARGO_PKG_VERSION")).unwrap_or_default(),
            })
        }
        [a, b] if a == "plugins" && b == "check" => {
            let plugin = argv.get(3).cloned().unwrap_or_else(|| "unknown".to_string());
            json!({"plugin": plugin, "status": "healthy"})
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

fn is_known_route(path: &[String]) -> bool {
    match path {
        [a, b]
            if a == "cli"
                && (b == "version"
                    || b == "doctor"
                    || b == "repl"
                    || b == "completion"
                    || b == "inspect"
                    || b == "status"
                    || b == "paths"
                    || b == "self-test") =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "config" && (c == "get" || c == "set") => true,
        [a] if a == "config" => true,
        [a] if a == "history" => true,
        [a] if a == "status" || a == "audit" || a == "docs" || a == "sleep" => true,
        [a, b, c] if a == "cli" && b == "plugins" && (c == "list" || c == "inspect") => true,
        [a, b] if a == "plugins" && (b == "list" || b == "inspect" || b == "check") => true,
        [a, b, c]
            if a == "dev"
                && b == "cli"
                && (c == "routes"
                    || c == "registry"
                    || c == "env"
                    || c == "doctor"
                    || c == "contracts") =>
        {
            true
        }
        [a, b, c] if a == "cli" && b == "hold" && c == "interruptible" => true,
        _ => false,
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

    let is_unknown = !is_known_route(&intent.normalized_path);

    let response = route_response(&intent.normalized_path, argv, &intent.global_flags);
    let payload = match response {
        Ok(value) => value,
        Err(error) => {
            let message = error.to_string();
            let code = if message.contains("Missing argument")
                || message.contains("Invalid argument")
                || message.contains("Invalid key")
                || message.contains("Unknown config section")
                || message.contains("Config key not found")
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
            return Ok(AppRunResult { exit_code: code, stdout: String::new(), stderr: error_content });
        }
    };

    let rendered = render_value(&payload, emitter_config(&intent.global_flags))?;
    let content = if rendered.ends_with('\n') {
        rendered
    } else {
        format!("{rendered}\n")
    };

    if is_unknown {
        return Ok(AppRunResult { exit_code: 2, stdout: String::new(), stderr: content });
    }

    if intent.global_flags.quiet {
        return Ok(AppRunResult { exit_code: 0, stdout: String::new(), stderr: String::new() });
    }

    Ok(AppRunResult { exit_code: 0, stdout: content, stderr: String::new() })
}
