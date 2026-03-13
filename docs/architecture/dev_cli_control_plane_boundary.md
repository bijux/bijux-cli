# Dev CLI Control-Plane Boundary

`bijux-dev-cli` is the owner of maintainer control-plane automation and report assembly for `bijux dev cli ...`.

## Boundary Rules

1. `bijux-dev-cli` owns maintainer automation and maintainer-facing report assembly.
2. Runtime crates own runtime law and read-only runtime data services.
3. Runtime crates must not own maintainer workflow orchestration or maintainer-facing dashboard formatting.
4. `bijux dev cli ...` is the canonical maintainer command surface.
5. `bijux` remains the only canonical executable.
6. `bijux-dev-cli` is a workspace crate, not a second public runtime executable package.
7. `bijux-dev-cli` is not a second runtime law center; runtime law remains in runtime crates and command contracts.

## Allowed Ownership

- `dev cli routes`, `registry`, `env`, `contracts`, `parity`, and `status` report assembly
- `dev cli runtime-identity`, `package-health`, `state-audit`, and `state-doctor` report assembly
- `dev cli docs-audit`, `crate-health`, `inventory`, and release-facing evidence reports
- maintainer text and machine-readable envelopes for those report surfaces

## Disallowed Ownership

- core parser and routing law for non-`dev cli` command surfaces
- runtime state mutation behavior
- plugin runtime execution law
- shared stdout/stderr and exit-code policy for end-user commands
- direct replacement of the canonical `bijux` surface

## Release Truth

Release truth for maintainer workflows comes from `dev cli release *`.

- status and readiness claims come from `bijux dev cli release status` and `readiness`
- evidence claims come from `bijux dev cli release evidence`
- blocker and gap claims come from `bijux dev cli release gaps`

## Query Boundary

- `bijux-cli` remains the canonical entrypoint and dispatch host.
- Runtime crates expose read-only structured query interfaces for maintainer reports.
- Query interfaces are data-only and do not render maintainer dashboards.
- Runtime crates must not import `bijux-dev-cli`.

## Evidence

- `artifacts/status/dev_cli_dispatch_ownership_report.json`
- `artifacts/status/bin_entrypoint_responsibility_diff.json`
- `artifacts/status/runtime_dev_leakage_report.json`
