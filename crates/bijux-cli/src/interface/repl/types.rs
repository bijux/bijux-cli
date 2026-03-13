use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::contracts::ExecutionPolicy;

/// REPL startup latency budget in milliseconds.
pub const REPL_STARTUP_LATENCY_BUDGET_MS: u128 = 50;
/// REPL memory budget in bytes.
pub const REPL_MEMORY_BUDGET_BYTES: usize = 2 * 1024 * 1024;

/// REPL startup command prefix.
pub(crate) const META_PREFIX: char = ':';
/// Max number of characters retained per history command.
pub const REPL_HISTORY_ENTRY_MAX_CHARS: usize = 8 * 1024;
/// Max bytes accepted when reading a persisted history file.
pub const REPL_HISTORY_FILE_MAX_BYTES: u64 = 8 * 1024 * 1024;
/// Max number of history entries retained by configuration.
pub const REPL_HISTORY_MAX_ENTRIES: usize = 50_000;
/// Max command completion candidates returned to callers.
pub const REPL_COMPLETION_MAX_CANDIDATES: usize = 512;
/// Max number of completion entries accepted per registry.
pub const REPL_COMPLETION_REGISTRY_MAX_ENTRIES: usize = 1024;
/// Max number of characters retained per completion entry.
pub const REPL_COMPLETION_ENTRY_MAX_CHARS: usize = 256;
/// Max number of characters retained for profile labels.
pub const REPL_PROFILE_MAX_CHARS: usize = 64;
/// Max number of characters retained for prompt text.
pub const REPL_PROMPT_MAX_CHARS: usize = 128;
/// Max diagnostics entries emitted during startup checks.
pub const REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES: usize = 128;
/// Max characters retained for `last_error` state.
pub const REPL_LAST_ERROR_MAX_CHARS: usize = 2_048;
/// Max characters retained in pending multiline REPL buffers.
pub const REPL_MULTILINE_BUFFER_MAX_CHARS: usize = 64 * 1024;
/// Max characters accepted for a fully assembled REPL command.
pub const REPL_COMMAND_MAX_CHARS: usize = 64 * 1024;

/// Stable REPL startup contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplStartupContract {
    /// Prompt format string.
    pub prompt: String,
    /// Whether profile/context is displayed in prompt.
    pub include_profile_context: bool,
    /// Effective startup policy.
    pub policy: ExecutionPolicy,
}

/// Stable REPL shutdown contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplShutdownContract {
    /// Session id.
    pub session_id: String,
    /// Number of commands executed.
    pub commands_executed: usize,
}

/// REPL session model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplSession {
    /// Session identifier.
    pub session_id: String,
    /// Prompt displayed to user.
    pub prompt: String,
    /// Profile label shown in prompt.
    pub profile: String,
    /// Effective execution policy.
    pub policy: ExecutionPolicy,
    /// Command counter.
    pub commands_executed: usize,
    /// Last mapped exit code as integer.
    pub last_exit_code: i32,
    /// Trace mode toggle.
    pub trace_mode: bool,
    /// Persistent command history buffer.
    pub history: Vec<String>,
    /// Max history size.
    pub history_limit: usize,
    /// Whether history persistence is enabled.
    pub history_enabled: bool,
    /// History file location.
    pub history_file: Option<PathBuf>,
    /// Pending multiline input buffer.
    pub pending_multiline: Option<String>,
    /// Last observed error message.
    pub last_error: Option<String>,
    /// Plugin completion hooks by namespace.
    pub plugin_completion_hooks: BTreeMap<String, Vec<String>>,
    /// Extension completion registries keyed by owner identifier.
    pub completion_registries: BTreeMap<String, Vec<String>>,
}

/// REPL emission stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Repl output frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplFrame {
    /// Stream target.
    pub stream: ReplStream,
    /// Serialized output.
    pub content: String,
}

/// Input event for interactive session loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplInput {
    /// Normal command line.
    Line(String),
    /// Ctrl-C interrupt event.
    Interrupt,
    /// EOF event.
    Eof,
}

/// Result of processing a REPL input event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplEvent {
    /// Keep session alive.
    Continue(Option<ReplFrame>),
    /// Exit session.
    Exit(Option<ReplFrame>),
    /// Interrupted command input.
    Interrupted(ReplFrame),
}

/// REPL runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum ReplError {
    /// Command parser failed.
    #[error(transparent)]
    Parser(#[from] crate::routing::parser::ParseError),
    /// Routing failed.
    #[error(transparent)]
    Route(#[from] crate::routing::registry::RouteError),
    /// Kernel failed.
    #[error("kernel execution failed")]
    Kernel(crate::kernel::KernelError),
    /// Core app execution failed.
    #[error("core execution failed: {0}")]
    Core(String),
    /// Output encoding failed.
    #[error(transparent)]
    Emit(#[from] crate::shared::output::EmitError),
    /// History serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid REPL command.
    #[error("invalid repl command: {0}")]
    InvalidMetaCommand(String),
    /// Invalid command-line tokenization input.
    #[error("invalid command input: {0}")]
    InvalidCommandInput(String),
    /// History replay index was invalid.
    #[error("history index out of bounds: {0}")]
    HistoryIndexOutOfBounds(usize),
}
