//! Help rendering for command paths.

use anyhow::Result;

use crate::routing::parser::root_command;

pub(crate) fn render_command_help(path: &[&str]) -> Result<String> {
    let mut cmd = root_command();
    let target =
        find_command_mut(&mut cmd, path).ok_or_else(|| anyhow::anyhow!("unknown help path"))?;
    let mut out = Vec::new();
    target.write_long_help(&mut out)?;
    Ok(decorate_help_text(String::from_utf8(out)?, path))
}

pub(crate) fn decorate_help_text(mut rendered: String, path: &[&str]) -> String {
    append_help_sections(&mut rendered, path);
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
        sections.push(grouped.to_string());
    }
    if let Some(subcommands) = help_subcommand_guide(path) {
        sections.push(subcommands.to_string());
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

fn help_grouped_guide(path: &[&str]) -> Option<&'static str> {
    match path {
        [] => Some(
            "Command groups:\n\
Runtime:\n\
  status      Runtime status summary\n\
  audit       Runtime audit report\n\
  docs        Documentation audit and links\n\
  sleep       Controlled sleep helper for diagnostics\n\
  doctor      Runtime health diagnostics\n\
  version     Runtime version report\n\
\n\
Configuration & Plugins:\n\
  config      Configuration management commands\n\
  plugins     Plugin lifecycle and diagnostics commands\n\
\n\
State & Interaction:\n\
  history     History state management\n\
  memory      Memory state management\n\
  repl        Interactive shell\n\
  completion  Shell completion generation\n\
  cli         Canonical runtime command namespace",
        ),
        _ => None,
    }
}

fn help_subcommand_guide(path: &[&str]) -> Option<&'static str> {
    match path {
        ["cli"] => Some(
            "Subcommand guide:\n\
  status     Runtime status summary\n\
  paths      Runtime state and filesystem paths\n\
  config     Runtime configuration operations\n\
  self-test  Deterministic runtime self-checks\n\
  plugins    Canonical plugin lifecycle namespace",
        ),
        ["config"] | ["cli", "config"] => Some(
            "Subcommand guide:\n\
  list    Print all key/value pairs\n\
  get     Read one key\n\
  set     Write one key=value pair\n\
  unset   Remove one key\n\
  clear   Remove all keys\n\
  reload  Validate and reload current file\n\
  export  Write config to a target path\n\
  load    Load config from a source path",
        ),
        ["plugins"] | ["cli", "plugins"] => Some(
            "Subcommand guide:\n\
  list            List discovered plugins\n\
  info            Show plugin inventory details\n\
  inspect         Inspect plugin contracts and compatibility\n\
  check           Validate a plugin namespace\n\
  enable          Enable a plugin namespace\n\
  disable         Disable a plugin namespace\n\
  install         Install plugin from manifest/source\n\
  uninstall       Remove a plugin namespace\n\
  scaffold        Generate a plugin template\n\
  doctor          Plugin health diagnostics\n\
  reserved-names  Show reserved plugin namespace rules\n\
  where           Show plugin state and install paths\n\
  explain         Explain plugin resolution outcome\n\
  schema          Show plugin manifest schema",
        ),
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
            "bijux config get foo".to_string(),
            "bijux config set foo=bar".to_string(),
            "bijux plugins list".to_string(),
        ],
        ["cli"] => vec![
            "bijux cli status".to_string(),
            "bijux cli paths".to_string(),
            "bijux cli config list".to_string(),
            "bijux cli plugins list".to_string(),
        ],
        ["status"] => vec!["bijux status".to_string(), "bijux status --format json".to_string()],
        ["audit"] => vec!["bijux audit".to_string(), "bijux audit --format json".to_string()],
        ["docs"] => vec!["bijux docs".to_string(), "bijux docs --format json".to_string()],
        ["sleep"] => vec!["bijux sleep 1".to_string(), "bijux sleep 250ms".to_string()],
        ["doctor"] => vec!["bijux doctor".to_string(), "bijux doctor --format json".to_string()],
        ["version"] => vec!["bijux version".to_string(), "bijux --version".to_string()],
        ["config"] => vec![
            "bijux config list".to_string(),
            "bijux config get foo".to_string(),
            "bijux config set foo=bar".to_string(),
            "bijux config export ./bijux.env".to_string(),
        ],
        ["plugins"] => vec![
            "bijux plugins list".to_string(),
            "bijux plugins inspect".to_string(),
            "bijux plugins check sample".to_string(),
            "bijux plugins install ./plugin.toml --source local".to_string(),
        ],
        ["plugins", "inspect"] | ["cli", "plugins", "inspect"] => vec![
            "bijux cli plugins inspect".to_string(),
            "bijux cli plugins inspect --format json".to_string(),
            "bijux help cli plugins inspect".to_string(),
        ],
        ["repl"] => vec!["bijux repl".to_string(), "bijux repl --format text".to_string()],
        ["completion"] => {
            vec!["bijux completion".to_string(), "bijux completion --format json".to_string()]
        }
        ["history"] => vec!["bijux history".to_string(), "bijux history clear".to_string()],
        ["memory"] => vec![
            "bijux memory list".to_string(),
            "bijux memory set session.id=abc123".to_string(),
            "bijux memory get session.id".to_string(),
        ],
        ["cli", "config"] => vec![
            "bijux cli config list".to_string(),
            "bijux cli config get foo".to_string(),
            "bijux cli config set foo=bar".to_string(),
        ],
        ["cli", "plugins"] => vec![
            "bijux cli plugins list".to_string(),
            "bijux cli plugins inspect".to_string(),
            "bijux cli plugins check sample".to_string(),
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
