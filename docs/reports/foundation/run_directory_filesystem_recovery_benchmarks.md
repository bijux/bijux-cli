# Run Directory Filesystem Recovery Benchmarks

Generated benchmark anchors for run-directory repair and recovery pathways.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| missing metadata recovery path | 8 | 14 | advisory |
| partial directory repair path | 10 | 18 | advisory |
| event-log corruption detection and report | 7 | 12 | advisory |
| node metadata corruption detection and report | 7 | 13 | advisory |
| post-corruption consistency verification | 11 | 19 | advisory |

## Notes

- Recovery benchmarks use deterministic corrupt-run fixtures.
- Trend drift alerts are used for regression detection.
