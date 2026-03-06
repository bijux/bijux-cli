# Memory budget smoke checks

- Smoke command: `cargo run -p bijux-dev-dag -- memory-smoke`
- Output artifact: `artifacts/memory/smoke.json`
- Runtime emits materialization memory sampling in run artifacts:
  - `observability.metrics.json` with `before_materialization_bytes` and `after_materialization_bytes`.

The smoke check records runtime execution timing and links memory budget enforcement to CI runner metrics and regression alerts.
