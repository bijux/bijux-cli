# Observability model

## Structured event contract

Runtime emits structured events under these stable categories:

- `plan`
- `schedule`
- `dispatch`
- `start`
- `retry`
- `timeout`
- `cache_hit`
- `cache_miss`
- `failure`
- `replay`
- `verify`

Additional diagnostic contracts:

- diagnostics kinds: validation, runtime-failure, policy-denial, recovery-anomaly
- stable machine-readable failure causes
- correlation IDs spanning planner/scheduler/worker/artifact/audit events
- replay span-link records for retry/replay ancestry

## Event sinks

Runtime event sinks share one contract and currently support:

- local file sink
- stdout sink
- remote collector sink (stub contract for future transport)

Metrics export formats:

- JSON file
- stdout JSON
- OTLP-compatible transport contract
- Prometheus text contract

## Metric families

Product metrics:

- run makespan
- success ratio
- cache reuse ratio
- artifact volume
- scheduler queue depth
- scheduler dispatch latency

Debug-only metrics:

- per-node queue delay
- per-node retries
- per-node output byte estimates
- scheduler starvation counters
- process memory samples around output materialization

## Timeline and visualization exports

Each run may emit:

- `observability.events.json`
- `observability.timeline.json`
- `observability.metrics.json`
- `observability.root-causes.json`
- `observability.graph-visualization.json`
- `observability.lineage-visualization.json`

These artifacts are intended for CLI reports and future UI rendering without format drift.

## Explainability commands

`bijux-dev-dag` provides:

- `dag explain-run --run-dir <dir>`
- `dag explain-node --run-dir <dir> --node-id <id>`
- `dag explain-artifact --run-dir <dir> --artifact-id <id>`
- `dag explain-schedule --run-dir <dir> --schedule-id <id>`

## Investigation and drift analysis

- `dag investigation-bundle --run-dir <dir> --run-id <id>` collects key evidence pointers.
- `dag drift-report --current-metrics <file> --baseline-metrics <file> --dag-name <name> --baseline-name <name>` reports metric drift.

Fixture examples:

- `benchmarks/fixtures/observability/retry_cancel_cache_failure.json`
- `benchmarks/fixtures/observability/investigation_bundle_demo.json`

## Redaction and sampling

- Redaction policy supports removing sensitive params/env/metadata keys.
- Sampling policy defines max spans/events for large run traces.

## Demo DAG

Observability demonstrations are sourced from benchmark observability fixtures and governed evidence scenarios.
