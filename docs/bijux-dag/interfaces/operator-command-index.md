---
title: Operator Command Index
audience: operators
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Operator Command Index

Use these commands when the run already exists and the job is to inspect,
verify, compare, or diagnose retained evidence.

## Run Inspection Commands

- `bijux-dag runs list`: enumerate available runs under an explicit root
- `bijux-dag runs show`: show compact status and timing for one run
- `bijux-dag runs inspect`: derive the structured inspection summary for one run
- `bijux-dag runs summary`: aggregate retained history into one repository-local
  overview
- `bijux-dag runs compare`: compare two retained runs across status, retries,
  cache hits, artifacts, timing, graph and execution fingerprints, graph
  inputs, selected nodes, node statuses, output hashes, and the first
  meaningful retained-evidence divergence
- `bijux-dag runs trend`: render one analytics point per retained run
- `bijux-dag runs failures`: aggregate failed node kinds across retained runs
- `bijux-dag runs flakes`: identify graph fingerprints with mixed retained
  outcomes
- `bijux-dag runs tree`: render node structure from run evidence
- `bijux-dag runs timeline`: render ordered execution events from
  `observability.timeline.json`, with node-trace projection only as a
  compatibility fallback
- `bijux-dag runs scheduler-checkpoint`: inspect retained scheduler checkpoint
  evidence including ready nodes, scheduled batch, resource-blocked nodes,
  inflight nodes, completed statuses, and the decision reason for that loop
- `bijux-dag runs diff`: compare two run directories
- `bijux-dag runs verify`: verify run integrity and compatibility
- `bijux-dag runs doctor`: diagnose corrupt or incomplete run evidence
- `bijux-dag runs explain-failure`: identify the first causal failure and group
  downstream affected nodes by terminal status

## Recovery Boundary

The public operator lane is inspect-first. Recovery or repair actions remain
deliberate internal surfaces rather than part of the default stable operator
path.

## Detailed Inventory

Use [Reference: Operator Command Index](reference/operator-command-index.md)
for the lower-level inventory that also names internal recovery surfaces.
