use std::time::{Duration, Instant};

use serde_json::json;

use super::session::startup_repl;
use super::types::{ReplSession, REPL_MEMORY_BUDGET_BYTES, REPL_STARTUP_LATENCY_BUDGET_MS};

/// Return last error message captured by REPL session.
#[must_use]
pub fn inspect_last_error(session: &ReplSession) -> Option<String> {
    session.last_error.clone()
}

/// Dump structured REPL diagnostics.
#[must_use]
pub fn session_diagnostics_dump(session: &ReplSession) -> String {
    let payload = json!({
        "session_id": session.session_id,
        "commands_executed": session.commands_executed,
        "last_exit_code": session.last_exit_code,
        "trace_mode": session.trace_mode,
        "history_size": session.history.len(),
        "history_limit": session.history_limit,
        "plugin_completion_hooks": session.plugin_completion_hooks.keys().collect::<Vec<_>>(),
        "completion_registries": session.completion_registries.keys().collect::<Vec<_>>(),
        "last_error": session.last_error,
    });
    format!("{}\n", serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string()))
}

/// Approximate REPL session memory use in bytes.
#[must_use]
pub fn estimated_session_memory_bytes(session: &ReplSession) -> usize {
    session.prompt.len()
        + session.profile.len()
        + session.history.iter().map(String::len).sum::<usize>()
        + session
            .plugin_completion_hooks
            .iter()
            .map(|(k, v)| k.len() + v.iter().map(String::len).sum::<usize>())
            .sum::<usize>()
        + session
            .completion_registries
            .iter()
            .map(|(k, v)| k.len() + v.iter().map(String::len).sum::<usize>())
            .sum::<usize>()
        + 1024
}

/// Benchmark average startup latency over N iterations.
#[must_use]
pub fn benchmark_startup_latency(iterations: usize) -> Duration {
    let runs = iterations.max(1);
    let started = Instant::now();
    for _ in 0..runs {
        let _ = startup_repl("benchmark", None);
    }
    let total = started.elapsed();
    Duration::from_nanos((total.as_nanos() / runs as u128) as u64)
}

/// Check REPL runtime budgets.
#[must_use]
pub fn check_repl_budgets(session: &ReplSession, startup_avg: Duration) -> Vec<String> {
    let mut warnings = Vec::new();
    if startup_avg.as_millis() > REPL_STARTUP_LATENCY_BUDGET_MS {
        warnings.push(format!(
            "startup latency {}ms exceeded {}ms budget",
            startup_avg.as_millis(),
            REPL_STARTUP_LATENCY_BUDGET_MS
        ));
    }

    let estimated = estimated_session_memory_bytes(session);
    if estimated > REPL_MEMORY_BUDGET_BYTES {
        warnings.push(format!(
            "estimated memory {} bytes exceeded {} bytes budget",
            estimated, REPL_MEMORY_BUDGET_BYTES
        ));
    }
    warnings
}
