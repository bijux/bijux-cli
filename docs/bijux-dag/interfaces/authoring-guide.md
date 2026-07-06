---
title: Authoring Guide
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Authoring Guide

Use the governed fixture set below when authoring or reviewing DAG definitions.

## Start from valid patterns

- `evidence/authoring/patterns/minimal.json`
- `evidence/authoring/patterns/medium.json`
- `evidence/authoring/patterns/pattern_chain.json`
- `evidence/authoring/patterns/pattern_diamond.json`
- `evidence/authoring/patterns/pattern_fanout.json`
- `evidence/authoring/patterns/pattern_aggregation.json`
- `evidence/authoring/patterns/pattern_cache_heavy.json`
- `evidence/authoring/patterns/pattern_replay_sensitive.json`

## Use negative fixtures to understand rejection behavior

- `evidence/authoring/negative/undeclared_outputs.json`
- `evidence/authoring/negative/invalid_refs.json`
- `evidence/authoring/negative/cycle.json`
- `evidence/authoring/negative/invalid_selectors.json`
- `evidence/authoring/negative/unsupported_adapter_payload.json`

## Working rule

- treat the positive fixtures as executable starting points
- treat the negative fixtures as explicit guardrails for unsupported authoring
  moves
