use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_cli_contracts::ExecutionPolicy;

/// REPL startup latency budget in milliseconds.
pub const REPL_STARTUP_LATENCY_BUDGET_MS: u128 = 50;
/// REPL memory budget in bytes.
pub const REPL_MEMORY_BUDGET_BYTES: usize = 2 * 1024 * 1024;

/// REPL startup command prefix.
pub(crate) const META_PREFIX: char = ':';

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
    /// Whether plugin reload command is allowed.
    pub plugin_reload_safe: bool,
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
    Parser(#[from] bijux_cli_routing::parser::ParseError),
    /// Routing failed.
    #[error(transparent)]
    Route(#[from] bijux_cli_routing::registry::RouteError),
    /// Kernel failed.
    #[error("kernel execution failed")]
    Kernel(bijux_cli_core::kernel::KernelError),
    /// Core app execution failed.
    #[error("core execution failed: {0}")]
    Core(String),
    /// Output encoding failed.
    #[error(transparent)]
    Emit(#[from] bijux_cli_output::EmitError),
    /// History serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid REPL command.
    #[error("invalid repl command: {0}")]
    InvalidMetaCommand(String),
    /// History replay index was invalid.
    #[error("history index out of bounds: {0}")]
    HistoryIndexOutOfBounds(usize),
    /// Plugin reload is blocked by safety policy.
    #[error("plugin reload is disabled by safety policy")]
    PluginReloadUnsafe,
}
