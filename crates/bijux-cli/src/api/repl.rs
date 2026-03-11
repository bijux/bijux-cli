#![forbid(unsafe_code)]
//! REPL surface facade.

pub use crate::interface::repl::{
    benchmark_startup_latency, check_repl_budgets, completion_candidates, configure_history,
    estimated_session_memory_bytes, execute_repl_input, execute_repl_line, flush_history,
    inspect_last_error, load_history, register_plugin_completion_hook,
    render_repl_command_reference, repl_argv_from_line, replay_history_command,
    session_diagnostics_dump, shutdown_repl, startup_repl, startup_repl_with_diagnostics,
    ReplError, ReplEvent, ReplFrame, ReplInput, ReplSession, ReplShutdownContract,
    ReplStartupContract, ReplStream, REPL_MEMORY_BUDGET_BYTES, REPL_STARTUP_LATENCY_BUDGET_MS,
};
