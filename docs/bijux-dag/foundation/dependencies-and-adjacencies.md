---
title: Dependencies and Adjacencies
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-04-06
---

# Dependencies and Adjacencies

Use this page when you want the honest answer to a basic maintenance question:
which dependencies can change DAG truth, and which crate boundaries must stay
clear for the stack to remain trustworthy?

The critical question is not dependency count. It is whether a dependency can
alter identity, execution meaning, replay truth, artifact integrity, or crate
direction.

## What Changes DAG Meaning

| Surface | Why it matters |
| --- | --- |
| identity and hashing | hashing and encoding libraries decide whether graph and artifact fingerprints remain stable |
| execution state | runtime dependencies influence how runs are classified, resumed, or compared |
| artifact persistence | retained evidence depends on durable serialization and integrity behavior |
| command modeling | CLI-layer dependencies shape operator entrypoints but should not redefine DAG semantics |
| crate adjacency | dependency shortcuts can collapse the boundary between graph truth, execution, evidence, and presentation |

## Dependencies With Real Semantic Pressure

- `clap`: CLI command models and route parsing in app/cli layers
- `serde`/`serde_json`: graph/run/artifact payload serialization
- `sha2`/`hex`: identity hashing and integrity checks
- `thiserror`: typed domain and runtime error surfaces

## Crate Boundaries That Must Stay Clear

- `dag-core` remains pure and side-effect free.
- `dag-runtime` may depend on `dag-core` and `dag-artifacts`.
- `dag-app` orchestrates runtime/core/artifact calls, not vice versa.
- `dag-cli` remains thin and does not absorb DAG semantics.

## What This Page Is Not Saying

- It is not claiming that every new crate is a design problem.
- It is not saying command-layer convenience should be banned.
- It is not replacing package ownership pages when you need exact crate
  responsibilities.

## Code Anchors

- `crates/bijux-dag-core/Cargo.toml`
- `crates/bijux-dag-runtime/Cargo.toml`
- `crates/bijux-dag-app/Cargo.toml`
- `crates/bijux-dag-cli/Cargo.toml`

## Continue Reading

- [Dependency Direction](../architecture/dependency-direction.md)
- [Dependency Governance](../quality/dependency-governance.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
