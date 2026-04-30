#![forbid(unsafe_code)]
//! Shared argv helpers for command argument extraction.

fn extras_window<'a>(argv: &'a [String], command_tokens: &[&str]) -> &'a [String] {
    fn match_tokens(argv: &[String], start: usize, tokens: &[&str]) -> Option<usize> {
        let end = start + tokens.len();
        if argv.len() < end {
            return None;
        }
        for (offset, token) in tokens.iter().enumerate() {
            if argv.get(start + offset).map(String::as_str) != Some(*token) {
                return None;
            }
        }
        Some(end)
    }

    let mut command_start = 1;
    while command_start < argv.len() {
        let token = argv[command_start].as_str();
        if token == "--quiet" || token == "-q" || token == "--pretty" || token == "--no-pretty" {
            command_start += 1;
            continue;
        }
        if token == "--format"
            || token == "-f"
            || token == "--log-level"
            || token == "--color"
            || token == "--config-path"
        {
            command_start += 2;
            continue;
        }
        if token.starts_with("--format=")
            || token.starts_with("--log-level=")
            || token.starts_with("--color=")
            || token.starts_with("--config-path=")
            || token == "--json"
            || token == "--text"
        {
            command_start += 1;
            continue;
        }
        break;
    }

    if let Some(extra_start) = match_tokens(argv, command_start, command_tokens) {
        return &argv[extra_start..];
    }

    if command_tokens.first().copied() == Some("cli") {
        if let Some(extra_start) = match_tokens(argv, command_start, &command_tokens[1..]) {
            return &argv[extra_start..];
        }
    }

    &[]
}

/// Return positional args for a command path while ignoring known global flags.
#[must_use]
pub fn command_positionals(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
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
            || token == "--profile"
            || token == "--from-profile"
            || token == "--to-profile"
        {
            i += 2;
            continue;
        }
        if token.starts_with("--format=")
            || token.starts_with("--log-level=")
            || token.starts_with("--color=")
            || token.starts_with("--config-path=")
            || token.starts_with("--profile=")
            || token.starts_with("--from-profile=")
            || token.starts_with("--to-profile=")
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
pub fn command_option_value(
    argv: &[String],
    command_tokens: &[&str],
    option: &str,
) -> Option<String> {
    let extras = extras_window(argv, command_tokens);
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == option {
            let next = extras.get(i + 1)?;
            if next.starts_with('-') {
                return None;
            }
            return Some(next.clone());
        }
        if token.starts_with(&(option.to_string() + "=")) {
            return token.split_once('=').map(|(_, value)| value.to_string());
        }
        i += 1;
    }

    None
}

/// Return true when the exact flag token is present in argv.
#[must_use]
pub fn command_has_flag(argv: &[String], flag: &str) -> bool {
    argv.iter().any(|arg| arg == flag)
}

#[cfg(test)]
mod tests {
    use super::{command_option_value, command_positionals};

    #[test]
    fn command_option_value_supports_space_and_equals_forms() {
        let spaced = vec![
            "bijux".to_string(),
            "history".to_string(),
            "--limit".to_string(),
            "5".to_string(),
        ];
        let equals = vec!["bijux".to_string(), "history".to_string(), "--limit=7".to_string()];

        assert_eq!(command_option_value(&spaced, &["history"], "--limit").as_deref(), Some("5"));
        assert_eq!(command_option_value(&equals, &["history"], "--limit").as_deref(), Some("7"));
    }

    #[test]
    fn command_option_value_treats_flag_followups_as_missing_value() {
        let argv = vec![
            "bijux".to_string(),
            "history".to_string(),
            "--filter".to_string(),
            "--sort".to_string(),
            "timestamp".to_string(),
        ];
        assert_eq!(command_option_value(&argv, &["history"], "--filter"), None);
    }

    #[test]
    fn command_option_value_respects_command_window() {
        let argv = vec![
            "bijux".to_string(),
            "history".to_string(),
            "--limit".to_string(),
            "9".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        assert_eq!(command_option_value(&argv, &["cli", "config", "get"], "--limit"), None);
    }

    #[test]
    fn command_window_accepts_root_alias_for_canonical_cli_tokens() {
        let argv = vec![
            "bijux".to_string(),
            "plugins".to_string(),
            "inspect".to_string(),
            "community".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        assert_eq!(
            command_positionals(&argv, &["cli", "plugins", "inspect"]),
            vec!["community".to_string()]
        );
    }

    #[test]
    fn command_positionals_ignore_profile_diff_options() {
        let argv = vec![
            "bijux".to_string(),
            "config".to_string(),
            "diff".to_string(),
            "--from-profile".to_string(),
            "dev".to_string(),
            "--to-profile".to_string(),
            "prod".to_string(),
            "cli.log_level".to_string(),
        ];
        assert_eq!(
            command_positionals(&argv, &["config", "diff"]),
            vec!["cli.log_level".to_string()]
        );
    }
}
