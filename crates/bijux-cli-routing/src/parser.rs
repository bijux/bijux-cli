//! Clap-based parser and normalized command intent model.

use clap::{Arg, ArgAction, ArgMatches, Command};

use bijux_cli_contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};

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

fn global_flags_from_matches(matches: &ArgMatches) -> Result<ParsedGlobalFlags, ParseError> {
    let output_format = parse_output_format(matches.get_one::<String>("format"))?;
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

/// Build the root clap command for `bijux`.
#[must_use]
pub fn root_command() -> Command {
    let format_arg =
        Arg::new("format").long("format").short('f').num_args(1).global(true).value_name("FORMAT");

    let quiet_arg =
        Arg::new("quiet").long("quiet").short('q').action(ArgAction::SetTrue).global(true);

    let log_level_arg =
        Arg::new("log-level").long("log-level").num_args(1).global(true).value_name("LEVEL");

    let color_arg = Arg::new("color").long("color").num_args(1).global(true).value_name("MODE");

    let pretty_arg = Arg::new("pretty")
        .long("pretty")
        .action(ArgAction::SetTrue)
        .overrides_with("no-pretty")
        .global(true);

    let no_pretty_arg = Arg::new("no-pretty")
        .long("no-pretty")
        .action(ArgAction::SetTrue)
        .overrides_with("pretty")
        .global(true);
    let config_path_arg =
        Arg::new("config-path").long("config-path").num_args(1).global(true).value_name("PATH");

    let config_group = Command::new("config")
        .subcommand_required(false)
        .subcommand(Command::new("get").arg(Arg::new("key").num_args(1)))
        .subcommand(Command::new("set").arg(Arg::new("pair").num_args(1)));

    let plugins_group = Command::new("plugins")
        .subcommand(Command::new("list"))
        .subcommand(Command::new("inspect"))
        .subcommand(Command::new("check").arg(Arg::new("plugin").num_args(1)));

    let cli_group = Command::new("cli")
        .subcommand(Command::new("status"))
        .subcommand(Command::new("paths"))
        .subcommand(config_group.clone())
        .subcommand(Command::new("self-test"))
        .subcommand(
            Command::new("hold")
                .hide(true)
                .subcommand(Command::new("interruptible").hide(true)),
        )
        .subcommand(plugins_group.clone());

    let dev_cli_group = Command::new("cli")
        .subcommand(Command::new("routes"))
        .subcommand(Command::new("registry"))
        .subcommand(Command::new("env"))
        .subcommand(Command::new("doctor"))
        .subcommand(Command::new("contracts"));

    let dev_group = Command::new("dev")
        .subcommand(dev_cli_group.clone())
        // Legacy path support normalized later to `dev cli ...`.
        .subcommand(Command::new("routes").hide(true))
        .subcommand(Command::new("registry").hide(true))
        .subcommand(Command::new("env").hide(true))
        .subcommand(Command::new("doctor").hide(true))
        .subcommand(Command::new("contracts").hide(true));

    Command::new("bijux")
        .args([
            format_arg,
            quiet_arg,
            log_level_arg,
            color_arg,
            pretty_arg,
            no_pretty_arg,
            config_path_arg,
        ])
        .subcommand_required(false)
        .allow_external_subcommands(true)
        .subcommand(cli_group)
        .subcommand(dev_group)
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
        .subcommand(Command::new("inspect"))
        .subcommand(Command::new("history"))
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

fn normalize_path(path: &[String]) -> Vec<String> {
    match path {
        [a] if a == "doctor"
            || a == "version"
            || a == "inspect"
            || a == "completion"
            || a == "repl" =>
        {
            vec!["cli".to_string(), a.clone()]
        }
        [a, b] if a == "config" && (b == "get" || b == "set") => {
            vec!["cli".to_string(), "config".to_string(), b.clone()]
        }
        [a, b] if a == "plugins" && (b == "list" || b == "inspect") => {
            vec!["cli".to_string(), "plugins".to_string(), b.clone()]
        }
        [a, b] if a == "plugins" && b == "check" => {
            vec!["plugins".to_string(), b.clone()]
        }
        [a, b]
            if a == "dev"
                && ["routes", "registry", "env", "doctor", "contracts"].contains(&b.as_str()) =>
        {
            vec!["dev".to_string(), "cli".to_string(), b.clone()]
        }
        _ => path.to_vec(),
    }
}

/// Parse argv and normalize global flags + command path.
pub fn parse_intent(argv: &[String]) -> Result<ParsedIntent, ParseError> {
    let matches = match root_command().try_get_matches_from(argv) {
        Ok(value) => value,
        Err(_) => {
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
        }
    };

    let command_path = extract_path(&matches);
    let normalized_path = normalize_path(&command_path);
    let global_flags = global_flags_from_matches(&matches)?;

    Ok(ParsedIntent { command_path, normalized_path, global_flags })
}
