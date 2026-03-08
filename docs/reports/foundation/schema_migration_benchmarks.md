# Schema Migration Benchmarks

Generated benchmark anchors for migration and compatibility checks.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| `dag migrate dag --dry-run` v0.1 -> v0.1 | 7 | 11 | advisory |
| `dag migrate run --dry-run` v0.1 -> v0.1 | 6 | 10 | advisory |
| compatibility fixture parse: graph/run/artifact/proof | 4 | 7 | advisory |
| forward-version rejection check | 3 | 6 | advisory |

## Notes

- Benchmarks are based on deterministic local fixtures.
- Use trend deltas for regression detection.
