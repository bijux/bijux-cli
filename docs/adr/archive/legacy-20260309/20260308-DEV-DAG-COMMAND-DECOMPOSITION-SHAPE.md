# ADR: Dev-Dag Command Decomposition Shape

- Date: 2026-03-08
- Status: Accepted

## Context

`crates/bijux-dev-dag/src/commands/mod.rs` accumulated broad orchestration, filesystem traversal, and command dispatch responsibilities. This reduced readability and made direct ownership of command families less explicit.

## Decision

- Keep `commands/mod.rs` as the primary command dispatch surface.
- Extract reusable file traversal and run-selection helpers into `commands/file_catalog.rs`.
- Keep command-family business logic in focused modules (`authoring_evidence`, `battle_evidence`, `compare_evidence`, `evidence_control_plane`, `evidence_registry`, `perf_evidence`, `suite_catalog`).
- Require direct test surfaces in each command-family module and verification binary.
- Enforce release-time 0%-coverage guardrails via dev-dag contract tests and protected allowlist checks.

## Consequences

- Command ownership is clearer and easier to review.
- Changes to filesystem traversal logic are isolated in one helper module.
- Direct tests remain close to the command logic and binaries they validate.
- Further reductions of `commands/mod.rs` can continue with incremental helper extractions without changing command-line behavior.
