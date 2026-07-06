---
title: Observability Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Observability Contract

Runtime observability must preserve required lifecycle events, reconstructible
timelines, and redaction-aware structured diagnostics.

## Scope

This contract covers runtime event records, timeline reconstruction, event-log
completeness verification, and sink behavior implemented in
`crates/bijux-dag-runtime/src/diagnostics/runtime/observability.rs`.

## Required runtime event names

The required lifecycle event catalog includes:

- `run_started`
- `node_ready`
- `node_started`
- `node_attempt_started`
- `node_attempt_finished`
- `node_scheduled`
- `node_finished`
- `run_finished`

The reference contract test
`required_runtime_event_names_are_present_for_reference_sequence` must remain
present and passing.

## Event record requirements

- required event fields must include a non-empty `name`
- `unix_ms` must be positive
- `run_id` must be present and non-empty
- event details must be auditable for sensitive material

## Timeline and completeness behavior

- `reconstruct_timeline_from_events` must produce a stable `TimelineExport`
- reconstructed timeline categories derive from the runtime `EventCategory`
- `verify_event_log_completeness` must report missing required names, field
  gaps, timestamp monotonicity, and timeline drift

## Related tests

- `crates/bijux-dag-runtime/tests/observability_contracts.rs`
- `crates/bijux-dag-runtime/tests/observability_deep_contracts.rs`

## Versioning and change policy

Any incompatible change to required runtime event names, event-field
requirements, timeline reconstruction, or completeness semantics must update
this contract and the linked tests in the same change.
