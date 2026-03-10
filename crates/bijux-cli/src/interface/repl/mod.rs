#![forbid(unsafe_code)]
//! REPL surface entrypoints.

pub use crate::repl::{
    benchmark_startup_latency, check_repl_budgets, completion_candidates, configure_history,
    estimated_session_memory_bytes, execute_repl_input, execute_repl_line, flush_history,
    inspect_last_error, load_history, register_plugin_completion_hook, render_repl_command_reference,
    replay_history_command, repl_argv_from_line, shutdown_repl, startup_repl,
    startup_repl_with_diagnostics, session_diagnostics_dump, ReplError, ReplEvent, ReplFrame,
    ReplInput, ReplSession, ReplShutdownContract, ReplStartupContract, ReplStream,
    REPL_MEMORY_BUDGET_BYTES, REPL_STARTUP_LATENCY_BUDGET_MS,
};
