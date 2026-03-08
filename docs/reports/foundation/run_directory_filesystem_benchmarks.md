# Run Directory Filesystem Benchmarks

Generated benchmark anchors for run-directory filesystem behavior.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| run directory creation under concurrency | 6 | 12 | advisory |
| manifest and index atomic writes | 5 | 10 | advisory |
| corrupted run directory verification | 9 | 16 | advisory |
| migration compatibility check | 7 | 13 | advisory |
| portability normalization handling | 4 | 8 | advisory |

## Notes

- Metrics are deterministic fixture-based trend anchors.
- Release checks rely on drift deltas, not hard fixed limits.
