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

- `cli/`: argument parsing, workspace discovery, command dispatch, and route handlers.
- `contracts/`: status inventories/runners and maintenance inventories/compliance/generators.
- `suites/`: control-plane/runtime/quality/resilience execution suites for status contracts.
- `reports/`: maintainer report builders grouped by business domain.
- `infra/`: filesystem/process/clock adapters used by reports and contracts.
- `schema/`: command registry and report envelope schema primitives.

`cli/routes/` is split by command families:

- `root.rs`
- `maintenance.rs`
- `release.rs`
- `evidence.rs`
- `config.rs`
- `python.rs`
- `rustdoc.rs`

`contracts/status/` uses a stable split:

- `model.rs`
- `inventory.rs`
- `run.rs`

## Architecture Rules

- `crates/*/src` path depth must stay `<= 7`.
- workspace root legacy shell directory is forbidden.
- suites must not contain `*_executor.rs` or `*_spec.rs` file names.
- legacy `app`, `platform`, `infrastructure`, `status_contracts`, and `contracts/native` namespaces are removed.

Enforcement is in:

- [`tests/architecture/layout_contracts.rs`](./tests/architecture/layout_contracts.rs)
- [`tests/architecture/ownership_boundaries.rs`](./tests/architecture/ownership_boundaries.rs)
- [`tests/architecture/depth_limit.rs`](./tests/architecture/depth_limit.rs)
