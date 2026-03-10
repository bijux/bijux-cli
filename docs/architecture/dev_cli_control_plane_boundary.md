# Dev CLI Control-Plane Boundary

`bijux-dev-cli` is the owner of maintainer control-plane automation and report assembly for `bijux dev cli ...`.

## Boundary Rules

1. `bijux-dev-cli` owns maintainer automation and maintainer-facing report assembly.
2. Runtime crates (`bijux-cli-core`, `bijux-cli-routing`, `bijux-cli-install`, `bijux-cli-plugin`, `bijux-cli-output`) own runtime law and structured-data services.
3. Runtime crates must not own maintainer workflow orchestration or maintainer-facing dashboard formatting.
4. `bijux dev cli ...` is the canonical maintainer command surface.
5. `bijux` remains the only canonical executable.
6. `bijux-dev-cli` is a workspace crate, not a second public runtime executable package.
7. `bijux-dev-cli` is not a second runtime law center; runtime law remains in runtime crates and command contracts.

## Current-Reality Boundary Freeze

Before extraction starts, maintainers must generate and review:

- `artifacts/status/dev_cli_owned_behaviors_inventory.json`
- `artifacts/status/runtime_owned_behaviors_inventory.json`
- `artifacts/status/misplaced_dev_behaviors_report.json`

These files represent the frozen baseline for extraction. Extraction work must reduce misplaced maintainer behavior while preserving command surface and output contracts.

## Route and Registry Ownership Freeze

- `dev cli routes` presentation assembly is owned by `bijux-dev-cli`.
- `dev cli registry` presentation assembly is owned by `bijux-dev-cli`.
- `bijux-cli-routing` exposes read-only query data for route and registry inventory.
- `bijux-cli-core` delegates these maintainer presentations to `bijux-dev-cli`.

## Env Contracts Parity Status Ownership Freeze

- `dev cli env` report assembly is owned by `bijux-dev-cli`.
- `dev cli contracts` report assembly is owned by `bijux-dev-cli`.
- `dev cli parity` report assembly is owned by `bijux-dev-cli`.
- `dev cli status` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli-core` only passes runtime-resolved inputs and delegates these report builds.

## Runtime Identity State Ownership Freeze

- `dev cli runtime-identity` report assembly is owned by `bijux-dev-cli`.
- `dev cli package-health` report assembly is owned by `bijux-dev-cli`.
- `dev cli state-audit` report assembly is owned by `bijux-dev-cli`.
- `dev cli state-doctor` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli-core` provides low-level state and install diagnostics inputs only.

## Script Docs Crate Health Ownership Freeze

- `dev cli inventory` and `dev cli script-audit` inventory assembly are owned by `bijux-dev-cli`.
- `dev cli docs-audit` report assembly is owned by `bijux-dev-cli`.
- `dev cli crate-health` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli-core` delegates these report surfaces and does not shape their presentation payloads.

## Operational Notes

- `bijux-cli-bin` remains the canonical entrypoint and dispatch host.
- `bijux-dev-cli` will become the implementation owner of maintainer workflows behind `bijux dev cli ...`.
- Runtime crates should expose minimal read-only query data needed by `bijux-dev-cli`.
