# Benchmark scenarios and baselines

This directory holds benchmark scenario definitions and baseline artifacts.

- `scenarios/`: benchmark workload definitions
- `micro/`: crate-level microbenchmark metadata and harness notes
- `system/`: end-to-end system benchmark metadata
- `baselines/`: approved baseline reports and schemas

Canonical system scenarios:
- `scenarios/tiny_canonical.json`
- `scenarios/medium_canonical.json`
- `scenarios/wide_canonical.json`
- `scenarios/deep_canonical.json`
- `scenarios/cache_heavy_canonical.json`
- `scenarios/replay_canonical.json`

Battle benchmark scenarios:
- `scenarios/many_small_nodes_scheduler_overhead.json`
- `scenarios/manifest_trace_write_amplification.json`
- `scenarios/replay_verification_cost.json`
