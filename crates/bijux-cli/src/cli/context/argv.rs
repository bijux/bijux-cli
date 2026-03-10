//! Shared command-argument helpers for CLI command families.

/// Read an option value from argv supporting `--opt value` and `--opt=value`.
#[must_use]
pub(crate) fn command_option_value(argv: &[String], name: &str) -> Option<String> {
    let prefixed = format!("{name}=");
    if let Some(found) = argv.iter().find(|arg| arg.starts_with(&prefixed)) {
        return Some(found[prefixed.len()..].to_string());
    }
    argv.iter()
        .position(|arg| arg == name)
        .and_then(|idx| argv.get(idx + 1))
        .cloned()
}

/// Return true when the flag token is present in argv.
#[must_use]
pub(crate) fn command_has_flag(argv: &[String], flag: &str) -> bool {
    argv.iter().any(|arg| arg == flag)
}
