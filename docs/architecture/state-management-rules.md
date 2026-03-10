# State Management Rules

## Purpose
This document freezes the state-management law for `config`, `history`, `memory`, and plugin registry state before new stateful features are added.

## Rules
1. Every stateful command must have a malformed-input resilience test.
2. Every stateful mutation must preserve previous on-disk state when a write fails.
3. Every stateful area must expose both machine-readable diagnostics and plain text diagnostics through `bijux dev cli state-audit` and `bijux dev cli state-doctor`.
4. Every stateful area must keep no-color output snapshots for diagnostics to prevent hidden formatting regressions.
5. New stateful features are blocked unless they first define parity expectations and failure behavior.
6. State path resolution is centralized; stateful commands must use one shared provider for config/history/plugins/registry/memory paths.
7. Corruption handling is a hard quality bar: truncation, malformed records, wrong-type state, partial-write artifacts, and rollback proofs must stay covered.

## Current Enforcement Points
- `crates/bijux-cli-bin/tests/config_set_parity.rs`
- `crates/bijux-cli-bin/tests/history_parity.rs`
- `crates/bijux-cli-bin/tests/memory_parity.rs`
- `crates/bijux-cli-bin/tests/diagnostics_parity.rs`
- `crates/bijux-cli-bin/tests/diagnostics_snapshots.rs`
- `bijux dev cli state-audit --format json`
- `crates/bijux-cli-bin/tests/diagnostics_parity.rs`
