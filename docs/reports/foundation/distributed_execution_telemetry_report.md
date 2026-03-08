# Distributed Execution Telemetry Report

Generated telemetry summary for distributed execution health.

## Tracked signals

- worker registration failure count
- worker capability mismatch count
- dispatch retry count
- timeout detection count
- network failure downgrade count
- artifact transfer integrity mismatch count
- distributed determinism mismatch count

## Source of truth

- `evidence/cache/distributed_execution/regression_corpus.json`
- `configs/suites/distributed_execution_stress.json`
