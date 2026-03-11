# bijux-dev-cli

`bijux-dev-cli` is the maintainer control-plane crate for `bijux dev cli ...` command workflows.

## Scope

- Owns maintainer-facing automation orchestration.
- Owns maintainer-facing report assembly.
- Keeps runtime command law and runtime state mutation rules in runtime crates.

## Non-Goals

- Defining runtime command law.
- Becoming a second executable.
- Replacing the canonical `bijux` binary entrypoint.

## Source Layout

`src/` is organized by stable ownership boundaries:

- `app/`: argument parsing, workspace discovery, route handling entrypoints.
- `contracts/`: status/maintenance contract catalogs and native executors.
- `reports/`: maintainer report builders grouped by business domain.
- `platform/`: shared command registry and report envelope primitives.
- `infrastructure/`: filesystem/process adapters used by report and contract code.
- `status_contracts/`: status contract inventory and runner services.

`contracts/native/` uses suite-oriented folders:

- `control_plane/`
- `runtime/`
- `resilience/`
- `quality/`

Each suite owns:

- `runner.rs`: execution dispatch for contract IDs in that suite.
- `catalog.rs`: contract inventory rows for that suite.
- `*_executor.rs` and `*_spec.rs`: concrete contract behavior and catalog entries.

## Architecture Rules

- `crates/*/src` path depth must stay `<= 7`.
- workspace root legacy shell directory is forbidden.
- single-file report modules must be `reports/<name>.rs` (no one-file `<name>/mod.rs` directories).
- legacy `contracts/maintenance/native` is removed; native suites live under `contracts/native`.

Enforcement is in [`tests/module_layout_contracts.rs`](./tests/module_layout_contracts.rs).
