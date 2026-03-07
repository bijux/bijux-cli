# Shallow Evidence Audit (2026-03-07)

## Scope
- Benchmarks fixtures and scenarios
- Comparison notes and scenarios
- Examples
- Top-level tests and fixture families

## Findings

### Benchmarks (Item 21)
- `benchmarks/fixtures/scheduling/enterprise/*` had docs-only references and no scenario execution linkage.
- `benchmarks/fixtures/scheduling/ha/*` had docs-only references and no scenario execution linkage.
- `benchmarks/fixtures/distributed/*` had docs-only references and no scenario execution linkage.
- Action: removed these fixture files and removed stale doc references.

### Comparisons (Items 22, 26, 33)
- External note files existed without machine-readable linkage to executable bijux scenarios.
- Action: added `comparisons/external/coverage_map.json` linking each note to scenario ids and the bijux baseline file.

### Examples (Item 23)
- Example set was executable and owned; no duplicate payload hashes were detected.
- Action: no deletion in this pass; `evidence/authoring/examples/hello.dag.json` remains flagged `move` in governance ledger.

### Top-level tests (Item 24)
- Top-level fixture families are active and consumed by contracts.
- Action: no fixture deletion in this pass; duplicate families remain explicitly marked in governance metadata.

## Deletions Applied
- `benchmarks/fixtures/distributed/transport_protocol_simulation.json`
- `benchmarks/fixtures/distributed/worker_lifecycle_simulation.json`
- `benchmarks/fixtures/scheduling/enterprise/load_spike.json`
- `benchmarks/fixtures/scheduling/enterprise/mass_backfill.json`
- `benchmarks/fixtures/scheduling/enterprise/trigger_storm.json`
- `benchmarks/fixtures/scheduling/ha/cold_restart_objective.json`
- `benchmarks/fixtures/scheduling/ha/split_brain_failover.json`
- `benchmarks/fixtures/scheduling/ha/trigger_storm_rebalance.json`
- `evidence/perf/scenarios/replay_path.json` (removed earlier in governance pass)

## Remaining Cleanup Queue
- Candidate docs-only benchmark fixtures under `benchmarks/fixtures/observability`, `benchmarks/fixtures/recovery`, and selected infrastructure backends.
- Runtime fixture families still marked `duplicate` in governance ledger should be consolidated into canonical battle/compat/fault families.
