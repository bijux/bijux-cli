#![forbid(unsafe_code)]
//! Shared argv helpers for command argument extraction.

fn extras_window<'a>(argv: &'a [String], command_tokens: &[&str]) -> &'a [String] {
    let extra_start = 1 + command_tokens.len();
    if argv.len() < extra_start {
        return &[];
    }
    for (idx, token) in command_tokens.iter().enumerate() {
        if argv.get(idx + 1).map(String::as_str) != Some(*token) {
            return &[];
        }
    }
    &argv[extra_start..]
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
    use super::command_option_value;

    #[test]
    fn command_option_value_supports_space_and_equals_forms() {
        let spaced = vec![
            "bijux".to_string(),
            "history".to_string(),
            "--limit".to_string(),
            "5".to_string(),
        ];
        let equals =
            vec!["bijux".to_string(), "history".to_string(), "--limit=7".to_string()];

        assert_eq!(
            command_option_value(&spaced, &["history"], "--limit").as_deref(),
            Some("5")
        );
        assert_eq!(
            command_option_value(&equals, &["history"], "--limit").as_deref(),
            Some("7")
        );
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
        assert_eq!(
            command_option_value(&argv, &["cli", "config", "get"], "--limit"),
            None
        );
    }
}
