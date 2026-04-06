# bijux-core-dev

Unified maintainer and governance control-plane for the `bijux-core` workspace.

## Scope

- Maintainer automation and diagnostics (`bijux-dev-cli` binary).
- Repository governance, contracts, evidence, and release verification (`bijux-dev-dag` binary).
- Shared control-plane reports, inventories, and suite orchestration.

## Layout

- `src/maintainer`: maintainer control-plane modules.
- `src/commands`, `src/suites`, `src/repo`, `src/report`: governance and evidence control-plane modules.
- `src/bin`: control-plane support binaries.
- `tests`: governance and maintainer contract suites.

## Non-goals

- End-user runtime command semantics (owned by `bijux-cli`).
- DAG semantic runtime ownership (owned by `bijux-dag-*` crates).
