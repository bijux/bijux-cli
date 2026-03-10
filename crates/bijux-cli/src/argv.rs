#![forbid(unsafe_code)]
//! Shared argv helpers for command argument extraction.

fn extras_window<'a>(argv: &'a [String], command_tokens: &[&str]) -> &'a [String] {
    let mut extra_start = 1 + command_tokens.len();
    if argv.len() < extra_start {
        return &[];
    }
    for (idx, token) in command_tokens.iter().enumerate() {
        if argv.get(idx + 1).map(String::as_str) != Some(*token) {
            extra_start = idx + 1;
            break;
        }
    }
    &argv[extra_start..]
}

/// Return positional args for a command path while ignoring known global flags.
#[must_use]
pub(crate) fn command_positionals(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
    let extras = extras_window(argv, command_tokens);
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

/// Read an option value from command extras, supporting `--opt value` and `--opt=value`.
#[must_use]
pub(crate) fn command_option_value(
    argv: &[String],
    command_tokens: &[&str],
    option: &str,
) -> Option<String> {
    let extras = extras_window(argv, command_tokens);
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == option {
            return extras.get(i + 1).cloned();
        }
        if token.starts_with(&(option.to_string() + "=")) {
            return token.split_once('=').map(|(_, value)| value.to_string());
        }
        i += 1;
    }

    None
}
