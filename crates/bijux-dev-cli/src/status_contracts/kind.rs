//! Status contract kind model.

/// Stable status contract category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusContractKind {
    Generate,
    Check,
    Enforce,
    Warn,
    Run,
    Status,
}

impl StatusContractKind {
    /// Return stable lowercase string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Generate => "generate",
            Self::Check => "check",
            Self::Enforce => "enforce",
            Self::Warn => "warn",
            Self::Run => "run",
            Self::Status => "status",
        }
    }

    /// Parse kind from lowercase string.
    #[must_use]
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "generate" => Some(Self::Generate),
            "check" => Some(Self::Check),
            "enforce" => Some(Self::Enforce),
            "warn" => Some(Self::Warn),
            "run" => Some(Self::Run),
            "status" => Some(Self::Status),
            _ => None,
        }
    }
}
