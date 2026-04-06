# bijux-dag-artifacts

Run artifact models and artifact persistence helpers.
Responsibility: Run artifact models, persistence services, integrity proofs, and lifecycle policy helpers.

## Why this crate exists
This crate is the authoritative implementation for artifact identity, lineage material, and artifact persistence/verification helpers.

## What must never enter this crate
- CLI command routing.
- Runtime scheduler and execution policy logic.
- Dev governance command orchestration.

See [CONTRACT.md](./CONTRACT.md).
