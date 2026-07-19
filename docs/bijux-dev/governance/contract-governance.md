---
title: Contract Governance
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Contract Governance

Use this page when a repository promise is about to change and you need to know
whether that change belongs in code, in a contract file, or in both.

Contract governance exists so that command surfaces, artifact schemas, and
maintainer evidence do not drift into separate stories. A contract is only
useful when readers, tests, and shipped behavior all describe the same thing.

## Governance Rules

- contract changes require matching tests and docs updates
- breaking contract changes need explicit compatibility notes
- contract files must remain machine-checkable and human-reviewable

## Contract Families You Will Encounter

- CLI command and output contracts
- DAG replay, diff, and artifact contracts
- maintainer evidence and reporting contracts

## What Reviewers Should Ask

| Question | Why it matters |
| --- | --- |
| did behavior change without a contract update? | that leaves the repository shipping undocumented promises |
| did the contract change without matching tests? | that turns the contract into unverified prose |
| is the change intentionally breaking? | release notes and compatibility boundaries must say so explicitly |

## Reader Shortcut

When behavior, contract text, and verification disagree, trust none of them in
isolation. The issue is not closed until all three surfaces align again.

## Code Anchors

- `contracts/`
- `crates/bijux-dev/src/commands/contract_governance.rs`
- `crates/bijux-dev/tests/evidence_schema_contracts.rs`

## Continue Reading

- [Dependency Governance](dependency-governance.md)
- [Core Compatibility and Schema](../../bijux-core/governance/compatibility-and-schema.md)
- [DAG Artifact Contracts](../../bijux-dag/interfaces/artifact-contracts.md)
