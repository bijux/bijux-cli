# REPL Recovery Law

REPL session behavior must stay resilient under hostile or malformed interaction.

Frozen requirements:

1. Malformed commands never crash the session.
2. Interrupt and EOF handling are deterministic and clear pending multiline state.
3. Plugin command failures do not prevent recovery to successful built-in commands.
4. Completion and startup remain available under broken plugin or corrupted history state.
5. REPL command results for shared commands remain equivalent to core command law.

Evidence sources:

- `artifacts/status/repl_hostile_session_report.json`
- `artifacts/status/repl_recovery_behavior_report.json`
- `crates/bijux-cli/tests/cli_surface/repl/repl_hostile_session_hardening.rs`
