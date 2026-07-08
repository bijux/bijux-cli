---
title: Repository Handbook
audience: mixed
type: index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Repository Handbook

This handbook explains the parts of `bijux-core` that sit above any single
crate or command. Read it when you need the shape of the repository, the
public-versus-private package line, or the release rules that keep `bijux` and
`bijux-dag` honest.

<div class="bijux-callout"><strong>Use this handbook when the question crosses products.</strong>
If the answer needs both <code>bijux</code> and <code>bijux-dag</code>, or if
it depends on package boundaries, release policy, or shared automation, this
is the right starting point.</div>

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="foundation/">Open foundation</a>
<a class="md-button" href="architecture/">Open architecture</a>
<a class="md-button" href="operations/">Open operations</a>
</div>

## What This Handbook Helps You Answer

- What does this repository actually publish today?
- Which crates are public, and which remain repository-internal?
- Where should a reader or contributor start before opening source files?
- Which repository rules affect both `bijux` and `bijux-dag`?
- Which root workflows validate, release, and document the whole workspace?

## Start Here

| Question | Best starting page |
| --- | --- |
| What does `bijux-core` publish today? | [Foundation](foundation/index.md) |
| Which package owns this behavior? | [Package Map](foundation/package-map.md) |
| Which crates are public and which stay internal? | [Package Boundary](foundation/package-boundary.md) |
| How is the workspace laid out and why? | [Core Architecture](architecture/index.md) |
| What do contributors run before review or release? | [Operations](operations/index.md) |

## Repository Snapshot

`bijux-core` publishes two public product families:

- `bijux`, the operator-facing command runtime
- `bijux-dag`, the local-first DAG runtime and crate family

It also carries repository-internal support crates that make those products
shippable and auditable:

- `bijux-cli-python` for Python packaging and bridge parity
- `bijux-dag-testkit` for deterministic DAG fixtures and shared assertions
- `bijux-dev` for repository diagnostics, governance, evidence, and release
  tooling

That split matters because many questions that look like one command problem
are really package-boundary or release-boundary questions.

## What You Will Find Here

- the public-versus-private crate line
- the workspace layout under `crates/`, `contracts/`, `docs/`, `makes/`, and
  `.github/workflows/`
- the release and validation rules that apply across both product families
- the ownership rules that keep package pages, READMEs, and shipped surfaces
  aligned

## When Not To Stay Here

- If the question is only about `bijux` behavior, move to the
  [CLI Handbook](../bijux-cli/index.md).
- If the question is only about DAG authoring, execution, replay, or retained
  evidence, move to the [DAG Handbook](../bijux-dag/index.md).
- If the question is about maintainer automation, release proof, or repository
  gates, move to the [Maintainer Handbook](../bijux-dev/index.md).

## Program Handbooks

- [CLI Handbook](../bijux-cli/index.md)
- [DAG Handbook](../bijux-dag/index.md)
- [Maintainer Handbook](../bijux-dev/index.md)
