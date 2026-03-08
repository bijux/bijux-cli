# Spec deprecation policy

## Scope

This policy governs deprecation for DAG spec fields, node kinds, and CLI surface behavior.

## Rules

- Every deprecation must include a changelog entry and explicit replacement guidance.
- Every deprecation must define removal criteria and earliest removal release.
- Every deprecation must preserve parser compatibility for the published compatibility window.
- Warnings must be deterministic and machine-readable where possible.

## Enforcement

- `bijux-dev-dag` release verification must fail if a breaking change ships without policy updates.
- Compatibility fixtures must be updated only when compatibility guarantees explicitly permit the change.
