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
    behavior[package public behavior] --> contracts[tracked contracts and schemas]
    contracts --> pinned[pinned snapshots and digests]
    pinned --> checks[drift and compatibility checks]
    checks --> review[review and merge decision]
    review --> behavior

    mismatch_code[behavior changed without contract update] --> checks
    mismatch_contract[contract changed without stated intent] --> checks
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
