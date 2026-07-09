---
title: Backlog Routing Ledger
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Backlog Routing Ledger

The backlog routing ledger explains how repository-level governance work is
classified before anyone starts changing code or docs. Its job is simple:
repository issues should enter the workspace with a named owner, an allowed
dependency lane, and a predictable evidence location.

The canonical source for the issue taxonomy is
`contracts/foundation/backlog_issue_class_routing.v1.json`.

## Why This Ledger Exists

Without an explicit routing model, repository work tends to collapse into vague
labels such as "governance," "cleanup," or "maintenance." That makes review,
ownership, and evidence weaker than they need to be.

This page translates the contract into reader-facing terms: what the issue
classes are for, who owns them, and what proof surfaces a reviewer should
expect to see.

## Durable Issue Classes

| Issue class | Owning crate | Evidence location | What belongs in this class |
| --- | --- | --- | --- |
| `foundation-ownership-boundary` | `bijux-dev` | `contracts/foundation/` | work that changes root ownership contracts, package boundaries, or other shared repository truth tables |
| `foundation-backlog-governance` | `bijux-dev` | `docs/bijux-core/foundation/` | work that keeps governance routing, root policy visibility, and handbook accountability understandable to readers |
| `foundation-compatibility-lanes` | `bijux-cli` | `contracts/schemas/` | work that changes compatibility lanes, schema-backed command output, or other machine-readable consumer boundaries |
| `foundation-release-gate` | `bijux-dev` | `crates/bijux-dev/tests/` | work that changes release criteria, hard gates, or the executable proof behind publication claims |
| `foundation-operator-diagnostics` | `bijux-cli` | `crates/bijux-cli/src/interface/cli/handlers/` | work that changes operator-facing diagnostic handling for the CLI runtime family |

## How To Use The Ledger

Use this page when you need to answer one of these questions:

- Is this governance work really about contracts, docs, release proof, or
  operator diagnostics?
- Which crate is expected to enforce the change?
- Where should evidence appear if the change is done correctly?

If an issue cannot be routed through one of these classes, that is usually a
sign that the taxonomy needs a deliberate update rather than an uncategorized
exception.

## What Good Routing Prevents

- repository work landing without a clear owning crate
- docs and contracts drifting because the evidence lane was never named
- release-gate changes being treated like ordinary prose edits
- operator diagnostics being routed through maintainer governance by accident

## Review Shortcut

When a repository-level change is proposed, reviewers can use this ledger in a
few seconds:

1. identify the issue class
2. confirm the owning crate matches the implementation
3. confirm the evidence location matches the claimed work
4. reject vague routing before the ambiguity spreads into code or docs

## Related Pages

- [Root Policy Surface Report](root-policy-surface-report.md)
- [Package Boundary](package-boundary.md)
- [Maintainer Handbook](../../bijux-dev/index.md)
