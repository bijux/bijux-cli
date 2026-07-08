---
title: Command Taxonomy
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Command Taxonomy

The DAG command surface is grouped by operator intent, not crate layout.

## Core groups

- define: validate and plan graph behavior
- execute: run and replay workflows
- inspect: list, show, inspect, tree, timeline, scheduler checkpoint, diff,
  verify, doctor, and explain runs
- operate: manage cache and inspect public command inventory

## Operator inspection lane

The operator inspection lane is the stable home for:

- `bijux-dag runs list`
- `bijux-dag runs show`
- `bijux-dag runs inspect`
- `bijux-dag runs summary`
- `bijux-dag runs compare`
- `bijux-dag runs trend`
- `bijux-dag runs failures`
- `bijux-dag runs flakes`
- `bijux-dag runs tree`
- `bijux-dag runs timeline`
  with `--node`, `--event`, `--since-unix-ms`, and `--until-unix-ms`
- `bijux-dag runs scheduler-checkpoint`
- `bijux-dag runs diff`
- `bijux-dag runs verify`
- `bijux-dag runs doctor`
- `bijux-dag runs explain-failure`
