---
title: Artifact and Contract Flow
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Artifact and Contract Flow

Artifact and contract flow defines how outputs are generated, validated,
published, and consumed across programs.

## Visual Summary

```mermaid
flowchart TD
    execution[cli or dag execution] --> artifacts[run and report artifacts]
    artifacts --> contracts[contract snapshots and schemas]
    contracts --> verification[validation suites]
    verification --> docs[documentation and release notes]
```

## Flow Stages

1. commands generate runtime artifacts and maintainer reports
2. contract assets capture schema and output expectations
3. verification suites check lockstep behavior and compatibility
4. docs and releases reference verified contract state

## Contract Surfaces

- DAG replay and diff schemas
- CLI output and route contracts
- maintainer evidence schemas and registry outputs
- documentation layout and link contracts

## Code Anchors

- `contracts/`
- `crates/bijux-dag-app/tests/`
- `crates/bijux-cli/tests/`
- `crates/bijux-dev/tests/`

## Next Reads

- [Testing and Validation](../governance/testing-and-validation.md)
- [Compatibility and Schema](../governance/compatibility-and-schema.md)
- [Documentation Standards](../governance/documentation-standards.md)
