//! Help rendering interception for clap-managed output.

use crate::interface::cli::help::decorate_help_text;
use crate::interface::cli::parser::root_command;

fn normalize_help_whitespace(raw: &str) -> String {
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
            let normalized = normalize_help_whitespace(&error.to_string());
            Some(decorate_help_text(normalized, &path_refs))
        }
        Err(_) => None,
    }
}

pub(super) fn try_render_clap_usage_error(argv: &[String]) -> Option<String> {
    match root_command().try_get_matches_from(argv) {
        Ok(_) => None,
        Err(error) if matches!(error.kind(), clap::error::ErrorKind::InvalidSubcommand) => None,
        Err(error)
            if !matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) =>
        {
            Some(normalize_help_whitespace(&error.to_string()))
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
