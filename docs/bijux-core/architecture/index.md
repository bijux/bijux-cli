---
title: Core Architecture
audience: mixed
type: section-index
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Core Architecture

This section explains the decisions that keep one repository from becoming one
indistinct runtime. `bijux-core` contains two user-facing command families, a
Python distribution bridge, retained DAG evidence, and maintainer-only proof
machinery. They share release governance, but they do not share ownership of
runtime behavior.

## Choose A Page By Decision

| Decision | Start here | Expected outcome |
| --- | --- | --- |
| place a new crate or dependency | [Workspace Topology](workspace-topology.md) and [Dependency Direction](dependency-direction.md) | an owner and an allowed dependency direction |
| change a public command or output | [Runtime Surfaces](runtime-surfaces.md) | the affected product contract and compatibility obligations |
| add configuration or retained state | [State and Configuration](state-and-configuration.md) | an explicit precedence rule and storage owner |
| change packaging or publication | [Distribution Model](distribution-model.md) | one verified source revision across every release surface |
| add a repository check or report | [Maintainer Control Plane](maintainer-control-plane.md) | proof machinery outside product runtime paths |
| change a schema, snapshot, or generated reference | [Artifact and Contract Flow](artifact-and-contract-flow.md) | implementation, authority, evidence, and handbook changes kept together |
| accept or mitigate structural debt | [Architecture Risks](architecture-risks.md) | a named risk, detection route, and release consequence |

## Invariants

- Product behavior is owned by the CLI or DAG crates that execute it.
- Public crates never require private maintainer crates at runtime or build
  time.
- Machine-readable contracts govern serialized shape; handbooks explain the
  supported behavior without becoming a second schema.
- Generated evidence can support a decision but cannot redefine the contract
  that generated it.
- A release claim is not complete until its code, tests, generated references,
  package pages, and operator guidance agree.

The workspace package classification and crates.io order are governed by
`contracts/foundation/workspace_package_boundary.v1.json`. The corresponding
contract tests compare that file with Cargo metadata; this page does not
duplicate its package list.

## Architecture Review

An architecture review should be able to answer four questions:

1. Which crate or repository surface owns the behavior?
2. Which dependencies are necessary, and do they point toward the owner?
3. Which executable contract detects semantic drift?
4. What evidence would block publication if the change were wrong?

If the owner or proof route is unclear, the change is not ready to be hidden
behind a new shared helper, script, or documentation page.

## Entry Points

- [System Overview](system-overview.md) for the complete ownership and proof
  flow
- [Foundation](../foundation/index.md) for product and package scope
- [Operations](../operations/index.md) for contributor and release procedures
- [Repository Handbook](../index.md) for reader-oriented navigation
