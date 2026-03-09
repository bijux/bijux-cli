# Run Summary Schema v0.1

The run summary object is emitted in run manifests under `run_summary`.

Fields:

- `total_nodes` (u32)
- `success` (u32)
- `failed` (u32)
- `skipped` (u32)
- `cached` (u32)

These counters are advisory aggregation surfaces and do not replace per-node trace truth.
