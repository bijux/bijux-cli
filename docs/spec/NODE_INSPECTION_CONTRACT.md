---
title: Node Inspection Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Node Inspection Contract

Explicit node inspection in `bijux-dag` must summarize one node from durable
run-directory evidence without inventing state that was not persisted.

## Scope

This contract governs the explicit-path `dag node <run_dir> --id <node-id>`
surface exercised by
`crates/bijux-dag-app/tests/node_inspection_contract.rs`.

## Required evidence view

Node inspection must expose:

- planned node fields from the persisted graph snapshot
- resolved params from `nodes/<node_id>/resolved_params.json` when present
- input artifacts from `nodes/<node_id>/inputs/index.json` when present
- output artifacts from `nodes/<node_id>/outputs/index.json` when present
- terminal attempt number plus per-attempt status and relative log paths from
  `nodes/<node_id>/attempts.json` when present
- terminal stdout and stderr paths plus tail excerpts from node log files when
  present
- configured cache policy together with the observed cache result derived from
  trace evidence
- failure, skip, transition-cause, and lifecycle-state evidence when present
- execution explanation derived from persisted trace, event, and dependency
  evidence

## Honesty rules

- inspection must derive its state from persisted files inside the run
  directory, not ambient repository state
- missing optional evidence must surface as explicit evidence gaps rather than
  silent omission disguised as success
- malformed required node evidence may fail inspection rather than being
  treated as valid data

## Related tests

- `crates/bijux-dag-app/tests/node_inspection_contract.rs`

## Related contracts

- [Node Execution Explanation Contract](NODE_EXECUTION_EXPLANATION_CONTRACT.md)

## Versioning and change policy

Any incompatible change to the node inspection payload, evidence-gap behavior,
or human-facing evidence summary must update this contract and the linked app
tests in the same change.
