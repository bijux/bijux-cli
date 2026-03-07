# Performance Evidence

Purpose: workload scenarios and approved baselines for performance trust.

Subdirectories:
- `scenarios/`
- `baselines/`
- `fixtures/`

Classification policy:
- Mark scenarios as `core`, `advisory`, or `experimental` in metadata.
- Keep the release-relevant set intentionally small and explicitly listed in metadata.
- Mark a scenario `release_blocking: true` only when a threshold reference is enforced.
- Keep exploratory scenarios as advisory or experimental and never claim release proof from them.

See `CONTRACT.md` for performance-specific enforcement rules.
