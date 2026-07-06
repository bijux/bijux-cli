---
title: Operator Inspection Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Operator Inspection Contract

Run inspection in `bijux-dag` must derive judgments from run-directory
evidence rather than ambient project state.

## Scope

This contract covers run summary generation, tree and timeline inspection,
failure explanation, imported-run support, and corrupt or unsupported run
classification exercised by `crates/bijux-dag-app/tests/operator_ux_contract.rs`.

## Inspection rules

- imported runs must remain inspectable through explicit run roots
- unsupported run-dir formats must be classified as unsupported rather than
  silently treated as healthy
- corrupt manifests or missing evidence must surface corrupt or incomplete
  integrity states
- timeline inspection must prefer `observability.timeline.json` when it exists
  and fall back to node-trace projection only for older or incomplete runs
- timeline inspection must preserve ordered event timestamps together with
  lifecycle labels, node identity, and any available terminal reason
- timing summaries must remain bounded by node trace timestamps

## Related tests

- `crates/bijux-dag-app/tests/operator_ux_contract.rs`

## Versioning and change policy

Any incompatible change to inspection classification, timing-summary behavior,
or imported-run handling must update this contract and the linked app tests in
the same change.
