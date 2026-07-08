---
title: Maintainer Control Plane
audience: maintainers
type: architecture
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-06
---

# Maintainer Surface

Use this page when the question is about the repository’s proving and control
machinery rather than the public product runtimes.

Maintainer capabilities are intentionally isolated from end-user runtime paths
so governance automation does not distort CLI or DAG behavior.

## What The Maintainer Surface Does

- repository structure and contract audits
- evidence generation and verification workflows
- release readiness and compatibility checks
- documentation governance and layout validation

## Why The Separation Matters

- Product runtimes should not absorb repository-governance logic.
- Maintainer commands need product facts, but they should consume them as
  evidence rather than rewrite product behavior.
- Release and audit flows should stay explainable without becoming hidden
  end-user features.

## Separation Rules

- product runtime crates do not import maintainer command logic
- maintainer crate may call product contracts as read-only inputs
- operational decisions must be explainable from generated evidence

## Separation Non-Goals

- maintainer commands must not become hidden user-facing runtime entrypoints
- governance reports must not override crate-level ownership boundaries
- release automation must not bypass required program contract checks

## Escalation Path

When a maintainer command needs product behavior changes, escalate in this
order:

1. update owning program handbook (`bijux-cli` or `bijux-dag`)
2. add contract/test evidence in the owning crate
3. update maintainer workflows that consume the new evidence

## Code Anchors

- `crates/bijux-dev/src/commands/mod.rs`
- `crates/bijux-dev/src/suites/`
- `crates/bijux-dev/src/report/`
- `crates/bijux-dev/src/maintainer/`

## Continue Reading

- [Dependency Direction](dependency-direction.md)
- [Repository Gates](../../bijux-dev/operations/repository-gates.md)
- [Quality Policy](../../bijux-dev/governance/quality-policy.md)
