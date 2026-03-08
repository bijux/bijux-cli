# bijux-dag-core

Core DAG model, parsing, validation, canonicalization, topology, and fingerprint logic.
Responsibility: DAG schema, parsing, canonicalization, validation, and deterministic semantic graph logic.

## Why this crate exists
This crate is the deterministic source for DAG semantics, validation rules, and graph identity behavior.

## What must never enter this crate
- Backend adapter implementations.
- Runtime scheduler/executor orchestration.
- CLI command routing logic.

See [CONTRACT.md](./CONTRACT.md).
