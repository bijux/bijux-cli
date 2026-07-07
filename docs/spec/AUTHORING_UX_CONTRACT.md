---
title: Authoring UX Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Authoring UX Contract

Authoring in `bijux-dag` must stay grounded in executable examples and
intentional negative fixtures, not prose-only claims.

## Scope

This contract covers the maintained authoring fixture set consumed by strict
graph parsing and validation guards.

## Positive authoring fixtures

- `evidence/dag/authoring/patterns/minimal.json`
- `evidence/dag/authoring/patterns/medium.json`
- `evidence/dag/authoring/patterns/pattern_chain.json`
- `evidence/dag/authoring/patterns/pattern_diamond.json`
- `evidence/dag/authoring/patterns/pattern_fanout.json`
- `evidence/dag/authoring/patterns/pattern_aggregation.json`
- `evidence/dag/authoring/patterns/pattern_cache_heavy.json`
- `evidence/dag/authoring/patterns/pattern_replay_sensitive.json`

These fixtures must parse and validate without error-level diagnostics.

## Negative authoring fixtures

- `evidence/dag/authoring/negative/undeclared_outputs.json`
- `evidence/dag/authoring/negative/invalid_refs.json`
- `evidence/dag/authoring/negative/cycle.json`
- `evidence/dag/authoring/negative/invalid_selectors.json`
- `evidence/dag/authoring/negative/unsupported_adapter_payload.json`

Negative fixtures must remain explicit proof for rejected authoring patterns.

## Related tests

- `crates/bijux-dag-core/tests/authoring_examples_contract.rs`
- `crates/bijux-dag-core/tests/validation_adversarial_contracts.rs`

## Versioning and change policy

Any incompatible change to authoring examples, negative validation outcomes, or
fixture-backed authoring claims must update this contract and the companion
guide in the same change.
