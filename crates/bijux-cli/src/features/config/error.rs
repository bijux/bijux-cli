#![forbid(unsafe_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConfigErrorKind {
    Validation,
    Parse,
    Persistence,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfigError {
    pub(crate) kind: ConfigErrorKind,
    pub(crate) message: String,
}

impl ConfigError {
    #[must_use]
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: ConfigErrorKind::Validation,
            message: message.into(),
        }
    }

    #[must_use]
    pub(crate) fn parse(message: impl Into<String>) -> Self {
        Self {
            kind: ConfigErrorKind::Parse,
            message: message.into(),
        }
    }

    #[must_use]
    pub(crate) fn persistence(message: impl Into<String>) -> Self {
        Self {
            kind: ConfigErrorKind::Persistence,
            message: message.into(),
        }
    }

    #[must_use]
    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self {
            kind: ConfigErrorKind::NotFound,
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}
