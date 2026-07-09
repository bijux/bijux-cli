---
title: Package Map
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Package Map

Use this page when you know the problem you are trying to solve, but you do
not yet know which package family owns it.

The point is not to memorize crate names. The point is to route a reader,
contributor, or reviewer to the right handbook and the right code surface
without burning time in the wrong part of the repository.

For the authoritative public-versus-private release answer, use
[Package Boundary](package-boundary.md).

## Start With The Job To Be Done

| If you need to understand... | Owning package family | Why that family owns it | Read next |
| --- | --- | --- | --- |
| `bijux` commands, mounted apps, plugin routing, layered config, REPL behavior, history, memory, or Python delivery | `bijux-cli` and `bijux-cli-python` | this family owns the operator-facing runtime and the Python packaging lane that delivers it | [CLI Handbook](../../bijux-cli/index.md) |
| graph parsing, validation, planning, execution, artifacts, replay, verification, or `bijux-dag` commands | `bijux-dag-core`, `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`, `bijux-dag-artifacts`, and `bijux-dag-testkit` | this family owns the DAG model, runtime policy, orchestration layer, executable wrapper, and retained evidence model | [DAG Handbook](../../bijux-dag/index.md) |
| release proof, repository diagnostics, docs automation, root gates, or evidence reporting | `bijux-dev` | this crate family owns repository control-plane behavior rather than end-user product behavior | [Maintainer Handbook](../../bijux-dev/index.md) |
| publication boundaries, shared contracts, or cross-product repository rules | repository root plus the owning crate family | these questions cross more than one product lane and usually need both docs and code context | [Repository Handbook](../index.md) |

## The Package Families In Plain Language

### CLI family

Use the CLI family when the reader is asking what the `bijux` command does,
how it is configured, how plugins are mounted, or how the same runtime is
delivered through Rust and Python packaging.

### DAG family

Use the DAG family when the reader is asking what a graph means, how a run is
planned or executed, what artifacts are retained, or how replay, comparison,
and verification behave.

### Maintainer family

Use the maintainer family when the reader is asking how the repository proves a
release, enforces root contracts, validates docs, or gathers evidence across
the workspace.

## Common Routing Mistakes

| When the question sounds like... | The real owner is usually... | Why |
| --- | --- | --- |
| "The `bijux` runtime can mount DAG commands, so this must be a CLI-only issue." | the DAG family once the question becomes graph execution, replay, or evidence | the CLI surface can launch DAG behavior without owning DAG semantics |
| "Python packaging is separate from the runtime." | the CLI family | the delivery format changes, but the public runtime story stays the same |
| "The release job is failing, so I should inspect product docs first." | the maintainer family | release proof, standards sync, and repository gates live above any one product handbook |
| "This schema lives under `contracts/`, so it is a root-only concern." | the root plus the crate that enforces it | shared contracts still need a concrete product or maintainer owner |

## A Fast Ownership Check

Use these questions when the route is still unclear:

1. Is the reader trying to run a product, author a DAG, or maintain the repo?
2. Does the behavior end at one executable, or does it cross products?
3. If a test failed, which crate or maintainer suite would be expected to
   catch that drift first?

If those answers still span more than one family, stay in the repository
handbook and then drill down from there.

## What This Page Should Help You Avoid

- treating crate names as if they were the same thing as product boundaries
- staying in repository-level docs after the owning family is already obvious
- debugging release or docs automation from product pages alone
- changing a downstream surface while missing the real owner upstream

## Continue Reading

- [Ownership Model](ownership-model.md)
- [Package Boundary](package-boundary.md)
- [Repository Packages](../packages/index.md)
