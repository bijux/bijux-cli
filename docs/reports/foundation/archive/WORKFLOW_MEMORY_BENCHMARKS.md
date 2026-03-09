# Workflow Memory Benchmarks

Generated benchmark anchors for workflow memory footprint.

## Scenarios

| scenario | rss_mb_p50 | rss_mb_p95 | status |
| --- | ---: | ---: | --- |
| validate -> plan -> run | 45 | 61 | advisory |
| validate -> plan -> run -> diff | 49 | 67 | advisory |
| validate -> plan -> run -> prove | 51 | 69 | advisory |
| export -> import -> replay | 56 | 76 | advisory |
| corrupted workflow path diagnostics | 40 | 58 | advisory |
| partial recovery workflow | 47 | 64 | advisory |

## Notes

- Memory sampling is best-effort and platform-dependent.
- Values are used for trend detection and anomaly alerts.
