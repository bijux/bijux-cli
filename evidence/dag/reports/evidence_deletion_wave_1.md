# Evidence Deletion Wave 1

Date: 2026-03-07

## Removed Assets

- `benchmarks/fixtures/infrastructure/backends/backend_degradation.json`
- `benchmarks/fixtures/infrastructure/backends/partial_cluster_outage.json`
- `benchmarks/fixtures/infrastructure/backends/queue_buildup.json`
- `comparisons/external/argo_notes.md`
- `comparisons/external/dagster_notes.md`
- `comparisons/external/prefect_notes.md`
- `comparisons/external/coverage_map.json`
- `examples/observability-gold-standard.dag.json`

## Why Removed

- Backend benchmark fixture family under `benchmarks/fixtures/infrastructure/backends/*` did not represent governed release-proof evidence.
- External comparison notes and coverage map were note artifacts, not executable comparison evidence.
- The observability gold-standard example was aspirational and not maintained as governed executable proof.

## Governance Locks Added

`configs/dag/policy/evidence_governance.json` now forbids reintroduction of removed categories via `forbidden_globs`:

- `benchmarks/fixtures/distributed/*`
- `benchmarks/fixtures/infrastructure/backends/*`
- `benchmarks/fixtures/scheduling/enterprise/*`
- `benchmarks/fixtures/scheduling/ha/*`
- `comparisons/external/*`
- `examples/observability-gold-standard.dag.json`

## Metadata Validation Outcome

- Evidence ledger no longer contains removed asset paths.
- Evidence governance contract now fails if forbidden paths reappear.
