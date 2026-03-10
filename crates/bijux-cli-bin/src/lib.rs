#![forbid(unsafe_code)]
//! REPL orchestration boundaries.

mod completion;
mod diagnostics;
mod execution;
mod history;
mod reference;
mod session;
mod types;

#[cfg(test)]
use bijux_cli_python as _;
#[cfg(test)]
use libc as _;

pub use completion::{completion_candidates, register_plugin_completion_hook};
pub use diagnostics::{
    benchmark_startup_latency, check_repl_budgets, estimated_session_memory_bytes,
    inspect_last_error, session_diagnostics_dump,
};
pub use execution::{execute_repl_input, execute_repl_line, repl_argv_from_line};
pub use history::{configure_history, flush_history, load_history, replay_history_command};
pub use reference::render_repl_command_reference;
pub use session::{shutdown_repl, startup_repl, startup_repl_with_diagnostics};
pub use types::{
    ReplError, ReplEvent, ReplFrame, ReplInput, ReplSession, ReplShutdownContract,
    ReplStartupContract, ReplStream, REPL_MEMORY_BUDGET_BYTES, REPL_STARTUP_LATENCY_BUDGET_MS,
};
