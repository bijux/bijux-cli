# ADR: Dev-Dag Helper Contract Surface

- Date: 2026-03-08
- Status: Accepted

## Context

The dev-dag helper modules (`repo`, `tooling`, `report`, and selected command helpers) mix stable workflow expectations with implementation glue. Without explicit boundaries, low-level helper changes can accidentally alter release-facing developer workflows.

## Decision

- Treat helper modules that resolve workspace root, write reports, and invoke tooling wrappers as public-ish contracts.
- Require direct in-file tests for helper modules, including very small modules.
- Keep generated helper health reports in `docs/reports/foundation/` and enforce them with contract tests.
- Keep command-family orchestration glue internal unless explicitly promoted through ADR.

## Public-ish helper contracts

- `repo/root.rs`
- `repo/layout.rs`
- `report/write.rs`
- `tooling/cargo.rs`
- `tooling/git.rs`
- `tooling/mod.rs`

## Internal glue (non-public contract by default)

- command-family wiring and dispatch internals in `commands/mod.rs`
- helper composition details not exposed through stable command outputs

## Consequences

- Helper behavior regressions are caught earlier by direct tests and helper fast suites.
- Review scope is clearer for changes that impact developer-facing release/evidence workflows.
