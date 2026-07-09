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

- `evidence/authoring/patterns/minimal.json`
- `evidence/authoring/patterns/medium.json`
- `evidence/authoring/patterns/pattern_chain.json`
- `evidence/authoring/patterns/pattern_diamond.json`
- `evidence/authoring/patterns/pattern_fanout.json`
- `evidence/authoring/patterns/pattern_aggregation.json`
- `evidence/authoring/patterns/pattern_cache_heavy.json`
- `evidence/authoring/patterns/pattern_replay_sensitive.json`

These fixtures must parse and validate without error-level diagnostics.

## Negative authoring fixtures

- `evidence/authoring/negative/undeclared_outputs.json`
- `evidence/authoring/negative/invalid_refs.json`
- `evidence/authoring/negative/cycle.json`
- `evidence/authoring/negative/invalid_selectors.json`
- `evidence/authoring/negative/unsupported_adapter_payload.json`

Negative fixtures must remain explicit proof for rejected authoring patterns.

## Related tests

- `crates/bijux-dag-core/tests/authoring_examples_contract.rs`
- `crates/bijux-dag-core/tests/validation_adversarial_contracts.rs`

## Versioning and change policy

Any incompatible change to authoring examples, negative validation outcomes, or
fixture-backed authoring claims must update this contract and the companion
guide in the same change.
