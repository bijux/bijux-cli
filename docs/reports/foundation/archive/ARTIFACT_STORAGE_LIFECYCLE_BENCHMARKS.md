# Artifact Storage Lifecycle Benchmarks

Generated benchmark anchors for artifact storage lifecycle behavior.

## Scenarios

| scenario | p50_ms | p95_ms | status |
| --- | ---: | ---: | --- |
| create -> store -> retrieve roundtrip | 8 | 14 | advisory |
| retention + gc planning on lineage snapshot | 12 | 19 | advisory |
| checksum verification on artifact index set | 11 | 17 | advisory |
| partial-write recovery marker handling | 7 | 12 | advisory |
| corruption detection for output index mismatch | 9 | 15 | advisory |

## Notes

- Benchmarks are deterministic fixture-based measurements.
- Release checks evaluate trend drift, not hard fixed ceilings.
