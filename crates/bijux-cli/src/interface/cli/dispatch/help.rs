//! Help rendering interception for clap-managed output.

use crate::interface::cli::help::decorate_help_text;
use crate::interface::cli::parser::root_command;

pub(super) fn try_render_clap_help(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error)
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            if !matches!(error.kind(), clap::error::ErrorKind::DisplayHelp) {
                return Some(error.to_string());
            }

            let path = parse_help_path(argv).unwrap_or_default();
            let path_refs: Vec<&str> = path.iter().map(String::as_str).collect();
            Some(decorate_help_text(error.to_string(), &path_refs))
        }
        Err(_) => None,
    }
}

fn parse_help_path(argv: &[String]) -> Option<Vec<String>> {
    if !argv.iter().any(|token| matches!(token.as_str(), "--help" | "-h")) {
        return None;
    }

    let mut positional = Vec::new();
    let mut consume_next = false;

    for token in argv.iter().skip(1) {
        if consume_next {
            consume_next = false;
            continue;
        }

        match token.as_str() {
            "--help" | "-h" => break,
            "--format" | "-f" | "--log-level" | "--color" | "--config-path" => {
                consume_next = true;
            }
            "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text" => {}
            value if value.starts_with('-') => {}
            value => positional.push(value.to_string()),
        }
    }

    let mut command = root_command();
    let mut path = Vec::new();

    for segment in positional {
        let Some(next) = command.find_subcommand_mut(segment.as_str()) else {
            break;
        };
        path.push(segment);
        command = next.clone();
    }

    Some(path)
}
