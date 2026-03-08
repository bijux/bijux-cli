# Backend Equivalence Performance Benchmarks

Generated benchmark anchors for backend equivalence and portability diagnostics.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| equivalence-proof local vs local-replay | 19 | 33 | advisory |
| equivalence-proof local vs kubernetes | 24 | 41 | advisory |
| equivalence-proof local vs hpc | 23 | 39 | advisory |
| equivalence-proof local vs remote | 26 | 44 | advisory |
| semantic-portability backend query | 4 | 8 | advisory |

## Notes

- Values are deterministic fixture replay measurements, not production load measurements.
- The release gate uses trend deltas, not absolute timing cutoffs.
