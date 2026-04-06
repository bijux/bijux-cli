# bijux-dag-app

Application orchestration crate for DAG commands and response formatting.
Responsibility: Application orchestration services, command response modeling, and user-facing render flows.

## Why this crate exists
This crate owns command routing, argument-to-service translation, and response shaping for `bijux dag` surfaces.

## What must never enter this crate
- Runtime execution engine internals.
- Artifact storage backend internals.
- Governance-only release policy logic.

See [CONTRACT.md](./CONTRACT.md).
