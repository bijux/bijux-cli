---
title: Authoring Guide
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Authoring Guide

Use the governed fixture set below when authoring or reviewing DAG definitions.
For the field-level DAG file contract, start with
[Graph Schema Reference](../reference/graph-schema.md).

## Start from valid patterns

- `evidence/dag/authoring/patterns/minimal.json`
- `evidence/dag/authoring/patterns/medium.json`
- `evidence/dag/authoring/patterns/pattern_chain.json`
- `evidence/dag/authoring/patterns/pattern_diamond.json`
- `evidence/dag/authoring/patterns/pattern_fanout.json`
- `evidence/dag/authoring/patterns/pattern_aggregation.json`
- `evidence/dag/authoring/patterns/pattern_cache_heavy.json`
- `evidence/dag/authoring/patterns/pattern_replay_sensitive.json`

## Use negative fixtures to understand rejection behavior

- `evidence/dag/authoring/negative/undeclared_outputs.json`
- `evidence/dag/authoring/negative/invalid_refs.json`
- `evidence/dag/authoring/negative/cycle.json`
- `evidence/dag/authoring/negative/invalid_selectors.json`
- `evidence/dag/authoring/negative/unsupported_adapter_payload.json`

## Working rule

- treat the positive fixtures as executable starting points
- treat the negative fixtures as explicit guardrails for unsupported authoring
  moves
