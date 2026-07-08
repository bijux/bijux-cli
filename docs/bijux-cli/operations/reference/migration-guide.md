---
title: Migration Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Migration Guide

Use this page when existing users, scripts, or wrapper binaries need a clear
path from older invocation habits to the current durable CLI forms.

The durable root-runtime forms are:

```text
bijux <root-command>
bijux <app> <verb>
```

Official product binaries remain the authoritative public operator surface when
the product ships one. The routed `bijux <app> ...` form is for root-managed
discovery and delegation.

For DAG specifically, use the
[DAG release boundary](../../../bijux-dag/foundation/release-boundary.md),
which is backed by the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`.

## Preferred Invocation Choices

- use `bijux-dag ...` for the public DAG command surface
- use `bijux dag ...` when you intentionally want root-managed app routing
- migrate deprecated alias wrappers such as `bijux-workflow ...` to either
  `bijux-dag ...` or `bijux dag ...`

## Compatibility Expectations

- deprecated alias wrappers are tolerated for migration periods
- the root runtime should preserve stdout/stderr discipline
- machine-readable output must remain stable across equivalent routes

## What Readers Should Migrate Deliberately

| Old habit | Safer target |
| --- | --- |
| deprecated alias wrapper | a current product binary or the explicit routed form |
| implicit product discovery assumptions | the documented public binary for that product |
| trusting equivalent routes without checking output discipline | validate stdout, stderr, and machine-readable output explicitly |

## Diagnostics

Use `bijux doctor shims` to find deprecated alias binaries such as
`bijux-workflow` while keeping declared product binaries such as `bijux-dag`
visible on PATH.

## Reader Shortcut

If a route is only "equivalent" by folklore, it is not equivalent enough for
automation. Migrate to the documented durable form and verify output behavior
before treating the move as complete.

## Continue Reading

- [Deployment Boundaries](deployment-boundaries.md)
- [Compatibility Commitments](../../interfaces/compatibility-commitments.md)
- [DAG Release Boundary](../../../bijux-dag/foundation/release-boundary.md)
