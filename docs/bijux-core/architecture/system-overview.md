---
title: System Overview
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# System Overview

`bijux-core` is a single Rust workspace that hosts runtime programs, DAG
execution crates, a Python bridge package, and a maintainer control-plane.

## Visual Summary

```mermaid
flowchart TB
    Root[bijux-core workspace]
    Root --> Programs[Runtime programs]
    Root --> DAG[DAG execution crates]
    Root --> Python[Python bridge package]
    Root --> Maintain[Maintainer control plane]
    Root --> Shared[Shared contracts and policies]

    Programs --> Shared
    DAG --> Shared
    Python --> Shared
    Maintain --> Shared
```

## System Components

- `crates/bijux-cli` owns CLI runtime behavior
- `crates/bijux-dag-*` owns DAG execution, replay, diff, and artifacts
- `crates/bijux-cli-python` owns Python packaging and bridge integration
- `crates/bijux-dev` owns maintainer-only diagnostics and governance workflows

## Architectural Boundary

- runtime behavior belongs to CLI and DAG program crates
- repository policy and release evidence belong to maintainer workflows
- shared workspace policy belongs to root manifests, configs, and make targets

## Non-Goals

- this page does not define command-by-command CLI behavior
- this page does not redefine DAG runtime semantics already owned by DAG docs
- this page does not replace executable contract and test evidence

## Boundary Smells

- adding maintainer-only logic into user runtime crates
- changing product behavior through repository scripts without crate-level review
- documenting cross-program policy without linking owning code anchors

## Code Anchors

- `Cargo.toml`
- `crates/bijux-cli/src/lib.rs`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dev/src/lib.rs`

## Next Reads

- [Workspace Topology](workspace-topology.md)
- [Dependency Direction](dependency-direction.md)
- [Repository Scope](../governance/repository-scope.md)
