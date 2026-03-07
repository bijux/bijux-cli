# Performance Evidence

Purpose: workload scenarios and approved baselines for performance trust.

Subdirectories:
- `scenarios/`
- `baselines/`
- `fixtures/`

Classification policy:
- Mark a scenario `release_blocking: true` only when an enforced release gate consumes its threshold.
- Keep non-gating scenarios as advisory with explicit `threshold_owner` in `evidence/perf/metadata.json`.

See `CONTRACT.md` for performance-specific enforcement rules.
