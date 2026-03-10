# bijux-cli

`bijux-cli` is the core runtime crate for the `bijux` command surface.

It owns command law, routing, runtime state behavior, and process-facing execution
for the Rust runtime.

## What This Crate Owns

- Canonical command routing and normalization (`src/routing`).
- Runtime command execution behavior (`src/app.rs`).
- CLI process entrypoint helpers (`src/entrypoint.rs` and `src/bin/bijux-rs.rs`).
- Execution-kernel primitives and exit mapping (`src/kernel.rs`).
- Runtime state behavior for config/history/memory/plugin command paths.
- Runtime query interfaces used by maintainer commands (`src/query.rs`, `src/routing/query.rs`, `src/routing/inventory.rs`, `src/install/query.rs`).

## What This Crate Does Not Own

- Maintainer control-plane report assembly and maintainer workflow orchestration.
  Those live in `bijux-dev-cli` and are invoked through delegation from runtime query data.
- A second executable surface. `bijux-rs` remains a thin process entrypoint.

## Module Map

- `src/routing`: command catalog, parser, registry, contracts, schema, read-only routing query inventory.
- `src/config`: config domain validation, parsing/serialization, storage, and command service helpers.
- `src/plugin`: plugin discovery, manifest validation, registry operations, and diagnostics.
- `src/install`: compatibility paths, completion scripts, install diagnostics, and runtime identity query helpers.
- `src/repl`: session, completion, history, execution, and diagnostics for interactive mode.
- `src/query.rs`: state and parity/status query interfaces for maintainer delegation.
- `src/app.rs`: top-level route dispatch and command behavior glue.
- `src/kernel.rs`: execution-kernel contracts and lifecycle/exit mapping primitives.

## Runtime Invariants

- All command behavior must resolve through routing normalization before execution.
- Help, error envelopes, and output formatting must stay deterministic across repeated runs.
- Dev CLI runtime commands must delegate report formatting to `bijux-dev-cli` from query data, not assemble report text directly in routing/runtime internals.
- Process entrypoint remains thin: decode argv, call runtime, write streams, map exit code.

## Testing Shape

- Integration coverage lives under `crates/bijux-cli/tests`.
- Routing law coverage is consolidated under `crates/bijux-cli/tests/routing.rs` with fixtures in `crates/bijux-cli/tests/data/fixtures/routing`.
- Command-surface and parity behavior is enforced through `tests/bin_surface.rs`.
