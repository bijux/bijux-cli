# Scheduler hardening report

## Scope

Captures scheduler semantic guarantees that are treated as release evidence.

## Canonical semantics

- canonical scheduling unit: node
- ready queue semantics: dependency indegree reaches zero
- deterministic tie-break: lexical `node_id`
- retry re-entry: explicit `retry_queue` -> `ready_queue`
- cache and skipped completions satisfy readiness as documented
- failure propagation mode controls downstream unlock behavior

## Determinism evidence

- fixed-input repeated run deterministic ordering checks
- downstream readiness de-duplication checks
- concurrency-level invariance checks across worker counts

## Timeline and invariants

- scheduler timeline reconstruction command:
  - `bijux-dev-dag dag scheduler-timeline --run-dir <path>`
- invariant suite gate:
  - `scheduler-invariants`

## Non-negotiable release linkage

Scheduler determinism is mandatory foundation evidence and remains required by foundation verification.
