# Git For Computation Graphs Mapping

## Purpose

This page defines the high-level mental mapping for the canonical one-liner.

## Mapping

| Git concept | Bijux DAG concept | Meaning |
| --- | --- | --- |
| repository tree | canonical graph | normalized structure and identity |
| commit | run | immutable execution record |
| blob | artifact | content + provenance identity |
| checkout | replay | re-materialize behavior under fidelity rules |
| diff | graph/run/artifact diff | semantic drift explanation |
| history log | run history | ancestry and execution lineage |

## Boundaries

- The mapping is conceptual, not a claim of identical implementation details.
- Execution support claims still follow `docs/reference/EXECUTION_SUPPORT_POLICY.md`.
