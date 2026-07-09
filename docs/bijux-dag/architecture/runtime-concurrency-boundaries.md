---
title: Runtime Concurrency Boundaries
audience: mixed
type: architecture
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Runtime Concurrency Boundaries

This page names the concurrency boundaries that `bijux-dag` currently
implements so operators and maintainers do not infer a broader scheduler or
replication model than the runtime actually proves.

## Core Boundary

The stable runtime is a local, single-controller execution engine.

- one controlling process owns selection, queueing, scheduler state, and node
  status transitions for a run
- parallel node execution may occur within the run, but that parallelism is
  coordinated by the same local controller boundary
- retained run evidence is the recovery boundary; there is no replicated
  control plane or multi-controller lease protocol in the stable surface

## Implemented Concurrency Surfaces

- worker pools, job budgets, and named resource capacities govern local
  parallel execution within one run
- scheduler checkpoints, timeline evidence, and run snapshots persist the
  decisions needed to inspect concurrent execution after the fact
- replay, diff, and cache verification treat retained evidence as the source of
  truth instead of relying on ambient process state

## Excluded Concurrency Claims

This repository does not currently claim:

- multi-controller failover
- distributed queue ownership
- remote worker lease coordination as a stable operator surface
- cluster-wide scheduler consensus

For the contract language behind these limits, use
[Known Limitations](../quality/known-limitations.md) and
[Concurrency Model](../../spec/CONCURRENCY_MODEL.md).
