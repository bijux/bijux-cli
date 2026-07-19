---
title: Policy Evaluation Trace
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Policy Evaluation Trace

The effective policy surface in `bijux-dag` must remain explainable, not merely
applied.

## Scope

This contract governs the trace emitted by `dag policy show-effective` and the
runtime-facing policy decisions it summarizes.

## Trace guarantees

- Policy evaluation trace must be available for operator/debug inspection.
- the trace must distinguish deny-network, deny-env, deny-clock, and clean-env
  decisions
- the trace must reflect the effective merged policy rather than raw CLI or
  environment inputs in isolation
- the trace may be advisory text, but it must stay deterministic for a given
  effective policy

## Governed rule names

The current rule identifiers are:

- `rule:deny_network`
- `rule:deny_env`
- `rule:deny_clock`
- `rule:clean_env`

## Related tests

- `crates/bijux-dag-app/tests/config_effective_command_contract.rs`
- `crates/bijux-dag-app/src/commands/config_resolution.rs`
- `crates/bijux-dag-app/src/commands/config_surface.rs`

## Versioning and change policy

Any incompatible change to policy trace availability, rule naming, or effective
policy explanation must update this contract and the linked tests in the same
change.
