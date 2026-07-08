---
title: Diagnostics Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Diagnostics Guide

Use this page when `bijux` behaves unexpectedly and you need the shortest path
from suspicion to a reproducible runtime snapshot.

The `doctor` surface is the first stop for runtime health checks because it
turns path, bridge, routing, and install questions into evidence instead of
guesswork.

## Core Topics

- `bijux doctor`
- `bijux doctor paths`
- `bijux doctor routing`
- `bijux doctor shims`
- `bijux doctor python`
- `bijux doctor <app>`

## Bundle Export

`bijux doctor --bundle` writes a reproducible evidence bundle under
`./artifacts/bijux-cli/doctor-bundle`.

Current bundle contents:

- `doctor.json`
- `docs.json`
- `config/generated-reference.md`

## What Each Surface Helps Diagnose

| Command | Best used for |
| --- | --- |
| `bijux doctor` | broad runtime health and obvious misconfiguration |
| `bijux doctor paths` | wrong state, config, or plugin path resolution |
| `bijux doctor routing` | route inventory and dispatch confusion |
| `bijux doctor shims` | deprecated wrappers and PATH ambiguity |
| `bijux doctor python` | bridge availability and interpreter selection |
| `bijux doctor --bundle` | capturing a supportable evidence package for later review |

## When To Use It

- attach a runtime snapshot to bug reports
- verify Python bridge availability and interpreter selection
- inspect root routing inventory before blaming app-level code
- confirm generated config documentation matches the shipped runtime schema

## Reader Shortcut

If a diagnosis depends on memory of how the machine is set up, run `doctor`
again and capture the output. The point of this surface is to replace folklore
with evidence.

## Continue Reading

- [Failure Recovery](failure-recovery.md)
- [Observability and Diagnostics](observability-and-diagnostics.md)
- [Security and Safety](security-and-safety.md)
