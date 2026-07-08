---
title: Change Control
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Change Control

Use this page when a maintainer-facing change looks small in code but may still
change release, governance, or verification behavior in ways that reviewers
need to classify explicitly.

Maintainer automation is risky for a simple reason: it can change how the
repository decides what is healthy, releasable, or trustworthy. The change bar
therefore depends on the surface being touched, not just the line count.

## What Requires Deliberate Review

- classify changes by affected command family and policy impact
- keep command behavior and docs updated together
- require explicit evidence for compatibility-sensitive changes
- keep `configs/dag/policy/test_taxonomy.json` aligned with the actual test-suite structure instead of accumulating legacy allowlists
- track temporary exceptions with expiry and owner

## Common Change Classes

| Change class | Why reviewers should care |
| --- | --- |
| command-surface change | scripts, docs, and release workflows may depend on the current interface |
| suite or policy change | the repository may start passing or failing for different reasons |
| report-shape change | downstream automation and human release proof may misread the result |
| release-path change | publication decisions can drift from documented policy |

## Review Checklist

- owning module is clear
- tests cover affected pathways
- docs and links updated
- risk notes updated when needed

## What This Page Is Not Saying

- It is not saying every maintainer edit needs heavyweight process.
- It is not replacing crate-level code review.
- It is not encouraging silent local exceptions when the durable rule belongs
  in policy or documentation.

## Code Anchors

- `crates/bijux-dev/src/commands/mod.rs`
- `crates/bijux-dev/src/suites/mod.rs`
- `crates/bijux-dev/src/policy/mod.rs`
- `configs/dag/policy/test_taxonomy.json`

## Continue Reading

- [Contract Governance](contract-governance.md)
- [Dependency Governance](dependency-governance.md)
- [Core Change Management](../../bijux-core/governance/change-management.md)
