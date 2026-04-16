---
title: Artifact Contracts
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Artifact Contracts

Artifact contracts tie payload bytes, metadata, identity, and lineage into one
verifiable evidence surface.

## Visual Summary

```mermaid
flowchart LR
    producer[core producer] --> schema[artifact schema]
    schema --> file[artifact payload and files]
    file --> consumer[consumer]
    schema --> version[version marker]
    version --> compatibility[compatibility rule]
    consumer --> validation[contract validation]
```

## Contract Surfaces

- outputs index and run outputs index
- run manifest and provenance records
- node trace and input/output index files
- integrity proofs and schema validation descriptors

## Code Anchors

- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/integrity/hash.rs`
- `crates/bijux-dag-artifacts/src/integrity/index.rs`
- `crates/bijux-dag-artifacts/src/integrity/proof.rs`
- `crates/bijux-dag-runtime/src/artifacts/`

## Contract Rules

- hash and lineage mismatches must be surfaced explicitly
- missing required evidence must not be treated as equivalent
- schema-bearing artifact files require compatibility review on shape changes

## Next Reads

- [Data Contracts](data-contracts.md)
- [State and Persistence](../architecture/state-and-persistence.md)
- [Documentation Standards](../quality/documentation-standards.md)
