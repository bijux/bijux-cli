# Benchmark Raw Data Retention

## Purpose
Define retention policy for benchmark raw outputs and derived summaries.

## Retention requirements
- Raw benchmark reports must remain available for every published benchmark claim.
- Derived reports and scorecards must reference the raw report locations.
- Raw data must be stored under committed evidence or reproducible artifact paths.

## Required link targets
- `evidence/perf/baselines/`
- `artifacts/benchmarks/` (when produced in CI)
- `evidence/reports/` for derived scorecards and comparisons

## Deletion policy
Raw reports may be compacted only when a successor baseline is committed and references are updated.
