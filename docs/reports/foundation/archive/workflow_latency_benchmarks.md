# Workflow Latency Benchmarks

Generated benchmark anchors for command-level workflow integrity.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| validate -> plan -> run | 62 | 99 | advisory |
| validate -> plan -> run -> inspect | 74 | 118 | advisory |
| validate -> plan -> run -> replay | 95 | 152 | advisory |
| export -> import -> inspect | 83 | 129 | advisory |
| history -> inspect -> explain | 58 | 90 | advisory |
| artifact hash -> inspect -> trace | 39 | 66 | advisory |

## Notes

- Metrics use deterministic local fixtures.
- Release checks evaluate drift over time, not absolute hard caps.
