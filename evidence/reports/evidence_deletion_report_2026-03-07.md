# Evidence Deletion Report (2026-03-07)

## Before vs After File Counts
| Root | Before | After | Delta |
|---|---:|---:|---:|
| benchmarks | 56 | 48 | -8 |
| comparisons | 14 | 17 | 3 |
| examples | 7 | 7 | 0 |
| tests | 60 | 57 | -3 |

## Removed Files
- benchmarks/scenarios/replay_path.json
- benchmarks/fixtures/distributed/transport_protocol_simulation.json
- benchmarks/fixtures/distributed/worker_lifecycle_simulation.json
- benchmarks/fixtures/scheduling/enterprise/load_spike.json
- benchmarks/fixtures/scheduling/enterprise/mass_backfill.json
- benchmarks/fixtures/scheduling/enterprise/trigger_storm.json
- benchmarks/fixtures/scheduling/ha/cold_restart_objective.json
- benchmarks/fixtures/scheduling/ha/split_brain_failover.json
- benchmarks/fixtures/scheduling/ha/trigger_storm_rebalance.json

## Rationale
- Removed scenario/fixture files without active executable measurement linkage in current benchmark contracts.
- Consolidated replay benchmark semantics under benchmarks/scenarios/replay_canonical.json.
- Reduced speculative enterprise/HA/distributed benchmark fixture surfaces to lower evidence noise.
