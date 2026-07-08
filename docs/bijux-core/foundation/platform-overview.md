---
title: Platform Overview
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# Platform Overview

Use this page when you want the plain-language answer to the first repository
question: what lives in `bijux-core`, and why is it all in one place?

`bijux-core` is a shared home for two public product families and one private
maintainer surface:

- `bijux`, the command runtime for mounted apps, plugins, config, diagnostics,
  history, memory, and REPL workflows
- `bijux-dag`, the local-first DAG toolchain for graph validation, execution,
  retained evidence, replay, and comparison
- `bijux-dev`, the private maintainer surface that audits, releases, and proves
  the repository without pretending to be an end-user product

## What Readers Usually Need To Know First

| Surface | What it is for | Where to read next |
| --- | --- | --- |
| `bijux` | command runtime behavior and operator workflows | [CLI handbook](../../bijux-cli/index.md) |
| `bijux-dag` | DAG authoring, execution, inspection, replay, and verification | [DAG handbook](../../bijux-dag/index.md) |
| `bijux-dev` | repository diagnostics, evidence, release checks, and governance automation | [Maintainer handbook](../../bijux-dev/index.md) |
| repository root | shared contracts, docs, release rules, and multi-product ownership | [Repository handbook](../index.md) |

## What The Repository Organizes

- `bijux-cli` owns the operator-facing command runtime and the Python bridge
- `bijux-dag-*` owns graph truth, execution, replay, and artifact semantics
- `bijux-dev` owns repository-health automation, evidence, and release control
- the repository root owns cross-program rules, shared docs, contracts, and
  automation entrypoints

## What You Can Rely On Today

- `bijux` is a public command runtime.
- `bijux-dag` is a public local-first DAG product.
- `bijux-cli-python`, `bijux-dag-testkit`, and `bijux-dev` are repository
  support crates, not end-user product surfaces.
- Simulated DAG namespaces and maintainer-only routes may exist in the code and
  docs, but they are not public `v0.4.0` product promises.

## Why The Split Exists

- command runtime behavior and DAG behavior have different public contracts
- release and documentation evidence must stay reviewable above both products
- maintainer automation should stay explicit instead of leaking into product
  packages

## What This Page Is Not Saying

- It is not saying the repository is one product with one audience.
- It is not saying every crate is public just because it sits in the workspace.
- It is not replacing package pages when you need exact crate ownership.

## Continue Reading

- [Repository Scope](repository-scope.md)
- [Package Map](package-map.md)
- [Package Boundary](package-boundary.md)
- [Core Architecture](../architecture/index.md)
