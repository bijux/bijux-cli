# DAG Memory Footprint Regression Benchmarks

Generated memory regression anchors for large DAG workloads.

## Scenarios

| scenario | rss_mb_p50 | rss_mb_p95 | status |
| --- | ---: | ---: | --- |
| validate + plan on 1,000-node DAG | 92 | 130 | advisory |
| validate + plan on 10,000-node DAG | 602 | 821 | advisory |
| runtime execution bookkeeping on deep chain | 111 | 159 | advisory |
| artifact indexing on large run output | 138 | 206 | advisory |
| provenance traversal over large lineage graph | 97 | 143 | advisory |

## Notes

- Memory figures are fixture-oriented trend anchors.
- Drift alerts trigger on slope and step-change anomalies.
