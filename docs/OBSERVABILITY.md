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

## Event sinks

Runtime event sinks share one contract and currently support:

- local file sink
- stdout sink
- remote collector sink (stub contract for future transport)

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
