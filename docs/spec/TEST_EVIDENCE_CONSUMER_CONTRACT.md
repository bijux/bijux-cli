# Test Evidence Consumer Contract

## Rule

Test code is a consumer of governed evidence assets. Test code does not define canonical scenario truth.

## Canonical ownership

- Authoring examples: `evidence/authoring/**`
- Battle scenarios: `evidence/battle/**`
- Cache and replay scenarios: `evidence/cache/**`
- Compatibility scenarios: `evidence/compat/**`
- Fault scenarios: `evidence/fault/**`
- Performance scenarios and baselines: `evidence/perf/**`
- Comparison scenarios and baselines: `evidence/compare/**`

## Forbidden ownership patterns

- Scenario assets under `tests/e2e/fixtures/**`
- Scenario assets under `tests/e2e/replay/fixtures/**`
- Scenario assets under `tests/e2e/compat/**`
- Scenario assets under `tests/e2e/container/**`
- Scenario assets under `benchmarks/scenarios/**`
- Scenario assets under `comparisons/scenarios/**`

## Enforcement surfaces

- `bijux-dev-dag verify evidence-ownership`
- `bijux-dev-dag verify evidence-drift`
- `bijux-dev-dag verify evidence-consumers`
- `bijux-dev-dag` contract and test suite id: `evidence-consumer-integrity`
