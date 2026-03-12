//! CLI argument extraction helpers for dev-cli routing.

/// Return option value from either `--name=value` or `--name value` forms.
#[must_use]
pub fn command_option_value(argv: &[String], name: &str) -> Option<String> {
    let prefixed = format!("{name}=");
    if let Some(found) = argv.iter().find(|arg| arg.starts_with(&prefixed)) {
        return Some(found[prefixed.len()..].to_string());
    }
    argv.iter().position(|arg| arg == name).and_then(|idx| {
        let next = argv.get(idx + 1)?;
        if next.starts_with('-') {
            return None;
        }
        Some(next.clone())
    })
}

/// Return passthrough args after `--`.
#[must_use]
pub fn command_passthrough_args(argv: &[String]) -> Vec<String> {
    argv.iter()
        .position(|arg| arg == "--")
        .and_then(|idx| argv.get(idx + 1..))
        .map_or_else(Vec::new, |tail| tail.to_vec())
}

/// Return true when `name` exists as a standalone flag.
#[must_use]
pub fn command_has_flag(argv: &[String], name: &str) -> bool {
    argv.iter().any(|arg| arg == name)
}

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

/// Return positional arguments that belong to a specific command path.
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

#[cfg(test)]
mod tests {
    use super::command_option_value;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_string()).collect()
    }

    #[test]
    fn command_option_value_reads_equals_form() {
        let args = argv(&["bijux", "dev", "cli", "evidence", "show", "--id=EVIDENCE-1001-TEST"]);
        assert_eq!(command_option_value(&args, "--id").as_deref(), Some("EVIDENCE-1001-TEST"));
    }

    #[test]
    fn command_option_value_reads_separate_token_form() {
        let args = argv(&["bijux", "dev", "cli", "evidence", "show", "--id", "EVIDENCE-1001-TEST"]);
        assert_eq!(command_option_value(&args, "--id").as_deref(), Some("EVIDENCE-1001-TEST"));
    }

    #[test]
    fn command_option_value_rejects_next_flag_as_value() {
        let args = argv(&["bijux", "dev", "cli", "evidence", "show", "--id", "--kind", "runtime"]);
        assert_eq!(command_option_value(&args, "--id"), None);
    }
}
