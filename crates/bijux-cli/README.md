# bijux-cli

`bijux-cli` is the Rust runtime crate behind the `bijux` executable.

## Scope

- Own command parsing, normalization, registry lookup, and execution.
- Own runtime-facing state behavior for config, history, memory, install diagnostics, plugins, and the REPL.
- Expose read-only query APIs used by maintainer tooling.
- Do not assemble maintainer reports; `bijux-dev-cli` owns that surface directly.

## Source Layout

- `src/api`: stable entrypoints used by the binary, tests, and the Python bridge.
- `src/bootstrap`: process wiring and exit-code handling.
- `src/contracts`: durable command, envelope, config, plugin, and query types.
- `src/features`: domain implementations for config, diagnostics, history, install, memory, and plugins.
- `src/infrastructure`: filesystem, process, environment, and state-store adapters.
- `src/interface`: CLI and REPL surfaces.
- `src/kernel`: execution pipeline and policy resolution.
- `src/routing`: command catalog, parser, and registry.
- `src/shared`: small cross-cutting helpers.

## Runtime Rules

- Commands are parsed and normalized before execution.
- Help, envelopes, and output formatting stay deterministic across repeated runs.
- Maintainer commands stay outside the runtime binary; this crate does not parse or execute `bijux-dev-cli` surfaces.
- The process entrypoint stays thin: decode argv, call the runtime, write streams, map exit codes.

## Tests

- `tests/architecture.rs`: boundary and ownership checks.
- `tests/integration.rs`: command behavior, parity, resilience, and REPL coverage.
- `tests/routing.rs`: parser, registry, schema, and routing law coverage.
- `tests/data/fixtures` and `tests/data/golden`: stable fixtures and snapshots.
