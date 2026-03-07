# Resource profile strategy

## Resource dimensions

Resource evidence tracks:

- wall time
- CPU time (where measurable)
- RSS and peak memory (where measurable)
- artifact bytes
- trace bytes
- process count

## Measurement quality levels

- authoritative: measured directly from runtime/process telemetry in controlled environment
- approximate: derived from filesystem size, wall-clock timing, or host-level summary sampling

## Scenario coverage

Resource profile scenarios must include parse/validate pressure, execution pressure, manifest/trace growth,
cache metadata growth, replay, and import/export memory behavior.

## Budget policy

- scenario-level artifact size budgets and trace/manifest budgets are defined under `benchmarks/scenarios/`.
- budget checks run in warning mode first and can be promoted to gate mode.

## Evidence outputs

Each benchmark report should include resource profile sections where feasible and
must declare measurement quality as `authoritative` or `approximate`.
