---
title: Workspace Topology
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Workspace Topology

Workspace topology documents where source, policies, contracts, and generated
assets live so contributors can navigate without guesswork.

## Visual Summary

```mermaid
flowchart TD
    root[repository root]
    root --> crates[crates/]
    root --> configs[configs/]
    root --> makes[makes/ and Makefile]
    root --> docs[docs/]
    root --> contracts[contracts/]
    root --> artifacts[artifacts/]

    crates --> runtime[runtime and maintainer crates]
    docs --> programs[core/cli/dag/dev handbooks]
    contracts --> api[contract snapshots and schemas]
    artifacts --> generated[generated evidence only]
```

## Topology Rules

- product and maintainer code lives under `crates/`
- shared build and test configuration lives under `configs/`
- orchestration entrypoints live under `makes/` and root `Makefile`
- handbook content lives under `docs/` with four top-level programs
- generated outputs stay under `artifacts/` and never become source of truth

## Documentation Programs

- `docs/bijux-core`
- `docs/bijux-cli`
- `docs/bijux-dag`
- `docs/bijux-dev`

## Code Anchors

- `Cargo.toml`
- `Makefile`
- `makes/root.mk`
- `docs/index.md`

## Next Reads

- [Runtime Surfaces](runtime-surfaces.md)
- [Package Ownership](../governance/package-ownership.md)
- [Documentation Standards](../governance/documentation-standards.md)
