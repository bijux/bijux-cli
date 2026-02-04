# Project Contract

## Goals
- Provide a strict, minimal DAG IR.
- Ensure deterministic execution order.
- Produce reproducible run artifacts.

## Non-Goals
- Distributed execution.
- Dynamic graph mutation at runtime.
- Implicit network access.

## Compatibility
- Spec versions live in `spec/`.
- Breaking changes require a new version file.

## Stability
- JSON parsing is strict (`deny_unknown_fields`).
- Canonical output uses stable ordering.
