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

fn is_known_help_global_flag(token: &str) -> bool {
    matches!(
        token,
        "--help" | "-h" | "--quiet" | "-q" | "--pretty" | "--no-pretty" | "--json" | "--text"
    )
}

fn help_global_flag_takes_value(token: &str) -> bool {
    matches!(token, "--format" | "-f" | "--log-level" | "--color" | "--config-path")
}

fn canonical_help_flag_name(token: &str) -> &'static str {
    match token {
        "--format" | "-f" => "--format",
        "--log-level" => "--log-level",
        "--color" => "--color",
        "--config-path" => "--config-path",
        _ => unreachable!("canonical_help_flag_name called with unsupported token"),
    }
}

fn parse_help_flag_with_equals(token: &str) -> Option<(&'static str, &str)> {
    const PREFIXES: [(&str, &str); 4] = [
        ("--format=", "--format"),
        ("--log-level=", "--log-level"),
        ("--color=", "--color"),
        ("--config-path=", "--config-path"),
    ];
    PREFIXES
        .iter()
        .find_map(|(prefix, canonical)| token.strip_prefix(prefix).map(|value| (*canonical, value)))
}

pub(super) fn parse_help_path_tokens(
    tokens: &[String],
    stop_on_help_flag: bool,
) -> std::result::Result<Vec<String>, String> {
    let mut positional = Vec::new();
    let mut pending_value_for: Option<&'static str> = None;
    let mut passthrough = false;

    for token in tokens {
        if let Some(flag) = pending_value_for.take() {
            if token == "--" {
                return Err(format!("Missing value for {flag}"));
            }
            continue;
        }

        if !passthrough {
            if token == "--" {
                passthrough = true;
                continue;
            }
            if stop_on_help_flag && matches!(token.as_str(), "--help" | "-h") {
                break;
            }
            if is_known_help_global_flag(token) {
                continue;
            }
            if let Some((flag, value)) = parse_help_flag_with_equals(token) {
                if value.is_empty() {
                    return Err(format!("Missing value for {flag}"));
                }
                continue;
            }
            if help_global_flag_takes_value(token) {
                pending_value_for = Some(canonical_help_flag_name(token));
                continue;
            }
            if token.starts_with('-') {
                return Err(format!("Unknown help flag: {token}"));
            }
        }

        positional.push(token.to_string());
    }

    if let Some(flag) = pending_value_for {
        return Err(format!("Missing value for {flag}"));
    }

    Ok(positional)
}

pub(super) fn parse_help_command_path(argv: &[String]) -> std::result::Result<Vec<String>, String> {
    parse_help_path_tokens(&argv[2..], false)
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
    let positional = parse_help_path_tokens(&argv[1..], true).ok()?;

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

#[cfg(test)]
mod tests {
    use super::parse_help_path_tokens;

    fn vec_of(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parse_help_path_tokens_rejects_missing_flag_values() {
        let missing = parse_help_path_tokens(&vec_of(&["--format"]), false);
        assert_eq!(missing, Err("Missing value for --format".to_string()));

        let missing_short = parse_help_path_tokens(&vec_of(&["-f"]), false);
        assert_eq!(missing_short, Err("Missing value for --format".to_string()));
    }

    #[test]
    fn parse_help_path_tokens_respects_passthrough_separator() {
        let parsed = parse_help_path_tokens(&vec_of(&["--", "--help", "--format"]), false)
            .expect("passthrough parsing should succeed");
        assert_eq!(parsed, vec_of(&["--help", "--format"]));
    }

    #[test]
    fn parse_help_path_tokens_rejects_unknown_flags_before_separator() {
        let parsed = parse_help_path_tokens(&vec_of(&["--unknown"]), false);
        assert_eq!(parsed, Err("Unknown help flag: --unknown".to_string()));
    }

    #[test]
    fn parse_help_path_tokens_rejects_empty_equals_values() {
        let parsed = parse_help_path_tokens(&vec_of(&["--format="]), false);
        assert_eq!(parsed, Err("Missing value for --format".to_string()));
    }
}
