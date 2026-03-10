#![forbid(unsafe_code)]
//! REPL startup latency and memory-use budget checks.

use std::time::{Duration, Instant};

use bijux_cli as _;
use bijux_cli::interface::repl::{estimated_session_memory_bytes, startup_repl};
use bijux_cli_python as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

#[test]
fn repl_startup_latency_stays_within_budget() {
    let started = Instant::now();
    for _ in 0..20 {
        let (_session, _startup) = startup_repl("benchmark", None);
    }
    let avg = started.elapsed().as_millis() / 20;
    assert!(avg <= 50, "repl startup average budget exceeded: {avg}ms");
}

#[test]
fn repl_startup_memory_estimate_stays_within_budget() {
    let (session, _) = startup_repl("benchmark", None);
    let estimated = estimated_session_memory_bytes(&session);
    assert!(estimated <= 512 * 1024, "repl startup memory estimate exceeded: {estimated} bytes");
}

#[test]
fn repl_startup_latency_with_plugin_completion_hooks_stays_within_budget() {
    let started = Instant::now();
    for _ in 0..10 {
        let (_session, _startup, diagnostics) =
            bijux_cli::interface::repl::startup_repl_with_diagnostics(
                "benchmark",
                None,
                &["community", "atlas", "plugins"],
            );
        assert!(diagnostics.is_empty() || diagnostics.len() <= 3);
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(2),
        "repl startup with plugin hooks budget exceeded: {elapsed:?}"
    );
}
