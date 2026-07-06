//! Help rendering for command paths.

use anyhow::Result;

use crate::contracts::{known_bijux_tool_by_query, known_bijux_tools};
use crate::routing::model::{
    CLI_CONFIG_SUBCOMMANDS, CLI_PLUGINS_SUBCOMMANDS, ROOT_APPS_SUBCOMMANDS,
    ROOT_INTERACTION_COMMANDS, ROOT_RUNTIME_COMMANDS, ROOT_STATE_COMMANDS,
};
use crate::routing::parser::root_command;

pub(crate) fn normalize_help_whitespace(raw: &str) -> String {
    let mut normalized = String::new();
    let mut previous_blank = false;
    let mut in_options_section = false;

    for line in raw.lines() {
        let trimmed = line.trim_end();
        let blank_line = trimmed.trim().is_empty();
        let section_header = !trimmed.starts_with(' ') && trimmed.ends_with(':');
        if trimmed == "Options:" {
            in_options_section = true;
        } else if in_options_section && section_header {
            in_options_section = false;
        }

        if blank_line {
            if in_options_section {
                continue;
            }
            if previous_blank {
                continue;
            }
            previous_blank = true;
            normalized.push('\n');
            continue;
        }

        previous_blank = false;
        normalized.push_str(trimmed);
        normalized.push('\n');
    }

    normalized
}

pub(crate) fn render_command_help(path: &[&str]) -> Result<String> {
    let mut argv = vec!["bijux".to_string()];
    argv.extend(path.iter().map(|segment| (*segment).to_string()));
    argv.push("--help".to_string());

    let rendered = match root_command().try_get_matches_from(argv) {
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            error.to_string()
        }
        Ok(_) => return Err(anyhow::anyhow!("unknown help path")),
        Err(_) => return Err(anyhow::anyhow!("unknown help path")),
    };

    let normalized = normalize_help_whitespace(&rendered);
    Ok(decorate_help_text(normalized, path))
}

pub(crate) fn render_known_tool_help(path: &[&str]) -> Option<String> {
    let (control_plane, query) = match path {
        [namespace] => (false, *namespace),
        ["dev", namespace] => (true, *namespace),
        _ => return None,
    };
    let tool = known_bijux_tool_by_query(query)?;
    let aliases = if tool.aliases.is_empty() {
        String::new()
    } else {
        format!("\nAliases:\n  {}\n", tool.aliases.join(", "))
    };
    let capabilities = if tool.capabilities.is_empty() {
        "  (none)\n".to_string()
    } else {
        format!("  {}\n", tool.capabilities.join(", "))
    };

    let (root_route, product_binary, install_note, examples) = if control_plane {
        (
            format!("bijux dev {} <command> ...", tool.namespace),
            tool.control_binary_name,
            format!("cargo install {}", tool.control_package_name),
            vec![
                format!("bijux dev {} status", tool.namespace),
                format!("{} --help", tool.control_binary_name),
            ],
        )
    } else {
        (
            format!("bijux {} <command> ...", tool.namespace),
            tool.runtime_binary_name,
            format!("cargo install {}", tool.runtime_package_name),
            vec![
                format!("bijux apps which {}", tool.namespace),
                format!("{} --help", tool.runtime_binary_name),
            ],
        )
    };

    let mut rendered = format!(
        "Official app help: {}\n\nSummary:\n  {}\n{}\nCapabilities:\n{}Routing:\n  root route: {}\n  product binary: {}\n\nInstall:\n  {}\n\nExamples:\n",
        tool.display_name, tool.help_summary, aliases, capabilities, root_route, product_binary, install_note
    );
    for example in examples {
        rendered.push_str("  ");
        rendered.push_str(&example);
        rendered.push('\n');
    }
    if !control_plane {
        rendered.push_str(&format!(
            "\nUse `{product_binary} --help` for the public command surface.\n"
        ));
    }
    Some(rendered)
}

pub(crate) fn decorate_help_text(mut rendered: String, path: &[&str]) -> String {
    append_help_sections(&mut rendered, path);
    if path == ["history"] {
        rendered.push_str(
            "\nHistory integrity:\n  malformed JSON payloads fail closed\n  use `bijux history clear --force` to reset corrupted history files\n",
        );
    }
    if path == ["version"] {
        rendered.push_str(
            "\nVersion output:\n  text: one-line summary (`bijux version <value>`)\n  json/yaml: includes semver, source, commit, and build profile fields\n",
        );
    }
    if matches!(path, ["plugins", "inspect"] | ["cli", "plugins", "inspect"]) {
        rendered.push_str(
            "\nCompatibility note: inspect output includes plugin compatibility warnings when present.\n",
        );
    }
    rendered
}

fn append_help_sections(rendered: &mut String, path: &[&str]) {
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }

    let mut sections = Vec::new();

    if let Some(grouped) = help_grouped_guide(path) {
        sections.push(grouped);
    }
    if path.is_empty() {
        sections.push(render_official_apps_section());
        sections.push(render_installed_plugins_section());
        sections.push(render_diagnostics_section());
    }
    if let Some(subcommands) = help_subcommand_guide(path) {
        sections.push(subcommands);
    }

    let examples = help_examples(path);
    if !examples.is_empty() {
        sections.push(render_examples(&examples));
    }

    if sections.is_empty() {
        return;
    }

    rendered.push('\n');
    rendered.push_str(&sections.join("\n\n"));
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
}

fn format_command_group_row(name: &str, commands: &[&str]) -> String {
    format!("{name:<12} {}", commands.join(", "))
}

fn render_official_apps_section() -> String {
    let mut out = String::from("Official apps:\n");
    for tool in known_bijux_tools() {
        let aliases = if tool.aliases.is_empty() {
            String::new()
        } else {
            format!(" ({})", tool.aliases.join(", "))
        };
        out.push_str(&format!("  {:<12} {}{}\n", tool.namespace, tool.help_summary, aliases));
    }
    out.trim_end().to_string()
}

fn render_installed_plugins_section() -> String {
    "Installed plugins:\n  Use `bijux plugins list` to inspect the current plugin inventory."
        .to_string()
}

fn render_diagnostics_section() -> String {
    "Diagnostics:\n  Use `bijux doctor` for runtime health and `bijux apps doctor` for app mount health."
        .to_string()
}

fn help_grouped_guide(path: &[&str]) -> Option<String> {
    match path {
        [] => Some(format!(
            "Management Commands:\n\
  {}\n\
  apps         apps\n\
  config       config\n\
  plugins      plugins\n\
  {}\n\
  {}\n\
\n\
Use `bijux help <command>` for command-specific help.",
            format_command_group_row("runtime", ROOT_RUNTIME_COMMANDS),
            format_command_group_row("state", ROOT_STATE_COMMANDS),
            format_command_group_row("interaction", ROOT_INTERACTION_COMMANDS),
        )),
        _ => None,
    }
}

fn render_subcommand_guide(rows: &[String]) -> String {
    let mut out = String::from("Subcommand guide:\n");
    for row in rows {
        out.push_str("  ");
        out.push_str(row);
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn config_subcommand_help(command: &str) -> &'static str {
    match command {
        "list" => "Print all key/value pairs",
        "get" => "Read one key",
        "set" => "Write one key=value pair",
        "unset" => "Remove one key",
        "clear" => "Remove all keys",
        "reload" => "Validate and reload current file",
        "validate" => "Validate layered config and project discovery",
        "schema" => "Show the config schema registry",
        "docs" => "Generate markdown config reference from the schema",
        "explain" => "Explain the effective value and source of one key",
        "diff" => "Compare effective config values between contexts",
        "repair" => "Recover a malformed config file and write a backup",
        "export" => "Write config to a target path",
        "load" => "Load config from a source path",
        _ => "Configuration command",
    }
}

fn plugin_subcommand_help(command: &str) -> &'static str {
    match command {
        "list" => "List discovered plugins",
        "info" => "Show plugin inventory details",
        "inspect" => "Inspect one plugin or the full inventory",
        "check" => "Validate a plugin namespace",
        "enable" => "Enable a plugin namespace",
        "disable" => "Disable a plugin namespace",
        "install" => "Install plugin from manifest/source",
        "uninstall" => "Remove a plugin namespace",
        "scaffold" => "Generate a plugin template",
        "doctor" => "Plugin health diagnostics",
        "reserved-names" => "Show reserved plugin namespace rules",
        "where" => "Show plugin state and install paths",
        "explain" => "Explain plugin resolution outcome",
        "schema" => "Show the current plugin manifest JSON schema",
        _ => "Plugin command",
    }
}

fn help_subcommand_guide(path: &[&str]) -> Option<String> {
    match path {
        ["cli"] => Some(render_subcommand_guide(&[
            "status     Runtime status summary".to_string(),
            "paths      Runtime state and filesystem paths".to_string(),
            "routes     Machine-readable route and alias inventory".to_string(),
            "shims      Legacy shim lifecycle policy and findings".to_string(),
            "script-contract Stable machine-readable automation contract".to_string(),
            "doctor     Runtime environment diagnostics".to_string(),
            "version    Runtime identity and provenance".to_string(),
            "repl       Interactive runtime shell".to_string(),
            "completion Shell completion output".to_string(),
            "config     Runtime configuration operations".to_string(),
            "self-test  Deterministic runtime self-checks".to_string(),
            "plugins    Canonical plugin lifecycle namespace".to_string(),
        ])),
        ["apps"] => {
            let rows = ROOT_APPS_SUBCOMMANDS
                .iter()
                .map(|command| match *command {
                    "list" => {
                        "list         List known official apps and resolution health".to_string()
                    }
                    "doctor" => {
                        "doctor       Diagnose official app mounts and discovery status".to_string()
                    }
                    "which" => {
                        "which        Show the exact resolved runtime entrypoint".to_string()
                    }
                    "version" => {
                        "version      Report manifest or probed runtime version".to_string()
                    }
                    "capabilities" => {
                        "capabilities Show declared app capability contract".to_string()
                    }
                    "schema" => {
                        "schema       Show the app mount descriptor JSON schema".to_string()
                    }
                    "validate-manifest" => {
                        "validate-manifest Validate one app mount manifest on disk".to_string()
                    }
                    "scaffold" => "scaffold     Generate a starter mounted app project".to_string(),
                    other => format!("{other:<12} App command"),
                })
                .collect::<Vec<_>>();
            Some(render_subcommand_guide(&rows))
        }
        ["config"] | ["cli", "config"] => {
            let rows = CLI_CONFIG_SUBCOMMANDS
                .iter()
                .map(|command| format!("{command:<8} {}", config_subcommand_help(command)))
                .collect::<Vec<_>>();
            Some(render_subcommand_guide(&rows))
        }
        ["plugins"] | ["cli", "plugins"] => {
            let rows = CLI_PLUGINS_SUBCOMMANDS
                .iter()
                .map(|command| format!("{command:<14} {}", plugin_subcommand_help(command)))
                .collect::<Vec<_>>();
            Some(render_subcommand_guide(&rows))
        }
        _ => None,
    }
}

fn render_examples(examples: &[String]) -> String {
    let mut out = String::from("Examples:\n");
    for example in examples {
        out.push_str("  ");
        out.push_str(example);
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn help_examples(path: &[&str]) -> Vec<String> {
    match path {
        [] => vec![
            "bijux status".to_string(),
            "bijux explain status".to_string(),
            "bijux apps list".to_string(),
            "bijux-dag --help".to_string(),
            "bijux install atlas --dry-run".to_string(),
            "bijux config get foo".to_string(),
            "bijux config set foo=bar".to_string(),
            "bijux plugins list".to_string(),
        ],
        ["explain"] => vec![
            "bijux explain status".to_string(),
            "bijux explain dag run".to_string(),
            "bijux explain plugins list --format json".to_string(),
        ],
        ["cli"] => vec![
            "bijux cli status".to_string(),
            "bijux cli paths".to_string(),
            "bijux cli routes --format json".to_string(),
            "bijux cli shims --format json".to_string(),
            "bijux cli script-contract --format json".to_string(),
            "bijux cli config list".to_string(),
            "bijux cli plugins list".to_string(),
        ],
        ["status"] => vec!["bijux status".to_string(), "bijux status --format json".to_string()],
        ["audit"] => vec!["bijux audit".to_string(), "bijux audit --format json".to_string()],
        ["docs"] => vec!["bijux docs".to_string(), "bijux docs --format json".to_string()],
        ["install"] => vec![
            "bijux install cli --dry-run".to_string(),
            "bijux install dev-cli --dry-run".to_string(),
            "bijux install atlas --dry-run".to_string(),
        ],
        ["apps"] => vec![
            "bijux apps list".to_string(),
            "bijux apps doctor".to_string(),
            "bijux apps which dag".to_string(),
            "bijux apps capabilities dag".to_string(),
            "bijux apps schema".to_string(),
            "bijux apps validate-manifest ./.bijux/apps/sample.mount.json".to_string(),
            "bijux apps scaffold python sample --path ./sample-app".to_string(),
        ],
        ["doctor"] => vec![
            "bijux doctor".to_string(),
            "bijux doctor --bundle".to_string(),
            "bijux doctor paths".to_string(),
            "bijux doctor python".to_string(),
            "bijux doctor routing --format json".to_string(),
            "bijux doctor dag".to_string(),
            "bijux doctor shims".to_string(),
        ],
        ["version"] => vec!["bijux version".to_string(), "bijux --version".to_string()],
        ["config"] => vec![
            "bijux config list".to_string(),
            "bijux config get foo".to_string(),
            "bijux config set foo=bar".to_string(),
            "bijux config diff --from-profile dev --to-profile prod".to_string(),
            "bijux config docs cli".to_string(),
            "bijux config export ./bijux.env".to_string(),
        ],
        ["plugins"] => vec![
            "bijux plugins list".to_string(),
            "bijux plugins inspect sample".to_string(),
            "bijux plugins check sample".to_string(),
            "bijux plugins install ./plugin.manifest.json".to_string(),
        ],
        ["plugins", "install"] => vec![
            "bijux plugins install ./plugin.manifest.json".to_string(),
            "bijux plugins install ./plugin.manifest.json --format json".to_string(),
            "bijux plugins install ./plugin.manifest.json --source community-catalog".to_string(),
        ],
        ["plugins", "scaffold"] => vec![
            "bijux plugins scaffold python sample-plugin".to_string(),
            "bijux plugins scaffold python sample-plugin --format json".to_string(),
            "bijux help plugins scaffold".to_string(),
        ],
        ["plugins", "inspect"] | ["cli", "plugins", "inspect"] => vec![
            "bijux cli plugins inspect sample".to_string(),
            "bijux cli plugins inspect sample --format json".to_string(),
            "bijux help cli plugins inspect".to_string(),
        ],
        ["repl"] => vec!["bijux repl".to_string(), "bijux repl --format text".to_string()],
        ["completion"] => vec![
            "bijux completion --shell bash".to_string(),
            "bijux completion --shell zsh --format json".to_string(),
        ],
        ["history"] => vec![
            "bijux history".to_string(),
            "bijux history clear".to_string(),
            "bijux history clear --force".to_string(),
        ],
        ["memory"] => vec![
            "bijux memory list".to_string(),
            "bijux memory set session.id=abc123".to_string(),
            "bijux memory get session.id".to_string(),
        ],
        ["cli", "config"] => vec![
            "bijux cli config list".to_string(),
            "bijux cli config get foo".to_string(),
            "bijux cli config set foo=bar".to_string(),
            "bijux cli config diff --from-profile dev --to-profile prod".to_string(),
        ],
        ["cli", "plugins"] => vec![
            "bijux cli plugins list".to_string(),
            "bijux cli plugins inspect sample".to_string(),
            "bijux cli plugins check sample".to_string(),
        ],
        ["cli", "plugins", "install"] => vec![
            "bijux cli plugins install ./plugin.manifest.json".to_string(),
            "bijux cli plugins install ./plugin.manifest.json --format json".to_string(),
            "bijux cli plugins install ./plugin.manifest.json --source community-catalog"
                .to_string(),
        ],
        ["cli", "plugins", "scaffold"] => vec![
            "bijux cli plugins scaffold python sample-plugin".to_string(),
            "bijux cli plugins scaffold python sample-plugin --format json".to_string(),
            "bijux help cli plugins scaffold".to_string(),
        ],
        _ => default_examples(path),
    }
}

fn default_examples(path: &[&str]) -> Vec<String> {
    if path.is_empty() {
        return Vec::new();
    }

    let joined = path.join(" ");
    vec![
        format!("bijux {joined}"),
        format!("bijux {joined} --format json"),
        format!("bijux help {joined}"),
    ]
}
