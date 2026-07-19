---
title: System Overview
audience: mixed
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# System Overview

Use this page when you want the architecture answer before the crate-by-crate
answer: what are the big moving parts in `bijux-core`, and how do they relate
without collapsing into one system diagram?

`bijux-core` is a single Rust workspace, but it is not a single runtime. It
contains:

- the `bijux` command runtime
- the `bijux-dag` graph and execution stack
- the Python bridge that distributes the CLI runtime
- the maintainer surfaces that validate, release, and audit the repository

## System Components

- `crates/bijux-cli` owns CLI runtime behavior
- `crates/bijux-dag-core`, `crates/bijux-dag-artifacts`, `crates/bijux-dag-runtime`,
  `crates/bijux-dag-app`, and `crates/bijux-dag-cli` own DAG execution,
  replay, diff, and artifacts
- `crates/bijux-cli-python` owns Python packaging and bridge integration
- `crates/bijux-dev` owns maintainer-only diagnostics and governance workflows

## What Each Layer Is For

| Layer | Main job |
| --- | --- |
| CLI runtime | parse commands, route work, and return stable operator-facing output |
| DAG stack | define graphs, execute work, retain evidence, and compare runs |
| Python bridge | ship the CLI runtime through Python packaging without inventing a different product |
| maintainer surface | prove release readiness, repository integrity, and documentation alignment |

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

## Continue Reading

- [Workspace Topology](workspace-topology.md)
- [Dependency Direction](dependency-direction.md)
- [Repository Scope](../foundation/repository-scope.md)
