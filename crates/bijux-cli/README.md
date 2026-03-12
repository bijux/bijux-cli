# bijux-cli

`bijux-cli` is the core runtime crate for the `bijux` command surface.

It owns command law, routing, runtime state behavior, and process-facing execution
for the Rust runtime.

## What This Crate Owns

- Canonical command routing and normalization (`src/routing`).
- Runtime command execution behavior (`src/app.rs`).
- CLI process entrypoint helpers (`src/bootstrap/run.rs` and `src/bin/bijux.rs`).
- Execution-kernel primitives and exit mapping (`src/kernel.rs`).
- Runtime state behavior for config/history/memory/plugin command paths.
- Runtime query interfaces used by maintainer commands (`src/contracts/query.rs`, `src/features/diagnostics/routing_inventory.rs`, `src/features/install/query.rs`, and diagnostics query modules).

## What This Crate Does Not Own

- Maintainer control-plane report assembly and maintainer workflow orchestration.
  Those live in `bijux-dev-cli` and are invoked through external binary delegation.

## Module Map

- `src/routing`: command catalog, parser, and registry.
- `src/contracts`: durable command/runtime/plugin/config contracts plus schema helpers and schema inventory query.
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
- Dev CLI runtime commands must delegate to the `bijux-dev-cli` executable, not assemble maintainer report payloads inside runtime routing internals.
- Process entrypoint remains thin: decode argv, call runtime, write streams, map exit code.

## Testing Shape

- Integration coverage lives under `crates/bijux-cli/tests`.
- Routing law coverage is consolidated under `crates/bijux-cli/tests/routing.rs` with fixtures in `crates/bijux-cli/tests/data/fixtures/routing`.
- Command-surface and parity behavior is enforced through `tests/bin_surface.rs`.
