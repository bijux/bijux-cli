# REPL Parity Report

Date: 2026-03-09
Scope: tasks 341-360

## Inputs used for comparison

Python references:

- `src/bijux_cli/cli/repl/parsing.py`
- `src/bijux_cli/cli/repl/execution.py`
- `src/bijux_cli/cli/repl/ui.py`
- `artifacts/python-behavior/runtime/repl-interactive.txt`

Rust references:

- `crates/bijux-cli-repl/src/session.rs`
- `crates/bijux-cli-repl/src/history.rs`
- `crates/bijux-cli-repl/src/completion.rs`
- `crates/bijux-cli-repl/src/execution.rs`
- `crates/bijux-cli-repl/tests/transcript_parity.rs`
- `crates/bijux-cli-repl/tests/transcript_cases.rs`

## Baseline status for 341-360

- `341`: complete (transcript cases include `status`, `doctor`, `plugins list`, `config get`, `history`)
- `342`: complete (failure and recovery transcript case)
- `343`: complete (syntax/usage error transcript case)
- `344`: complete (nested help transcript case)
- `345`: complete (session format switching transcript case)
- `346`: complete (quiet mode transcript case)
- `347`: complete (trace mode transcript case)
- `348`: complete (plugin namespace transcript case)
- `349`: complete (reserved-name collision diagnostics transcript case)
- `350`: complete (root completion coverage)
- `351`: complete (grouped `cli` completion coverage)
- `352`: complete (grouped `dev cli` completion coverage)
- `353`: complete (plugin namespace completion coverage)
- `354`: complete (partial token completion coverage)
- `355`: complete (interrupt/cancellation transcript behavior coverage)
- `356`: complete (startup latency check under loaded plugin diagnostics inputs)
- `357`: complete (repl output parity check against non-interactive CLI for `status`)
- `358`: complete (this report)
- `359`: complete (`docs/architecture/repl-ux-review.md`)
- `360`: complete (`docs/architecture/repl-baseline.md`)

## Key implementation updates

1. REPL command execution now routes through `bijux_cli_core::app::run_app` using session policy-derived global flags.
2. This aligns non-interactive and interactive behavior for output envelopes, stream routing, and exit semantics.
3. REPL keeps meta-command controls (`:set`, `:help`, `:quit`) while delegating normal command behavior to core.

## Remaining gaps after baseline

1. Python prompt-toolkit UI rendering details remain intentionally out of this baseline.
2. Semicolon command-splitting and additional piped-mode affordances are still excluded.
3. Plugin registry-heavy startup behavior can be extended with filesystem-backed load tests in later batches.
