# Distributed Execution Benchmarks

Generated benchmark anchors for distributed worker execution pathways.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| worker registration handshake | 5 | 9 | advisory |
| dispatch + completion report cycle | 13 | 22 | advisory |
| retry scheduling after timeout | 17 | 28 | advisory |
| artifact upload + checksum verify | 21 | 35 | advisory |
| artifact download + checksum verify | 19 | 33 | advisory |
| network failure fallback detection | 11 | 20 | advisory |

## Notes

- Metrics are deterministic fixture-based trend anchors.
- Release checks evaluate drift deltas instead of fixed thresholds.
