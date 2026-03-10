# Dev CLI Control-Plane Boundary

`bijux-dev-cli` is the owner of maintainer control-plane automation and report assembly for `bijux dev cli ...`.

## Boundary Rules

1. `bijux-dev-cli` owns maintainer automation and maintainer-facing report assembly.
2. Runtime crates (`bijux-cli`, `bijux-cli-routing`, `bijux-cli::install`, `bijux-cli-plugin`, `bijux-cli-output`) own runtime law and structured-data services.
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
- `bijux-cli` delegates these maintainer presentations to `bijux-dev-cli`.

## Env Contracts Parity Status Ownership Freeze

- `dev cli env` report assembly is owned by `bijux-dev-cli`.
- `dev cli contracts` report assembly is owned by `bijux-dev-cli`.
- `dev cli parity` report assembly is owned by `bijux-dev-cli`.
- `dev cli status` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli` only passes runtime-resolved inputs and delegates these report builds.

## Runtime Identity State Ownership Freeze

- `dev cli runtime-identity` report assembly is owned by `bijux-dev-cli`.
- `dev cli package-health` report assembly is owned by `bijux-dev-cli`.
- `dev cli state-audit` report assembly is owned by `bijux-dev-cli`.
- `dev cli state-doctor` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli` provides low-level state and install diagnostics inputs only.

## Script Docs Crate Health Ownership Freeze

- `dev cli inventory` and `dev cli script-audit` inventory assembly are owned by `bijux-dev-cli`.
- `dev cli docs-audit` report assembly is owned by `bijux-dev-cli`.
- `dev cli crate-health` report assembly is owned by `bijux-dev-cli`.
- `bijux-cli` delegates these report surfaces and does not shape their presentation payloads.

## Operational Notes

- `bijux-cli` remains the canonical entrypoint and dispatch host.
- `bijux-dev-cli` will become the implementation owner of maintainer workflows behind `bijux dev cli ...`.
- Runtime crates should expose minimal read-only query data needed by `bijux-dev-cli`.

## Bin And Routing Ownership Freeze

- `bijux-cli` owns process entrypoint concerns only: argv decoding, `run_app()` invocation, stream writes, and process exit code.
- `bijux-cli` does not implement maintainer workflow routing branches or maintainer payload formatting.
- `bijux-cli-routing` owns command identity, normalization, and route resolution only.
- `bijux-cli-routing` does not own maintainer report assembly or maintainer dashboard formatting.
- Dispatch ownership evidence must be generated at:
  - `artifacts/status/dev_cli_dispatch_ownership_report.json`
  - `artifacts/status/bin_entrypoint_responsibility_diff.json`

## Runtime Query Interface Freeze

- Runtime crates expose read-only structured query interfaces for maintainer reports:
  - `bijux-cli-routing`: route and registry inventory, contracts schema inventory.
  - `bijux-cli::install`: runtime identity diagnostics query.
  - `bijux-cli`: state diagnostics and parity/status artifact availability query.
- Query interfaces are data-only and must not render text or assemble maintainer dashboards.
- Runtime crates other than `bijux-cli` must not import `bijux-dev-cli`.
- The query interface layer is the bridge between runtime data and maintainer report assembly.

## Runtime Dev Leakage Rule

- Runtime crates are audited for maintainer-workflow leakage at:
  - `artifacts/status/runtime_dev_leakage_report.json`
- The expected steady state is zero leakage score across runtime crates.
- Any remaining leakage must be explicitly justified and tracked before release.
