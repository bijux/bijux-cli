# Large DAG Scalability Benchmarks

Generated benchmark anchors for large DAG behavior.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| planner on 1,000-node DAG | 96 | 152 | advisory |
| planner on 10,000-node DAG | 1380 | 2140 | advisory |
| scheduler on large fan-out DAG | 121 | 188 | advisory |
| scheduler on large fan-in DAG | 129 | 201 | advisory |
| replay planning under large DAG | 177 | 281 | advisory |
| diff under large DAG | 205 | 328 | advisory |

## Notes

- Metrics use deterministic synthetic fixtures.
- Release policy evaluates trend changes, not absolute cutoffs.
