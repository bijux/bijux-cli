---
title: Package Map
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Package Map

Use the package map when you know the behavior you care about, but you do not
yet know which package or handbook owns it.

For canonical public/private publication status, use
[Package Boundary](package-boundary.md).

## Fast Ownership Guide

| If the question is about... | Owning package family | Open next |
| --- | --- | --- |
| `bijux` command behavior, config, REPL, plugin routing, or Python distribution | `bijux-cli` and `bijux-cli-python` | [CLI Handbook](../../bijux-cli/index.md) |
| graph semantics, DAG execution, replay, retained artifacts, or `bijux-dag` command behavior | `bijux-dag-core`, `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`, `bijux-dag-artifacts`, `bijux-dag-testkit` | [DAG Handbook](../../bijux-dag/index.md) |
| repository diagnostics, evidence reports, release proof, docs automation, or repository gates | `bijux-dev` | [Maintainer Handbook](../../bijux-dev/index.md) |

## What This Map Helps You Avoid

- It prevents readers from staying in the wrong handbook too long.
- It prevents crate names from being mistaken for product boundaries.
- It keeps maintainer and product surfaces from collapsing into one story.

## Typical Misroutes

| Misroute | Better reading path |
| --- | --- |
| `bijux dag ...` mounted through the root runtime is treated as only a CLI problem | move to the DAG handbook once the real question is graph execution or evidence |
| Python distribution questions are treated as separate from the CLI runtime | stay with the CLI family; the distribution is different, the runtime story is not |
| release, docs, or gate failures are debugged from product pages alone | move to the maintainer handbook when the real problem is repository proof or automation |

## How To Use This Page

- Use it to choose the owner.
- Leave it once the owner is clear.
- Return only when the answer crosses more than one family.

## Continue Reading

- [Ownership Model](ownership-model.md)
- [Package Boundary](package-boundary.md)
- [Repository Packages](../packages/index.md)
