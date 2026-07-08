---
title: Scope and Non-Goals
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-05
---

# Scope and Non-Goals

Use this page when you need the honest boundary of the shipped `bijux-dag`
product: what the local-first DAG runtime defends today, and what still sits
outside the `v0.4.0` promise.

Scope discipline matters more in DAG tooling than in many CLIs, because readers
will otherwise assume that every backend, every orchestration style, and every
control-plane idea in the repository is already part of the supported product.

## What `bijux-dag` Is For

`bijux-dag` is a local-first runtime for validated workflows with retained
evidence. It exists to help an operator answer practical questions with proof:
what would run, what did run, what changed, and which artifact or trace proves
that conclusion.

## In Scope

- DAG model validity and canonical semantics
- deterministic run and artifact evidence surfaces
- replay and diff contract vocabularies (`equivalent`, `drift`, `incomplete`/`unknown`)
- bounded backend capability semantics and explicit downgrade handling

## What Readers Should Not Assume

- Not every backend is promised to behave identically.
- Hidden experimental, simulated, and internal routes are not part of the
  normal operator contract.
- The DAG product is not claiming to replace organization-wide compliance,
  policy, or fleet-control systems.

## Non-Goals

- claiming equal behavior across all backends and environments
- masking missing evidence as successful equivalence
- collapsing graph/run/artifact scopes into one generic change signal
- shipping simulated platform-control namespaces as stable operator APIs
- replacing organization security/compliance policy systems

The current hidden experimental and simulation surfaces remain constrained by
`LIM-005` and `LIM-006` in [Known Limitations](../quality/known-limitations.md).
The post-`v0.4.0` promotion path for scheduling, remote workers, and cluster
backends lives in the [Bijux Dag Roadmap](../../tracking/bijux-dag-roadmap.md).

## Practical Reading Rule

- Stay here when the question is what the shipped DAG product really promises.
- Move to [Release Boundary](release-boundary.md) when you need the stable,
  experimental, simulated, and internal lane split.
- Move to package pages when you already know the issue belongs to graph truth,
  runtime execution, app orchestration, or artifact storage.

## Code Anchors

- `crates/bijux-dag-core/src/`
- `crates/bijux-dag-runtime/src/replay/`
- `crates/bijux-dag-app/src/routes/diff_routes.rs`
- `crates/bijux-dag-artifacts/src/integrity/`

## Continue Reading

- [Ownership Boundary](ownership-boundary.md)
- [Release Boundary](release-boundary.md)
- [Bijux Dag Roadmap](../../tracking/bijux-dag-roadmap.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Known Limitations](../quality/known-limitations.md)
