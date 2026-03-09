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

#[cfg(test)]
mod tests {
    use super::{normalize_key, validate_value};

    #[test]
    fn key_rejects_empty_and_whitespace_only() {
        assert!(normalize_key("").is_err());
        assert!(normalize_key("   ").is_err());
    }

    #[test]
    fn key_accepts_lower_mixed_underscore_and_alphanumeric() {
        assert_eq!(normalize_key("alpha").expect("lower"), "alpha");
        assert_eq!(normalize_key("MixedCase").expect("mixed"), "mixedcase");
        assert_eq!(normalize_key("_").expect("underscore"), "_");
        assert_eq!(normalize_key("a1b2").expect("alphanumeric"), "a1b2");
    }

    #[test]
    fn key_rejects_invalid_punctuation_dots_dashes_and_non_ascii() {
        assert!(normalize_key("bad!key").is_err());
        assert!(normalize_key("group.key").is_err());
        assert!(normalize_key("bad-key").is_err());
        assert!(normalize_key("näme").is_err());
    }

    #[test]
    fn value_accepts_ascii_empty_and_spaces() {
        assert!(validate_value("plain-ascii").is_ok());
        assert!(validate_value("").is_ok());
        assert!(validate_value("value with spaces").is_ok());
    }

    #[test]
    fn value_rejects_newline_tab_and_control_characters() {
        assert!(validate_value("line\nbreak").is_err());
        assert!(validate_value("tab\tvalue").is_err());
        assert!(validate_value("vert\u{000B}tab").is_err());
    }
}
