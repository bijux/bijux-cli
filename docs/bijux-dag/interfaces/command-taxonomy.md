---
title: Command Taxonomy
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Command Taxonomy

The DAG command surface is grouped by operator intent, not crate layout.

## Core groups

- define: validate and plan graph behavior
- execute: run and replay workflows
- inspect: list, show, inspect, tree, timeline, diff, verify, doctor, and
  explain runs
- operate: manage cache, config, policy, and environment controls

## Operator inspection lane

The operator inspection lane is the stable home for:

- `dag runs list`
- `dag runs show`
- `dag runs inspect`
- `dag runs tree`
- `dag runs timeline`
- `dag runs diff`
- `dag runs verify`
- `dag runs doctor`
- `dag runs explain-failure`
