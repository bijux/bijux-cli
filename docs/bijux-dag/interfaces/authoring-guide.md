---
title: Authoring Guide
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Authoring Guide

Use this page when you are authoring, reviewing, or debugging a DAG definition
and need the governed fixture set that defines the supported authoring
experience.

For field-level schema details, start with
[Graph Schema Reference](graph-schema.md). For the contract that
governs the fixture set itself, open
[Authoring UX Contract](../../spec/AUTHORING_UX_CONTRACT.md).

## Valid starting patterns

- `evidence/authoring/patterns/minimal.json`
- `evidence/authoring/patterns/medium.json`
- `evidence/authoring/patterns/pattern_chain.json`
- `evidence/authoring/patterns/pattern_diamond.json`
- `evidence/authoring/patterns/pattern_fanout.json`
- `evidence/authoring/patterns/pattern_aggregation.json`
- `evidence/authoring/patterns/pattern_cache_heavy.json`
- `evidence/authoring/patterns/pattern_replay_sensitive.json`

These fixtures are the maintained positive patterns. They must parse and
validate without error-level diagnostics.

## Rejection fixtures

- `evidence/authoring/negative/undeclared_outputs.json`
- `evidence/authoring/negative/invalid_refs.json`
- `evidence/authoring/negative/cycle.json`
- `evidence/authoring/negative/invalid_selectors.json`
- `evidence/authoring/negative/unsupported_adapter_payload.json`

Use these fixtures when you need to understand refusal behavior or confirm that
an unsupported authoring move still fails for the right reason.

## Working rule

- treat the positive fixtures as executable starting points
- treat the negative fixtures as explicit guardrails for unsupported authoring
  moves
- update this guide and the authoring contract together when the governed
  fixture set changes
