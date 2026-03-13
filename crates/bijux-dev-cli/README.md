# bijux-dev-cli

`bijux-dev-cli` is the maintainer control-plane crate for `bijux dev cli ...`.

## Scope

- Own maintainer automation, report assembly, and release/docs support commands.
- Keep runtime command law and runtime state mutation rules in runtime crates.
- Ship a separate maintainer binary that the runtime can delegate to when needed.

## Non-Goals

- Defining end-user runtime command behavior.
- Replacing the canonical `bijux` binary entrypoint.
- Reimplementing runtime state rules that already live in `bijux-cli`.

## Source Layout

- `src/cli`: argument parsing, workspace discovery, dispatch, and route handlers.
- `src/contracts`: maintenance and status inventories, models, and runners.
- `src/infra`: filesystem, process, and clock adapters.
- `src/reports`: structured maintainer reports grouped by domain.
- `src/runtime`: maintainer entrypoint and execution helpers.
- `src/schema`: report and registry data types.
- `src/suites`: composed control-plane, quality, resilience, and runtime checks.

## Tests

- `tests/architecture`: layout, depth, and ownership boundaries.
- `tests/contracts`: status inventory and suite catalog contracts.
- `tests/e2e`: route-level maintainer behavior.

## Constraints

- `crates/*/src` path depth stays `<= 7`.
- legacy namespace patterns removed by architecture checks stay removed.
- suites do not use `*_executor.rs` or `*_spec.rs` names.
