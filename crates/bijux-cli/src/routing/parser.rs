//! Clap-based parser and normalized command intent model.

use clap::{Arg, ArgAction, ArgMatches, Command};

use super::catalog::normalize_command_path;
use super::model::{DEV_CLI_SUBCOMMANDS, DEV_LEGACY_ALIASES};
use crate::contracts::{ColorMode, LogLevel, OutputFormat, PrettyMode};

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

fn with_hidden_leaf_subcommands(mut command: Command, names: &[&'static str]) -> Command {
    for name in names {
        command = command.subcommand(Command::new(*name).hide(true));
    }
    command
}

fn with_dev_cli_surface_subcommands(mut command: Command) -> Command {
    for subcommand in DEV_CLI_SUBCOMMANDS {
        if matches!(
            *subcommand,
            "maintenance"
                | "rustdoc"
                | "release"
                | "evidence"
                | "config"
                | "python"
                | "repo"
                | "contracts"
        ) {
            continue;
        }

        let item = if matches!(*subcommand, "atlas" | "di" | "list-products" | "list-plugins") {
            Command::new(*subcommand).hide(true)
        } else {
            Command::new(*subcommand)
        };
        command = command.subcommand(item);
    }

    command
}

/// Build the root clap command for `bijux`.
#[must_use]
#[allow(clippy::too_many_lines)]
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
        .subcommand(config_group.clone())
        .subcommand(Command::new("self-test"))
        .subcommand(
            Command::new("hold").hide(true).subcommand(Command::new("interruptible").hide(true)),
        )
        .subcommand(plugins_group.clone());

    let dev_cli_group =
        with_dev_cli_surface_subcommands(
            Command::new("cli")
                .subcommand(
                    Command::new("maintenance")
                        .subcommand(Command::new("remaining"))
                        .subcommand(Command::new("migrated"))
                        .subcommand(Command::new("diff"))
                        .subcommand(Command::new("audit"))
                        .subcommand(Command::new("generators"))
                        .subcommand(
                            Command::new("generate")
                                .arg(Arg::new("id").long("id").num_args(1))
                                .arg(Arg::new("source").long("source").num_args(1)),
                        )
                        .subcommand(Command::new("generate-all"))
                        .subcommand(Command::new("requirements"))
                        .subcommand(Command::new("flaky-tests"))
                        .subcommand(
                            Command::new("status")
                                .subcommand(Command::new("inventory"))
                                .subcommand(
                                    Command::new("run")
                                        .arg(Arg::new("id").long("id").num_args(1))
                                        .arg(Arg::new("source").long("source").num_args(1))
                                        .arg(
                                            Arg::new("args")
                                                .num_args(0..)
                                                .trailing_var_arg(true)
                                                .allow_hyphen_values(true),
                                        ),
                                )
                                .subcommand(
                                    Command::new("run-all")
                                        .arg(Arg::new("kind").long("kind").num_args(1))
                                        .arg(
                                            Arg::new("args")
                                                .num_args(0..)
                                                .trailing_var_arg(true)
                                                .allow_hyphen_values(true),
                                        ),
                                ),
                        )
                        .subcommand(Command::new("package-metadata"))
                        .subcommand(Command::new("e2e-contract"))
                        .subcommand(
                            Command::new("pip-audit")
                                .arg(Arg::new("report-path").long("report-path").num_args(1)),
                        )
                        .subcommand(Command::new("capture-python-behavior"))
                        .subcommand(
                            Command::new("provenance-statement")
                                .arg(Arg::new("tag").long("tag").num_args(1).required(true))
                                .arg(
                                    Arg::new("output-dir")
                                        .long("output-dir")
                                        .num_args(1)
                                        .required(true),
                                ),
                        ),
                )
                .subcommand(
                    Command::new("rustdoc")
                        .subcommand(Command::new("audit"))
                        .subcommand(Command::new("coverage"))
                        .subcommand(Command::new("broken-links"))
                        .subcommand(Command::new("public-api"))
                        .subcommand(Command::new("examples"))
                        .subcommand(Command::new("migrate-website-api-docs"))
                        .subcommand(Command::new("build-proof"))
                        .subcommand(Command::new("workspace-coverage-proof"))
                        .subcommand(Command::new("python-link-proof")),
                )
                .subcommand(
                    Command::new("release")
                        .subcommand(Command::new("status"))
                        .subcommand(Command::new("evidence"))
                        .subcommand(Command::new("readiness"))
                        .subcommand(Command::new("diff"))
                        .subcommand(Command::new("gaps"))
                        .subcommand(Command::new("summary"))
                        .subcommand(Command::new("manifest"))
                        .subcommand(Command::new("notes"))
                        .subcommand(Command::new("behavior-changes"))
                        .subcommand(Command::new("intentional-differences"))
                        .subcommand(Command::new("unresolved-gaps"))
                        .subcommand(Command::new("compatibility-leftovers")),
                )
                .subcommand(
                    Command::new("evidence")
                        .subcommand(Command::new("list"))
                        .subcommand(Command::new("show").arg(Arg::new("id").long("id").num_args(1)))
                        .subcommand(Command::new("audit"))
                        .subcommand(Command::new("stale"))
                        .subcommand(Command::new("matrix"))
                        .subcommand(Command::new("website-export"))
                        .subcommand(Command::new("ci-export"))
                        .subcommand(Command::new("release-export"))
                        .subcommand(Command::new("command-map"))
                        .subcommand(Command::new("parity-map")),
                )
                .subcommand(
                    Command::new("config")
                        .subcommand(Command::new("rust-owner"))
                        .subcommand(Command::new("python-owner"))
                        .subcommand(Command::new("ownership"))
                        .subcommand(Command::new("drift"))
                        .subcommand(Command::new("shape"))
                        .subcommand(Command::new("evidence-map")),
                )
                .subcommand(
                    Command::new("python")
                        .subcommand(Command::new("bridge-status"))
                        .subcommand(Command::new("surface-status"))
                        .subcommand(Command::new("sovereignty-audit"))
                        .subcommand(Command::new("drift"))
                        .subcommand(Command::new("packaging")),
                )
                .subcommand(
                    Command::new("repo")
                        .subcommand(Command::new("health"))
                        .subcommand(Command::new("drift"))
                        .subcommand(Command::new("inventories"))
                        .subcommand(Command::new("generated"))
                        .subcommand(Command::new("stale")),
                )
                .subcommand(
                    Command::new("contracts")
                        .arg(Arg::new("all").long("all").action(ArgAction::SetTrue))
                        .arg(Arg::new("kind").long("kind").num_args(1).value_parser([
                            "generate", "check", "enforce", "warn", "run", "status",
                        ])),
                ),
        );

    // Legacy path support normalized later to `dev cli ...`.
    let dev_group = with_hidden_leaf_subcommands(
        Command::new("dev").subcommand(dev_cli_group.clone()),
        DEV_LEGACY_ALIASES,
    );

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
        .subcommand(Command::new("atlas"))
        .subcommand(
            Command::new("history")
                .subcommand(Command::new("clear"))
                .arg(Arg::new("limit").long("limit").short('l').num_args(1))
                .arg(Arg::new("filter").long("filter").short('F').num_args(1))
                .arg(Arg::new("sort").long("sort").num_args(1)),
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
    let Ok(matches) = root_command().try_get_matches_from(argv) else {
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

    let command_path = extract_path(&matches);
    let normalized_path = normalize_command_path(&command_path);
    let global_flags = global_flags_from_matches(&matches)?;

    Ok(ParsedIntent { command_path, normalized_path, global_flags })
}
