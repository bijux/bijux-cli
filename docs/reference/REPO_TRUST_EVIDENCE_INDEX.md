# Repository Trust Evidence Index

## Contracts
- planner, scheduler, storage, observability, security, invariants, comparison, adoption, anti-drift

## Tests and fixtures
- runtime contract tests
- app contract tests
- compatibility fixtures
- comparison scenarios
- integration fixtures

## Governance checks
- `bijux-dev-dag repo run --domain governance`
- `bijux-dev-dag verify evidence-ownership`
- `bijux-dev-dag verify evidence-drift`
- `bijux-dev-dag verify evidence-consumers`
- `bijux-dev-dag invariants-report`
- `bijux-dev-dag comparison-evidence-report`
- `bijux-dev-dag drift-dashboard`
- `bijux-dev-dag repo-trust-summary`

## Benchmarks
- benchmark scenarios and baselines in `evidence/perf/` and generated reports in `artifacts/benchmarks/`

## Drift controls
- anti-drift policy and dashboard tracking
