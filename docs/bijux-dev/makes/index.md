---
title: Make System
audience: mixed
type: index
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Make System

The root Make surface composes shared standards, repository adapters, product
commands, and hosted workflow entrypoints. It exists to make repeated
operations discoverable without hiding which package, command, or evidence
surface owns the result.

Start with `make help` for the live target catalog. Use these pages when you
need ownership, execution, failure, or extension semantics that a one-line help
entry cannot provide.

## Route By Question

| Question | Page |
| --- | --- |
| How are shared and local Make fragments composed? | [Make Execution Model](make-system-overview.md) |
| Which root targets should contributors use first? | [Root Entrypoints](root-entrypoints.md) |
| Which package or command owns a failed target? | [Make Dispatch Boundaries](package-dispatch.md) |
| How do hosted workflows delegate to Make? | [CI Targets](ci-targets.md) |
| Which targets validate, build, and publish releases? | [Release Surfaces](release-surfaces.md) |
| How should a new target preserve status and artifacts? | [Make Target Authoring](authoring-rules.md) |

## Ownership Rule

Shared files under `.bijux/shared/` are managed outputs from `bijux-std`.
Repository-specific adapters live under `makes/`. Product semantics remain in
the owning crate or package. GitHub workflows own triggers, permissions, and
hosted setup, but should delegate repository behavior to a named Make target.

When a target fails, repair the owning layer rather than adding a second path
that happens to pass.
