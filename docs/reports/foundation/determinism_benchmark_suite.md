# Determinism Benchmark Suite

Generated benchmark anchors for deterministic kernel behavior.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| repeated run equivalence check | 22 | 36 | advisory |
| scheduler deterministic ordering check | 11 | 18 | advisory |
| replay deterministic planning check | 17 | 28 | advisory |
| deterministic diff ordering check | 14 | 24 | advisory |
| deterministic explain ordering check | 12 | 20 | advisory |

## Notes

- Benchmarks are deterministic fixture-based checks.
- Drift thresholds are evaluated by trend deltas in release verification.
