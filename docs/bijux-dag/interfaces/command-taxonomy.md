---
title: Command Taxonomy
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Command Taxonomy

The DAG command surface is grouped by operator intent rather than by crate
layout.

## Core Groups

- define: validate and plan graph behavior
- execute: run and replay workflows
- inspect: list, show, inspect, summary, compare, trend, failures, flakes,
  tree, timeline, scheduler checkpoint, diff, verify, doctor, and explain runs
- operate: manage cache and inspect public command inventory

## Operator Inspection Lane

The stable operator inspection lane centers on:

- `bijux-dag runs list`
- `bijux-dag runs show`
- `bijux-dag runs inspect`
- `bijux-dag runs tree`
- `bijux-dag runs timeline`
- `bijux-dag runs diff`
- `bijux-dag runs verify`
- `bijux-dag runs doctor`
- `bijux-dag runs explain-failure`

The wider retained-history analytics lane also includes `summary`, `compare`,
`trend`, `failures`, and `flakes` under the same `runs` family.

## Detailed Taxonomy

Use [Reference: Command Taxonomy](command-taxonomy.md) for the
fuller route grouping and subordinate command list.
