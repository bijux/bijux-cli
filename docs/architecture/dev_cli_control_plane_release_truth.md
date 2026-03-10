# Dev CLI Control-Plane Release Truth

## Maintainer Dashboard Defaults

- Default maintainer dashboard: `bijux dev cli status`
- Default migration dashboard: `bijux dev cli parity`
- Default install/runtime truth command: `bijux dev cli runtime-identity`
- Default state truth command: `bijux dev cli state-audit`

## What Belongs In `bijux-dev-cli`

- Maintainer-facing report assembly for `bijux dev cli ...` commands.
- Control-plane orchestration that combines runtime query inputs into maintainer outputs.
- Evidence bundles and release-facing truth summaries for maintainer workflows.

## What Does Not Belong In `bijux-dev-cli`

- Runtime command law and end-user behavior contracts.
- Plugin registry mutation rules and install state mutation rules.
- Output rendering infrastructure shared by non-maintainer command surfaces.

## Canonical Ownership Freeze

`bijux-dev-cli` is the canonical control-plane crate for `bijux-cli` maintainer workflows.
Runtime crates remain the source of runtime law and read-only query data.
