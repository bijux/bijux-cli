//! CLI argument extraction helpers for maintainer routing.

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

    let mut legacy_tokens = Vec::with_capacity(command_tokens.len() + 2);
    legacy_tokens.extend(["dev", "cli"]);
    legacy_tokens.extend(command_tokens.iter().copied());

    let Some(extra_start) = match_tokens(argv, command_start, &legacy_tokens) else {
        return &[];
    };
    &argv[extra_start..]
}

/// Return option value from either `--name=value` or `--name value` forms.
#[must_use]
pub fn command_option_value(
    argv: &[String],
    command_tokens: &[&str],
    name: &str,
) -> Option<String> {
    let extras = extras_window(argv, command_tokens);
    let mut found = None;
    let prefixed = format!("{name}=");
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == "--" {
            break;
        }
        if token == name {
            let Some(next) = extras.get(i + 1) else {
                found = None;
                break;
            };
            if next == "--" || next.starts_with('-') {
                found = None;
            } else {
                found = Some(next.clone());
                i += 2;
                continue;
            }
        } else if let Some(value) = token.strip_prefix(&prefixed) {
            found = Some(value.to_string());
        }
        i += 1;
    }
    found
}

/// Return passthrough args after `--`.
#[must_use]
pub fn command_passthrough_args(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
    let extras = extras_window(argv, command_tokens);
    extras
        .iter()
        .position(|arg| arg == "--")
        .and_then(|idx| extras.get(idx + 1..))
        .map_or_else(Vec::new, |tail| tail.to_vec())
}

/// Return true when `name` exists as a standalone flag.
#[must_use]
pub fn command_has_flag(argv: &[String], command_tokens: &[&str], name: &str) -> bool {
    let extras = extras_window(argv, command_tokens);
    for token in extras {
        if token == "--" {
            break;
        }
        if token == name {
            return true;
        }
    }
    false
}

/// Return positional arguments that belong to a specific command path.
#[must_use]
pub fn command_positionals(argv: &[String], command_tokens: &[&str]) -> Vec<String> {
    let extras = extras_window(argv, command_tokens);
    let mut positional = Vec::new();
    let mut i = 0;
    while i < extras.len() {
        let token = &extras[i];
        if token == "--" {
            break;
        }
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

#[cfg(test)]
mod tests {
    use super::command_option_value;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn command_option_value_reads_equals_form() {
        let args = argv(&["bijux-dev-cli", "evidence", "show", "--id=EVIDENCE-1001-TEST"]);
        assert_eq!(
            command_option_value(&args, &["evidence", "show"], "--id").as_deref(),
            Some("EVIDENCE-1001-TEST")
        );
    }

    #[test]
    fn command_option_value_reads_separate_token_form() {
        let args = argv(&["bijux-dev-cli", "evidence", "show", "--id", "EVIDENCE-1001-TEST"]);
        assert_eq!(
            command_option_value(&args, &["evidence", "show"], "--id").as_deref(),
            Some("EVIDENCE-1001-TEST")
        );
    }

    #[test]
    fn command_option_value_rejects_next_flag_as_value() {
        let args = argv(&["bijux-dev-cli", "evidence", "show", "--id", "--kind", "runtime"]);
        assert_eq!(command_option_value(&args, &["evidence", "show"], "--id"), None);
    }

    #[test]
    fn command_option_value_uses_last_occurrence_before_passthrough_marker() {
        let args = argv(&[
            "bijux-dev-cli",
            "contracts",
            "--all",
            "--kind",
            "first",
            "--kind=second",
            "--",
            "--kind",
            "ignored",
        ]);
        assert_eq!(
            command_option_value(&args, &["contracts"], "--kind").as_deref(),
            Some("second")
        );
    }
}
