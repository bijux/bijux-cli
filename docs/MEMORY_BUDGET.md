# Memory budget smoke checks (provisional)

- Smoke command: `cargo run -p bijux-dev-dag -- memory-smoke`
- Output artifact: `artifacts/memory/smoke.json`
- Runtime emits materialization memory sampling in run artifacts:
  - `observability.metrics.json` with `before_materialization_bytes` and `after_materialization_bytes`.

This smoke check is provisional and records early runtime timing and memory signals.
It is not a release guarantee until strict measured memory gates are enforced.
