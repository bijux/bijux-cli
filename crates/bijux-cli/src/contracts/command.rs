use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Canonical command namespace segment.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
pub struct Namespace(pub String);

impl Namespace {
    /// Build a normalized namespace.
    pub fn new(raw: &str) -> Result<Self, String> {
        let normalized = Self::normalize(raw);
        if normalized.is_empty() {
            return Err("namespace cannot be empty".to_string());
        }
        if !normalized.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        {
            return Err("namespace must be lowercase kebab-case".to_string());
        }
        if normalized.starts_with('-') || normalized.ends_with('-') || normalized.contains("--") {
            return Err(
                "namespace cannot start/end with '-' or contain consecutive '-'".to_string()
            );
        }
        Ok(Self(normalized))
    }

    /// Normalize namespace input to lowercase kebab-case candidate.
    #[must_use]
    pub fn normalize(raw: &str) -> String {
        raw.trim()
            .to_ascii_lowercase()
            .replace(['_', ' ', '/'], "-")
            .split('-')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Borrow normalized namespace string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Canonical command path composed from namespace segments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandPath {
    /// Ordered namespace segments from root to leaf.
    pub segments: Vec<Namespace>,
}

impl CommandPath {
    /// Build a command path with normalized segments.
    pub fn new(raw_segments: &[&str]) -> Result<Self, String> {
        let mut segments = Vec::with_capacity(raw_segments.len());
        for value in raw_segments {
            segments.push(Namespace::new(value)?);
        }
        if segments.is_empty() {
            return Err("command path requires at least one segment".to_string());
        }
        Ok(Self { segments })
    }

    /// Parse and normalize command path from a whitespace- or slash-delimited string.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let input = raw.trim();
        if input.is_empty() {
            return Err("command path cannot be empty".to_string());
        }
        let segments: Vec<&str> = input
            .split(|ch: char| ch.is_ascii_whitespace() || ch == '/')
            .filter(|segment| !segment.is_empty())
            .collect();
        Self::new(&segments)
    }

    /// Join path segments as a single command string.
    #[must_use]
    pub fn to_command_string(&self) -> String {
        self.segments.iter().map(Namespace::as_str).collect::<Vec<_>>().join(" ")
    }
}

/// Stable command metadata used by help and inspect APIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommandMetadata {
    /// Canonical command path.
    pub path: CommandPath,
    /// Human-readable summary.
    pub summary: String,
    /// Whether the command is hidden from help.
    pub hidden: bool,
    /// Stable aliases.
    pub aliases: Vec<CommandPath>,
}

/// Stable namespace metadata used by route-tree introspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct NamespaceMetadata {
    /// Namespace identifier.
    pub name: Namespace,
    /// Whether this namespace is reserved.
    pub reserved: bool,
    /// Owning product or component.
    pub owner: String,
}
