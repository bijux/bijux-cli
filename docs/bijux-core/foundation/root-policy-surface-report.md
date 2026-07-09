---
title: Root Policy Surface Report
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Root Policy Surface Report

This page is the short guide to the contracts that keep repository-level claims
honest. When a release, handbook, or root workflow says "this is the supported
surface," one of the contracts listed here is usually the file that freezes
that claim and the test suite that proves it.

The primary inventory for this page is
`contracts/foundation/root_policy_surface_inventory.v1.json`.

## What Counts As A Root Policy Surface

A root policy surface is not just any file under `contracts/`. It is a shared
contract that affects how the repository is published, routed, reviewed, or
validated across more than one package family.

These files matter because they stop repository-wide decisions from turning
into folklore.

## The Most Important Root Contracts

| Repository question | Governing contract | Owning crate | What the contract decides |
| --- | --- | --- | --- |
| Which `bijux-dev-dag` commands are part of the visible maintainer surface? | `contracts/foundation/maintainer_command_surface.v1.json` | `bijux-dev` | the stable maintainer command inventory and the command names reviewers should expect to see |
| Which governance issue classes exist and where their evidence must live? | `contracts/foundation/backlog_issue_class_routing.v1.json` | `bijux-dev` | the durable issue taxonomy for repository-level governance work |
| Which workspace crates are public releases and which stay private support code? | `contracts/foundation/workspace_package_boundary.v1.json` | `bijux-dev` | public-versus-private publication intent across the workspace |
| Which `bijux-dag` operator surfaces are stable, experimental, simulated, internal, or refused? | `contracts/foundation/dag_release_truth_table.v1.json` | `bijux-dev` | the release boundary that keeps DAG claims and docs from overstating what ships |
| Which product namespaces and aliases are reserved by the runtime? | `contracts/official_product_namespace_registry.json` | `bijux-cli` | namespace ownership and collision policy for mounted product surfaces |

## What This Report Helps A Reader Do

Use this page to answer questions like:

- Which file actually governs this repository-wide claim?
- Which crate is supposed to keep that claim enforced?
- Is this a product-surface promise, a release-boundary rule, or a maintainer
  control-plane rule?
- Where should I look first when a docs claim and a test result disagree?

## How These Contracts Stay Honest

Each contract in the root inventory is paired with one or more executable
checks under the owning crate. The contract is not only documentation. It is a
machine-readable boundary that a suite is expected to enforce.

That is why these files sit above one handbook page or one crate README. They
govern repository behavior that has to remain consistent across multiple
surfaces.

## Reading Pattern

Use the following route when a root claim looks suspicious:

1. find the claim in a handbook, README, workflow, or release note
2. identify the governing contract from this page
3. inspect the owning crate and enforcing suite
4. decide whether the drift is in docs, contract data, or executable checks

## Related Pages

- [Backlog Routing Ledger](backlog-routing-ledger.md)
- [Package Boundary](package-boundary.md)
- [Maintainer Command Surface](../../bijux-dev/operations/command-surface.md)
