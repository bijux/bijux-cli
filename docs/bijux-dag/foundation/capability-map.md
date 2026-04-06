---
title: Capability Map
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Capability Map

This page maps DAG capabilities to concrete crate ownership so operators and
maintainers can locate responsible code quickly.

## Visual Summary

```mermaid
flowchart TB
    define["definition capabilities"] --> core["dag-core parse validate canonicalize"]
    execute["execution capabilities"] --> runtime["dag-runtime engine scheduler policy"]
    orchestrate["command capabilities"] --> app["dag-app routes and output contracts"]
    persist["artifact capabilities"] --> artifacts["dag-artifacts integrity and lifecycle"]
    invoke["process capabilities"] --> cli["dag-cli binary entry"]
```

## Capability Inventory

- definition validation and canonical identity generation
- execution planning and node scheduling over DAG dependencies
- run identity and run-history inspection surfaces
- artifact lineage, integrity proofs, and portability bundle workflows
- replay and diff classification for release decisions

## Code Anchors

- `crates/bijux-dag-core/src/`
- `crates/bijux-dag-runtime/src/`
- `crates/bijux-dag-app/src/routes/`
- `crates/bijux-dag-artifacts/src/`
- `crates/bijux-dag-cli/src/main.rs`

## Next Reads

- [Domain Language](domain-language.md)
- [Module Map](../architecture/module-map.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
