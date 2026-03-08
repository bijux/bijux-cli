# Crate Responsibility Alignment

This document aligns current code ownership boundaries with runtime contraction governance.

## Aligned Responsibilities

- `bijux-dag-core`: graph identity, canonicalization, validation.
- `bijux-dag-runtime`: execution engine, scheduler, runtime adapter boundaries.
- `bijux-dag-artifacts`: artifact identity, lineage, storage interfaces.
- `bijux-dag-app`: command routing and operator UX.
- `bijux-dev-dag`: release evidence and governance checks.

## Drift Handling

- boundary drift is blocked by crate responsibility guardrails contracts.
