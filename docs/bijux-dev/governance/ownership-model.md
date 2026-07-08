---
title: Ownership Model
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Ownership Model

Use this page when the question is "who owns this maintainer behavior?" rather
than "which command mentioned it first?"

The point of the ownership model is to stop maintainer tooling from gradually
absorbing product semantics or hiding product behavior behind repository
automation.

## Ownership Rules

- `bijux-dev` owns governance automation and evidence orchestration
- CLI and DAG crates own product runtime behavior and user contracts
- shared policy updates require coordinated documentation across handbooks

## What `bijux-dev` Should Own

| Surface | Why it belongs here |
| --- | --- |
| suites and contract checks | they evaluate repository health across products |
| evidence reporting | they summarize health, drift, and release proof for maintainers |
| release automation | it coordinates publication and verification across crate families |
| governance diagnostics | it checks whether docs, contracts, and workflows stay aligned |

## Boundary Violations

- maintainer commands changing product behavior semantics
- product crates importing maintainer-only policy logic
- docs claims with no owning code anchor

## Reader Shortcut

If the answer changes what an operator sees in `bijux` or `bijux-dag`, the
owning product crate or handbook should lead. If the answer changes how the
repository checks, proves, or releases that behavior, `bijux-dev` is the
likely owner.

## Code Anchors

- `crates/bijux-dev/src/lib.rs`
- `crates/bijux-dev/src/maintainer/`
- `crates/bijux-dev/src/suites/`

## Continue Reading

- [Change Control](change-control.md)
- [Contract Governance](contract-governance.md)
- [Core Package Ownership](../../bijux-core/governance/package-ownership.md)
