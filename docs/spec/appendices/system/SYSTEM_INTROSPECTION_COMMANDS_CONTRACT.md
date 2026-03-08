# System Introspection Commands Contract

## Purpose

This contract defines durable expectations for operator-facing introspection in
`bijux-dag`. Introspection commands must provide deterministic, machine-readable
diagnostics without mutating run state.

## Command Surface

The introspection surface is composed of command entrypoints and their backing
handlers.

- `dag.run-inspect` -> `run_dag_run_inspect`
- `dag.scheduler-timeline` -> `run_dag_scheduler_timeline`
- `storage-health` -> `run_storage_health`
- `backend-registry-report` -> `run_backend_registry_report`
- `cache-coverage-report` -> `run_cache_coverage_report`
- `verify.evidence-replay` -> `run_evidence_replay_verify`
- `drift-dashboard` -> `run_drift_dashboard`
- `verify.evidence-drift` -> `run_evidence_drift_verify`

## Determinism Rules

- JSON object keys and list ordering are deterministic for equal inputs.
- Text output ordering is deterministic for equal inputs.
- Introspection commands must never depend on wall-clock ordering for stable
  report fields.

## Integrity Rules

- Commands detect malformed or missing metadata and report clear anomalies.
- Commands must return parseable JSON when `--json` is requested.
- Introspection evidence files must be schema-validated where schemas exist.

## Coverage Rules

Introspection verification must cover:

- execution trace inspection
- artifact store health inspection
- run history integrity inspection
- scheduler state inspection
- backend capability inspection
- replay compatibility inspection
- cache state inspection
- provenance graph inspection
- artifact lineage graph inspection
- runtime diagnostics inspection
- deterministic output ordering
- schema validation
- snapshot stability
- telemetry and anomaly reporting
- stress behavior

## Non-goals

- release distribution workflows
- runtime mutation or repair semantics

