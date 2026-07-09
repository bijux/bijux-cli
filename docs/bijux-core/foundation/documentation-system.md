---
title: Documentation System
audience: mixed
type: explanation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Documentation System

`bijux-core` keeps documentation authority split by durable responsibility so
operators, maintainers, and contributors can tell which page governs which
claim.

## Authority Layers

- Foundation pages define repository scope, package boundaries, durable
  language, and ownership rules.
- Architecture pages explain subsystem shape, boundary decisions, and
  integration rules inside the repository.
- Interface pages describe operator-visible or maintainer-visible surfaces and
  point back to their governing contracts.
- Operations pages cover repeatable procedures, verification flows, and
  evidence-backed maintenance steps.
- Report pages summarize governed facts generated from repository policy,
  contract tests, or curated evidence.

## Reading Rules

- Start in Foundation when the question is about what belongs in this
  repository, which crate owns a capability, or which terms are allowed to
  harden into durable names.
- Use Architecture when the repository boundary is already clear and the next
  question is subsystem shape or integration behavior.
- Use Interfaces when the claim is about a command, schema, API, or another
  surfaced contract.
- Use Operations when the task is procedural and must stay aligned with the
  repository's verification flow.
- Treat report pages as derived summaries. When a report and a governing
  contract disagree, the contract wins and the report must be refreshed.

## Durable Authoring Rules

- Repository docs must cite repository-relative contracts, schemas, and source
  files that exist on disk.
- Release-boundary claims must point to the corresponding truth-table or
  package-boundary contract.
- Generated reports must say what governs them and should not invent authority
  that lives elsewhere.

## Next Reads

- [Foundation Index](index.md)
- [Package Boundary](package-boundary.md)
- [Repository Scope](repository-scope.md)
- [Root Policy Surface Report](root-policy-surface-report.md)
