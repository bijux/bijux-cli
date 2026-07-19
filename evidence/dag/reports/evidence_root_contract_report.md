# Evidence Root Contract Report

Date: 2026-03-07
Owner: bijux-dev-dag

## Contract result
The repository enforces `evidence/` as the sole root pillar for executable proof assets.

## Verified controls
- Evidence metadata completeness: `bijux-dev-dag verify evidence-ownership`
- Legacy-root drift freeze: `bijux-dev-dag verify evidence-drift`
- Consumer-path integrity: `bijux-dev-dag verify evidence-consumers`
- Domain governance checks:
  - `verify evidence-authoring`
  - `verify evidence-battle`
  - `verify evidence-cache`
  - `verify evidence-compat`
  - `verify evidence-fault`
  - `verify evidence-perf`
  - `verify evidence-compare`

## Root contract statement
- Proof assets are governed under `evidence/authoring`, `evidence/battle`, `evidence/cache`, `evidence/compat`, `evidence/fault`, `evidence/operator`, `evidence/perf`, and `evidence/compare`.
- Retired roots `examples/`, `benchmarks/`, and `comparisons/` are forbidden by policy and layout guardrails.

## Execution entrypoint
`make evidence-all`
