# System Formal Invariants Contract

## Purpose

This contract defines system-level invariants that must hold across DAG
evaluation, replay, diff, identity, backend equivalence, and import/export
operations.

## System Invariant Domains

- core DAG execution invariants
- artifact lineage invariants
- replay equivalence invariants
- diff semantic invariants
- scheduler fairness invariants
- run identity invariants
- artifact identity invariants
- backend equivalence invariants
- determinism invariants

## Verification Expectations

- invariants are checked for successful runs
- invariants are checked for failed runs
- invariants are checked for partial runs
- invariants are checked during replay flows
- invariants are checked during import/export flows
- invariant failures are explicitly logged
- invariant drift is detected through deterministic corpus checks

## Operator Surface

- `invariants-report` is the canonical command surface for invariant status.
- invariant verification artifacts are machine-readable and versioned.

