# Determinism

## What Must Be Stable
- Canonical JSON output for a DAG.
- Fingerprints for graphs and nodes.
- Execution order for ready nodes (stable topo ordering).
- Artifact layout and file naming.

## What May Vary
- Actual wall-clock timestamps.
- Runtime performance and scheduling latency.

## Forbidden
- Non-deterministic scheduling that changes trace ordering.
- Reading undeclared environment variables.
- Hidden runtime-only fields that affect fingerprints.
