# Observability and diagnostics

Audience: operators and maintainers.
Owner: runtime and operations teams.
Status: stable.

## Contracted event model

Runtime emits structured event categories for stable diagnostics:

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

Diagnostics kinds include: `validation`, `runtime-failure`, `policy-denial`, and `recovery-anomaly`.
Correlated identifiers span planner/scheduler/worker/artifact/audit streams.

## Event sinks and outputs

Supported sinks:

- local file
- stdout
- remote collector (contract placeholder)

Export formats:

- JSON file
- stdout JSON
- OTLP-compatible transport
- Prometheus text

Per-run artifacts may include:

- `observability.events.json`
- `observability.timeline.json`
- `observability.metrics.json`
- `observability.root-causes.json`
- `observability.graph-visualization.json`
- `observability.lineage-visualization.json`

## Product metrics and diagnostics

Core metrics:

- run makespan
- success ratio
- cache reuse ratio
- artifact volume
- scheduler queue depth
- scheduler dispatch latency

Diagnostic metrics:

- per-node queue delay and retries
- per-node output byte estimates
- scheduler starvation counters
- memory samples during output materialization
- process memory samples around output materialization

## Capacity and reliability model

Use these evidence-oriented metrics for capacity planning:

- queue depth and dispatch lag
- backend saturation and scheduler pressure
- storage growth and retention trends
- cost and resource envelopes for run families

Redaction and sampling controls apply to sensitive parameters, environment values, and metadata.

## Drift and regression evidence

- Drift reporting compares current and baseline metrics for replay-heavy and benchmark workloads.
- Resource baselines should be run from approved benchmark suites only.

Approximate measurements are acceptable only when explicitly labeled as such; authoritative measurements must reference controlled benchmark or runtime telemetry.

## Evidence locations and references

- Benchmark baseline procedures are tracked under `evidence/perf/` and benchmark execution tooling in `bijux-dev-dag`.
- Deep investigations should prefer:
  - `dag investigation-bundle`
  - `dag drift-report`
  - explicit evidence artifacts and diff reports

Normative output and evidence contracts are defined in `docs/spec/`.
