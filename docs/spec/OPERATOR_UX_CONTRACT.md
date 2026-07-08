---
title: Operator UX Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Operator UX Contract

Operator-facing run inspection in `bijux-dag` must stay concise, explicit, and
trace-coherent across human and machine output.

## Scope

This contract covers operator commands that inspect existing run directories,
report integrity state, summarize timing, and explain failure boundaries as
exercised by `crates/bijux-dag-app/tests/operator_ux_contract.rs`.

## Required operator run surfaces

The governed operator inspection lanes are:

- `dag runs list`
- `dag runs show`
- `dag runs inspect`
- `dag runs tree`
- `dag runs timeline`
- `dag runs scheduler-checkpoint`
- `dag runs diff`
- `dag runs verify`
- `dag runs doctor`
- `dag runs explain-failure`

## Output expectations

- human output must remain concise enough to scan quickly
- timing summaries must remain coherent with trace evidence
- timeline inspection must expose timestamps and causes in human output
- timeline inspection filters must remain available for node, event, and
  inclusive time windows
- scheduler checkpoint inspection must surface decision reason, ready nodes,
  scheduled nodes, resource-blocked nodes, inflight nodes, and completed
  statuses when checkpoint evidence exists
- scheduler checkpoint inspection must distinguish retained, absent, and corrupt
  checkpoint evidence explicitly instead of implying a scheduler state that was
  not actually recorded
- failure explanation must identify the first causal failure rather than only
  list every failed or skipped node
- failure explanation must separate propagated failed nodes from propagated
  skipped or cancelled nodes
- failure explanation must group downstream affected nodes by terminal status
- failure explanation must surface failure class, code, message, and stable
  reason for the primary failure
- integrity state must distinguish healthy, incomplete, corrupt, and
  unsupported runs
- explicit `--root` input must be sufficient without ambient repository state

## Related tests

- `crates/bijux-dag-app/tests/operator_ux_contract.rs`

## Versioning and change policy

Any incompatible change to operator command vocabulary, integrity-state
classification, or inspection summary semantics must update this contract and
the linked app tests in the same change.
