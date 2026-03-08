# Run History Size Growth Report

Generated: 2026-03-08

## Scope

Run history size-growth behavior is validated over synthetic many-run fixtures and
query pagination surfaces.

## Evidence

- large fixture determinism and scale:
  - `crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs`
- history query performance under load:
  - `crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs`

## Current posture

- history traversal remains deterministic at high run counts
- paged query behavior remains stable under load
