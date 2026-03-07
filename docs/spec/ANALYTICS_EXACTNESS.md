# Analytics Exactness Model

## Exact analytics
- `runs compare` field-by-field values copied from run summaries.
- `runs failures` counts derived from recorded trace failure kinds.
- `runs summary` totals (run count, retries, cache hits, artifact counts).

## Heuristic analytics
- `runs flakes` as status divergence grouped by graph fingerprint.
- trend interpretation over incomplete history.
- determinism and replay signals inferred from observed retries and outcomes.

## Interpretation boundary
Heuristic outputs are indicators for investigation and are not formal correctness proofs.
