---
title: Node Execution Explanation Contract
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Node Execution Explanation Contract

Blocked-node explanation in `bijux-dag` must classify why one node did or did
not execute from durable run-directory evidence.

## Scope

This contract governs:

- `bijux-dag node <run_dir> --id <node_id>`
- `bijux-dag explain <run_dir> --node <node_id>`

It is exercised by:

- `crates/bijux-dag-app/src/inspect/node_execution_explanation.rs`
- `crates/bijux-dag-app/tests/node_inspection_contract.rs`

## Required classification surface

Node execution explanation must emit exactly one durable classification from
the following set:

- `executed`
- `dependency_blocked`
- `trigger_rule_blocked`
- `branch_skipped`
- `resource_blocked`
- `selector_excluded`
- `cache_reused`
- `policy_denied`
- `unknown`

The explanation payload must also surface:

- whether the node actually executed
- a machine-readable `reason`
- a human-readable `summary`
- blocking upstream node ids when they are knowable
- trigger-rule name and trigger reason when they are knowable
- parent trigger statuses when they are persisted
- scheduler blocking reasons when they are persisted
- the evidence sources used to derive the explanation

## Evidence rules

- explanation must prefer persisted node-trace evidence when a valid
  `nodes/<node_id>/trace.json` exists
- explanation may supplement trace evidence with
  `observability.events.json`, `run-log.index.json`, the persisted graph
  snapshot, and persisted upstream node traces
- explanation must not depend on ambient Git state, the current working
  directory, or unstored runtime memory
- explanation must distinguish nodes that executed and failed from nodes that
  never executed
- `dag explain <run_dir> --node <node_id>` must still classify blocked nodes
  when the node never produced a trace file
- `dag node <run_dir> --id <node_id>` may continue to require required node
  evidence, including `trace.json`

## Reason mapping

The explanation classifier must use persisted evidence to map these durable
operator meanings:

- `dependency_blocked`: upstream failure or non-success dependency state kept
  the node from running
- `trigger_rule_blocked`: dependency statuses were available but did not
  satisfy the node trigger rule
- `branch_skipped`: branch selection excluded the node
- `resource_blocked`: scheduler evidence reports resource or parallelism
  pressure
- `selector_excluded`: include or exclude selectors removed the node before
  execution
- `cache_reused`: outputs were reused from cache instead of re-executing the
  node
- `policy_denied`: policy evaluation denied execution before the node ran

## Versioning and change policy

Any incompatible change to the classification taxonomy, required explanation
fields, or missing-trace behavior must update this contract and the linked app
tests in the same change.

## Related tests

- `crates/bijux-dag-app/tests/node_inspection_contract.rs`
- `crates/bijux-dag-app/tests/diff_explain_contract.rs`
- `crates/bijux-dag-app/tests/replay_semantic_surface_contracts.rs`
