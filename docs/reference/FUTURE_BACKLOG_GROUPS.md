# Future Backlog Groups

Use these groups when adding new work items so roadmap growth stays aligned with architecture ownership.

## Kernel

Graph semantics, identity, planner invariants, scheduler correctness, run/artifact contracts.

## Runtime

Execution engine behavior, concurrency/cancellation semantics, cache/replay correctness, artifact durability.

## Adapters

Backend capability negotiation, adapter SDK contracts, integration compatibility.

## Evidence

Battle scenarios, trust-property mapping, release evidence set, drift controls.

## Ecosystem

Docs, onboarding, packaging, external integrations, contributor workflow tooling.

## Rules

- Every backlog item must belong to exactly one group.
- Items that alter trust boundaries must include an evidence update plan.
- New command surfaces require CLI contract and compatibility notes.
