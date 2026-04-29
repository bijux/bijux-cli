---
title: Contract Governance
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Contract Governance

This page explains how `bijux-dev` keeps contract files honest.

The rule is straightforward: contract text, executable checks, and released
behavior must keep telling the same story.

## Contract Flow

```mermaid
flowchart LR
    behavior["runtime behavior"] --> contract["contract definition"]
    behavior --> tests["contract checks"]
    contract --> release["release trust"]
    tests --> release
```

## Governance Rules

- contract changes require matching tests and docs updates
- breaking contract changes need explicit compatibility notes
- contract files must remain machine-checkable and human-reviewable

## Contract Families

- CLI command and output contracts
- DAG replay, diff, and artifact contracts
- maintainer evidence and reporting contracts

## Reading Rule

Use this page when behavior, contract text, and verification no longer line up,
or when a change is about to introduce a new contract surface that must stay
reviewable over time.

## Code Anchors

- `contracts/`
- `crates/bijux-dev/src/commands/contract_governance.rs`
- `crates/bijux-dev/tests/evidence_schema_contracts.rs`

## Next Reads

- [Dependency Governance](dependency-governance.md)
- [Core Compatibility and Schema](../../bijux-core/governance/compatibility-and-schema.md)
- [DAG Artifact Contracts](../../bijux-dag/interfaces/artifact-contracts.md)
