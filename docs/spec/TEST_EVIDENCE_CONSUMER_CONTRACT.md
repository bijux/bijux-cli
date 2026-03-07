# Test Evidence Consumer Contract

## Rule

Test code is a consumer of governed evidence assets. Test code does not define canonical scenario truth.
Evidence consumers must resolve assets through typed access helpers, not direct filesystem reads of the registry.
Evidence consumption is read-only; tests must not mutate files under `evidence/`.

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
- `bijux-dev-dag repo evidence-resolve-by-id --id <asset-id>`
- `bijux-dev-dag repo evidence-resolve-by-family --family <family>`
- `bijux-dev-dag repo evidence-resolve-by-trust-property --trust-property <trust-id>`
- `bijux-dev-dag repo evidence-resolve-by-consumer --consumer <consumer-id>`
- `bijux-dev-dag` contract and test suite id: `evidence-consumer-integrity`
- `bijux-dag-testkit` evidence access helpers:
  - `load_evidence_registry_checked`
  - `resolve_evidence_asset_by_id_checked`
  - `evidence_asset_ids`

## Consumer mapping reports

- `evidence/reports/evidence_consumers_inventory.md`
- `evidence/reports/evidence_assets_to_consumers.md`
- `evidence/reports/evidence_consumers_to_families.md`
- `evidence/reports/evidence_consumption_by_crate.md`
