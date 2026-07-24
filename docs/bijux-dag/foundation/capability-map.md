---
title: Capability Map
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-23
---

# Capability Map

`bijux-dag` turns a declared graph into retained evidence that can support a
bounded operational conclusion. Graph truth, execution behavior, evidence
integrity, and replay attribution remain separate authorities throughout that
lifecycle.

## Operational Capabilities

| Capability | Operator result | Acceptance evidence |
| --- | --- | --- |
| graph truth | parsed, validated, canonicalized, fingerprinted, and lowered graph | canonical graph identity, validation result, and execution plan |
| execution | planned and scheduled nodes with policy-aware outcome classification | attempt records, streams, declared outputs, backend identity, and final run state |
| retained evidence | inspectable runs, artifacts, traces, and integrity material | manifest, output index, hashes, trace records, and verification result |
| replay | equivalence, drift, refusal, or unknown classification | source identity, compatibility lane, retained inputs, and first divergent fact |
| comparison | attributed differences between graphs, runs, nodes, or artifacts | normalized identities and structured divergence report |
| cache decisions | explained reuse, miss, invalidation, or corruption refusal | cache key inputs, proof material, and recorded decision reason |

## Core Capability Inventory

| Capability | What it covers | Primary owning crates |
| --- | --- | --- |
| definition truth | parsing, validation, canonicalization, topology, semantic identity, planner lowering | `bijux-dag-core` |
| execution | run planning, node scheduling, adapter boundaries, policy evaluation, runtime diagnostics | `bijux-dag-runtime` |
| operator workflows | command orchestration, response shaping, inspect routes, replay UX, diff UX | `bijux-dag-app`, `bijux-dag-cli` |
| evidence | run manifests, output indexes, trace files, integrity proofs, lifecycle helpers | `bijux-dag-artifacts`, `bijux-dag-runtime` |
| attribution | replay outcomes, run comparison, first-divergence reporting, cache and rerun explanations | `bijux-dag-runtime`, `bijux-dag-app` |

## Capability Lifecycle

```mermaid
flowchart LR
    source["Graph source"]
    core["Parse, validate, canonicalize"]
    plan["Lower to an execution plan"]
    runtime["Schedule and execute"]
    artifacts["Retain manifests, traces, and outputs"]
    inspect["Inspect, replay, compare, and verify"]
    decision["Bounded operator conclusion"]

    source --> core --> plan --> runtime --> artifacts --> inspect --> decision
    artifacts -. integrity or compatibility failure .-> decision
```

Every later conclusion depends on the earlier authority. Execution cannot make
an invalid graph valid. A successful process cannot make incomplete artifacts
authoritative. Replay and comparison cannot infer equality when identity,
integrity, or compatibility evidence is missing.

## Stable Operator Outcomes

The public `bijux-dag` surface is built around a small number of operator
questions:

- is this graph valid and what will execute
- what happened during this run
- can this run be replayed faithfully
- what changed between two runs or two graph versions
- which artifact or node proves that conclusion

The handbook pages under [Interfaces](../interfaces/index.md) and
[Operations](../operations/index.md) are organized around those questions.

## Current Execution Boundary

| Lane | Current statement | Evidence required for a claim |
| --- | --- | --- |
| local host and container | stable operator surface for declared local nodes and supported container engines | accepted attempt records, declared outputs, retained streams, and run integrity |
| Kubernetes Job | stable bounded submission lane for container nodes using `kubectl` and a shared persistent volume claim | backend identity, submitted workload, shared storage assumptions, status, logs, and cleanup outcome |
| shared-filesystem SLURM | stable bounded submission lane through `sbatch` and `sacct` | submission identity, shared path assumptions, scheduler status, output collection, and cleanup outcome |
| generic HPC or remote workers | unreleased | no stable product claim; shared types or modeled reports do not promote the lane |
| simulated control-plane namespaces | modeled and opt-in | simulation evidence only, never production equivalence |

Stable means the bounded lane is part of the current release contract. It does
not mean all clusters, engines, security policies, storage classes, or failure
modes are equivalent.

## Capability Limits

- A stable backend lane is bounded to its declared submission, storage,
  collection, and cleanup contract; it does not imply universal cluster
  equivalence.
- Simulated control-plane routes produce model evidence only and do not prove
  a deployed service or production backend.
- A completed process is not accepted run evidence until required artifacts
  and integrity material verify.
- Replay refuses equivalence when identity, compatibility, or retained inputs
  are insufficient. Unknown is a valid result, not permission to infer
  success.

## Code Anchors

- `crates/bijux-dag-core/src/`
- `crates/bijux-dag-runtime/src/`
- `crates/bijux-dag-app/src/routes/`
- `crates/bijux-dag-artifacts/src/`
- `crates/bijux-dag-cli/src/main.rs`

## Authorities

- [DAG Packages](../packages/index.md)
- [Domain Language](domain-language.md)
- [Release Boundary](release-boundary.md)
- [Module Map](../architecture/module-map.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
