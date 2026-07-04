# bijux-dev

`bijux-dev` is the repository-internal maintainer and governance control plane
for `bijux-core`.

## What this crate provides

- Maintainer automation and diagnostics through the `bijux-dev-cli` binary.
- Repository governance, contracts, evidence, and release verification flows.
- Shared reports, inventories, and suite orchestration used by repository gates.

## Publication status

`bijux-dev` is intentionally not published. It exists to validate and release
the repository products, not to act as a public runtime dependency.

## Layout

- `src/maintainer`: maintainer control-plane modules.
- `src/commands`, `src/suites`, `src/repo`, `src/report`: governance and evidence control-plane modules.
- `src/bin`: control-plane support binaries.
- `tests`: governance and maintainer contract suites.

## Non-goals

- End-user runtime command semantics (owned by `bijux-cli`).
- DAG semantic runtime ownership (owned by `bijux-dag-*` crates).

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [Maintainer handbook](https://bijux.io/bijux-core/bijux-dev/)
