# Sandbox Security Benchmarks

Generated benchmark anchors for sandbox and isolation behavior.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| path authorization boundary check | 4 | 8 | advisory |
| symlink escape rejection check | 5 | 9 | advisory |
| environment shaping and denylist enforcement | 6 | 11 | advisory |
| container contract validation | 7 | 12 | advisory |
| sandbox policy denial reporting | 6 | 10 | advisory |

## Notes

- Metrics are deterministic fixture-based trend anchors.
- Release checks evaluate drift deltas instead of fixed ceilings.
