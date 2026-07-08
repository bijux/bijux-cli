# bijux-dev

`bijux-dev` is the repository-internal maintainer and governance control plane
for `bijux-core`.

## Release Status

- repository-internal crate
- intentionally not published
- exists to validate, document, and release the public products

## What This Crate Owns

- maintainer automation and diagnostics through `bijux-dev-cli`
- repository governance, contracts, evidence, and release verification flows
- shared reports, inventories, and suite orchestration used by repository gates

## Typical Entry Points

- `bijux-dev-cli` for maintainer automation and diagnostics
- `bijux-dev-dag` for repository-owned DAG governance and release surfaces
- `bijux-dev-cli docs write-dag-cli-reference` for rewriting the checked-in DAG
  CLI reference pages without relying on ignored tests

## Layout

- `src/maintainer`: maintainer control-plane modules
- `src/commands`, `src/suites`, `src/repo`, `src/report`: governance and
  evidence control-plane modules
- `src/bin`: control-plane support binaries
- `tests`: governance and maintainer contract suites

## Reach For Another Surface When

- you need end-user runtime command semantics: `bijux-cli`
- you need DAG execution behavior or artifact semantics: `bijux-dag-*`
- you need repository handbook guidance instead of control-plane code:
  `docs/bijux-dev/`

## Non-Goals

- end-user runtime command semantics
- DAG semantic runtime ownership

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [Maintainer handbook](https://bijux.io/bijux-core/bijux-dev/)
