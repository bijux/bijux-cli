# Observability Contract

## Scope

Defines operator-facing observability guarantees for runtime runs.

## Layers

- logs: structured event records
- metrics: typed run/node/scheduler metrics
- traces: timeline and attempt-level state transitions

## Required Runtime Events

Required event names:

- `run_started`
- `node_ready`
- `node_scheduled`
- `node_started`
- `node_attempt_started`
- `node_attempt_finished`
- `node_failed`
- `run_finished`

Required event fields:

- `name`
- `unix_ms`
- `run_id`
- `category`

## Required Metrics

Scheduler metrics:

- queue depth
- ready count
- running count
- completed count
- retry count
- cache hits
- cache misses
- failure count
- dispatch latency
- concurrency pressure

Run metrics:

- makespan
- success ratio
- parallelism utilization
- cache reuse ratio
- artifact volume
- planning duration
- scheduling wait duration
- execution duration
- trace write duration
- manifest finalize duration
- replay compare duration

## Timeline and Debug Artifacts

- `observability.timeline.json` is required for completed and failed runs.
- `observability.events.json` is required for completed and failed runs.
- `observability.root-causes.json` is required for failed runs.

## Secret Redaction

Observability payloads must not include raw secret/token/password values in
public runtime event details.

## Contract Checks

- Event name and required field checks are enforced in runtime tests.
- Control-plane suite `observability-contract` validates docs/test alignment.

## Operator vs Developer Surfaces

Operator and developer observability surface split is tracked in
`docs/tracking/OBSERVABILITY_SURFACE_PLAN.md`.
