//! Clap-based parser and normalized command intent model.

use clap::{Arg, ArgAction, ArgMatches, Command};

use super::catalog::normalize_command_path;
use crate::contracts::{
    ColorMode, LogLevel, OutputFormat, PrettyMode, KNOWN_BIJUX_TOOL_NAMESPACES,
};

/// Parsed and normalized global options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedGlobalFlags {
    /// Optional output format override.
    pub output_format: Option<OutputFormat>,
    /// Optional pretty mode override.
    pub pretty_mode: Option<PrettyMode>,
    /// Optional color mode override.
    pub color_mode: Option<ColorMode>,
    /// Optional log-level override.
    pub log_level: Option<LogLevel>,
    /// Quiet mode.
    pub quiet: bool,
    /// Optional explicit config file path override.
    pub config_path: Option<String>,
}

/// Intent model normalized from clap matches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedIntent {
    /// Original path extracted from command tokens.
    pub command_path: Vec<String>,
    /// Normalized path after alias rewriting.
    pub normalized_path: Vec<String>,
    /// Parsed global flags regardless of placement.
    pub global_flags: ParsedGlobalFlags,
}

/// Errors emitted by parser normalization.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ParseError {
    /// Unknown format value.
    #[error("invalid format: {0}")]
    InvalidFormat(String),
    /// Unknown color mode value.
    #[error("invalid color mode: {0}")]
    InvalidColor(String),
    /// Unknown log level value.
    #[error("invalid log level: {0}")]
    InvalidLogLevel(String),
}

fn parse_output_format(raw: Option<&String>) -> Result<Option<OutputFormat>, ParseError> {
    raw.map(|v| match v.as_str() {
        "json" => Ok(OutputFormat::Json),
        "yaml" => Ok(OutputFormat::Yaml),
        "text" => Ok(OutputFormat::Text),
        other => Err(ParseError::InvalidFormat(other.to_string())),
    })
    .transpose()
}

fn parse_color(raw: Option<&String>) -> Result<Option<ColorMode>, ParseError> {
    raw.map(|v| match v.as_str() {
        "auto" => Ok(ColorMode::Auto),
        "always" => Ok(ColorMode::Always),
        "never" => Ok(ColorMode::Never),
        other => Err(ParseError::InvalidColor(other.to_string())),
    })
    .transpose()
}

fn parse_log_level(raw: Option<&String>) -> Result<Option<LogLevel>, ParseError> {
    raw.map(|v| match v.as_str() {
        "trace" => Ok(LogLevel::Trace),
        "debug" => Ok(LogLevel::Debug),
        "info" => Ok(LogLevel::Info),
        "warning" => Ok(LogLevel::Warning),
        "error" => Ok(LogLevel::Error),
        "critical" => Ok(LogLevel::Critical),
        other => Err(ParseError::InvalidLogLevel(other.to_string())),
    })
    .transpose()
}

fn is_global_flag_without_value(token: &str) -> bool {
    matches!(token, "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text")
}

fn is_global_flag_with_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn is_global_flag_with_equals(token: &str) -> bool {
    token.starts_with("--format=")
        || token.starts_with("--log-level=")
        || token.starts_with("--color=")
        || token.starts_with("--config-path=")
}

fn parse_argv_with_global_flags_front(argv: &[String]) -> Vec<String> {
    if argv.is_empty() {
        return Vec::new();
    }

    let mut globals = Vec::new();
    let mut command_tail = Vec::new();
    let mut idx = 1;

    while idx < argv.len() {
        let token = argv[idx].as_str();
        if token == "--" {
            command_tail.extend(argv.iter().skip(idx).cloned());
            break;
        }
        if is_global_flag_without_value(token) || is_global_flag_with_equals(token) {
            globals.push(argv[idx].clone());
            idx += 1;
            continue;
        }
        if is_global_flag_with_value(token) {
            globals.push(argv[idx].clone());
            if let Some(value) = argv.get(idx + 1) {
                globals.push(value.clone());
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }

        command_tail.push(argv[idx].clone());
        idx += 1;
    }

    let mut normalized = Vec::with_capacity(1 + globals.len() + command_tail.len());
    normalized.push(argv[0].clone());
    normalized.extend(globals);
    normalized.extend(command_tail);
    normalized
}

fn global_flags_from_matches(matches: &ArgMatches) -> Result<ParsedGlobalFlags, ParseError> {
    let output_format = if matches.get_flag("json") {
        Some(OutputFormat::Json)
    } else if matches.get_flag("text") {
        Some(OutputFormat::Text)
    } else {
        parse_output_format(matches.get_one::<String>("format"))?
    };
    let color_mode = parse_color(matches.get_one::<String>("color"))?;
    let log_level = parse_log_level(matches.get_one::<String>("log-level"))?;

    let pretty_mode = if matches.get_flag("pretty") {
        Some(PrettyMode::Pretty)
    } else if matches.get_flag("no-pretty") {
        Some(PrettyMode::Compact)
    } else {
        None
    };

    Ok(ParsedGlobalFlags {
        output_format,
        pretty_mode,
        color_mode,
        log_level,
        quiet: matches.get_flag("quiet"),
        config_path: matches.get_one::<String>("config-path").cloned(),
    })
}

fn delegated_command(name: &'static str, hidden: bool) -> Command {
    Command::new(name).hide(hidden).allow_external_subcommands(true)
}

fn build_dev_group() -> Command {
    let mut dev_cli = Command::new("cli").allow_external_subcommands(true);
    for command in crate::routing::model::DEV_CLI_VISIBLE_SUBCOMMANDS {
        let entry = if crate::routing::model::DEV_CLI_NESTED_SUBCOMMANDS.contains(command) {
            delegated_command(command, false)
        } else {
            Command::new(*command)
        };
        dev_cli = dev_cli.subcommand(entry);
    }
    for command in crate::routing::model::DEV_CLI_HIDDEN_SUBCOMMANDS {
        dev_cli = dev_cli.subcommand(delegated_command(command, true));
    }

    let mut command =
        Command::new("dev").hide(true).allow_external_subcommands(true).subcommand(dev_cli);

    for namespace in KNOWN_BIJUX_TOOL_NAMESPACES {
        command = command.subcommand(delegated_command(namespace, true));
    }

    command
}

/// Build the root clap command for `bijux`.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn root_command() -> Command {
    let format_arg = Arg::new("format")
        .long("format")
        .short('f')
        .num_args(1)
        .global(true)
        .value_name("FORMAT")
        .help("Output format: text, json, or yaml");

    let quiet_arg = Arg::new("quiet")
        .long("quiet")
        .short('q')
        .action(ArgAction::SetTrue)
        .global(true)
        .help("Suppress command output");

    let log_level_arg = Arg::new("log-level")
        .long("log-level")
        .num_args(1)
        .global(true)
        .value_name("LEVEL")
        .help("Log verbosity level");

    let color_arg = Arg::new("color")
        .long("color")
        .num_args(1)
        .global(true)
        .value_name("MODE")
        .help("ANSI color policy");

    let pretty_arg = Arg::new("pretty")
        .long("pretty")
        .action(ArgAction::SetTrue)
        .overrides_with("no-pretty")
        .global(true)
        .help("Pretty-print structured output");

    let no_pretty_arg = Arg::new("no-pretty")
        .long("no-pretty")
        .action(ArgAction::SetTrue)
        .overrides_with("pretty")
        .global(true)
        .help("Emit compact structured output");
    let config_path_arg = Arg::new("config-path")
        .long("config-path")
        .num_args(1)
        .global(true)
        .value_name("PATH")
        .help("Use explicit config file path");
    let json_arg = Arg::new("json")
        .long("json")
        .action(ArgAction::SetTrue)
        .overrides_with_all(["text", "format"])
        .hide(true)
        .global(true);
    let text_arg = Arg::new("text")
        .long("text")
        .action(ArgAction::SetTrue)
        .overrides_with_all(["json", "format"])
        .hide(true)
        .global(true);

    let config_group = Command::new("config")
        .subcommand_required(false)
        .subcommand(Command::new("list"))
        .subcommand(Command::new("get").arg(Arg::new("key").num_args(1)))
        .subcommand(Command::new("set").arg(Arg::new("pair").num_args(1)))
        .subcommand(Command::new("unset").arg(Arg::new("key").num_args(1)))
        .subcommand(Command::new("clear"))
        .subcommand(Command::new("reload"))
        .subcommand(Command::new("export").arg(Arg::new("path").num_args(1)))
        .subcommand(Command::new("load").arg(Arg::new("path").num_args(1)));

    let plugins_group = Command::new("plugins")
        .subcommand(Command::new("list"))
        .subcommand(Command::new("info"))
        .subcommand(Command::new("inspect"))
        .subcommand(Command::new("check").arg(Arg::new("plugin").num_args(1)))
        .subcommand(Command::new("enable").arg(Arg::new("plugin").num_args(1)))
        .subcommand(Command::new("disable").arg(Arg::new("plugin").num_args(1)))
        .subcommand(
            Command::new("install")
                .arg(Arg::new("manifest").num_args(1))
                .arg(Arg::new("source").long("source").num_args(1))
                .arg(Arg::new("trust").long("trust").num_args(1)),
        )
        .subcommand(Command::new("uninstall").arg(Arg::new("namespace").num_args(1)))
        .subcommand(
            Command::new("scaffold")
                .arg(Arg::new("kind").num_args(1))
                .arg(Arg::new("namespace").num_args(1))
                .arg(Arg::new("path").long("path").num_args(1))
                .arg(Arg::new("force").long("force")),
        )
        .subcommand(Command::new("doctor"))
        .subcommand(Command::new("reserved-names"))
        .subcommand(Command::new("where"))
        .subcommand(Command::new("explain").arg(Arg::new("plugin").num_args(1)))
        .subcommand(Command::new("schema"));

    let cli_group = Command::new("cli")
        .subcommand(Command::new("status"))
        .subcommand(Command::new("paths"))
        .subcommand(Command::new("inspect").hide(true))
        .subcommand(config_group.clone())
        .subcommand(Command::new("self-test"))
        .subcommand(
            Command::new("hold").hide(true).subcommand(Command::new("interruptible").hide(true)),
        )
        .subcommand(plugins_group.clone());

    Command::new("bijux")
        .args([
            format_arg,
            quiet_arg,
            log_level_arg,
            color_arg,
            pretty_arg,
            no_pretty_arg,
            config_path_arg,
            json_arg,
            text_arg,
        ])
        .subcommand_required(false)
        .allow_external_subcommands(true)
        .subcommand(cli_group)
        .subcommand(build_dev_group())
        // Legacy roots kept for alias normalization.
        .subcommand(Command::new("status"))
        .subcommand(Command::new("audit"))
        .subcommand(Command::new("docs"))
        .subcommand(Command::new("sleep").arg(Arg::new("duration").num_args(1)))
        .subcommand(Command::new("doctor"))
        .subcommand(Command::new("version"))
        .subcommand(config_group)
        .subcommand(plugins_group)
        .subcommand(Command::new("repl"))
        .subcommand(Command::new("completion"))
        .subcommand(Command::new("inspect").hide(true))
        .subcommand(
            Command::new("history")
                .subcommand(Command::new("clear"))
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .short('l')
                        .num_args(1)
                        .value_parser(clap::value_parser!(usize)),
                )
                .arg(Arg::new("filter").long("filter").short('F').num_args(1))
                .arg(Arg::new("sort").long("sort").num_args(1).value_parser(["timestamp"])),
        )
        .subcommand(
            Command::new("memory")
                .subcommand(Command::new("list"))
                .subcommand(Command::new("get").arg(Arg::new("key").num_args(1)))
                .subcommand(Command::new("set").arg(Arg::new("pair").num_args(1)))
                .subcommand(Command::new("delete").arg(Arg::new("key").num_args(1)))
                .subcommand(Command::new("clear")),
        )
}

fn extract_path(matches: &ArgMatches) -> Vec<String> {
    let mut out = Vec::<String>::new();
    let mut curr = matches;

    while let Some((name, next)) = curr.subcommand() {
        out.push(name.to_string());
        curr = next;
    }

    out
}

/// Parse argv and normalize global flags + command path.
pub fn parse_intent(argv: &[String]) -> Result<ParsedIntent, ParseError> {
    let Ok(raw_matches) = root_command().try_get_matches_from(argv) else {
        // Keep parser deterministic for routing tests by returning empty intent on clap usage failures.
        return Ok(ParsedIntent {
            command_path: Vec::new(),
            normalized_path: Vec::new(),
            global_flags: ParsedGlobalFlags {
                output_format: None,
                pretty_mode: None,
                color_mode: None,
                log_level: None,
                quiet: false,
                config_path: None,
            },
        });
    };

    let command_path = extract_path(&raw_matches);
    let normalize_external_globals = matches!(
        command_path.as_slice(),
        [a, b, ..] if a == "dev" && (b == "cli" || KNOWN_BIJUX_TOOL_NAMESPACES.contains(&b.as_str()))
    ) || matches!(
        command_path.as_slice(),
        [a, ..] if KNOWN_BIJUX_TOOL_NAMESPACES.contains(&a.as_str())
    );

    let global_flags = if normalize_external_globals {
        let parse_argv = parse_argv_with_global_flags_front(argv);
        let Ok(reparsed) = root_command().try_get_matches_from(&parse_argv) else {
            return Ok(ParsedIntent {
                command_path: Vec::new(),
                normalized_path: Vec::new(),
                global_flags: ParsedGlobalFlags {
                    output_format: None,
                    pretty_mode: None,
                    color_mode: None,
                    log_level: None,
                    quiet: false,
                    config_path: None,
                },
            });
        };
        global_flags_from_matches(&reparsed)?
    } else {
        global_flags_from_matches(&raw_matches)?
    };

    let normalized_path = normalize_command_path(&command_path);

    Ok(ParsedIntent { command_path, normalized_path, global_flags })
}

#[cfg(test)]
mod tests {
    use super::root_command;

    #[test]
    fn dev_cli_help_lists_registered_subcommands() {
        let argv =
            vec!["bijux".to_string(), "dev".to_string(), "cli".to_string(), "--help".to_string()];
        let help = match root_command().try_get_matches_from(argv) {
            Err(error) if matches!(error.kind(), clap::error::ErrorKind::DisplayHelp) => {
                error.to_string()
            }
            other => panic!("expected clap help output, got {other:?}"),
        };

        assert!(help.contains("Commands:"));
        assert!(help.contains("maintenance"));
        assert!(help.contains("runtime-identity"));
    }
}
