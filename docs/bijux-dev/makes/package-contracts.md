---
title: Package Contracts
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-12
---

# Package Contracts

The make layer exposes repository workflows, but it should not blur ownership
between product packages and maintainer automation.

## Contract Boundaries

- `rust.mk` verifies and publishes Rust package surfaces
- `python.mk` owns Python packaging and publication
- `docs.mk` owns handbook build and navigation integrity
- `dag.mk` owns DAG governance and evidence entrypoints
- `gh.mk` supports workflow decisions and publish planning

## Contract Rule

Make targets may orchestrate multiple surfaces, but the docs should still name
which file and package own each step.

## Next Reads

- [Package Dispatch](package-dispatch.md)
- [Release Surfaces](release-surfaces.md)
- [Ownership Model](../../bijux-core/foundation/ownership-model.md)
