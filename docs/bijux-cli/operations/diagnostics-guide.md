---
title: Diagnostics Guide
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-29
---

# Diagnostics Guide

The `doctor` surface is the first stop for runtime health checks.

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

## When To Use It

- attach a runtime snapshot to bug reports
- verify Python bridge availability and interpreter selection
- inspect root routing inventory before blaming app-level code
- confirm generated config documentation matches the shipped runtime schema
