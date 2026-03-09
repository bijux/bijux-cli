# Runtime Observability Benchmarks

Generated benchmark anchors for runtime telemetry and diagnostics.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| telemetry capture on successful run | 14 | 23 | advisory |
| telemetry capture on replay + diff | 21 | 35 | advisory |
| telemetry capture on prove + verify | 24 | 39 | advisory |
| failure-path diagnostics snapshot | 17 | 28 | advisory |
| cancellation-path diagnostics snapshot | 16 | 27 | advisory |
| partial-rerun diagnostics snapshot | 18 | 30 | advisory |

## Notes

- Benchmarks use deterministic local fixtures.
- Release checks evaluate trend regressions, not static absolute ceilings.
