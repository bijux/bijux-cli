---
title: Backlog Routing Ledger
audience: maintainers
type: governance
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-08
---

# Backlog Routing Ledger

This ledger records the durable issue classes that govern repository-level work
and the evidence surfaces that keep those classes reviewable.

| Goal | Issue class | Owning crate | Evidence location | Status | Note |
| --- | --- | --- | --- | --- | --- |
| Root policy inventory stays linked from maintainer docs | foundation-backlog-governance | bijux-dev | docs/bijux-core/foundation/root-policy-surface-report.md | done | Keeps the maintained root policy inventory visible from handbook and package surfaces. |
| Backlog issue classes remain owned and categorized | foundation-backlog-governance | bijux-dev | contracts/foundation/backlog_issue_class_routing.v1.json | done | Freezes the routing taxonomy so repository work does not fall back to uncategorized intake. |
| Release-gate ownership stays tied to executable proof | foundation-release-gate | bijux-dev | crates/bijux-dev/tests/ | done | Keeps release-boundary and publication checks anchored in executable maintainer suites. |
