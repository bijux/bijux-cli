# ADR: dev-dag Governance Scope

## Status
Accepted

## Context
`bijux-dev-dag` accumulated mixed concerns, including checks that risked becoming authoritative for runtime semantics.

## Decision
`bijux-dev-dag` remains governance and verification orchestration only:
- Repository policy checks and generated governance reports.
- Contract and release checks that validate other crates.
- No authoritative ownership of runtime execution semantics or schema definitions.

## Consequences
- Runtime/core/artifacts keep source-of-truth ownership.
- Governance automation remains strict but non-authoritative.
- Drift checks can enforce scope boundaries with clear failure modes.
