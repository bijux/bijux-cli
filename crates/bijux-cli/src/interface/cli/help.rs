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
    append_help_guidance(&mut rendered, path);
    if matches!(
        path,
        ["inspect"] | ["cli", "inspect"] | ["plugins", "inspect"] | ["cli", "plugins", "inspect"]
    ) {
        rendered.push_str(
            "\nCompatibility note: inspect output includes plugin compatibility warnings when present.\n",
        );
    }
    rendered
}

fn append_help_guidance(rendered: &mut String, path: &[&str]) {
    let Some(guidance) = help_guidance(path) else {
        return;
    };

    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    rendered.push('\n');
    rendered.push_str(guidance);
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
}

fn help_guidance(path: &[&str]) -> Option<&'static str> {
    match path {
        [] => Some(
            "Command guide:\n\
  cli         Runtime command family for status, paths, config, and plugins\n\
  dev         Maintainer-only engineering and repository diagnostics commands\n\
  status      Runtime status summary\n\
  audit       Runtime audit report\n\
  docs        Documentation audit and links\n\
  sleep       Controlled sleep helper for diagnostics\n\
  doctor      Runtime health diagnostics\n\
  version     Runtime version report\n\
  config      Configuration management commands\n\
  plugins     Plugin lifecycle and diagnostics commands\n\
  repl        Interactive shell\n\
  completion  Shell completion generation\n\
  inspect     Route and schema inspection\n\
  atlas       Product and namespace registry report\n\
  history     History state management\n\
  memory      Memory state management\n\
\n\
Examples:\n\
  bijux status\n\
  bijux config get foo\n\
  bijux config set foo=bar\n\
  bijux plugins list\n\
  bijux help plugins",
        ),
        ["cli"] => Some(
            "Examples:\n\
  bijux cli status\n\
  bijux cli paths\n\
  bijux cli config list\n\
  bijux cli plugins list",
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
  load    Load config from a source path\n\
\n\
Examples:\n\
  bijux config list\n\
  bijux config get foo\n\
  bijux config set foo=bar\n\
  bijux config export ./bijux.env",
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
  schema          Show plugin manifest schema\n\
\n\
Examples:\n\
  bijux plugins list\n\
  bijux plugins inspect\n\
  bijux plugins check sample\n\
  bijux plugins install ./plugin.toml --source local",
        ),
        _ => None,
    }
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
