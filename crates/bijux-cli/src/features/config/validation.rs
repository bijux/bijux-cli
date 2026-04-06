#![forbid(unsafe_code)]

use super::error::ConfigError;

pub(crate) fn normalize_key(raw: &str) -> Result<String, ConfigError> {
    let key = raw.trim();
    if key.is_empty() {
        return Err(ConfigError::validation("Key cannot be empty"));
    }
    if !key.is_ascii() {
        return Err(ConfigError::validation(
            "Non-ASCII characters are not allowed in keys or values.",
        ));
    }
    if key.contains('.') {
        return Err(ConfigError::validation(format!("Unknown config section in key: {key}")));
    }

    let normalized = key
        .strip_prefix("BIJUXCLI_")
        .or_else(|| key.strip_prefix("BIJUX_"))
        .unwrap_or(key)
        .to_ascii_lowercase();
    if !normalized.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        return Err(ConfigError::validation(
            "Invalid key: only alphanumerics and underscore allowed.",
        ));
    }

    Ok(normalized)
}

pub(crate) fn validate_value(value: &str) -> Result<(), ConfigError> {
    if !value.is_ascii() {
        return Err(ConfigError::validation(
            "Non-ASCII characters are not allowed in keys or values.",
        ));
    }
    if value.chars().any(|ch| matches!(ch, '\r' | '\n' | '\t' | '\u{000B}' | '\u{000C}')) {
        return Err(ConfigError::validation(
            "Control characters are not allowed in config values.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_key, validate_value};

    #[test]
    fn key_rejects_empty_and_whitespace_only() {
        let empty = normalize_key("").expect_err("empty key should fail validation");
        let spaces = normalize_key("   ").expect_err("whitespace key should fail validation");
        assert!(empty.to_string().contains("Key cannot be empty"));
        assert!(spaces.to_string().contains("Key cannot be empty"));
    }

    #[test]
    fn key_accepts_lower_mixed_underscore_and_alphanumeric() {
        assert_eq!(normalize_key("alpha").expect("lower"), "alpha");
        assert_eq!(normalize_key("MixedCase").expect("mixed"), "mixedcase");
        assert_eq!(normalize_key("BIJUXCLI_ALPHA").expect("prefix"), "alpha");
        assert_eq!(normalize_key("BIJUX_ALPHA").expect("legacy prefix"), "alpha");
        assert_eq!(normalize_key("_").expect("underscore"), "_");
        assert_eq!(normalize_key("a1b2").expect("alphanumeric"), "a1b2");
    }

    #[test]
    fn key_rejects_invalid_punctuation_dots_dashes_and_non_ascii() {
        let punctuation = normalize_key("bad!key").expect_err("punctuation should be rejected");
        let dotted = normalize_key("group.key").expect_err("dot should be rejected");
        let dashed = normalize_key("bad-key").expect_err("dash should be rejected");
        let non_ascii = normalize_key("näme").expect_err("non-ascii should be rejected");
        assert!(punctuation.to_string().contains("Invalid key"));
        assert!(dotted.to_string().contains("Unknown config section"));
        assert!(dashed.to_string().contains("Invalid key"));
        assert!(non_ascii.to_string().contains("Non-ASCII"));
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
        assert!(validate_value("accent-é").is_err(), "non-ascii values should be rejected");
    }
}
