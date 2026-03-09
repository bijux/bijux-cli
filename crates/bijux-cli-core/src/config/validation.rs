#![forbid(unsafe_code)]

use super::error::ConfigError;

pub(crate) fn normalize_key(raw: &str) -> Result<String, ConfigError> {
    let key = raw.trim();
    if key.is_empty() {
        return Err(ConfigError::validation("Key cannot be empty"));
    }
    if !key.is_ascii() {
        return Err(ConfigError::validation("Non-ASCII characters are not allowed in keys or values."));
    }
    if key.contains('.') {
        return Err(ConfigError::validation(format!("Unknown config section in key: {key}")));
    }

    let normalized = key.strip_prefix("BIJUXCLI_").unwrap_or(key).to_ascii_lowercase();
    if !normalized.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(ConfigError::validation("Invalid key: only alphanumerics and underscore allowed."));
    }

    Ok(normalized)
}

pub(crate) fn validate_value(value: &str) -> Result<(), ConfigError> {
    if !value.is_ascii() {
        return Err(ConfigError::validation("Non-ASCII characters are not allowed in keys or values."));
    }
    if value
        .chars()
        .any(|ch| matches!(ch, '\r' | '\n' | '\t' | '\u{000B}' | '\u{000C}'))
    {
        return Err(ConfigError::validation("Control characters are not allowed in config values."));
    }
    Ok(())
}
