use std::time::{Duration, Instant};

use serde_json::json;

use super::session::startup_repl;
use super::types::{
    ReplSession, REPL_MEMORY_BUDGET_BYTES, REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES,
    REPL_STARTUP_LATENCY_BUDGET_MS,
};
use crate::shared::telemetry::truncate_chars;

const MAX_DIAGNOSTIC_SESSION_ID_CHARS: usize = 128;
const MAX_DIAGNOSTIC_ERROR_CHARS: usize = 1024;

/// Return last error message captured by REPL session.
#[must_use]
pub fn inspect_last_error(session: &ReplSession) -> Option<String> {
    session.last_error.clone()
}

/// Dump structured REPL diagnostics.
#[must_use]
pub fn session_diagnostics_dump(session: &ReplSession) -> String {
    let (session_id, session_id_truncated) =
        truncate_chars(&session.session_id, MAX_DIAGNOSTIC_SESSION_ID_CHARS);
    let (last_error, last_error_truncated) = match session.last_error.as_deref() {
        Some(value) => {
            let (bounded, truncated) = truncate_chars(value, MAX_DIAGNOSTIC_ERROR_CHARS);
            (Some(bounded), truncated)
        }
        None => (None, false),
    };
    let plugin_completion_hooks = session
        .plugin_completion_hooks
        .keys()
        .take(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES)
        .collect::<Vec<_>>();
    let completion_registries = session
        .completion_registries
        .keys()
        .take(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES)
        .collect::<Vec<_>>();
    let payload = json!({
        "session_id": session_id,
        "session_id_truncated": session_id_truncated,
        "commands_executed": session.commands_executed,
        "last_exit_code": session.last_exit_code,
        "trace_mode": session.trace_mode,
        "history_size": session.history.len(),
        "history_limit": session.history_limit,
        "plugin_completion_hooks": plugin_completion_hooks,
        "plugin_completion_hooks_omitted":
            session.plugin_completion_hooks.len().saturating_sub(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES),
        "completion_registries": completion_registries,
        "completion_registries_omitted":
            session.completion_registries.len().saturating_sub(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES),
        "last_error": last_error,
        "last_error_truncated": last_error_truncated,
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
    let avg_nanos = total.as_nanos() / runs as u128;
    let nanos = match u64::try_from(avg_nanos) {
        Ok(value) => value,
        Err(_) => u64::MAX,
    };
    Duration::from_nanos(nanos)
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

#[cfg(test)]
mod tests {
    use super::session_diagnostics_dump;
    use crate::interface::repl::session::startup_repl;
    use crate::interface::repl::types::REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES;

    #[test]
    fn diagnostics_dump_bounds_large_fields() {
        let (mut session, _) = startup_repl("", None);
        session.session_id = "x".repeat(512);
        session.last_error = Some("e".repeat(4096));
        for idx in 0..(REPL_STARTUP_DIAGNOSTIC_MAX_ENTRIES + 5) {
            session
                .plugin_completion_hooks
                .insert(format!("plugin-{idx}"), vec!["status".to_string()]);
            session
                .completion_registries
                .insert(format!("owner-{idx}"), vec!["status".to_string()]);
        }

        let dump = session_diagnostics_dump(&session);
        let payload: serde_json::Value =
            serde_json::from_str(&dump).expect("diagnostics should be valid json");
        assert_eq!(payload["session_id"].as_str().unwrap_or_default().len(), 128);
        assert_eq!(payload["last_error"].as_str().unwrap_or_default().len(), 1024);
        assert_eq!(payload["plugin_completion_hooks_omitted"], 5);
        assert_eq!(payload["completion_registries_omitted"], 5);
    }
}
