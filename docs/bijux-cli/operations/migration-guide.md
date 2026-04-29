---
title: Migration Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Migration Guide

The durable command form is:

```text
bijux <product> <verb>
```

## Preferred Migrations

- `bijux-dag ...` -> `bijux dag ...`
- `bijux-canon ...` -> `bijux canon ...`
- `bijux-atlas ...` -> `bijux atlas ...`

## Compatibility Expectations

- legacy shims are tolerated for migration periods
- the root runtime should preserve stdout/stderr discipline
- machine-readable output must remain stable across equivalent routes

## Diagnostics

Use `bijux doctor shims` to find old `bijux-<app>` wrappers still visible on
PATH.
